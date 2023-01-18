use lazy_static::lazy_static;
use secp256k1::SecretKey as SecretKey2;
use std::str::FromStr;
use web3::contract::tokens::Tokenize;
use web3::futures::lock::Mutex;
use web3::types::H256;
use web3::{
    self,
    api::Eth,
    contract::{tokens::Tokenizable, Contract, Options},
    ethabi::Token,
    transports::Http,
    types::{BlockNumber, TransactionReceipt, H160, U256},
};

lazy_static! {
    pub static ref PRIKEY_CONTRACT_LOCK: Mutex<()> = Mutex::new(());
}

//use crate::utils::{extract_keypair_from_config, handle_error};
use crate::{extract_keypair_from_str, handle_error, CampaignInfo};

#[derive(Debug, Clone, Default)]
pub struct NewCampaignData {
    pub bnum: U256,
    pub deposit: U256,
    pub commit_balkline: U256,
    pub commit_deadline: U256,
}

impl Tokenize for NewCampaignData {
    fn into_tokens(self) -> Vec<Token> {
        let mut res: Vec<Token> = Vec::new();
        res.push(self.bnum.into_token());
        res.push(self.deposit.into_token());
        res.push(self.commit_balkline.into_token());
        res.push(self.commit_deadline.into_token());
        res
    }
}

#[derive(Debug, Clone, Default)]
pub struct CommitData {
    pub campaign_id: u128,
    pub hs: Vec<u8>,
}

impl Tokenize for CommitData {
    fn into_tokens(self) -> Vec<Token> {
        let mut res: Vec<Token> = Vec::new();
        res.push(self.campaign_id.into_token());
        res.push(Token::FixedBytes(self.hs));
        res
    }
}

#[derive(Debug, Clone, Default)]
pub struct RevealData {
    pub campaign_id: u128,
    pub s: U256,
}

impl Tokenize for RevealData {
    fn into_tokens(self) -> Vec<Token> {
        let mut res: Vec<Token> = Vec::new();
        res.push(self.campaign_id.into_token());
        res.push(self.s.into_token());
        res
    }
}

#[derive(Debug, Clone, Default)]
pub struct RandaoContract {
    pub sec_key: String,
    pub contract_addr: String,
    pub abi_content: String,
    pub gas: u32,
    pub gas_price: u128,
}

impl RandaoContract {
    pub async fn get_campaign_info(
        &self,
        eth: Eth<Http>,
        campaign_id: u128,
        sec_key: &str,
    ) -> web3::contract::Result<CampaignInfo> {
        let (_root_sk, root_addr) = extract_keypair_from_str(sec_key.to_string());
        let contr_addr: H160 = self.contract_addr.parse().unwrap();

        let contract = Contract::from_json(eth, contr_addr, self.abi_content.as_bytes())?;
        let opt = Options::default();

        let token_id: U256 = campaign_id.into();
        let mut campaign_info = CampaignInfo::default();
        let rerult: std::result::Result<CampaignInfo, web3::contract::Error> = contract
            .query("getCampaign", token_id, root_addr, opt, None)
            .await;
        match rerult {
            Ok(data) => {
                campaign_info = data.clone();
            }
            Err(e) => {
                println!("get_campaign_info erro:{:?}", handle_error(e));
            }
        }
        Ok(campaign_info)
    }

    pub async fn new_campaign(
        &self,
        eth: Eth<Http>,
        gas: u32,
        gas_price: u128,
        args: NewCampaignData,
    ) -> web3::contract::Result<TransactionReceipt> {
        let contr_addr: H160 = self.contract_addr.parse().unwrap();
        let contract = Contract::from_json(eth.clone(), contr_addr, self.abi_content.as_bytes())?;
        let secretkey = SecretKey2::from_str(&self.sec_key).unwrap();

        let opt = Options {
            gas: Some(gas.into()),
            gas_price: Some(gas_price.into()),
            value: Some(args.deposit),
            ..Default::default()
        };

        let result = {
            let _guard = PRIKEY_CONTRACT_LOCK.lock().await;
            contract
                .signed_call_with_confirmations("newCampaign", args, opt, 1, &secretkey)
                .await?
        };

        println!("new_campaign ok");
        Ok(result)
    }

