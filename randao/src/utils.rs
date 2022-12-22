use crate::config::Config;
use crate::{BlockClient, CampaignInfo, U256};
use sha3::{Digest, Keccak256};
use std::str::FromStr;
use std::thread::sleep;
use std::time::Duration;
use url::Url;
use web3::contract::Error;
use web3::types::BlockNumber::Number;
use web3::types::{Address, H256};

pub fn log_cpus() -> u64 {
    num_cpus::get() as u64
}

pub fn phy_cpus() -> u64 {
    num_cpus::get_physical() as u64
}

pub fn real_network(network: &str) -> Vec<Option<String>> {
    match network {
        "local" => vec![Some("http://localhost:8545".to_string())],
        "anvil" => vec![Some(
            "https://prod-testnet.prod.findora.org:8545".to_string(),
        )],
        "main" => vec![Some(
            "https://prod-mainnet.prod.findora.org:8545".to_string(),
        )],
        "mock" => vec![Some(
            "https://dev-mainnetmock.dev.findora.org:8545".to_string(),
        )],
        "test" => vec![Some("http://34.211.109.216:8545".to_string())],
        "qa01" => vec![Some("https://dev-qa01.dev.findora.org:8545".to_string())],
        "qa02" => vec![Some("https://dev-qa02.dev.findora.org:8545".to_string())],
        n => {
            // comma seperated network endpoints
            n.split(',')
                .filter_map(|s| {
                    let ns = s.trim();
                    if ns.is_empty() || Url::parse(ns).is_err() {
                        None
                    } else {
                        Some(Some(ns.to_string()))
                    }
                })
                .collect::<Vec<_>>()
        }
    }
}

#[inline(always)]
pub fn extract_keypair_from_config(config: &Config) -> (secp256k1::SecretKey, Address) {
    let sk_str = config.root_secret.clone();
    let root_sk = secp256k1::SecretKey::from_str(sk_str.trim()).unwrap();
    let s = secp256k1::Secp256k1::signing_only();
    let root_pk = secp256k1::PublicKey::from_secret_key(&s, &root_sk);
    let mut res = [0u8; 64];
    res.copy_from_slice(&root_pk.serialize_uncompressed()[1..65]);
    let root_addr = Address::from(H256::from_slice(Keccak256::digest(&res).as_slice()));
    (root_sk, root_addr)
}

#[inline(always)]
pub fn extract_keypair_from_str(sk_str: String) -> (secp256k1::SecretKey, Address) {
    let root_sk = secp256k1::SecretKey::from_str(sk_str.trim()).unwrap();
    let s = secp256k1::Secp256k1::signing_only();
    let root_pk = secp256k1::PublicKey::from_secret_key(&s, &root_sk);
    let mut res = [0u8; 64];
    res.copy_from_slice(&root_pk.serialize_uncompressed()[1..65]);
    let root_addr = Address::from(H256::from_slice(Keccak256::digest(&res).as_slice()));
    (root_sk, root_addr)
}

pub fn check_parallel_args(max_par: u64) {
    if max_par > log_cpus() * 1000 {
        panic!(
            "Two much working thread, maybe overload the system {}/{}",
            max_par,
            log_cpus(),
        )
    }
    if max_par == 0 {
        panic!("Invalid parallel parameters: max {}", max_par);
    }
}

pub fn calc_pool_size(keys: usize, max_par: usize) -> usize {
    let mut max_pool_size = keys * 2;
    if max_pool_size > max_par {
        max_pool_size = max_par;
    }
    max_pool_size
}

#[inline(always)]
pub fn handle_error(error: Error) -> String {
    match error {
        Error::InvalidOutputType(s) => format!("Invalid output type: {}", s),
        Error::Abi(e) => format!("Abi error: {}", e),
        Error::Api(e) => format!("Api error: {}", e),
        Error::Deployment(e) => format!("Deployment error: {}", e),
        Error::InterfaceUnsupported => "Contract does not support this interface.".to_string(),
    }
}

pub fn wait_blocks(client: &BlockClient) {
    let mut isRunning = true;
    let initialBlockNumber = client.block_number().unwrap();
    while isRunning {
        let currentBlockNumber = client.block_number().unwrap();

        if currentBlockNumber > initialBlockNumber {
            isRunning = false;
        }
        sleep(Duration::from_millis(500));
    }
}

pub fn check_campaign_info(
    client: &BlockClient,
    campaign_info: &CampaignInfo,
    config: &Config,
) -> bool {
    let block_number = client.block_number().unwrap();
    let (root_sk, root_addr) = extract_keypair_from_str(client.config.root_secret.clone());
    let balance = client.balance(root_addr, Some(Number(block_number)));
    if U256::from_str(config.chain.opts.minGasReserve.as_str()).unwrap() >= balance {
        return false;
    }

    let mut num = (campaign_info.bountypot.as_u128()
        / (campaign_info.deposit.as_u128() / (campaign_info.commitNum.as_u128() + 1)));
    if config.chain.opts.maxDeposit > i32::try_from(campaign_info.deposit).unwrap()
        && config.chain.opts.minRateOfReturn <= num as f32
        && campaign_info.bnum - campaign_info.commitBalkline > U256::from(block_number.as_u64())
        && campaign_info.commitDeadline > U256::from(config.chain.opts.minRevealWindow)
        && config.chain.opts.minRevealWindow > config.chain.opts.maxRevealDelay
    {
        return true;
    }
    false
}
