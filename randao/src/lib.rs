pub mod config;
pub mod contract;
pub mod error;
pub mod utils;
use crate::{
    contract::{NewCampaignData, RandaoContract},
    error::{Error, InternalError},
    utils::{extract_keypair_from_config, handle_error},
};
use anyhow::bail;
use bip0039::{Count, Language, Mnemonic};
use bip32::{DerivationPath, XPrv};
use lazy_static::lazy_static;
use libsecp256k1::{PublicKey, SecretKey};
use log::{error, info, warn};
use rand::Rng;
use reqwest::{Client, Url};
use secp256k1::SecretKey as SecretKey2;
use std::{
    fs::OpenOptions,
    io::{Read, Write},
    path::Path,
};

use crate::config::CampaignInfo;
use crate::utils::extract_keypair_from_str;
use config::Config;
use prometheus::{labels, opts};
use prometheus::{register_gauge, Gauge};
use serde::{Deserialize, Serialize};
use sha3::{Digest, Keccak256};
use std::{
    error::Error as StdError,
    fs,
    future::Future,
    path::PathBuf,
    str::FromStr,
    sync::{
        atomic::{AtomicU32, Ordering},
        Arc,
    },
    time,
    time::Duration,
};
use tokio::{runtime::Runtime, sync::mpsc::Receiver, sync::Mutex};
use web3::types::BlockNumber::Number;
use web3::{
    self,
    api::Eth,
    contract::{tokens::Tokenizable, Contract, Options},
    ethabi::{Int, Token, Uint},
    transports::Http,
    types::{
        Address, Block, BlockId, BlockNumber, Bytes, Transaction, TransactionId,
        TransactionReceipt, H160, H256, U128, U256, U64,
    },
};

const _FRC20_ADDRESS: u64 = 0x1000;
pub const BLOCK_TIME: u64 = 16;

lazy_static! {
    pub(crate) static ref CUR_TASKS: Arc<AtomicU32> = Arc::new(AtomicU32::new(0));
    pub(crate) static ref MAX_TASKS: Arc<AtomicU32> = Arc::new(AtomicU32::new(2));
    // total success tasks、total tasks cost time、average tasks cost time queue
    pub(crate) static ref RES_QUEUE_SECS: Arc<Mutex<(u32, u128, Vec::<u128>)>> = Arc::new(Mutex::new((0, 0, Vec::new())));

}

lazy_static! {
    pub static ref ONGOING_CAMPAIGNS: Gauge = register_gauge!(opts!(
        "http_requests_total",
        "Number of HTTP requests made.",
        labels! {"handler" => "all",}
    ))
    .unwrap();
}

const PULL_INTERVAL: u64 = 50;
const RES_QUEUE_MAX_LEN: usize = 10;
const UPDATE_INTERVAL: u64 = 300;
const DELTA_RANGE: u128 = 100;

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct KeyPair {
    pub address: String,
    pub private: String,
}

