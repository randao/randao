#[macro_use]
extern crate serde;


mod config;
use randao::WorkThd;
use uuid::Uuid;
mod commands;
mod contract;
mod api;

use std::thread::sleep;
use std::{path::PathBuf, sync::{
     Arc,
}, thread, time::Duration};

use std::sync::Mutex;
use clap::Parser;
use lazy_static::lazy_static;
use log::{error, info};
use nix::{
    libc,
};
use randao::{
    config::*, contract::*, error::Error, utils::*, BlockClient,
};
use std::process;
use std::sync::atomic::{AtomicBool, Ordering as Order};
use prometheus::{IntGauge, Registry};
use web3::types::{ U256, TransactionReceipt};
use actix_cors::Cors;
use actix_web::{middleware, web, App, HttpServer, post, HttpResponse, Responder};
use prometheus::{Encoder, TextEncoder};

use crate::api::ApiResult;

lazy_static! {
    static ref STOP: AtomicBool = AtomicBool::new(false);
}

#[derive(Clone)]
struct MainThread{
    client:Arc<Mutex<BlockClient>>,
    registry:Registry,
    work_count: Arc<Mutex<u128>>,
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(exit);
}

#[post("/exit")]
async fn exit() -> impl Responder {
    STOP.store(true, Order::SeqCst);
    return ApiResult::<i32>::new().code(400).with_msg("exit success ");
}

impl MainThread {
    pub fn set_up() -> Self{
        let opt = Opts::parse();
        let config: Config = Config::parse_from_file(&opt.config);
        let client = Arc::new(Mutex::new(BlockClient::setup(&config, None)));
        let registry = Registry::new();
        let thread_count = IntGauge::new("thread_count", "Number of threads currently running").unwrap();
        registry.register(Box::new(thread_count.clone()));
        MainThread{
            client,
            registry,
            work_count: Arc::new(Mutex::new(0))
        }
    }

    pub async fn run_http_server(&self) -> std::io::Result<()> {
        HttpServer::new(move || {
            App::new()
                .wrap(middleware::Compress::default())
                .wrap(middleware::Logger::default())
                //.wrap(Cors::default().send_wildcard())
                .wrap(Cors::permissive())
                .default_service(web::route().to(api::notfound))
                .service(web::scope("/randao").configure(init))
        })
            .keep_alive(std::time::Duration::from_secs(300))
            .bind(self.client.lock().unwrap().config.http_listen.clone())?
            .run()
            .await
    }

    async fn metrics(registry: Registry) -> HttpResponse {
        let mut buffer = Vec::new();
        let encoder = TextEncoder::new();
        let metric_families = registry.gather();
        encoder.encode(&metric_families, &mut buffer).unwrap();
        HttpResponse::Ok()
            .content_type("text/plain")
            .body(String::from_utf8(buffer).unwrap())
    }
}

extern "C" fn handle_sig(sig_no: libc::c_int) {
    info!("signal_handler has been runned {:?}", sig_no);
    STOP.store(true, Order::SeqCst);
}

fn main()-> std::io::Result<()> {
    thread::spawn(move || {
        let main_thread = MainThread::set_up();
        let  rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        rt.block_on(async {
            main_thread.run_http_server().await;
        })
    });
    match run_main() {
        Ok(_) => {}
        Err(error) => {
            let e = format!("main thread err:{:?}", error);
            error!("{}", e);
            println!("{}", e);
        }
    }
    process::exit(0);
}

fn run_main() -> Result<U256, Error> {
    let opt = Opts::parse();
    let config: Config = Config::parse_from_file(&opt.config);
    let mut client = BlockClient::setup(&config, None);
    client.contract_setup(
        &config.root_secret.clone(),
        &config.chain.participant.clone(),
        "Randao_sol_Randao.abi",
        10000000,
        10000000000,
    );

    let client_arc = Arc::new(client);
    let chain_id = client_arc.chain_id().unwrap();
    let block = client_arc.current_block().unwrap();

    contract_new_campaign(&client_arc);

    if chain_id.to_string() != client_arc.config.chain.chainId {
        return Err(Error::CheckChainErr);
    }
    let campaign_num = match client_arc.contract_campaign_num() {
        None => {
            return Err(Error::GetNumCampaignsErr);
        }
        Some(num) => num,
    };
    info!(
        "chain_name:{:?}, chain_id:{:?}, block_num, endpoind:{:?}, campaigns_num:{:?}, randao:{:?}",
        client_arc.config.chain.name,
        chain_id,
        client_arc.config.chain.endpoint,
        block.number,
        client_arc.config.chain.opts.randao
    );

    let mut handle_vec = Vec::new();

    while !STOP.load(Order::SeqCst) {
        contract_new_campaign(&client_arc);
        let local_client = client_arc.clone();

        let new_campaign_num = match local_client.contract_campaign_num() {
            None => {
                return Err(Error::GetNumCampaignsErr);
            }
            Some(num) => num,
        };

        if new_campaign_num > campaign_num {
            let campaign_id = new_campaign_num.as_u128() - 1;
            let info = local_client.contract_get_campaign_info(campaign_id).unwrap();
            if local_client.config.chain.opts.maxCampaigns <= i32::try_from(new_campaign_num).unwrap() {
               break //return Err(Error::GetNumCampaignsErr);
            }
            if !check_campaign_info(&local_client, &info, &local_client.config) {
               continue //return Err(Error::CheckCampaignsInfoErr);
            }
            let t = thread::spawn(move || {
                let uuid = Uuid::new_v4().to_string();
                let work_thd = WorkThd::new(uuid,campaign_id.clone(),info,&local_client,   local_client.config.clone());
                let (uuid, campaign_id, randao_num, my_bounty) = work_thd.do_task().unwrap();

                info!("campaign_id:{:?},  randao:{:?}", campaign_id, randao_num);
            });
            handle_vec.push(t);
        }
        sleep(Duration::from_millis(5000));
    }
    for t in handle_vec {
        t.join().unwrap();
    }
    return Ok(U256::from(1));
}