    pub async fn follow(
        &self,
        eth: Eth<Http>,
        gas: u32,
        gas_price: u128,
        campaign_id: u128,
        deposit: u128,
        follow_sec_key: &str,
    ) -> web3::contract::Result<H256> {
        let contr_addr: H160 = self.contract_addr.parse().unwrap();
        let contract = Contract::from_json(eth.clone(), contr_addr, self.abi_content.as_bytes())?;
        let secretkey = SecretKey2::from_str(follow_sec_key).unwrap();
        let (_root_sk, root_addr) = extract_keypair_from_str(follow_sec_key.to_string());
        let token_id: U256 = campaign_id.into();
        let opt_chech = Options::default();

        let result = contract
            .estimate_gas("follow", token_id.clone(), root_addr, opt_chech)
            .await;
        match result {
            Ok(_) => {
                let mut opt = Options {
                    gas: Some(gas.into()),
                    gas_price: Some(gas_price.into()),
                    value: Some(deposit.into()),
                    ..Default::default()
                };

                let hash = {
                    let _guard = PRIKEY_CONTRACT_LOCK.lock().await;
                    opt.nonce = Some(
                        eth.transaction_count(root_addr, Some(BlockNumber::Pending))
                            .await?,
                    );
                    contract
                        .signed_call("follow", token_id, opt, &secretkey)
                        .await?
                };
                // let result = eth
                //     .transaction_receipt(hash)
                //     .await?
                //     .expect("transaction_receipt error");
                println!("follow signed_call ok");
                Ok(hash)
            }
            Err(e) => {
                let info = self
                    .get_campaign_info(eth, campaign_id, follow_sec_key)
                    .await?;
                println!("follow erro:{:?} , and info :{:?}", handle_error(e), info);
                Err(web3::contract::Error::InterfaceUnsupported)
            }
        }
    }

    pub async fn gas_new_campaign(
        &self,
        eth: Eth<Http>,
        _gas: u32,
        _gas_price: u128,
        args: NewCampaignData,
    ) -> web3::contract::Result<U256> {
        let contr_addr: H160 = self.contract_addr.parse().unwrap();
        let contract = Contract::from_json(eth, contr_addr, self.abi_content.as_bytes())?;
        let (_root_sk, root_addr) = extract_keypair_from_str(self.sec_key.to_string());

        let opt = Options::default();
        //(args.bnum, args.deposit,args.commitBalkline, args.commitDeadline),
        let result = contract
            .estimate_gas("newCampaign", args, root_addr, opt)
            .await?;
        Ok(result)
    }

    pub async fn campaign_num(&self, eth: Eth<Http>) -> web3::contract::Result<U256> {
        let (_root_sk, root_addr) = extract_keypair_from_str(self.sec_key.to_string());
        let contr_addr: H160 = self.contract_addr.parse().unwrap();

        let contract = Contract::from_json(eth, contr_addr, self.abi_content.as_bytes())?;
        let opt = Options::default();

        let mut campaigns_id = U256::default();
        let rerult: std::result::Result<U256, web3::contract::Error> = contract
            .query("numCampaigns", (), root_addr, opt, None)
            .await;
        match rerult {
            Ok(data) => {
                campaigns_id = data.clone();
                println!("campaign_num result: {:?}", campaigns_id);
            }
            Err(e) => {
                println!("campaign_num erro:{:?}", handle_error(e));
            }
        }
        Ok(campaigns_id)
    }

    pub async fn sha_commit(&self, eth: Eth<Http>, _s: &str) -> web3::contract::Result<Vec<u8>> {
        let (_root_sk, root_addr) = extract_keypair_from_str(self.sec_key.to_string());
        let contr_addr: H160 = self.contract_addr.parse().unwrap();

        let contract = Contract::from_json(eth, contr_addr, self.abi_content.as_bytes())?;
        let opt = Options::default();

        let token_id: U256 = U256::from_dec_str(_s).unwrap();
        let mut encode_hash = Vec::default();
        let rerult: std::result::Result<Vec<u8>, web3::contract::Error> = contract
            .query("shaCommit", token_id, root_addr, opt, None)
            .await;
        match rerult {
            Ok(data) => {
                encode_hash = data.clone();
                println!("sha_commit: {:?}", encode_hash);
            }
            Err(e) => {
                println!("sha_commit erro:{:?}", handle_error(e));
            }
        }
        Ok(encode_hash)
    }

    pub async fn commit(
        &self,
        eth: Eth<Http>,
        campaign_id: u128,
        deposit: u128,
        commit_sec_key: &str,
        _hs: Vec<u8>,
    ) -> web3::contract::Result<H256> {
        let contr_addr: H160 = self.contract_addr.parse().unwrap();
        let contract = Contract::from_json(eth.clone(), contr_addr, self.abi_content.as_bytes())?;
        let secretkey = SecretKey2::from_str(commit_sec_key).unwrap();
        let (_root_sk, root_addr) = extract_keypair_from_str(commit_sec_key.to_string());
        let opt_chech = Options {
            value: Some(deposit.into()),
            ..Default::default()
        };

        let token = CommitData {
            campaign_id: campaign_id.clone(),
            hs: _hs.clone(),
        };

        let result = contract
            .estimate_gas("commit", token.clone(), root_addr, opt_chech)
            .await;
        match result {
            Ok(_) => {
                let mut opt = Options {
                    gas: Some(self.gas.into()),
                    gas_price: Some(self.gas_price.into()),
                    value: Some(deposit.into()),
                    ..Default::default()
                };
                let hash = {
                    let _guard = PRIKEY_CONTRACT_LOCK.lock().await;
                    opt.nonce = Some(
                        eth.transaction_count(root_addr, Some(BlockNumber::Pending))
                            .await?,
                    );
                    contract
                        .signed_call("commit", token.clone(), opt, &secretkey)
                        .await?
                };
                // let result = eth
                //     .transaction_receipt(hash)
                //     .await?
                //     .expect("transaction_receipt error");
                println!("commit signed_call ok");
                Ok(hash)
            }
            Err(e) => {
                let info = self
                    .get_campaign_info(eth, campaign_id, commit_sec_key)
                    .await?;
                println!("commit erro:{:?} , and info :{:?}", handle_error(e), info);
                Err(web3::contract::Error::InterfaceUnsupported)
            }
        }
    }