#[inline(always)]
pub fn one_eth_key() -> KeyPair {
    let mnemonic = Mnemonic::generate_in(Language::English, Count::Words12);
    let bs = mnemonic.to_seed("");
    let ext = XPrv::derive_from_path(&bs, &DerivationPath::from_str("m/44'/60'/0'/0/0").unwrap())
        .unwrap();

    let secret = SecretKey::parse_slice(&ext.to_bytes()).unwrap();
    let public = PublicKey::from_secret_key(&secret);

    let mut res = [0u8; 64];
    res.copy_from_slice(&public.serialize()[1..65]);
    let public = H160::from(H256::from_slice(Keccak256::digest(&res).as_slice()));

    KeyPair {
        address: eth_checksum::checksum(&format!("{:?}", public)),
        private: hex::encode(secret.serialize()),
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct TxMetric {
    pub to: Address,
    pub amount: U256,
    pub hash: Option<H256>, // Tx hash
    pub status: u64,        // 1 - success, other - fail
    pub wait: u64,          // seconds for waiting tx receipt
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct TransferMetrics {
    pub from: Address,
    pub total: u64,
    pub succeed: u64,
    pub txs: Vec<TxMetric>,
}

#[derive(Debug)]
pub struct BlockClient {
    pub web3: Arc<web3::Web3<Http>>,
    pub eth: Arc<web3::api::Eth<Http>>,
    pub accounts: Arc<web3::api::Accounts<Http>>,
    pub _root_sk: secp256k1::SecretKey,
    pub root_addr: Address,
    pub config: config::Config,
    pub randao_contract: RandaoContract,
    rt: Arc<Runtime>,
}

impl Clone for BlockClient {
    fn clone(&self) -> Self {
        BlockClient {
            web3: self.web3.clone(),
            eth: self.eth.clone(),
            accounts: self.accounts.clone(),
            _root_sk: self._root_sk.clone(),
            root_addr: self.root_addr.clone(),
            config: self.config.clone(),
            randao_contract: self.randao_contract.clone(),
            rt: self.rt.clone(),
        }
    }
}

#[derive(Debug)]
pub struct NetworkInfo {
    pub chain_id: U256,
    pub block_number: U64,
    pub gas_price: U256,
    pub frc20_code: Option<Bytes>,
}

impl BlockClient {
    pub fn setup(config: &Config, timeout: Option<u64>) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(timeout.unwrap_or(3)))
            .build()
            .unwrap();
        let url = Url::parse(config.chain.endpoint.as_str()).unwrap();
        let transport = Http::with_client(client, url);
        let web3 = Arc::new(web3::Web3::new(transport));
        let eth = Arc::new(web3.eth());
        let accounts = Arc::new(web3.accounts());
        let (_root_sk, root_addr) = extract_keypair_from_config(&config);
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let rt_arc = Arc::new(rt);
        Self {
            web3,
            eth,
            accounts,
            _root_sk,
            root_addr,
            rt: rt_arc,
            randao_contract: RandaoContract::default(),
            config: config.clone(),
        }
    }

    pub fn chain_id(&self) -> Option<U256> {
        self.rt.block_on(self.eth.chain_id()).ok()
    }

    pub fn block_number(&self) -> Option<U64> {
        self.rt.block_on(self.eth.block_number()).ok()
    }

    pub fn current_block(&self) -> Option<Block<H256>> {
        self.rt
            .block_on(self.eth.block(BlockId::Number(BlockNumber::Latest)))
            .unwrap_or_default()
    }

    pub fn block_with_tx_hashes(&self, id: BlockId) -> Option<Block<H256>> {
        self.block_with_tx_hashes_inner(id, None, None)
    }

    pub fn block_with_tx_hashes_inner(
        &self,
        id: BlockId,
        interval: Option<u64>,
        times: Option<u64>,
    ) -> Option<Block<H256>> {
        let interval = interval.unwrap_or(1);
        let mut retries = 1u64;
        loop {
            if let Ok(Some(block)) = self.rt.block_on(self.eth.block(id)) {
                break Some(block);
            }
            if times == Some(retries) || times == Some(0u64) {
                break None;
            }
            warn!("retries {}", retries);
            retries += 1;
            std::thread::sleep(Duration::from_secs(interval));
        }
    }

    pub fn nonce(&self, from: Address, block: Option<BlockNumber>) -> Option<U256> {
        self.rt
            .block_on(self.eth.transaction_count(from, block))
            .ok()
    }

    pub fn pending_nonce(&self, from: Address) -> Option<U256> {
        self.pending_nonce_inner(from, Some(3), None)
    }

    pub fn pending_nonce_inner(
        &self,
        from: Address,
        interval: Option<u64>,
        times: Option<u64>,
    ) -> Option<U256> {
        let interval = interval.unwrap_or(5);
        let mut tries = 1u64;
        loop {
            match self
                .rt
                .block_on(self.eth.transaction_count(from, Some(BlockNumber::Pending)))
            {
                Ok(nonce) => break Some(nonce),
                Err(e) => error!("failed to get nonce, tries {}, {:?}", tries, e),
            }
            std::thread::sleep(Duration::from_secs(interval));
            if times == Some(tries) || times == Some(0u64) {
                break None;
            }
            tries += 1;
        }
    }

    pub fn gas_price(&self) -> Option<U256> {
        self.rt.block_on(self.eth.gas_price()).ok()
    }

    #[allow(unused)]
    pub fn transaction(&self, id: TransactionId) -> Option<Transaction> {
        self.rt
            .block_on(self.eth.transaction(id))
            .unwrap_or_default()
    }

    pub fn transaction_receipt(&self, hash: H256) -> Option<TransactionReceipt> {
        self.rt
            .block_on(self.eth.transaction_receipt(hash))
            .unwrap_or_default()
    }

    #[allow(unused)]
    pub fn accounts(&self) -> Vec<Address> {
        self.rt.block_on(self.eth.accounts()).unwrap_or_default()
    }

    pub fn balance(&self, address: Address, number: Option<BlockNumber>) -> U256 {
        self.rt
            .block_on(self.eth.balance(address, number))
            .unwrap_or_default()
    }

    pub fn wait_for_tx_receipt(
        &self,
        hash: H256,
        interval: Duration,
        times: u64,
    ) -> (u64, Option<TransactionReceipt>) {
        let mut wait = 0;
        let mut retry = times;
        loop {
            if let Some(receipt) = self.transaction_receipt(hash) {
                wait = times + 1 - retry;
                break (wait, Some(receipt));
            } else {
                std::thread::sleep(interval);
                retry -= 1;
                if retry == 0 {
                    break (wait, None);
                }
            }
        }
    }

    pub fn parse_error(&self, err: Option<&dyn StdError>) -> Error {
        match err {
            Some(e) => {
                let err_str = e.to_string();
                if err_str.contains("broadcast_tx_sync") {
                    Error::GetNumCampaignsErr
                } else if err_str.contains("Transaction check error") {
                    Error::CheckChainErr
                } else if err_str.contains("error sending request") {
                    Error::CheckCampaignsInfoErr
                } else if err_str.contains("InternalError") {
                    if err_str.contains("InvalidNonce") {
                        Error::TxInternalErr(InternalError::InvalidNonce(err_str))
                    } else {
                        Error::TxInternalErr(InternalError::Other(err_str))
                    }
                } else {
                    Error::Unknown(err_str)
                }
            }
            None => Error::Unknown("empty error".to_string()),
        }
    }

    pub fn contract_deploy(&self, deploy_json: DeployJson) -> anyhow::Result<()> {
        self.rt.block_on(async {
            let mut vf = Vec::new();
            for deploy_obj in deploy_json.deploy_obj {
                let DeployJsonObj {
                    code_path,
                    abi_path,
                    sec_key,
                    gas,
                    gas_price,
                    args,
                } = deploy_obj;
                let args = parse_args_csv(&args)?;
                let eth = (*self.eth.clone()).clone();

                let f = move || async move {
                    let succeed = match contract_deploy(
                        eth, &sec_key, &code_path, &abi_path, gas, gas_price, args,
                    )
                    .await
                    {
                        Ok(v) => {
                            println!("contract address: {:?}", v);
                            true
                        }
                        Err(e) => {
                            println!("deploy contract failed: {:?}", e);
                            false
                        }
                    };
                    if !succeed {
                        println!("deploy failed");
                    }
                    Ok(())
                };

                vf.push(f);
            }

            let (success_task, total_times) = multi_tasks_impl(vf).await?;

            println!(
                "success task: {} total times: {} average time: {}",
                success_task,
                total_times,
                if success_task == 0 {
                    0
                } else {
                    total_times / success_task as u128
                }
            );

            anyhow::Ok(())
        })?;

        Ok(())
    }

    pub fn contract_call(&self, call_obj: CallJsonObj) -> anyhow::Result<()> {
        self.rt.block_on(async {
            let CallJsonObj {
                contract_addr,
                abi_path,
                sec_key,
                gas,
                gas_price,
                func_name,
                args,
            } = call_obj;
            let args = parse_args_csv(&args)?;
            let eth = (*self.eth.clone()).clone();
            let result = contract_call(
                eth,
                &contract_addr,
                &sec_key,
                &abi_path,
                gas,
                gas_price,
                &func_name,
                args,
            )
            .await;

            match result {
                Ok(v) => {
                    println!("transaction hash: {:?}", v);
                    true
                }
                Err(e) => {
                    println!("call contract failed: {:?}", e);
                    false
                }
            };
            anyhow::Ok(())
        })?;
        Ok(())
    }

    pub fn contract_query(&self, query_json: QueryJson) -> anyhow::Result<()> {
        self.rt.block_on(async {
            let QueryJson {
                sec_key,
                contract_addr,
                abi_path,
                func_name,
                args,
            } = query_json;

            let args = parse_args_csv(&args)?;
            let (_root_sk, root_addr) = extract_keypair_from_str(sec_key.to_string());
            let eth = (*self.eth.clone()).clone();
            let account = format!("0x{:x}", root_addr);
            let result =
                contract_query(eth, &contract_addr, &account, &abi_path, &func_name, args).await?;
            println!("query result: {:?}", result);
            anyhow::Ok(())
        })?;

        Ok(())
    }

    pub fn contract_setup(
        &mut self,
        sec_key: &str,
        contract_addr: &str,
        abi_path: &str,
        gas: u32,
        gas_price: u128,
    ) {
        self.randao_contract = RandaoContract {
            sec_key: sec_key.to_string(),
            contract_addr: contract_addr.to_string(),
            abi_path: abi_path.to_string(),
            gas: gas,
            gas_price: gas_price,
        };
    }

    pub fn contract_new_campaign(
        &self,
        gas: u32,
        gas_price: u128,
        campaign_data: NewCampaignData,
    ) -> Option<TransactionReceipt> {
        self.rt.block_on(async {
            let eth = (*self.eth.clone()).clone();
            let result = self
                .randao_contract
                .new_campaign(eth, gas, gas_price, campaign_data)
                .await;
            let value = match result {
                Ok(v) => Some(v),
                Err(e) => {
                    println!("call contract_new_campaign failed: {:?}", e);
                    None
                }
            };
            value
        })
    }

    pub fn contract_follow(
        &self,
        gas: u32,
        gas_price: u128,
        campaign_id: u128,
        deposit: u128,
        follow_sec_key: &str,
    ) -> Option<TransactionReceipt> {
        self.rt.block_on(async {
            let eth = (*self.eth.clone()).clone();
            let result = self
                .randao_contract
                .follow(eth, gas, gas_price, campaign_id, deposit, follow_sec_key)
                .await;
            let value = match result {
                Ok(v) => Some(v),
                Err(_e) => None,
            };
            value
        })
    }

    pub fn gas_new_campaign(
        &self,
        gas: u32,
        gas_price: u128,
        campaign_data: NewCampaignData,
    ) -> Option<U256> {
        self.rt.block_on(async {
            let eth = (*self.eth.clone()).clone();
            let result = self
                .randao_contract
                .gas_new_campaign(eth, gas, gas_price, campaign_data)
                .await;
            let value = match result {
                Ok(v) => {
                    println!("gas_new_campaign hash: {:?}", v);
                    Some(v)
                }
                Err(e) => {
                    println!("call gas_new_campaign failed: {:?}", e);
                    None
                }
            };
            value
        })
    }

    pub fn contract_get_campaign_info(&self, campaign_id: u128) -> Option<CampaignInfo> {
        self.rt
            .block_on(async {
                let eth = (*self.eth.clone()).clone();
                let sec = self.config.root_secret.as_str();
                self.randao_contract
                    .get_campaign_info(eth, campaign_id, sec)
                    .await
            })
            .ok()
    }

    pub fn contract_campaign_num(&self) -> Option<U256> {
        self.rt.block_on(async {
            let eth = (*self.eth.clone()).clone();
            let result = self.randao_contract.campaign_num(eth).await;
            let value = match result {
                Ok(v) => Some(v),
                Err(e) => {
                    println!("call contract failed: {:?}", e);
                    None
                }
            };
            value
        })
    }

    pub fn contract_sha_commit(&self, _s: &str) -> Option<Vec<u8>> {
        self.rt.block_on(async {
            let eth = (*self.eth.clone()).clone();
            let result = self.randao_contract.sha_commit(eth, _s).await;
            let value = match result {
                Ok(v) => Some(v),
                Err(e) => {
                    println!("call contract_sha_commit failed: {:?}", e);
                    None
                }
            };
            value
        })
    }

    pub fn contract_commit(
        &self,
        campaign_id: u128,
        deposit: u128,
        commit_sec_key: &str,
        _hs: Vec<u8>,
    ) -> Option<TransactionReceipt> {
        self.rt.block_on(async {
            let eth = (*self.eth.clone()).clone();
            let result = self
                .randao_contract
                .commit(eth, campaign_id, deposit, commit_sec_key, _hs)
                .await;
            let value = match result {
                Ok(v) => {
                    if v.status.unwrap() == U64::zero() {
                        println!("commit receipt:{:?}", v);
                    }
                    Some(v)
                }
                Err(e) => {
                    println!("call contract_commit failed: {:?}", e);
                    None
                }
            };
            value
        })
    }

    pub fn contract_reveal(
        &self,
        campaign_id: u128,
        _deposit: u128,
        commit_sec_key: &str,
        _s: &str,
    ) -> Option<TransactionReceipt> {
        self.rt.block_on(async {
            let eth = (*self.eth.clone()).clone();
            let result = self
                .randao_contract
                .reveal(eth, campaign_id, commit_sec_key, _s)
                .await;
            let value = match result {
                Ok(v) => {
                    if v.status.unwrap() == U64::zero() {
                        println!("reveal receipt:{:?}", v);
                    }
                    Some(v)
                }
                Err(e) => {
                    println!("call contract_reveal failed: {:?}", e);
                    None
                }
            };
            value
        })
    }

    pub fn contract_get_my_bounty(&self, campaign_id: u128, commit_sec_key: &str) -> Option<U256> {
        self.rt.block_on(async {
            let eth = (*self.eth.clone()).clone();
            let result = self
                .randao_contract
                .get_campaign_query(eth, "getMyBounty", campaign_id, commit_sec_key)
                .await;
            let value = match result {
                Ok(v) => Some(v),
                Err(e) => {
                    println!("call contract_get_my_bounty failed: {:?}", e);
                    None
                }
            };
            value
        })
    }

    pub fn contract_get_random(&self, campaign_id: u128, commit_sec_key: &str) -> Option<U256> {
        self.rt.block_on(async {
            let eth = (*self.eth.clone()).clone();
            let result = self
                .randao_contract
                .get_campaign_query(eth, "getRandom", campaign_id, commit_sec_key)
                .await;
            let value = match result {
                Ok(v) => Some(v),
                Err(e) => {
                    println!("call contract_reveal failed: {:?}", e);
                    None
                }
            };
            value
        })
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct DeployJsonObj {
    pub code_path: String,
    pub abi_path: String,
    pub sec_key: String,
    pub gas: u64,
    pub gas_price: u64,
    pub args: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct DeployJson {
    pub deploy_obj: Vec<DeployJsonObj>,
}
#[derive(Serialize, Deserialize, Clone)]
pub struct CallJsonObj {
    pub contract_addr: String,
    pub abi_path: String,
    pub sec_key: String,
    pub gas: u32,
    pub gas_price: u32,
    pub func_name: String,
    pub args: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct CallJson {
    pub call_obj: Vec<CallJsonObj>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct QueryJson {
    pub sec_key: String,
    pub contract_addr: String,
    pub abi_path: String,
    pub func_name: String,
    pub args: String,
}

pub fn parse_deploy_json(pat: &PathBuf) -> anyhow::Result<DeployJson> {
    let deploy_json_bytes = fs::read(pat)?;
    let deply_json_obj: DeployJson = serde_json::from_slice(deploy_json_bytes.as_slice())?;

    Ok(deply_json_obj)
}

pub fn parse_call_json(pat: &PathBuf) -> anyhow::Result<CallJson> {
    let call_json_bytes = fs::read(pat)?;
    let call_json_obj: CallJson = serde_json::from_slice(call_json_bytes.as_slice())?;

    Ok(call_json_obj)
}

pub fn parse_query_json(pat: &PathBuf) -> anyhow::Result<QueryJson> {
    let query_json_bytes = fs::read(pat)?;
    let query_json_obj: QueryJson = serde_json::from_slice(query_json_bytes.as_slice())?;

    Ok(query_json_obj)
}

fn parse_args_csv(args: &str) -> anyhow::Result<Vec<Token>> {
    let mut res: Vec<Token> = Vec::new();

    let args_str = args.to_string();
    let mut csv_reader1 = csv::Reader::from_reader(args_str.as_bytes());

    if let Ok(args) = csv_reader1.headers() {
        for arg in args {
            if arg.is_empty() {
                bail!("arg format error!!!");
            } else if let Ok(arg_bool) = arg.parse::<bool>() {
                res.push(arg_bool.into_token());
            } else if let Ok(arg_uint) = arg.parse::<Uint>() {
                res.push(Token::Uint(arg_uint));
            } else if let Ok(arg_int) = arg.parse::<Int>() {
                res.push(arg_int.into_token());
            } else if let Ok(arg_address) = arg.parse::<Address>() {
                res.push(arg_address.into_token());
            } else if let Ok(arg_h160) = arg.parse::<H160>() {
                res.push(arg_h160.into_token());
            } else if let Ok(arg_h256) = arg.parse::<H256>() {
                res.push(arg_h256.into_token());
            } else if let Ok(arg_u128) = arg.parse::<U128>() {
                res.push(arg_u128.into_token());
            } else if let Ok(arg_u256) = arg.parse::<U256>() {
                res.push(arg_u256.into_token());
            } else {
                let arg_string = arg.to_string();
                res.push(arg_string.into_token());
            }
        }
    }

    Ok(res)
}

async fn contract_deploy(
    eth: Eth<Http>,
    sec_key: &str,
    code_path: &str,
    abi_path: &str,
    gas: u64,
    gas_price: u64,
    args: Vec<Token>,
) -> web3::contract::Result<H160> {
    let byetcode = fs::read(code_path).unwrap();
    let abi = fs::read(abi_path).unwrap();

    let (_root_sk, _root_addr) = extract_keypair_from_str(sec_key.to_string());
    let secretkey = SecretKey2::from_str(sec_key).unwrap();
    let contract = if args.is_empty() {
        Contract::deploy(eth, &abi)?
            .confirmations(1)
            .poll_interval(time::Duration::from_millis(PULL_INTERVAL))
            .options(Options::with(|opt| {
                opt.gas = Some(gas.into());
                opt.gas_price = Some(gas_price.into());
                // opt.nonce = Some(nonce + nonce_add);
            }))
            .sign_with_key_and_execute(
                std::str::from_utf8(&byetcode).unwrap(),
                (),
                &secretkey,
                Some(2153),
            )
            .await?
    } else {
        Contract::deploy(eth, &abi)?
            .confirmations(1)
            .poll_interval(time::Duration::from_millis(PULL_INTERVAL))
            .options(Options::with(|opt| {
                opt.gas = Some(gas.into());
                opt.gas_price = Some(gas_price.into());
                // opt.nonce = Some(nonce + nonce_add);
            }))
            .sign_with_key_and_execute(
                std::str::from_utf8(&byetcode).unwrap(),
                args,
                &secretkey,
                None,
            )
            .await?
    };

    Ok(contract.address())
}

#[allow(clippy::too_many_arguments)]
async fn contract_call(
    eth: Eth<Http>,
    contr_addr: &str,
    sec_key: &str,
    // _account: &str,
    abi_path: &str,
    gas: u32,
    gas_price: u32,
    func_name: &str,
    args: Vec<Token>,
) -> web3::contract::Result<H256> {
    let abi = fs::read(abi_path).unwrap();
    let contr_addr: H160 = contr_addr.parse().unwrap();
    // let _account: H160 = _account.parse().unwrap();
    let contract = Contract::from_json(eth, contr_addr, &abi)?;
    let secretkey = SecretKey2::from_str(sec_key).unwrap();

    let opt = Options {
        gas: Some(gas.into()),
        gas_price: Some(gas_price.into()),
        ..Default::default()
    };

    let transaction_hash = if args.is_empty() {
        contract.signed_call(func_name, (), opt, &secretkey).await?
    } else {
        contract
            .signed_call(func_name, args, opt, &secretkey)
            .await?
    };

    Ok(transaction_hash)
}

async fn contract_query(
    eth: Eth<Http>,
    contr_addr: &str,
    account: &str,
    abi_path: &str,
    func_name: &str,
    args: Vec<Token>,
) -> web3::contract::Result<U128> {
    let abi = fs::read(abi_path).unwrap();
    let contr_addr: H160 = contr_addr.parse().unwrap();
    let _account: H160 = account.parse().unwrap();

    let contract = Contract::from_json(eth, contr_addr, &abi)?;
    // let _secretkey = SecretKey::from_str(_sec_key).unwrap();
    let opt = Options::default();

    let id = 3;
    let token_id: U256 = id.into();
    let result: U128 = if args.is_empty() {
        contract.query(func_name, (), _account, opt, None).await?
    } else {
        contract
            .query(func_name, token_id, _account, opt, None)
            .await?
    };

    println!("result:{:?}", result);
    let mut ret = [0; 2];
    ret[0] = 0;
    ret[1] = 0;
    Ok(U128(ret))
}

async fn multi_tasks_impl<F, T>(vf: Vec<F>) -> anyhow::Result<(u32, u128)>
where
    F: FnOnce() -> T,
    T: Future<Output = anyhow::Result<()>> + Send + 'static,
{
    let mut task_queue = Vec::with_capacity(vf.len());
    for f in vf {
        let af = f();
        let task = tokio::spawn(async move {
            CUR_TASKS.store(CUR_TASKS.load(Ordering::Acquire) + 1, Ordering::Release);

            let beg_time = get_timestamp();

            if af.await.is_ok() {
                let end_time = get_timestamp();
                update_res_queue_secs(end_time - beg_time).await;
            }
            CUR_TASKS.store(CUR_TASKS.load(Ordering::Acquire) - 1, Ordering::Release);
        });
        task_queue.push(task);

        while MAX_TASKS.load(Ordering::Acquire) <= CUR_TASKS.load(Ordering::Acquire) {
            let task = task_queue.pop().unwrap();
            task.await?;
        }
    }

    let (tx1, rx1) = tokio::sync::mpsc::channel(2);
    tokio::spawn(max_tasks_update(rx1));

    for task in task_queue {
        task.await?;
    }

    tx1.send(()).await?;
    let success_task = RES_QUEUE_SECS.lock().await.0;
    let total_times = RES_QUEUE_SECS.lock().await.1;

    anyhow::Ok((success_task, total_times))
}

async fn max_tasks_update(mut rx: Receiver<()>) {
    loop {
        let res_queue_secs = RES_QUEUE_SECS.lock().await;
        let average_time_queue = &res_queue_secs.2;
        if average_time_queue.len() > 1 {
            let end_cost_time = average_time_queue.iter().last().unwrap();
            let mut big: u8 = 0;
            let mut less: u8 = 0;

            for cost in average_time_queue.iter().rev().skip(1) {
                if end_cost_time > cost && (end_cost_time - *cost) > DELTA_RANGE {
                    big += 1;
                }
                if end_cost_time < cost && (*cost - end_cost_time) > DELTA_RANGE {
                    less += 1;
                }
            }

            match big.cmp(&less) {
                std::cmp::Ordering::Greater => {
                    MAX_TASKS.store(2 * MAX_TASKS.load(Ordering::Acquire), Ordering::Release);
                }
                std::cmp::Ordering::Less => {
                    MAX_TASKS.store(MAX_TASKS.load(Ordering::Acquire) - 1, Ordering::Release);
                }
                std::cmp::Ordering::Equal => {
                    let end_cost_time2 = average_time_queue
                        .iter()
                        .rev()
                        .skip(1)
                        .rev()
                        .last()
                        .unwrap();

                    if end_cost_time > end_cost_time2
                        && end_cost_time - end_cost_time2 > DELTA_RANGE
                    {
                        MAX_TASKS.store(2 * MAX_TASKS.load(Ordering::Acquire), Ordering::Release);
                    } else if end_cost_time < end_cost_time2
                        && end_cost_time2 - end_cost_time > DELTA_RANGE
                    {
                        MAX_TASKS.store(MAX_TASKS.load(Ordering::Acquire) - 1, Ordering::Release);
                    }
                }
            }
        }

        tokio::time::sleep(tokio::time::Duration::from_millis(UPDATE_INTERVAL)).await;
        if rx.try_recv().is_ok() {
            break;
        }
    }
}

async fn update_res_queue_secs(interval: u128) {
    let mut res_queue_secs = RES_QUEUE_SECS.lock().await;

    res_queue_secs.0 += 1;
    res_queue_secs.1 += interval;
    let aveage_time = res_queue_secs.1 / res_queue_secs.0 as u128;
    res_queue_secs.2.push(aveage_time);

    while res_queue_secs.2.len() > RES_QUEUE_MAX_LEN {
        res_queue_secs.2.pop();
    }
}

fn get_timestamp() -> u128 {
    use std::time::{SystemTime, UNIX_EPOCH};

    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(n) => n.as_millis(),
        Err(_) => panic!("SystemTime before UNIX EPOCH!"),
    }
}

#[derive(Clone, Serialize, Deserialize)]
struct TaskStatus {
    step: u8,
    hs: Vec<u8>,
    randao_num: U256,
    _s: String,
}

pub struct WorkThd {
    uuid: String,
    campaign_id: u128,
    campaign_info: CampaignInfo,
    cli: BlockClient,
    cfg: Config,
}

impl WorkThd {
    pub fn new(
        uuid: String,
        campaign_id: u128,
        campaign_info: CampaignInfo,
        cli: &BlockClient,
        cfg: Config,
    ) -> WorkThd {
        // 1)
        WorkThd {
            uuid: uuid,
            campaign_id: campaign_id,
            campaign_info: campaign_info,
            cli: cli.clone(),
            cfg: cfg,
        }
    }

    pub fn do_task(&self) -> anyhow::Result<(String, u128, U256, U256)> {
        let block_number = self
            .cli
            .block_number()
            .ok_or(anyhow::format_err!("block_number err"))?;
        let (_, root_addr) = extract_keypair_from_str(self.cli.config.root_secret.clone());
        let balance = self.cli.balance(root_addr, Some(Number(block_number)));

        let mut task_status = TaskStatus {
            step: 0,
            hs: Vec::new(),
            randao_num: U256::zero(),
            _s: String::new(),
        };

        let status_path_str = self.uuid.clone() + ".json";
        let status_path = Path::new(&status_path_str);

        let mut status_file;
        if status_path.exists() && status_path.is_file() {
            status_file = OpenOptions::new()
                .read(true)
                .write(true)
                .open(&status_path)?;

            let mut status_str = String::new();
            status_file.read_to_string(&mut status_str)?;
            task_status = serde_json::from_str(&status_str[..])?;
        } else {
            status_file = OpenOptions::new()
                .read(true)
                .write(true)
                .create_new(true)
                .open(&status_path)?;
        }

        if task_status.step == 0 {
            // 2)
            let mut rng = rand::thread_rng();
            let mut _s = U256::zero();
            _s.0 = rng.gen::<[u64; 4]>();
            let _s = _s.to_string();

            let hs = self
                .cli
                .contract_sha_commit(&_s)
                .ok_or(anyhow::format_err!("sha_commit err"))
                .and_then(|v| {
                    info!(
                        "sha_commit succeed, campaignID={:?}, hs={:?}",
                        self.campaign_id, v
                    );
                    Ok(v)
                })
                .or_else(|e| {
                    error!(
                        "sha_commit failed, campaignID={:?}, err={:?}",
                        self.campaign_id,
                        e.to_string()
                    );
                    Err(e)
                })?;

            task_status.step = 1;
            task_status.hs = hs;
            task_status._s = _s;
            status_file.write(serde_json::to_string(&task_status)?.as_bytes())?;
            status_file.flush()?;
        }

        if task_status.step == 1 {
            let mut curr_block_number = U256::from(
                self.cli
                    .block_number()
                    .ok_or(anyhow::format_err!("block_number err"))?
                    .as_u64(),
            );
            let balkline = self.campaign_info.bnum - self.campaign_info.commit_balkline;
            let deadline = self.campaign_info.bnum - self.campaign_info.commit_deadline;
            while !(curr_block_number >= balkline && curr_block_number <= deadline) {
                utils::wait_blocks(&self.cli);
                curr_block_number = U256::from(
                    self.cli
                        .block_number()
                        .ok_or(anyhow::format_err!("block_number err"))?
                        .as_u64(),
                );
            }

            let commit_tx_receipt = self
                .cli
                .contract_commit(
                    self.campaign_id,
                    self.campaign_info.deposit.as_u128(),
                    &self.cfg.secret_key.consumer_secret,
                    task_status.hs.clone(),
                )
                .ok_or(anyhow::format_err!("commit err"))
                .and_then(|v| {
                    info!(
                        "Commit succeed, campaignID={:?}, tx={:?} gasPrice={:?}",
                        self.campaign_id, v.transaction_hash, v.gas_used
                    );
                    Ok(v)
                })
                .or_else(|e| {
                    error!(
                        "Commit failed, campaignID={:?}, err={:?}",
                        self.campaign_id,
                        e.to_string()
                    );
                    Err(e)
                })?;
            info!("commit transaction receipt :{:?}", commit_tx_receipt);

            task_status.step = 2;
            status_file.write(serde_json::to_string(&task_status)?.as_bytes())?;
            status_file.flush()?;

            ONGOING_CAMPAIGNS.inc();
        }

        // 3)
        if task_status.step == 2 {
            let mut curr_block_number = U256::from(
                self.cli
                    .block_number()
                    .ok_or(anyhow::format_err!("block_number err"))?
                    .as_u64(),
            );
            let deadline = self.campaign_info.bnum - self.campaign_info.commit_deadline;
            let bnum = self.campaign_info.bnum;
            while !(curr_block_number > deadline && curr_block_number < bnum) {
                utils::wait_blocks(&self.cli);
                curr_block_number = U256::from(
                    self.cli
                        .block_number()
                        .ok_or(anyhow::format_err!("block_number err"))?
                        .as_u64(),
                );
            }

            let reveal_tx_receipt = self
                .cli
                .contract_reveal(
                    self.campaign_id,
                    self.campaign_info.deposit.as_u128(),
                    &self.cfg.secret_key.consumer_secret,
                    task_status._s.as_str(),
                )
                .ok_or(anyhow::format_err!("reveal err"))
                .and_then(|v| {
                    info!(
                        "Reveal succeed, campaignID={:?}, tx={:?} gasPrice={:?}",
                        self.campaign_id, v.transaction_hash, v.gas_used
                    );
                    Ok(v)
                })
                .or_else(|e| {
                    error!(
                        "Reveal failed, fines={:?}, campaignID={:?}, err={:?}",
                        self.campaign_id,
                        3,
                        e.to_string()
                    );
                    Err(e)
                })?;
            info!("reveal transaction receipt :{:?}", reveal_tx_receipt);

            task_status.step = 3;
            status_file.write(serde_json::to_string(&task_status)?.as_bytes())?;
            status_file.flush()?;

            ONGOING_CAMPAIGNS.inc();
        }

        // 4)
        if task_status.step == 3 {
            let randao_num = self
                .cli
                .contract_get_random(self.campaign_id, &self.cfg.secret_key.consumer_secret)
                .ok_or(anyhow::format_err!("get_random err"))
                .and_then(|v| {
                    info!(
                        "get Random succeed, campaignID={:?}, randao num={:?}",
                        self.campaign_id, v
                    );
                    Ok(v)
                })
                .or_else(|e| {
                    error!(
                        "get Random failed, campaignID={:?}, err={:?}",
                        self.campaign_id,
                        e.to_string()
                    );
                    Err(e)
                })?;
            info!("randao_num :{:?}", randao_num);
            task_status.step = 4;
            task_status.randao_num = randao_num;

            status_file.write(serde_json::to_string(&task_status)?.as_bytes())?;
            status_file.flush()?;
        }

        if task_status.step == 4 {
            let mut curr_block_number = U256::from(
                self.cli
                    .block_number()
                    .ok_or(anyhow::format_err!("block_number err"))?
                    .as_u64(),
            );
            let bnum = self.campaign_info.bnum;
            println!("--------------bnum: {:?}--------------", bnum);
            while !(curr_block_number >= bnum) {
                utils::wait_blocks(&self.cli);
                curr_block_number = U256::from(
                    self.cli
                        .block_number()
                        .ok_or(anyhow::format_err!("block_number err"))?
                        .as_u64(),
                );
            }

            let my_bounty = self
                .cli
                .contract_get_my_bounty(self.campaign_id, &self.cfg.secret_key.consumer_secret)
                .ok_or(anyhow::format_err!("get_my_bounty err"))
                .and_then(|v| {
                    info!(
                        "Bounty claimed, campaignID={:?}, bounty={:?}",
                        self.campaign_id, v
                    );
                    Ok(v)
                })
                .or_else(|e| {
                    error!(
                        "Get bounty failed, campaignID={:?}, err={:?}",
                        self.campaign_id,
                        e.to_string()
                    );
                    Err(e)
                })?;
            info!("my_bounty :{:?}", my_bounty);

            task_status.step = 5;
            status_file.write(serde_json::to_string(&task_status)?.as_bytes())?;
            status_file.flush()?;
            std::mem::drop(status_file);

            fs::remove_file(&status_path)?;

            if my_bounty <= balance {
                anyhow::bail!("my_bounty less than balance")
            }

            ONGOING_CAMPAIGNS.dec();

            return Ok((
                self.uuid.clone(),
                self.campaign_id,
                task_status.randao_num,
                my_bounty,
            ));
        }

        anyhow::bail!("task status step error!!!")
    }
}