fn contract_new_campaign(client: &BlockClient) ->Option<TransactionReceipt>
{
    let block_num = client.block_number().unwrap();
    let bnum = block_num.as_u64() + 20;
    let commitBalkline: u128 = 16;
    let commitDeadline: u128 = 8;
    let deposit: u128 = 1000000000000000000;
    //let  arg = format!("{:?},{:?},{:?},{:?}", bnum, deposit, commitBalkline, commitDeadline);

    let new_data = NewCampaignData {
        bnum: bnum.into(),
        deposit: deposit.into(),
        commitBalkline: commitBalkline.into(),
        commitDeadline: commitDeadline.into(),
    };
     client.contract_new_campaign(1000000, 10000000000, new_data)
}

#[test]
fn test_create_new_campaign(){
    let config: PathBuf = PathBuf::from("config.json");
    let config: Config = Config::parse_from_file(&config);
    let mut client = BlockClient::setup(&config, None);
    client.contract_setup(
        &config.root_secret.clone(),
        &config.chain.participant.clone(),
        "Randao_sol_Randao.abi",
        10000000,
        10000000000,
    );
    let block_num = client.block_number().unwrap();
    let bnum = block_num.as_u64() + 10;
    let commitBalkline: u128 = 8;
    let commitDeadline: u128 = 4;
    let deposit: u128 = 1000000000000000000;

    let new_data = NewCampaignData {
        bnum: bnum.into(),
        deposit: deposit.into(),
        commitBalkline: commitBalkline.into(),
        commitDeadline: commitDeadline.into(),
    };
    client.contract_new_campaign(1000000, 10000000000, new_data);
    let campaign_id = client.contract_campaign_num().unwrap();
    let campaign = campaign_id.as_u128() - 1;
    assert!(campaign > 0, "Campaign ID must be greater than 0");
}

#[test]
fn test_contract_new_campaign(){
    let config: PathBuf = PathBuf::from("config.json");
    let config: Config = Config::parse_from_file(&config);
    let mut client = BlockClient::setup(&config, None);
    client.contract_setup(
        &config.root_secret.clone(),
        &config.chain.participant.clone(),
        "Randao_sol_Randao.abi",
        10000000,
        10000000000,
    );
    let block_num = client.block_number().unwrap();
    let bnum = block_num.as_u64() + 10;
    let commitBalkline: u128 = 8;
    let commitDeadline: u128 = 4;
    let deposit: u128 = 1000000000000000000;

    let new_data = NewCampaignData {
        bnum: bnum.into(),
        deposit: deposit.into(),
        commitBalkline: commitBalkline.into(),
        commitDeadline: commitDeadline.into(),
    };
    client.contract_new_campaign(1000000, 10000000000, new_data);
    let campaign_id = client.contract_campaign_num().unwrap();
    let campaign = campaign_id.as_u128() - 1;
    assert!(campaign > 0, "Campaign ID must be greater than 0");

    let result = client.contract_follow(
        1000000,
        10000000000,
        campaign,
        deposit,
        &config.secret_key.follower_secret,
    );
    assert!(result.is_some());

    for _ in 0..1 {
        wait_blocks(&client);
    }
    let _s = "131242344353464564564574574567456";
    let hs = client.contract_sha_commit(_s.clone()).unwrap();
    client.contract_commit(campaign, deposit, &config.secret_key.consumer_secret, hs);
    for _ in 0..1 {
        wait_blocks(&client);
    }
    client.contract_reveal(campaign, deposit, &config.secret_key.consumer_secret, _s);
    let info = client.contract_get_campaign_info(campaign).unwrap();
    println!("campaign info :{:?}", info);
    for _ in 0..1 {
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

    let uuid = Uuid::new_v4().to_string();
    let work_thd = WorkThd::new(uuid, campaign, info, &client, config);
    let (uuid, campaign_id, randao_num, my_bounty) = work_thd.do_task().unwrap();

    println!(
        "uuid: {:?} campaign_id: {:?} randao_num: {:?} my_bounty :{:?}",
        uuid, campaign_id, randao_num, my_bounty
    );
}