    pub async fn reveal(
        &self,
        eth: Eth<Http>,
        campaign_id: u128,
        commit_sec_key: &str,
        _s: &str,
    ) -> web3::contract::Result<H256> {
        let contr_addr: H160 = self.contract_addr.parse().unwrap();
        let contract = Contract::from_json(eth.clone(), contr_addr, self.abi_content.as_bytes())?;
        let secretkey = SecretKey2::from_str(commit_sec_key).unwrap();
        let (_root_sk, root_addr) = extract_keypair_from_str(commit_sec_key.to_string());
        let mut opt = Options {
            gas: Some(self.gas.into()),
            gas_price: Some(self.gas_price.into()),
            ..Default::default()
        };
        let token = RevealData {
            campaign_id,
            s: U256::from_dec_str(_s).unwrap(),
        };

        let result = contract
            .estimate_gas("reveal", token.clone(), root_addr, opt.clone())
            .await;
        match result {
            Ok(_) => {
                let hash = {
                    let _guard = PRIKEY_CONTRACT_LOCK.lock().await;
                    opt.nonce = Some(
                        eth.transaction_count(root_addr, Some(BlockNumber::Pending))
                            .await?,
                    );
                    contract
                        .signed_call("reveal", token.clone(), opt, &secretkey)
                        .await?
                };
                // let result = eth
                //     .transaction_receipt(hash)
                //     .await?
                //     .expect("transaction_receipt error");
                println!("reveal signed_call ok");
                Ok(hash)
            }
            Err(e) => {
                let info = self
                    .get_campaign_info(eth, campaign_id, commit_sec_key)
                    .await?;
                println!("reveal erro:{:?} , and info :{:?}", handle_error(e), info);
                Err(web3::contract::Error::InterfaceUnsupported)
            }
        }
    }

    pub async fn refund_bounty(
        &self,
        eth: Eth<Http>,
        campaign_id: u128,
        sec_key: &str,
    ) -> web3::contract::Result<H256> {
        let contr_addr: H160 = self.contract_addr.parse().unwrap();
        let contract = Contract::from_json(eth.clone(), contr_addr, self.abi_content.as_bytes())?;
        let secretkey = SecretKey2::from_str(sec_key).unwrap();
        let (_root_sk, root_addr) = extract_keypair_from_str(sec_key.to_string());
        let mut opt = Options {
            gas: Some(self.gas.into()),
            gas_price: Some(self.gas_price.into()),
            ..Default::default()
        };
        let token = campaign_id.into_token();

        let result = contract
            .estimate_gas("refundBounty", token.clone(), root_addr, opt.clone())
            .await;
        match result {
            Ok(_) => {
                let hash = {
                    let _guard = PRIKEY_CONTRACT_LOCK.lock().await;
                    opt.nonce = Some(
                        eth.transaction_count(root_addr, Some(BlockNumber::Pending))
                            .await?,
                    );
                    contract
                        .signed_call("refundBounty", token.clone(), opt, &secretkey)
                        .await?
                };
                // let result = eth
                //     .transaction_receipt(hash)
                //     .await?
                //     .expect("transaction_receipt error");
                println!("refundBounty signed_call ok");
                Ok(hash)
            }
            Err(e) => {
                let info = self.get_campaign_info(eth, campaign_id, sec_key).await?;
                println!(
                    "refundBounty erro:{:?} , and info :{:?}",
                    handle_error(e),
                    info
                );
                Err(web3::contract::Error::InterfaceUnsupported)
            }
        }
    }

    pub async fn get_campaign_query(
        &self,
        eth: Eth<Http>,
        fun_name: &str,
        campaign_id: u128,
        sec_key: &str,
    ) -> web3::contract::Result<U256> {
        let contr_addr: H160 = self.contract_addr.parse().unwrap();
        let contract = Contract::from_json(eth.clone(), contr_addr, self.abi_content.as_bytes())?;
        let (_root_sk, root_addr) = extract_keypair_from_str(sec_key.to_string());
        let opt = Options {
            gas: Some(self.gas.into()),
            gas_price: Some(self.gas_price.into()),
            ..Default::default()
        };
        let token: Token = campaign_id.into_token();
        let result = contract
            .estimate_gas(fun_name, token.clone(), root_addr, opt.clone())
            .await;
        match result {
            Ok(_) => {
                let result = contract
                    .query(fun_name, token.clone(), root_addr, opt, None)
                    .await?;
                println!("get_campaign_query ok");
                Ok(result)
            }
            Err(e) => {
                let info = self.get_campaign_info(eth, campaign_id, sec_key).await?;
                println!(
                    "get_campaign_query erro:{:?} , and info :{:?}",
                    handle_error(e),
                    info
                );
                Err(web3::contract::Error::InterfaceUnsupported)
            }
        }
    }
}
