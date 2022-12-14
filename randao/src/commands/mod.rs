use crate::{
    db::{Db, Proto},
    profiler,
};
use chrono::NaiveDateTime;
use clap::{Parser, Subcommand};
use randao::{error::Result, BLOCK_TIME};
use serde::{Deserialize, Serialize};
use std::{
    fmt::{Display, Formatter},
    io::BufRead,
    path::{Path, PathBuf},
    rc::Rc,
};
use web3::types::{Address, H256};

#[derive(Debug)]
pub enum TestMode {
    Basic,
    Contract,
}

impl std::str::FromStr for TestMode {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "basic" => Ok(Self::Basic),
            "contract" => Ok(Self::Contract),
            _ => Err("Invalid mode: basic and contract are supported".to_owned()),
        }
    }
}

#[derive(Debug)]
pub enum Network {
    Local,
    Anvil,
    Main,
    Test,
    Qa(u32, Option<u32>),
    Node(String),
}

#[derive(Debug)]
pub enum ContractOP {
    Deploy,
    Call,
    Query,
}

const LOCAL_URL: &str = "http://localhost:8545";
const ANVIL_URL: &str = "https://prod-testnet.prod.findora.org:8545";
const MAIN_URL: &str = "https://prod-mainnet.prod.findora.org:8545";
const MY_TEST_URL: &str = "http://34.211.109.216:8545";

impl Network {
    pub fn get_url(&self) -> String {
        match self {
            Network::Local => LOCAL_URL.to_owned(),
            Network::Anvil => ANVIL_URL.to_owned(),
            Network::Main => MAIN_URL.to_owned(),
            Network::Test => MY_TEST_URL.to_owned(),
            Network::Qa(cluster, node) => {
                if let Some(node) = node {
                    format!(
                        "http://dev-qa{:0>2}-us-west-2-full-{:0>3}-open.dev.findora.org:8545",
                        cluster, node
                    )
                } else {
                    format!("https://dev-qa{:0>2}.dev.findora.org:8545", cluster)
                }
            }
            Network::Node(url) => url.to_owned(),
        }
    }
}

impl std::str::FromStr for Network {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_owned().as_str() {
            "local" => Ok(Self::Local),
            "anvil" => Ok(Self::Anvil),
            "main" => Ok(Self::Main),
            "test" => Ok(Self::Test),
            network if network.starts_with("qa") => {
                // --network qa,01,02
                let segs: Vec<&str> = network.splitn(3, ',').collect();
                if segs.len() < 2 {
                    return Err("Please provide a cluster num at least".to_owned());
                }
                if segs.first() != Some(&"qa") {
                    return Err("Just for qa environment".to_owned());
                }
                return if let Some(cluster) = segs.get(1).and_then(|&num| num.parse::<u32>().ok()) {
                    segs.get(2).map_or(Ok(Self::Qa(cluster, None)), |&num| {
                        num.parse::<u32>()
                            .map_or(Err("Node num should be a 32-bit integer".to_owned()), |node| {
                                Ok(Self::Qa(cluster, Some(node)))
                            })
                    })
                } else {
                    Err("QA env num is a 32-bit integer".to_owned())
                };
            }
            network if network.starts_with("node") => {
                let segs: Vec<&str> = network.splitn(2, ',').collect();
                if let Some(node) = segs.get(1) {
                    Ok(Self::Node(node.to_string()))
                } else {
                    Err("Please provide a node".to_owned())
                }
            }
            _ => Err("Invalid network".to_owned()),
        }
    }
}

impl std::str::FromStr for ContractOP {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_owned().as_str() {
            "deploy" => Ok(Self::Deploy),
            "call" => Ok(Self::Call),
            "query" => Ok(Self::Query),
            _ => Err("Invalid network".to_owned()),
        }
    }
}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about=None)]
pub(crate) struct Cli {
    #[clap(subcommand)]
    pub(crate) command: Option<Commands>,
}

#[allow(dead_code)]
#[derive(Debug, Default, Serialize, Deserialize)]
struct BlockInfo {
    height: u64,
    timestamp: i64,
    txs: u64,
    valid_txs: u64,
    block_time: Option<u64>,
    begin: u64,
    snapshot: u64,
    end: u64,
    commit: u64,
    commit_evm: u64,
}

impl Display for BlockInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let block_time = self.block_time.unwrap_or(0);
        write!(
            f,
            "{},{},{},{},{},{},{},{}",
            self.height, block_time, self.txs, self.begin, self.snapshot, self.end, self.commit, self.commit_evm
        )
    }
}

