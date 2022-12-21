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
use log::{debug, error, info};
use randao::{
    config::*, contract::*, error::Error, one_eth_key, parse_call_json, parse_deploy_json,
    parse_query_json, utils::*, BlockClient, CallJson, CallJsonObj, DeployJson, DeployJsonObj,
    KeyPair, QueryJson,
};
use rayon::prelude::*;
use web3::types::BlockNumber::Number;
use web3::types::{Address, Block, BlockId, BlockNumber, TransactionId, H256, U256, U64};

fn main() -> std::io::Result<()> {
    let opt = Opts::parse();
    let config: Config = Config::parse_from_file(&opt.config);
    let mut client = BlockClient::setup(&config, None);
    /*let DeployJsonObj = DeployJsonObj{
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
    let result= client.contract_deploy(ob_json);*/

    let (tx, rx) = std::sync::mpsc::channel();
    let child_thread = thread::spawn(move || {
        loop {
            let sum = rx.recv().unwrap();
            // 等待接收主线程计算完成的信号
            println!("子线程计算：1 + 1 = {}", sum);
        }
    });
    run_main(&client);

    loop {
        let chain_id = client.chain_id();
        let call_data = CallJsonObj {
            contract_addr: "0x81A1F0EaAe2a930B3CE1477e67500db7C6cA5719".to_string(),
            abi_path: "Randao_sol_Randao.abi".to_string(),
            sec_key: config.secret.clone(),
            gas: 100000,
            gas_price: 1000000,
            func_name: "newCampaign".to_string(),
            args: "newCampaign".to_string(),
        };
        test_contract_new_campaign();
        /* let num = 3u128;
        let query_data = QueryJson{
            sec_key : config.secret.clone(),
            contract_addr:"0x81A1F0EaAe2a930B3CE1477e67500db7C6cA5719".to_string(),
            abi_path : "Randao_sol_Randao.abi".to_string(),
            func_name: "multiply".to_string(),
            args: num.to_string()
        };

        client.contract_query(query_data);
        client.contract_get_campaign_query(1);*/

        // 计算1+1
        let sum = 1 + 1;
        // 向子线程发送计算完成的信号
        tx.send(sum).unwrap();
        thread::sleep(Duration::from_millis(20000));
    }

    child_thread.join().unwrap();
    Ok(())
}
fn run_main(client: &BlockClient) -> Result<U256, Error> {
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

    loop {
        let new_campaign_num = match client.contract_campaign_num() {
            None => {
                return Err(Error::GetNumCampaignsErr);
            }
            Some(num) => num,
        };
        if new_campaign_num > campaign_num {
            let campaign_id = new_campaign_num.as_u128() - 1;
            let info = client.contract_get_campaign_info(campaign_id).unwrap();
            if client.config.chain.opts.maxCampaigns <= i32::try_from(new_campaign_num).unwrap() {
                return Err(Error::GetNumCampaignsErr);
            }
            if !check_campaign_info(client, &info, &client.config) {
                return Err(Error::CheckCampaignsInfoErr);
            }

            //new_thread();
        }
        sleep(Duration::from_millis(500));
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
        &config.secret.clone(),
        "0x81A1F0EaAe2a930B3CE1477e67500db7C6cA5719",
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

    let work_thd = WorkThd::new(client, campaign, info, config);
    work_thd.do_task().unwrap();
}
