pub mod config;
pub mod contract;
pub mod error;
pub mod utils;

use crate::{
    config::CampaignInfo,
    contract::{NewCampaignData, RandaoContract},
    error::{Error, InternalError},
    utils::{extract_keypair_from_config, extract_keypair_from_str, handle_error},
};
use bip0039::{Count, Language, Mnemonic};
use bip32::{DerivationPath, XPrv};
use lazy_static::lazy_static;
use libsecp256k1::{PublicKey, SecretKey};
use log::{error, info, warn};
use rand::Rng;
use reqwest::{Client, Url};
use std::{
    fs::OpenOptions,
    io::{Read, Seek, Write},
    path::Path,
};

pub use config::Config;
use prometheus::{labels, opts};
use prometheus::{register_gauge, Gauge};
use serde::{Deserialize, Serialize};
use sha3::{Digest, Keccak256};
use std::{
    error::Error as StdError,
    fs,
    str::FromStr,
    sync::{atomic::AtomicU32, Arc},
    time::Duration,
};
use tokio::{runtime::Runtime, sync::Mutex};
use web3::{
    self,
    transports::Http,
    types::{
        Address, Block, BlockId, BlockNumber, Bytes, Transaction, TransactionId,
        TransactionReceipt, H160, H256, U256, U64,
    },
};

const _FRC20_ADDRESS: u64 = 0x1000;
pub const BLOCK_TIME: u64 = 16;

