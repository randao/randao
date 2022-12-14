
use serde_json::from_str;
use std::path::PathBuf;
use std::sync::Arc;
use serde::*;

#[derive(clap::Parser, Debug)]
pub struct Opts {
    /// Config file
    #[clap(
    short = 'c',
    long = "config",
    parse(from_os_str),
    default_value = "config.json"
    )]
    pub config: PathBuf,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Config {
    pub chain:Chain,
    pub secret:String
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Chain {
    pub  name: String,
    pub  chainId: String,
    pub endpoint: String,
    pub participant: String,
    pub opts: ChainOpts,
}
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct ChainOpts {
    pub   randao: String,
    pub   gasLimit: String,
    pub  maxGasPrice: String,
    pub  minGasReserve: String,
    pub  maxDeposit: i32,
    pub  minRateOfReturn: f32,
    pub  minRevealWindow: i32,
    pub   maxRevealDelay: i32,
    pub  maxCampaigns: i32,
    pub  startBlock: i32,
}

impl Config {
    pub fn parse_from_file(file: &PathBuf) -> Self {
        use std::fs::read_to_string;

        println!("file :{:?} ", file);
        let confstr = read_to_string(file).expect("confile read");
        from_str(&confstr).expect("confile deser")
    }

    pub fn show() {
        let de: Self = Default::default();
        println!("{}", serde_json::to_string_pretty(&de).unwrap())
    }
}

