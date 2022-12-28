#[macro_use]
extern crate serde;

mod config;
use randao::WorkThd;
use uuid::Uuid;
mod api;
mod contract;

use std::thread::sleep;
use std::{sync::Arc, thread, time::Duration};

use actix_cors::Cors;
use actix_web::{middleware, post, web, App, HttpServer, Responder};
use clap::Parser;
use log::{error, info};
use nix::libc;

use randao::{config::*, contract::*, error::Error, utils::*, BlockClient, ONGOING_CAMPAIGNS};
use std::process;
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering as Order};
use std::sync::Mutex;
use web3::types::{TransactionReceipt, U256};

use hyper::{
    header::CONTENT_TYPE,
    service::{make_service_fn, service_fn},
    Body, Request, Response, Server,
};
use prometheus::{Encoder, Gauge, HistogramVec, TextEncoder};

use lazy_static::lazy_static;
use prometheus::{labels, opts, register_gauge, register_histogram_vec};

use crate::api::ApiResult;

lazy_static! {
    static ref STOP: AtomicBool = AtomicBool::new(false);
    static ref MUTEX: Mutex<()> = Mutex::new(());
}

#[derive(Clone)]
struct MainThread {
    client: Arc<Mutex<BlockClient>>,
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
    pub fn set_up() -> Self {
        let opt = Opts::parse();
        let config: Config = Config::parse_from_file(&opt.config);
        let client = Arc::new(Mutex::new(BlockClient::setup(&config, None)));
        MainThread {
            client,
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
}

extern "C" fn handle_sig(sig_no: libc::c_int) {
    info!("signal_handler has been runned {:?}", sig_no);
    STOP.store(true, Order::SeqCst);
}

lazy_static! {
    static ref HTTP_BODY_GAUGE: Gauge = register_gauge!(opts!(
        "http_response_size_bytes",
        "The HTTP response sizes in bytes.",
        labels! {"handler" => "all",}
    ))
    .unwrap();
    static ref HTTP_REQ_HISTOGRAM: HistogramVec = register_histogram_vec!(
        "http_request_duration_seconds",
        "The HTTP request latencies in seconds.",
        &["handler"]
    )
    .unwrap();
}

async fn prometheus_http_svr_req(_req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    let encoder = TextEncoder::new();

    ONGOING_CAMPAIGNS.get();

    let timer = HTTP_REQ_HISTOGRAM.with_label_values(&["all"]).start_timer();

    let metric_families = prometheus::gather();
    let mut buffer = vec![];
    encoder.encode(&metric_families, &mut buffer).unwrap();
    HTTP_BODY_GAUGE.set(buffer.len() as f64);

    let response = Response::builder()
        .status(200)
        .header(CONTENT_TYPE, encoder.format_type())
        .body(Body::from(buffer))
        .unwrap();

    timer.observe_duration();

    Ok(response)
}

async fn prometheus_http_svr_start() {
    let addr = ([0, 0, 0, 0], 9090).into();
    info!("Prometheus Listening on http://{}", addr);

    let serve_future = Server::bind(&addr).serve(make_service_fn(|_| async {
        Ok::<_, hyper::Error>(service_fn(prometheus_http_svr_req))
    }));

    if let Err(err) = serve_future.await {
        panic!("Server Error: {}", err);
    }
}

fn main() -> std::io::Result<()> {
    thread::spawn(move || {
        let main_thread = MainThread::set_up();
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        rt.block_on(async {
            main_thread.run_http_server().await;
        })
    });

    thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        rt.block_on(async {
            prometheus_http_svr_start().await;
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

    //test
    contract_new_campaign(&client_arc);

    if chain_id.to_string() != client_arc.config.chain.chain_id {
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

    let guard = MUTEX.lock().unwrap();
    match read_uuids() {
        Ok(uuids) => {
            drop(guard);
            for uuid in uuids{

            }
        },
        Err(err) => {
            error!("Error loading UUID file: {:?}", err);
        },
    }

    while !STOP.load(Order::SeqCst) {

        //test
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
            let info = local_client
                .contract_get_campaign_info(campaign_id)
                .unwrap();
            if local_client.config.chain.opts.max_campaigns
                <= i32::try_from(new_campaign_num).unwrap()
            {
                break; //return Err(Error::GetNumCampaignsErr);
            }
            if !check_campaign_info(&local_client, &info, &local_client.config) {
                continue; //return Err(Error::CheckCampaignsInfoErr);
            }
            let t = thread::spawn(move || {
                let uuid = Uuid::new_v4().to_string();

                let guard = MUTEX.lock().unwrap();
                store_uuid(&Uuid::from_str(uuid.as_str().clone()).unwrap()).unwrap();
                drop(guard);

                let work_thd = WorkThd::new(
                    uuid,
                    campaign_id.clone(),
                    info,
                    &local_client,
                    local_client.config.clone(),
                );
                let (uuid, campaign_id, randao_num, my_bounty) = work_thd.do_task().unwrap();
                info!("campaign_id:{:?},  randao:{:?}", campaign_id, randao_num);

                let guard = MUTEX.lock().unwrap();
                remove_uuid(&Uuid::from_str(uuid.as_str()).unwrap()).unwrap();
                drop(guard);
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

fn contract_new_campaign(client: &BlockClient) -> Option<TransactionReceipt> {
    let block_num = client.block_number().unwrap();
    let bnum = block_num.as_u64() + 20;
    let commitBalkline: u128 = 16;
    let commitDeadline: u128 = 8;
    let deposit: u128 = 1000000000000000000;
    //let  arg = format!("{:?},{:?},{:?},{:?}", bnum, deposit, commitBalkline, commitDeadline);

    let new_data = NewCampaignData {
        bnum: bnum.into(),
        deposit: deposit.into(),
        commit_balkline: commitBalkline.into(),
        commit_deadline: commitDeadline.into(),
    };
    client.contract_new_campaign(1000000, 10000000000, new_data)
}

#[test]
fn test_create_new_campaign() {
    let config: std::path::PathBuf = std::path::PathBuf::from("config.json");
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
        commit_balkline: commitBalkline.into(),
        commit_deadline: commitDeadline.into(),
    };
    client.contract_new_campaign(1000000, 10000000000, new_data);
    let campaign_id = client.contract_campaign_num().unwrap();
    let campaign = campaign_id.as_u128() - 1;
    assert!(campaign > 0, "Campaign ID must be greater than 0");
}

#[test]
fn test_contract_new_campaign() {
    use std::path::PathBuf;
    
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
        commit_balkline: commitBalkline.into(),
        commit_deadline: commitDeadline.into(),
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