pub const RANDAO_PATH: &str = "/tmp/.randao/campaigns/";
// pub const CONF_PATH: &str = "/tmp/.randao/config/config.json";
pub const KEY_PATH: &str = "/tmp/.randao/keys/";
lazy_static! {
    pub static ref CONF_PATH: std::sync::Mutex<String> =
        std::sync::Mutex::new("config.json".to_string());
}

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

    pub fn contract_setup(
        &mut self,
        sec_key: &str,
        contract_addr: &str,
        abi_content: &str,
        gas: u32,
        gas_price: u128,
    ) {
        self.randao_contract = RandaoContract {
            sec_key: sec_key.to_string(),
            contract_addr: contract_addr.to_string(),
            abi_content: abi_content.to_string(),
            gas,
            gas_price,
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
    ) -> Option<H256> {
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
                let sec = self.config.chain.participant.as_str();
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
    ) -> Option<H256> {
        self.rt.block_on(async {
            let eth = (*self.eth.clone()).clone();
            let result = self
                .randao_contract
                .commit(eth, campaign_id, deposit, commit_sec_key, _hs)
                .await;
            let value = match result {
                Ok(v) => {
                    // if v.status.unwrap() == U64::zero() {
                    //     println!("commit receipt:{:?}", v);
                    // }
                    println!("commit hash:{:?}", v);
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
    ) -> Option<H256> {
        self.rt.block_on(async {
            let eth = (*self.eth.clone()).clone();
            let result = self
                .randao_contract
                .reveal(eth, campaign_id, commit_sec_key, _s)
                .await;
            let value = match result {
                Ok(v) => {
                    // if v.status.unwrap() == U64::zero() {
                    //     println!("reveal receipt:{:?}", v);
                    // }
                    println!("reveal hash:{:?}", v);
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

    pub fn contract_refund_bounty(&self, campaign_id: u128, sec_key: &str) -> Option<H256> {
        self.rt.block_on(async {
            let eth = (*self.eth.clone()).clone();
            let result = self
                .randao_contract
                .refund_bounty(eth, campaign_id, sec_key)
                .await;
            let value = match result {
                Ok(v) => {
                    // if v.status.unwrap() == U64::zero() {
                    //     println!("contract_refund_bounty receipt:{:?}", v);
                    // }
                    println!("contract_refund_bounty hash:{:?}", v);
                    Some(v)
                }
                Err(e) => {
                    println!("call contract_refund_bounty failed: {:?}", e);
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

    pub fn contract_comfirm_and_get_receipt(&self, hash: H256) -> Option<TransactionReceipt> {
        self.rt.block_on(async {
            let result = self.eth.transaction_receipt(hash).await;
            let value = match result {
                Ok(Some(v)) => {
                    if v.status.unwrap() == U64::zero() {
                        println!("contract_comfirm_and_get_receipt contract receipt:{:?}", v);
                    }
                    println!("contract_comfirm_and_get_receipt contract hash:{:?}", v);
                    Some(v)
                }
                Ok(None) => {
                    println!("contract_comfirm_and_get_receipt contract error");
                    None
                }
                Err(e) => {
                    println!("contract_comfirm_and_get_receipt call failed: {:?}", e);
                    None
                }
            };
            value
        })
    }
}

#[derive(Clone, Serialize, Deserialize)]
struct TaskStatus {
    step: u8,
    hs: Vec<u8>,
    randao_num: U256,
    _s: String,
    campaign_id: u128,
    campaign_info: CampaignInfo,
    tx_hash: H256,
}

pub struct WorkThd {
    campaign_id: u128,
    campaign_info: Option<CampaignInfo>,
    cli: BlockClient,
    _cfg: Config,
}

impl WorkThd {
    pub fn new(
        campaign_id: u128,
        campaign_info: CampaignInfo,
        cli: &BlockClient,
        _cfg: Config,
    ) -> WorkThd {
        // 1)
        WorkThd {
            campaign_id,
            campaign_info: Some(campaign_info),
            cli: cli.clone(),
            _cfg,
        }
    }

    pub fn new_from_campaign_id(campaign_id: u128, cli: &BlockClient, _cfg: Config) -> WorkThd {
        // 1)
        WorkThd {
            campaign_id,
            campaign_info: None,
            cli: cli.clone(),
            _cfg,
        }
    }

    pub fn do_task(&mut self) -> anyhow::Result<(u128, U256, U256)> {
        let mut task_status = TaskStatus {
            step: 0,
            hs: Vec::new(),
            randao_num: U256::zero(),
            _s: String::new(),
            campaign_id: 0u128,
            campaign_info: Default::default(),
            tx_hash: H256::zero(),
        };

        let status_path_str = RANDAO_PATH.to_string() + &self.campaign_id.to_string() + ".json";
        let status_path = Path::new(&status_path_str);

        let mut status_file;
        if status_path.exists() {
            if self.campaign_info.is_some() {
                anyhow::bail!("campaign_ids json exists, but call WorkThd::new() init!!!");
            }

            if status_path.is_file() {
                status_file = OpenOptions::new()
                    .read(true)
                    .write(true)
                    .open(&status_path)?;

                let mut status_str = String::new();
                status_file.read_to_string(&mut status_str)?;
                task_status = serde_json::from_str(&status_str[..])?;
            } else {
                anyhow::bail!("campaign_ids json file is not file!!!");
            }
        } else {
            if self.campaign_info.is_none() {
                anyhow::bail!("campaign_ids json not exists, but call WorkThd::new_from_campaign_id() init!!!");
            }

            task_status.campaign_id = self.campaign_id;
            std::mem::swap(
                &mut task_status.campaign_info,
                self.campaign_info.as_mut().unwrap(),
            );

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
                        task_status.campaign_id, v
                    );
                    Ok(v)
                })
                .or_else(|e| {
                    error!(
                        "sha_commit failed, campaignID={:?}, err={:?}",
                        task_status.campaign_id,
                        e.to_string()
                    );
                    Err(e)
                })?;

            task_status.step = 1;
            task_status.hs = hs;
            task_status._s = _s;

            status_file.rewind()?;
            status_file.write_all(serde_json::to_string(&task_status)?.as_bytes())?;
            status_file.flush()?;
        }

        if task_status.step == 1 {
            let mut curr_block_number = U256::from(
                self.cli
                    .block_number()
                    .ok_or(anyhow::format_err!("block_number err"))?
                    .as_u64(),
            );
            let balkline =
                task_status.campaign_info.bnum - task_status.campaign_info.commit_balkline;
            let deadline =
                task_status.campaign_info.bnum - task_status.campaign_info.commit_deadline;
            while !(curr_block_number >= balkline && curr_block_number <= deadline) {
                utils::wait_blocks(&self.cli);
                curr_block_number = U256::from(
                    self.cli
                        .block_number()
                        .ok_or(anyhow::format_err!("block_number err"))?
                        .as_u64(),
                );
            }

            let commit_tx_hash = self
                .cli
                .contract_commit(
                    task_status.campaign_id,
                    task_status.campaign_info.deposit.as_u128(),
                    &self.cli.randao_contract.sec_key,
                    task_status.hs.clone(),
                )
                .ok_or(anyhow::format_err!("commit err"))
                .and_then(|v| {
                    // info!(
                    //     "Commit succeed, campaignID={:?}, tx={:?} gasPrice={:?}",
                    //     task_status.campaign_id, v.transaction_hash, v.gas_used
                    // );
                    info!(
                        "Commit succeed, campaignID={:?}, tx={:?}",
                        task_status.campaign_id, v
                    );
                    Ok(v)
                })
                .or_else(|e| {
                    error!(
                        "Commit failed, campaignID={:?}, err={:?}",
                        task_status.campaign_id,
                        e.to_string()
                    );
                    Err(e)
                })?;
            info!("commit transaction hash :{:?}", commit_tx_hash);

            task_status.step = 2;
            task_status.tx_hash = commit_tx_hash;

            status_file.rewind()?;
            status_file.write_all(serde_json::to_string(&task_status)?.as_bytes())?;
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
            let deadline =
                task_status.campaign_info.bnum - task_status.campaign_info.commit_deadline;
            let bnum = task_status.campaign_info.bnum;
            while !(curr_block_number > deadline && curr_block_number < bnum) {
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
                .contract_comfirm_and_get_receipt(task_status.tx_hash)
                .ok_or(anyhow::format_err!("get_receipt error"))?;
            info!(
                "Commit succeed, campaignID={:?}, tx={:?} gasPrice={:?}",
                task_status.campaign_id,
                commit_tx_receipt.transaction_hash,
                commit_tx_receipt.gas_used
            );

            let reveal_tx_hash = self
                .cli
                .contract_reveal(
                    task_status.campaign_id,
                    task_status.campaign_info.deposit.as_u128(),
                    &self.cli.randao_contract.sec_key,
                    task_status._s.as_str(),
                )
                .ok_or(anyhow::format_err!("reveal err"))
                .and_then(|v| {
                    // info!(
                    //     "Reveal succeed, campaignID={:?}, tx={:?} gasPrice={:?}",
                    //     task_status.campaign_id, v.transaction_hash, v.gas_used
                    // );
                    info!(
                        "Reveal succeed, campaignID={:?}, tx={:?}",
                        task_status.campaign_id, v
                    );
                    Ok(v)
                })
                .or_else(|e| {
                    error!(
                        "Reveal failed, fines={:?}, campaignID={:?}, err={:?}",
                        task_status.campaign_id,
                        3,
                        e.to_string()
                    );
                    Err(e)
                })?;
            info!("reveal transaction receipt :{:?}", reveal_tx_hash);

            task_status.step = 3;
            task_status.tx_hash = reveal_tx_hash;

            status_file.rewind()?;
            status_file.write_all(serde_json::to_string(&task_status)?.as_bytes())?;
            status_file.flush()?;

            ONGOING_CAMPAIGNS.inc();
        }

        // 4)
        if task_status.step == 3 {
            let mut curr_block_number = U256::from(
                self.cli
                    .block_number()
                    .ok_or(anyhow::format_err!("block_number err"))?
                    .as_u64(),
            );
            let bnum = task_status.campaign_info.bnum;
            while !(curr_block_number >= bnum) {
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
                .contract_comfirm_and_get_receipt(task_status.tx_hash)
                .ok_or(anyhow::format_err!("get_receipt error"))?;
            info!(
                "Reveal succeed, campaignID={:?}, tx={:?} gasPrice={:?}",
                task_status.campaign_id,
                commit_tx_receipt.transaction_hash,
                commit_tx_receipt.gas_used
            );

            let randao_num = self
                .cli
                .contract_get_random(task_status.campaign_id, &self.cli.randao_contract.sec_key)
                .ok_or(anyhow::format_err!("get_random err"))
                .and_then(|v| {
                    info!(
                        "get Random succeed, campaignID={:?}, randao num={:?}",
                        task_status.campaign_id, v
                    );
                    Ok(v)
                })
                .or_else(|e| {
                    error!(
                        "get Random failed, campaignID={:?}, err={:?}",
                        task_status.campaign_id,
                        e.to_string()
                    );
                    Err(e)
                })?;
            info!("randao_num :{:?}", randao_num);
            task_status.step = 4;
            task_status.randao_num = randao_num;

            status_file.rewind()?;
            status_file.write_all(serde_json::to_string(&task_status)?.as_bytes())?;
            status_file.flush()?;
        }

        if task_status.step == 4 {
            let my_bounty = self
                .cli
                .contract_get_my_bounty(task_status.campaign_id, &self.cli.randao_contract.sec_key)
                .ok_or(anyhow::format_err!("get_my_bounty err"))
                .and_then(|v| {
                    info!(
                        "Bounty claimed, campaignID={:?}, bounty={:?}",
                        task_status.campaign_id, v
                    );
                    Ok(v)
                })
                .or_else(|e| {
                    error!(
                        "Get bounty failed, campaignID={:?}, err={:?}",
                        task_status.campaign_id,
                        e.to_string()
                    );
                    Err(e)
                })?;
            info!("my_bounty :{:?}", my_bounty);

            task_status.step = 5;

            status_file.rewind()?;
            status_file.write_all(serde_json::to_string(&task_status)?.as_bytes())?;
            status_file.flush()?;
            std::mem::drop(status_file);

            fs::remove_file(&status_path)?;

            if my_bounty < task_status.campaign_info.deposit {
                anyhow::bail!("my_bounty less than deposit");
            }

            ONGOING_CAMPAIGNS.dec();

            return Ok((task_status.campaign_id, task_status.randao_num, my_bounty));
        }

        anyhow::bail!("task status step error!!!")
    }
}
