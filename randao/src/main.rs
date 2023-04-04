#[macro_use]
extern crate serde;

mod api;

use actix_cors::Cors;
use actix_web::{middleware, post, web, App, HttpServer, Responder};
use api::ApiResult;
use clap::Parser;
use hyper::{
    header::CONTENT_TYPE,
    service::{make_service_fn, service_fn},
    Body, Request, Response, Server,
};
use lazy_static::lazy_static;
use log::{error, info};
use prometheus::{
    labels, opts, register_gauge, register_histogram_vec, Encoder, Gauge, HistogramVec, TextEncoder,
};
use randao::config::Opts;
use randao::RANDAO_PATH;
use randao::{
    config::*, contract::*, error::Error, utils::*, BlockClient, WorkThd, ONGOING_CAMPAIGNS,
};
use std::{
    net::SocketAddr,
    str::FromStr,
    fs::create_dir,
    ops::{Add, AddAssign, SubAssign},
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering as Order},
        Arc, Mutex,
    },
    thread::{self, sleep},
    time::Duration,
};
use web3::types::{H256, U256};

// const CHECK_CNT: u8 = 5;

lazy_static! {
    static ref STOP: AtomicBool = AtomicBool::new(false);
    static ref MUTEX: Mutex<()> = Mutex::new(());
    static ref THREAD_CNT: std::sync::Mutex<u16> = std::sync::Mutex::new(0);
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
    pub fn set_up(randao_cfg: &Config) -> Self {
        let client = Arc::new(Mutex::new(BlockClient::setup(randao_cfg, None)));
        MainThread { client }
    }

    pub async fn run_http_svr(&self) -> std::io::Result<()> {
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

    pub async fn prometheus_http_svr_start(&self) {
        // let addr = ([0, 0, 0, 0], 9090).into();
        info!(
            "Prometheus Listening on http://{}",
            self.client.lock().unwrap().config.prometheus_listen.clone()
        );
        let addr = SocketAddr::from_str(&self.client.lock().unwrap().config.prometheus_listen.clone()).unwrap();
        let serve_future = Server::bind(
            &addr,
        )
        .serve(make_service_fn(|_| async {
            Ok::<_, hyper::Error>(service_fn(prometheus_http_svr_req))
        }));

        if let Err(err) = serve_future.await {
            panic!("Server Error: {}", err);
        }
    }
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

fn main() -> anyhow::Result<()> {
    // env::set_var("RUST_LOG", "info,all=info");
    // env_logger::init();

    let mut opts: Opts = Opts::parse();
    println!("opts: {:?}", opts);

    opts.datadir.push_str(&RANDAO_PATH.lock().unwrap());
    *RANDAO_PATH.lock().unwrap() = opts.datadir;

    let randao_cfg = PathBuf::from(opts.config);
    if !randao_cfg.exists() {
        create_dir(&randao_cfg)?;
    } else if !randao_cfg.is_file() {
        anyhow::bail!("randao config is not file!!!");
    }

    let randao_cfg: Config = Config::parse_from_file(&randao_cfg);

    let randao_path = PathBuf::from(RANDAO_PATH.lock().unwrap().to_owned());
    if !randao_path.exists() || !randao_path.is_dir() {
        anyhow::bail!("randao folder is incorrect!!!");
    }

    // let key_path = Path::new(KEY_PATH);
    // if !key_path.exists() || !key_path.is_dir() {
    //     anyhow::bail!("key folder is incorrect!!!");
    // }

    let randao_cfg2 = randao_cfg.clone();
    let randao_cfg3 = randao_cfg.clone();
    if !opts.is_campagin {
        thread::spawn(move || {
            let main_thread = MainThread::set_up(&randao_cfg2);
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();
            rt.block_on(async {
                let _ = main_thread.run_http_svr().await;
            })
        });

        thread::spawn(move || {
            let main_thread = MainThread::set_up(&randao_cfg3);
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();
            rt.block_on(async {
                let _ = main_thread.prometheus_http_svr_start().await;
            })
        });
    }

    run_main(&randao_cfg, opts.is_campagin)
        .or_else(|e| anyhow::bail!("main thread err:{:?}", e))?;

    Ok(())
}

fn run_main(randao_cfg: &Config, is_campagin: bool) -> Result<U256, Error> {
    let mut client = BlockClient::setup(&randao_cfg, None);

    let abi_content = include_str!("../Randao_sol_Randao.abi");
    client.contract_setup(
        &randao_cfg.chain.participant.clone(),
        &randao_cfg.chain.opts.randao.clone(),
        abi_content,
        10000000,
        10000000000,
    );

    let cli_arc = Arc::new(client);
    let chain_id = cli_arc.chain_id().unwrap();
    let block = cli_arc.current_block().unwrap();

    if is_campagin {
        contract_new_campaign(&cli_arc);
    }

    if chain_id.to_string() != cli_arc.config.chain.chain_id {
        return Err(Error::CheckChainErr);
    }
    info!(
        "chain_name:{:?}, chain_id:{:?}, block_num, endpoind:{:?}, campaigns_num:{:?}, randao:{:?}",
        cli_arc.config.chain.name,
        chain_id,
        cli_arc.config.chain.endpoint,
        block.number,
        cli_arc.config.chain.opts.randao
    );

    let mut handle_vec: Vec<std::thread::JoinHandle<Result<(), Error>>> = Vec::new();
    // let max_thds_cnt: usize = num_cpus::get() * 2;
    // let mut check_cnt: u8 = 0;

    let mut campaign_ids = {
        let _guard = MUTEX
            .lock()
            .or_else(|e| Err(Error::Unknown(format!("{:?}", e))))?;
        read_campaign_ids(&RANDAO_PATH.lock().unwrap()).or_else(|e| {
            error!("Error loading campaign_id file: {:?}", e);
            Ok(Vec::new())
        })?
    };
    info!("campaign_ids {:?}", campaign_ids);

    let mut campaign_num = if !campaign_ids.is_empty() {
        let mut max_campagin_id = 0;
        for campaign_id in campaign_ids.clone() {
            if campaign_id > max_campagin_id {
                max_campagin_id = campaign_id;
            }
        }
        U256::from(max_campagin_id + 1)
    } else {
        U256::from(0)
    };

    while !STOP.load(Order::SeqCst) {
        if is_campagin {
            contract_new_campaign(&cli_arc);
            sleep(Duration::from_millis(1000));
            continue;
        }

        let local_client = cli_arc.clone();

        let new_campaign_num = match local_client.contract_campaign_num() {
            None => {
                return Err(Error::GetNumCampaignsErr);
            }
            Some(num) => num,
        };

        if new_campaign_num <= campaign_num && campaign_ids.is_empty() {
            sleep(Duration::from_millis(5000));
            continue;
        }

        // if new_campaign_num > campaign_num || !campaign_ids.is_empty() {
        if campaign_ids.is_empty() {
            campaign_num = new_campaign_num;
        }

        let mut campaign_id = new_campaign_num.as_u128() - 1;
        let info = local_client
            .contract_get_campaign_info(campaign_id)
            .unwrap();
        if local_client.config.chain.opts.max_campaigns
            <= i32::try_from(handle_vec.len())
                .or_else(|e| Err(Error::Unknown(format!("{:?}", e))))?
        {
            handle_vec
                .pop()
                .unwrap()
                .join()
                .map_err(|e| Error::Unknown(format!("{:?}", e)))??;
            THREAD_CNT.lock().unwrap().sub_assign(1);

            println!("thread count greater than max campaign");
            continue; //return Err(Error::GetNumCampaignsErr);
        }
        if !check_campaign_info(&local_client, &info, &local_client.config) {
            println!("campaign info is incorrect");
            continue; //return Err(Error::CheckCampaignsInfoErr);
        }

        let is_new_campaign_id = {
            let _guard = MUTEX.lock().unwrap();
            if campaign_ids.is_empty() {
                store_campaign_id(&RANDAO_PATH.lock().unwrap(), campaign_id).unwrap();
                true
            } else {
                campaign_id = campaign_ids.pop().unwrap();
                false
            }
        };

        let t = thread::spawn(move || {
            let mut work_thd;
            if is_new_campaign_id {
                work_thd = WorkThd::new(
                    campaign_id,
                    info,
                    &local_client,
                    local_client.config.clone(),
                    RANDAO_PATH.lock().unwrap().to_owned(),
                );
            } else {
                work_thd = WorkThd::new_from_campaign_id(
                    campaign_id,
                    &local_client,
                    local_client.config.clone(),
                    RANDAO_PATH.lock().unwrap().to_owned(),
                );
            }
            println!("work thread begin!!!");
            match work_thd
                .do_task()
                .or_else(|e| Err(Error::Unknown(format!("{:?}", e))))
            {
                Ok((campaign_id, randao_num, _my_bounty)) => {
                    println!("work thread end success!!!");
                    info!("campaign_id:{:?},  randao:{:?}", campaign_id, randao_num);
                    THREAD_CNT.lock().unwrap().add_assign(1);
                }
                Err(e) => {
                    println!("work thread err: {:?}", e);
                    THREAD_CNT.lock().unwrap().add_assign(1);
                }
            };

            {
                let _guard = MUTEX.lock().unwrap();
                remove_campaign_id(&RANDAO_PATH.lock().unwrap(), campaign_id)
                    .or_else(|e| Err(Error::Unknown(format!("{:?}", e))))?
            }
            Ok(())
        });
        handle_vec.push(t);
        while *THREAD_CNT.lock().unwrap() != 0 {
            handle_vec
                .pop()
                .unwrap()
                .join()
                .map_err(|e| Error::Unknown(format!("{:?}", e)))??;
            THREAD_CNT.lock().unwrap().sub_assign(1);
        }
        println!(
            "-------------------thread count: {:?}-------------------",
            handle_vec.len()
        );

        // check_cnt += 1;
        // if check_cnt == CHECK_CNT {
        //     while handle_vec.len() > max_thds_cnt {
        //         handle_vec
        //             .pop()
        //             .unwrap()
        //             .join()
        //             .map_err(|e| Error::Unknown(format!("{:?}", e)))??;
        //     }
        //     check_cnt = 0;
        // }
    }
    //     sleep(Duration::from_millis(5000));
    // }
    for t in handle_vec {
        t.join().map_err(|e| Error::Unknown(format!("{:?}", e)))??;
    }
    return Ok(U256::from(1));
}

fn contract_new_campaign(client: &BlockClient) -> Option<H256> {
    let block_num = client.block_number().unwrap();
    let bnum = block_num.as_u64() + 20;
    let commit_balkline: u128 = 16;
    let commit_deadline: u128 = 8;
    let deposit: u128 = 1000000000000000000;

    let new_data = NewCampaignData {
        bnum: bnum.into(),
        deposit: deposit.into(),
        commit_balkline: commit_balkline.into(),
        commit_deadline: commit_deadline.into(),
    };
    println!("----------------------contract_new_campaign begin----------------------");
    let res = client.contract_new_campaign(1000000, 10000000000, new_data);
    println!("----------------------contract_new_campaign end----------------------");
    res
}

#[test]
fn test_create_new_campaign() {
    use serde_json::{from_str, Value};
    use std::env;
    use std::fs;
    use std::path::Path;

    let conf_path = PathBuf::from("config.json");
    let config: Config = Config::parse_from_file(&conf_path);
    let mut client = BlockClient::setup(&config, None);
    client.contract_setup(
        &config.chain.participant.clone(),
        &config.chain.opts.randao.clone(),
        "Randao_sol_Randao.abi",
        10000000,
        10000000000,
    );
    let block_num = client.block_number().unwrap();
    let bnum = block_num.as_u64() + 10;
    let commit_balkline: u128 = 8;
    let commit_deadline: u128 = 4;
    let deposit: u128 = 1000000000000000000;

    let new_data = NewCampaignData {
        bnum: bnum.into(),
        deposit: deposit.into(),
        commit_balkline: commit_balkline.into(),
        commit_deadline: commit_deadline.into(),
    };
    client.contract_new_campaign(1000000, 10000000000, new_data);
    let campaign_id = client.contract_campaign_num().unwrap();
    let campaign = campaign_id.as_u128() - 1;
    assert!(campaign > 0, "Campaign ID must be greater than 0");

    let current_dir = env::current_dir().unwrap();
    let file_path = Path::new(&current_dir).join("src/test-keys/test-key.json");
    let file_contents = fs::read_to_string(file_path).unwrap();

    // Parse the JSON object
    let keys: Value = from_str(&file_contents).unwrap();

    // Extract the secrets from the JSON object
    let founder = keys["founder"]["secret"].as_str().unwrap();
    let follower = keys["follower"]["secret"].as_str().unwrap();
    let consumer = keys["consumer"]["secret"].as_str().unwrap();
    let committer = keys["committer"]["secret"].as_str().unwrap();

    println!("Founder secret: {}", founder);
    println!("Follower secret: {}", follower);
    println!("Consumer secret: {}", consumer);
    println!("Committer secret: {}", committer);
}

#[test]
fn test_contract_new_campaign() {
    use serde_json::{from_str, Value};
    use std::env;
    use std::fs;
    use std::path::Path;
    use std::path::PathBuf;

    let current_dir = env::current_dir().unwrap();
    let file_path = Path::new(&current_dir).join("src/test-keys/test-key.json");
    let file_contents = fs::read_to_string(file_path).unwrap();

    // Parse the JSON object
    let keys: Value = from_str(&file_contents).unwrap();

    // Extract the secrets from the JSON object
    let _founder = keys["founder"]["secret"].as_str().unwrap();
    let follower = keys["follower"]["secret"].as_str().unwrap();
    let consumer = keys["consumer"]["secret"].as_str().unwrap();
    let _committer = keys["committer"]["secret"].as_str().unwrap();

    let conf_path = PathBuf::from("config.json");
    let config: Config = Config::parse_from_file(&conf_path);
    let mut client = BlockClient::setup(&config, None);
    client.contract_setup(
        &config.chain.participant,
        &config.chain.opts.randao,
        "Randao_sol_Randao.abi",
        10000000,
        10000000000,
    );
    let block_num = client.block_number().unwrap();
    let bnum = block_num.as_u64() + 10;
    let commit_balkline: u128 = 7;
    let commit_deadline: u128 = 3;
    let deposit: u128 = 1000000000000000000;

    let new_data = NewCampaignData {
        bnum: bnum.into(),
        deposit: deposit.into(),
        commit_balkline: commit_balkline.into(),
        commit_deadline: commit_deadline.into(),
    };
    client.contract_new_campaign(1000000, 10000000000, new_data);
    let campaign_id = client.contract_campaign_num().unwrap();
    assert!(campaign_id.as_u128() > 0);
    let campaign_id = campaign_id.as_u128() - 1;
    assert!(campaign_id > 0, "Campaign ID must be greater than 0");

    let result = client.contract_follow(1000000, 10000000000, campaign_id, deposit, follower);
    assert!(result.is_some());

    for _ in 0..1 {
        wait_blocks(&client);
    }
    let _s = "131242344353464564564574574567456";
    let hs = client.contract_sha_commit(_s).unwrap();
    client.contract_commit(campaign_id, deposit, consumer, hs);
    for _ in 0..1 {
        wait_blocks(&client);
    }
    client.contract_reveal(campaign_id, deposit, consumer, _s);
    let info = client.contract_get_campaign_info(campaign_id).unwrap();
    println!("campaign info :{:?}", info);
    for _ in 0..1 {
        wait_blocks(&client);
    }
    let randao_num = client.contract_get_random(campaign_id, consumer).unwrap();
    println!("randao_num :{:?}", randao_num);
    let my_bounty = client
        .contract_get_my_bounty(campaign_id, consumer)
        .unwrap();
    println!("my_bounty :{:?}", my_bounty);

    let mut work_thd = WorkThd::new(campaign_id, info, &client, config, RANDAO_PATH.lock().unwrap().to_owned());
    let (campaign_id, randao_num, my_bounty) = work_thd.do_task().unwrap();

    println!(
        "campaign_id: {:?} randao_num: {:?} my_bounty :{:?}",
        campaign_id, randao_num, my_bounty
    );
}
