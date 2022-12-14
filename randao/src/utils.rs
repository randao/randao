use sha3::{Digest, Keccak256};
use std::{path::Path, str::FromStr};
use url::Url;
use web3::types::{Address, H256};
use crate::config::Config;
use std::fmt;

pub fn log_cpus() -> u64 {
    num_cpus::get() as u64
}

pub fn phy_cpus() -> u64 {
    num_cpus::get_physical() as u64
}

pub fn real_network(network: &str) -> Vec<Option<String>> {
    match network {
        "local" => vec![Some("http://localhost:8545".to_string())],
        "anvil" => vec![Some("https://prod-testnet.prod.findora.org:8545".to_string())],
        "main" => vec![Some("https://prod-mainnet.prod.findora.org:8545".to_string())],
        "mock" => vec![Some("https://dev-mainnetmock.dev.findora.org:8545".to_string())],
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
pub fn extract_keypair_from_config(config: &Config) -> (secp256k1::SecretKey, Address)
{
    let sk_str = config.secret.clone();
    let root_sk = secp256k1::SecretKey::from_str(sk_str.trim()).unwrap();
    let s = secp256k1::Secp256k1::signing_only();
    let root_pk = secp256k1::PublicKey::from_secret_key(&s, &root_sk);
    let mut res = [0u8; 64];
    res.copy_from_slice(&root_pk.serialize_uncompressed()[1..65]);
    let root_addr = Address::from(H256::from_slice(Keccak256::digest(&res).as_slice()));
    println!("root_addr :{:x}", root_addr);
    (root_sk, root_addr)
}

#[inline(always)]
pub fn extract_keypair_from_str(sk_str: String) -> (secp256k1::SecretKey, Address)
{
    let root_sk = secp256k1::SecretKey::from_str(sk_str.trim()).unwrap();
    let s = secp256k1::Secp256k1::signing_only();
    let root_pk = secp256k1::PublicKey::from_secret_key(&s, &root_sk);
    let mut res = [0u8; 64];
    res.copy_from_slice(&root_pk.serialize_uncompressed()[1..65]);
    let root_addr = Address::from(H256::from_slice(Keccak256::digest(&res).as_slice()));
    println!("root_addr :{:x}", root_addr);
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