#[allow(unused)]
fn parse_abcid<P>(abcid: P, db: Rc<Db>) -> Result<()>
where
    P: AsRef<Path> + std::fmt::Debug,
{
    let abci_log = std::fs::File::open(abcid)?;
    std::io::BufReader::new(abci_log)
        .lines()
        .filter_map(|line| line.map_or(None, |l| if l.contains("tps,") { Some(l) } else { None }))
        .for_each(|line| {
            let words = line[52..].split(',').collect::<Vec<_>>();
            match words.last().map(|w| w.trim()) {
                Some("end of begin_block") => {
                    // tps,begin_block,31,31,td_height 781,end of begin_block
                    let height = words[words.len() - 2].split_whitespace().collect::<Vec<_>>()[1]
                        .parse::<u64>()
                        .unwrap();
                    if let Ok(raw_bi) = db.get(height) {
                        let mut bi: BlockInfo = serde_json::from_str(raw_bi.as_str()).unwrap();
                        bi.snapshot = words[2].parse::<u64>().unwrap();
                        bi.begin = words[3].parse::<u64>().unwrap();
                        let new_raw = serde_json::to_string(&bi).unwrap();
                        db.insert(bi.height, new_raw.as_bytes())
                            .expect("failed to update a block info");
                    }
                }
                Some("end of end_block") => {
                    // tps,end_block,6,td_height 781,end of end_block
                    let height = words[words.len() - 2].split_whitespace().collect::<Vec<_>>()[1]
                        .parse::<u64>()
                        .unwrap();
                    if let Ok(raw_bi) = db.get(height) {
                        let mut bi: BlockInfo = serde_json::from_str(raw_bi.as_str()).unwrap();
                        bi.end = words[2].parse::<u64>().unwrap();
                        let new_raw = serde_json::to_string(&bi).unwrap();
                        db.insert(bi.height, new_raw.as_bytes())
                            .expect("failed to update a block info");
                    }
                }
                Some("end of commit") => {
                    // tps,commit,2,60,62,td_height 781,end of commit
                    let height = words[words.len() - 2].split_whitespace().collect::<Vec<_>>()[1]
                        .parse::<u64>()
                        .unwrap();
                    if let Ok(raw_bi) = db.get(height) {
                        let mut bi: BlockInfo = serde_json::from_str(raw_bi.as_str()).unwrap();
                        bi.commit_evm = words[3].parse::<u64>().unwrap();
                        bi.commit = words[4].parse::<u64>().unwrap();
                        let new_raw = serde_json::to_string(&bi).unwrap();
                        db.insert(bi.height, new_raw.as_bytes())
                            .expect("failed to update a block info");
                    }
                }
                _ => {}
            }
        });
    Ok(())
}

fn parse_tendermint<P>(tendermint: P, db: Rc<Db>) -> Result<(u64, u64)>
where
    P: AsRef<Path> + std::fmt::Debug,
{
    let mut min_height = u64::MAX;
    let mut max_height = u64::MIN;
    let tm_log = std::fs::File::open(tendermint)?;
    for line in std::io::BufReader::new(tm_log).lines() {
        match line {
            Ok(l) if l.contains("Executed block") => {
                let mut blk = (None, None, None, None);
                // I[2022-04-07|02:17:07.759] Executed block module=state height=191 validTxs=3368 invalidTxs=666
                // parse timestamp
                // %Y-%m-%d|%H:%M:%S.%.3f
                let time_str = &l[2..25];
                blk.0 = NaiveDateTime::parse_from_str(time_str, "%Y-%m-%d|%H:%M:%S%.3f")
                    .map(|dt| dt.timestamp())
                    .ok();
                for word in l.split_whitespace() {
                    let kv = word.split('=').collect::<Vec<_>>();
                    if kv.len() != 2 {
                        continue;
                    } else {
                        match kv[0] {
                            "height" => blk.1 = kv[1].parse::<u64>().ok(),
                            "validTxs" => blk.2 = kv[1].parse::<u64>().ok(),
                            "invalidTxs" => blk.3 = kv[1].parse::<u64>().ok(),
                            _ => {}
                        }
                    }
                }
                let bi = BlockInfo {
                    height: blk.1.unwrap(),
                    timestamp: blk.0.unwrap(),
                    txs: blk.2.unwrap() + blk.3.unwrap(),
                    valid_txs: blk.2.unwrap(),
                    ..Default::default()
                };
                if min_height > bi.height {
                    min_height = bi.height;
                }
                if max_height < bi.height {
                    max_height = bi.height
                }
                let raw_data = serde_json::to_string(&bi).unwrap();
                db.insert(bi.height, raw_data.as_bytes())
                    .expect("failed to insert a block info");
                //blocks.insert(bi.height, std::cell::RefCell::new(bi));
            }
            _ => {}
        }
    }
    Ok((min_height, max_height))
}

impl Cli {
    pub(crate) fn parse_args() -> Self {
        Cli::parse()
    }

