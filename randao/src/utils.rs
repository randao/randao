use crate::config::Config;
use crate::RANDAO_PATH;
use crate::{BlockClient, CampaignInfo, U256};

use sha3::{Digest, Keccak256};
use std::str::FromStr;
use std::thread::sleep;
use std::time::Duration;
use web3::contract::Error;
use web3::types::BlockNumber::Number;
use web3::types::{Address, H256};

use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::Path;
use uuid::Uuid;

#[inline(always)]
pub fn extract_keypair_from_config(config: &Config) -> (secp256k1::SecretKey, Address) {
    let sk_str = config.chain.participant.clone();
    let _root_sk = secp256k1::SecretKey::from_str(sk_str.trim()).unwrap();
    let s = secp256k1::Secp256k1::signing_only();
    let root_pk = secp256k1::PublicKey::from_secret_key(&s, &_root_sk);
    let mut res = [0u8; 64];
    res.copy_from_slice(&root_pk.serialize_uncompressed()[1..65]);
    let root_addr = Address::from(H256::from_slice(Keccak256::digest(&res).as_slice()));
    (_root_sk, root_addr)
}

#[inline(always)]
pub fn extract_keypair_from_str(sk_str: String) -> (secp256k1::SecretKey, Address) {
    let _root_sk = secp256k1::SecretKey::from_str(sk_str.trim()).unwrap();
    let s = secp256k1::Secp256k1::signing_only();
    let root_pk = secp256k1::PublicKey::from_secret_key(&s, &_root_sk);
    let mut res = [0u8; 64];
    res.copy_from_slice(&root_pk.serialize_uncompressed()[1..65]);
    let root_addr = Address::from(H256::from_slice(Keccak256::digest(&res).as_slice()));
    (_root_sk, root_addr)
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
    let initial_block_number = client.block_number().unwrap();
    while is_running {
        let current_block_number = client.block_number().unwrap();

        if current_block_number > initial_block_number {
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
    let (_root_sk, root_addr) = extract_keypair_from_str(client.config.chain.participant.clone());
    let balance = client.balance(root_addr, Some(Number(block_number)));
    if U256::from_str(config.chain.opts.min_gas_reserve.as_str()).unwrap() >= balance {
        return false;
    }

    println!("{:?}, block_number :{:?}", campaign_info, block_number);

    let num = campaign_info.bountypot.as_u128()
        / (campaign_info.deposit.as_u128() / (campaign_info.commit_num.as_u128() + 1));

    let wei = u128::try_from(campaign_info.deposit).unwrap();
    let eth = wei as f64 / 1_000_000_000_000_000_000f64;

    if config.chain.opts.max_deposit as f64 > eth
        && config.chain.opts.min_rate_of_return <= num as f32
        && campaign_info.bnum - campaign_info.commit_balkline > U256::from(block_number.as_u64())
        && campaign_info.commit_deadline > U256::from(config.chain.opts.min_reveal_window)
        && config.chain.opts.min_reveal_window > config.chain.opts.max_reveal_delay
    {
        return true;
    }
    false
}

pub fn store_uuid(uuid: &Uuid) -> Result<(), std::io::Error> {
    fs::create_dir_all(RANDAO_PATH)?;
    let path = RANDAO_PATH.to_string() + "uuids.txt";
    let path = Path::new(&(path));
    if !path.exists() {
        File::create(path)?;
    }
    let mut file = OpenOptions::new().append(true).open(&path)?;

    writeln!(file, "{}", uuid)?;
    Ok(())
}

pub fn remove_uuid(uuid: &Uuid) -> Result<(), std::io::Error> {
    let mut uuids = read_uuids().unwrap();
    let path = RANDAO_PATH.to_string() + "uuids.txt";
    uuids.retain(|u| u != uuid);
    let mut file = OpenOptions::new().write(true).truncate(true).open(&path)?;
    for uuid in uuids {
        writeln!(file, "{}", uuid)?;
    }
    Ok(())
}

pub fn read_uuids() -> Result<Vec<Uuid>, std::io::Error> {
    let path = RANDAO_PATH.to_string() + "uuids.txt";
    let mut file = File::open(&path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    if contents.len() == 0 {
        return Ok(Vec::new());
    }
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

pub fn delete_all_uuids() -> Result<(), std::io::Error> {
    let path = RANDAO_PATH.to_string() + "uuids.txt";
    let mut file = OpenOptions::new().write(true).truncate(true).open(&path)?;
    file.write_all(b"")?;
    fs::remove_file(path)?;
    fs::remove_dir_all(RANDAO_PATH)?;
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
    let _ = delete_all_uuids();
}
