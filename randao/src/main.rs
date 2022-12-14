#[macro_use]
extern crate serde;

mod config;
use std::borrow::Borrow;
use tokio::time::timeout;
mod commands;
mod db;
mod profiler;

use std::{
    thread,
    cell::RefCell,
    cmp::Ordering,
    ops::{Mul, MulAssign, Sub},
    path::PathBuf,
    str::FromStr,
    sync::{
        atomic::{AtomicU64, Ordering::Relaxed},
        mpsc, Arc,
    },
    time::Duration,
};

use commands::*;
use randao::{one_eth_key, parse_call_json, parse_deploy_json, parse_query_json,
             utils::*, KeyPair, BlockClient, config::*, DeployJson, DeployJsonObj};
use log::{debug, error, info};
use rayon::prelude::*;
use web3::types::{Address, Block, BlockId, BlockNumber, TransactionId, H256, U256, U64};
use clap::Parser;

fn main() -> std::io::Result<()> {
    let opt = Opts::parse();
    let config:Config = Config::parse_from_file(&opt.config);
    let (tx, rx) = std::sync::mpsc::channel();
    let child_thread = thread::spawn(move || {
        loop {
            // 等待接收主线程计算完成的信号
            println!("子线程计算：1 + 1 = {}", sum);
        }
    });

    loop {
        let client = BlockClient::setup(&config, None);
        let chain_id = client.chain_id();
        println!("chain_id:{:?}",chain_id );
        let DeployJsonObj = DeployJsonObj{
            code_path : "Randao_sol_Randao.bin".to_string(),
            abi_path: "Randao_sol_Randao.abi".to_string(),
            sec_key : config.secret.clone(),
            gas:1000000,
            gas_price:10000000000,
            args: "".to_string(),
        };
        let obj_vec = vec![DeployJsonObj];

        let ob_json  = DeployJson{
            deploy_obj:obj_vec,
        };
        let result= client.contract_deploy(ob_json);
        thread::sleep(Duration::from_millis(20000));
        // 计算1+1
        let sum = 1 + 1;
        // 向子线程发送计算完成的信号
        tx.send(sum).unwrap();
    }

    child_thread.join().unwrap();
    Ok(())
}