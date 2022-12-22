#[macro_use]
extern crate serde;

mod config;
use randao::WorkThd;
use std::borrow::Borrow;
use tokio::time::timeout;
mod commands;
mod contract;

use std::thread::sleep;
use std::{
    cell::RefCell,
    cmp::Ordering,
    ops::{Mul, MulAssign, Sub},
    path::PathBuf,
    str::FromStr,
    sync::{
        atomic::{AtomicU64, Ordering::Relaxed},
        mpsc, Arc,
    },
    thread,
    time::Duration,
};

use clap::Parser;
use commands::*;
use lazy_static::lazy_static;
use log::{debug, error, info};
use nix::{
    libc,
    sys::signal::{self, SigHandler, Signal},
};
use randao::{
    config::*, contract::*, error::Error, one_eth_key, parse_call_json, parse_deploy_json,
    parse_query_json, utils::*, BlockClient, CallJson, CallJsonObj, DeployJson, DeployJsonObj,
    KeyPair, QueryJson,
};
use rayon::prelude::*;
use std::process;
use std::sync::atomic::{AtomicBool, Ordering as Order};
use web3::types::BlockNumber::Number;
use web3::types::{Address, Block, BlockId, BlockNumber, TransactionId, H256, U256, U64};

lazy_static! {
    static ref STOP: AtomicBool = AtomicBool::new(false);
}

extern "C" fn handle_sigint(sig_no: libc::c_int) {
    info!("signal_handler has been runned {:?}", sig_no);
    STOP.store(true, Order::SeqCst);
}

fn main() {
    match run_main() {
        Ok(_) => {}
        Err(error) => {
            let e = format!("main thread err:{:?}", error);
            error!("{}", e);
        }
    }
    process::exit(0);
    //test_contract_new_campaign();
}
fn run_main() -> Result<U256, Error> {
    let handler = SigHandler::Handler(handle_sigint);
    unsafe { signal::signal(Signal::SIGTERM, handler) }.unwrap();

    let opt = Opts::parse();
    let config: Config = Config::parse_from_file(&opt.config);
    let mut client = BlockClient::setup(&config, None);
    let chain_id = client.chain_id().unwrap();
    let block = client.current_block().unwrap();

    if chain_id.to_string() != client.config.chain.chainId {
        return Err(Error::CheckChainErr);
    }
    let mut campaign_num = match client.contract_campaign_num() {
        None => {
            return Err(Error::GetNumCampaignsErr);
        }
        Some(num) => num,
    };
    info!(
        "chain_name:{:?}, chain_id:{:?}, block_num, endpoind:{:?}, campaigns_num:{:?}, randao:{:?}",
        client.config.chain.name,
        chain_id,
        client.config.chain.endpoint,
        block.number,
        client.config.chain.opts.randao
    );
    let mut handle_vec = Vec::new();
    while !STOP.load(Order::SeqCst) {
        let mut local_client = client.clone();
        let new_campaign_num = match local_client.contract_campaign_num() {
            None => {
                return Err(Error::GetNumCampaignsErr);
            }
            Some(num) => num,
        };

        let t = thread::spawn(move || {
            println!("1+1 = 3");
        });
        handle_vec.push(t);
        /*if new_campaign_num > campaign_num {
            let campaign_id = new_campaign_num.as_u128() - 1;
            let info = local_client.contract_get_campaign_info(campaign_id).unwrap();
            if local_client.config.chain.opts.maxCampaigns <= i32::try_from(new_campaign_num).unwrap() {
                return Err(Error::GetNumCampaignsErr);
            }
            if !check_campaign_info(&local_client, &info, &local_client.config) {
                return Err(Error::CheckCampaignsInfoErr);
            }
            let t = thread::spawn(move || {
                let work_thd = WorkThd::new(&local_client, campaign_id.clone(), info, local_client.config.clone());
                let result = work_thd.do_task().unwrap_or_else(|error| U256::from({
                    error!("do_task error: {}", error);
                    -1
                }));

                info!("campaign_id:{:?},  randao:{:?}",campaign_id,result);
            });
            handle_vec.push(t);
        }*/
        sleep(Duration::from_millis(1000));
    }
    for t in handle_vec {
        t.join().unwrap();
    }
    return Ok(U256::from(1));
}

fn test_contract_new_campaign() {
    let opt = Opts::parse();
    let config: Config = Config::parse_from_file(&opt.config);
    let mut client = BlockClient::setup(&config, None);
    let block_num = client.block_number().unwrap();
    let bnum = block_num.as_u64() + 10;
    let commitBalkline: u128 = 8;
    let commitDeadline: u128 = 4;
    let deposit: u128 = 1000000000000000000;
    //let  arg = format!("{:?},{:?},{:?},{:?}", bnum, deposit, commitBalkline, commitDeadline);
    client.contract_setup(
        &config.root_secret.clone(),
        "0x0CCe486D83bA3BD519BB457d746fc6D7b3a6a620",
        "Randao_sol_Randao.abi",
        10000000,
        10000000000,
    );
    let new_data = NewCampaignData {
        bnum: bnum.into(),
        deposit: deposit.into(),
        commitBalkline: commitBalkline.into(),
        commitDeadline: commitDeadline.into(),
    };
    client.contract_new_campaign(1000000, 10000000000, new_data);
    let campaign_id = client.contract_campaign_num().unwrap();
    let campaign = campaign_id.as_u128() - 1;
    client.contract_follow(
        1000000,
        10000000000,
        campaign,
        deposit,
        &config.secret_key.follower_secret,
    );
    for i in 0..1 {
        wait_blocks(&client);
    }
    let _s = "131242344353464564564574574567456";
    let hs = client.contract_sha_commit(_s.clone()).unwrap();
    client.contract_commit(campaign, deposit, &config.secret_key.consumer_secret, hs);
    for i in 0..1 {
        wait_blocks(&client);
    }
    client.contract_reveal(campaign, deposit, &config.secret_key.consumer_secret, _s);
    let info = client.contract_get_campaign_info(campaign).unwrap();
    println!("campaign info :{:?}", info);
    for i in 0..1 {
        wait_blocks(&client);
    }
    let randao_num = client
        .contract_get_random(campaign, &config.secret_key.consumer_secret)
        .unwrap();
    println!("randao_num :{:?}", randao_num);
    let my_bounty = client
        .contract_get_my_bounty(campaign, &config.secret_key.consumer_secret)
        .unwrap();
    println!("my_bounty :{:?}", my_bounty);

    let work_thd = WorkThd::new(&client, campaign, info, config);
    work_thd.do_task().unwrap();
}