    pub(crate) fn etl_cmd<P>(abcid: &Option<P>, tendermint: &Option<P>, redis: &str, load: bool) -> Result<()>
    where
        P: AsRef<Path> + std::fmt::Debug,
    {
        log::info!("{:?} {:?} {} {}", abcid, tendermint, redis, load);

        let proto = if &redis[..4] == "unix" { Proto::Unix } else { Proto::Url };
        let db = Rc::new(Db::new(Some(proto), None, redis, Some(6379), Some(0))?);

        let (min_height, max_height) = tendermint.as_ref().map_or_else(
            || (u64::MAX, u64::MIN),
            |tendermint| parse_tendermint(tendermint, db.clone()).unwrap_or((u64::MAX, u64::MIN)),
        );
        abcid.as_ref().map(|abcid| parse_abcid(abcid, db.clone()));

        for h in min_height..=max_height {
            if let Ok(bi) = db.get(h) {
                let bi = serde_json::from_str::<BlockInfo>(bi.as_str()).unwrap();
                let last_bi = {
                    if h == 0 {
                        None
                    } else if let Ok(bi) = db.get(h - 1) {
                        serde_json::from_str::<BlockInfo>(bi.as_str()).ok()
                    } else {
                        None
                    }
                };

                let (block_time, tps) = match last_bi {
                    Some(last) if bi.timestamp > last.timestamp => {
                        let time = bi.timestamp - last.timestamp;
                        let tps = bi.txs as f64 / time as f64;
                        (time, tps)
                    }
                    _ => (0i64, 0f64),
                };
                log::info!("{},{},{},{},{:.3}", bi.height, bi.txs, bi.valid_txs, block_time, tps,);
            }
        }
        Ok(())
    }

    pub(crate) fn profiler(network: &str, enabled: bool) -> Result<()> {
        let url = format!("{}/configuration", network);
        profiler::set_profiler(url.as_str(), enabled)
    }
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Fund Ethereum accounts
    Fund {
        /// ethereum-compatible network
        #[clap(long)]
        network: Network,

        /// http request timeout, seconds
        #[clap(long)]
        timeout: Option<u64>,

        /// block time of the network
        #[clap(long, default_value_t = BLOCK_TIME)]
        block_time: u64,

        /// the number of Eth Account to be fund
        #[clap(long, default_value_t = 0)]
        count: u64,

        /// how much 0.1-eth to fund
        #[clap(long, default_value_t = 1)]
        amount: u64,

        /// load keys from file
        #[clap(long)]
        load: bool,

        /// re-deposit account with insufficient balance
        #[clap(long)]
        redeposit: bool,
    },
    /// check ethereum account information
    Info {
        /// ethereum-compatible network
        #[clap(long)]
        network: Network,

        /// http request timeout, seconds
        #[clap(long)]
        timeout: Option<u64>,

        /// ethereum address
        #[clap(long)]
        account: Address,
    },

    /// Transaction Operations
    Transaction {
        /// ethereum-compatible network
        #[clap(long)]
        network: Network,

        /// http request timeout, seconds
        #[clap(long)]
        timeout: Option<u64>,

        /// transaction hash
        #[clap(long)]
        hash: H256,
    },

    /// Block Operations
    Block {
        /// ethereum-compatible network
        #[clap(long)]
        network: Network,

        /// http request timeout, seconds
        #[clap(long)]
        timeout: Option<u64>,

        /// start block height
        #[clap(long)]
        start: Option<u64>,

        /// block count, could be less than zero
        #[clap(long)]
        count: Option<i64>,
    },

    /// ETL procession
    Etl {
        /// abcid log file
        #[clap(long)]
        abcid: Option<String>,

        /// tendermint log file
        #[clap(long)]
        tendermint: Option<String>,

        /// redis db address
        #[clap(long, default_value = "127.0.0.1")]
        redis: String,

        /// load data
        #[clap(long)]
        load: bool,
    },

    /// Profiler operations
    Profiler {
        ///  Findora submission server endpoint
        #[clap(long)]
        network: String,

        /// Profiler switch
        #[clap(long)]
        enable: bool,
    },
    /// Contract Operations
    Contract {
        /// ethereum-compatible network
        #[clap(long)]
        network: Network,

        /// contract operation
        #[clap(long)]
        optype: ContractOP,

        /// contract operation
        #[clap(long)]
        config: PathBuf,

        /// http request timeout, seconds
        #[clap(long)]
        timeout: Option<u64>,
    },
    /// Test
    Test {
        /// Ethereum web3-compatible network
        #[clap(long)]
        network: Network,

        /// Test mode: basic transfer transaction, contract call transaction
        #[clap(long)]
        mode: TestMode,

        /// Delay time for next batch of transactions
        #[clap(long, default_value_t = 15)]
        delay: u64,

        /// The max thread pool size
        #[clap(long, default_value_t = 200)]
        max_threads: u64,

        /// The count of transactions sent by a source key
        #[clap(long, default_value_t = 0)]
        count: u64,

        /// the source account file
        #[clap(long, parse(from_os_str), value_name = "FILE", default_value = "source_keys.001")]
        source: PathBuf,

        /// block time of the network
        #[clap(long, default_value_t = BLOCK_TIME)]
        block_time: u64,

        /// http request timeout, seconds
        #[clap(long, default_value_t = 60)]
        timeout: u64,

        /// if need to retry to sending transactions
        #[clap(long)]
        need_retry: bool,

        /// If need to check balance of source keys
        #[clap(long)]
        check_balance: bool,
    },
}
