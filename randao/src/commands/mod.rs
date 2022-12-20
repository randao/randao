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
                        num.parse::<u32>().map_or(
                            Err("Node num should be a 32-bit integer".to_owned()),
                            |node| Ok(Self::Qa(cluster, Some(node))),
                        )
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
            self.height,
            block_time,
            self.txs,
            self.begin,
            self.snapshot,
            self.end,
            self.commit,
            self.commit_evm
        )
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
        #[clap(
            long,
            parse(from_os_str),
            value_name = "FILE",
            default_value = "source_keys.001"
        )]
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
