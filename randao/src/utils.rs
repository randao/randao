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

use std::fs::{OpenOptions, File};
use std::io::{Read, Seek, Write};
use std::sync::RwLock;
use std::sync::Mutex;
use lazy_static::lazy_static;
use uuid::Uuid;
use std::path::Path;

pub fn log_cpus() -> u64 {
    num_cpus::get() as u64
}

pub fn phy_cpus() -> u64 {
    num_cpus::get_physical() as u64
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
    let mut is_running = true;
    let initialBlockNumber = client.block_number().unwrap();
    while is_running {
        let currentBlockNumber = client.block_number().unwrap();

        if currentBlockNumber > initialBlockNumber {
            is_running = false;
        }
        sleep(Duration::from_millis(500));
    }
}

#[warn(unused_parens)]
pub fn check_campaign_info(
    client: &BlockClient,
    campaign_info: &CampaignInfo,
    config: &Config,
) -> bool {
    let block_number = client.block_number().unwrap();
    let (_root_sk, root_addr) = extract_keypair_from_str(client.config.root_secret.clone());
    let balance = client.balance(root_addr, Some(Number(block_number)));
    if U256::from_str(config.chain.opts.minGasReserve.as_str()).unwrap() >= balance {
        return false;
    }

    println!("{:?}, block_number :{:?}", campaign_info, block_number);

    let num = campaign_info.bountypot.as_u128()
        / (campaign_info.deposit.as_u128() / (campaign_info.commitNum.as_u128() + 1));

    let wei = u128::try_from(campaign_info.deposit).unwrap();
    let eth = wei as f64 / 1_000_000_000_000_000_000f64;

    if config.chain.opts.maxDeposit as f64 > eth
        && config.chain.opts.minRateOfReturn <= num as f32
        && campaign_info.bnum - campaign_info.commitBalkline > U256::from(block_number.as_u64())
        && campaign_info.commitDeadline > U256::from(config.chain.opts.minRevealWindow)
        && config.chain.opts.minRevealWindow > config.chain.opts.maxRevealDelay
    {
        return true;
    }
    false
}

fn store_uuid(uuid: &Uuid) -> Result<(), std::io::Error> {
    let path = Path::new("uuids.txt");
    if !path.exists() {
        File::create(path)?;
    }
    let mut file = OpenOptions::new()
        .append(true)
        .open("uuids.txt")?;

    write!(file, "{}\n", uuid)?;
    Ok(())
}

fn remove_uuid(uuid: &Uuid) -> Result<(), std::io::Error> {
    let mut uuids = read_uuids().unwrap();
    uuids.retain(|u| u != uuid);
    let mut file = OpenOptions::new().write(true).truncate(true).open("uuids.txt")?;
    for uuid in uuids {
        write!(file, "{}\n", uuid)?;
    }
    Ok(())
}

fn read_uuids() -> Result<Vec<Uuid>, std::io::Error> {
    let mut file = File::open("uuids.txt")?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let uuid_strings: Vec<&str> = contents.split('\n').collect();
    let mut uuids = Vec::new();
    for uuid_string in uuid_strings {
        if uuid_string.is_empty() {
            continue;
        }
        let uuid = Uuid::from_str(uuid_string).unwrap();
        uuids.push(uuid);
    }
    Ok(uuids)
}

fn delete_all_uuids() -> Result<(), std::io::Error>  {
    let path = "uuids.txt";
    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(path)?;
    file.write_all(b"")?;
    Ok(())
}
#[test]
fn test_uuid_store_and_remove() {
    let uuid1 = Uuid::new_v4();
    let uuid2 = Uuid::new_v4();
    let uuid3 = Uuid::new_v4();

    store_uuid(&uuid1).unwrap();
    store_uuid(&uuid2).unwrap();
    store_uuid(&uuid3).unwrap();

    let uuids = read_uuids().unwrap();
    assert_eq!(uuids.len(), 3);
    assert!(uuids.contains(&uuid1));
    assert!(uuids.contains(&uuid2));
    assert!(uuids.contains(&uuid3));

    remove_uuid(&uuid2).unwrap();

    let uuids = read_uuids().unwrap();
    assert_eq!(uuids.len(), 2);
    assert!(uuids.contains(&uuid1));
    assert!(!uuids.contains(&uuid2));
    assert!(uuids.contains(&uuid3));
    delete_all_uuids();
}