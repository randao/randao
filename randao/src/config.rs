use serde::*;
use serde_json::from_str;
use std::path::PathBuf;
use web3::contract::tokens::Detokenize;
use web3::contract::Error;
use web3::ethabi::Token;
use web3::types::U256;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct CampaignInfo {
    pub bnum: U256,
    pub deposit: U256,
    pub commitBalkline: U256,
    pub commitDeadline: U256,
    pub random: U256,
    pub settled: bool,
    pub bountypot: U256,
    pub commitNum: U256,
    pub revealsNum: U256,
}

impl CampaignInfo {
    fn from_token(tokens: Vec<Token>) -> Result<Self, Error> {
        // 检查 tokens 数组的长度
        if tokens.len() != 9 {
            println!(
                "Expected 9 elements, got a list of length {}: {:?}",
                tokens.len(),
                tokens
            );
            return Err(Error::InvalidOutputType(format!(
                "Expected 9 elements, got a list of length {}: {:?}",
                tokens.len(),
                tokens
            )));
        }

        // 将 tokens 中的元素转换成相应的类型
        let bnum = tokens[0].to_owned().into_uint().unwrap();
        let deposit = tokens[1].to_owned().into_uint().unwrap();
        let commitBalkline = tokens[2].to_owned().into_uint().unwrap();
        let commitDeadline = tokens[3].to_owned().into_uint().unwrap();
        let random = tokens[4].to_owned().into_uint().unwrap();
        let settled = tokens[5].to_owned().into_bool().unwrap();
        let bountypot = tokens[6].to_owned().into_uint().unwrap_or(U256::from(0));
        let commitNum = tokens[7].to_owned().into_uint().unwrap_or(U256::from(0));
        let revealsNum = tokens[8].to_owned().into_uint().unwrap_or(U256::from(0));

        Ok(CampaignInfo {
            bnum,
            deposit,
            commitBalkline,
            commitDeadline,
            random,
            settled,
            bountypot,
            commitNum,
            revealsNum,
        })
    }
}

impl Detokenize for CampaignInfo {
    fn from_tokens(mut tokens: Vec<Token>) -> Result<Self, Error> {
        if tokens.len() != 1 {
            Err(Error::InvalidOutputType(format!(
                "Expected single element, got a list: {:?}",
                tokens
            )))
        } else {
            match tokens[0].to_owned() {
                Token::Tuple(tokens) | Token::Array(tokens) => CampaignInfo::from_token(tokens),
                other => {
                    println!(
                        "Expected 9 elements, got a list of length {}: {:?}",
                        tokens.len(),
                        tokens
                    );
                    Err(Error::InvalidOutputType(format!(
                        "Expected `Array`, got {:?}",
                        other
                    )))
                }
            }
        }
    }
}

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
    pub chain: Chain,
    pub secret: String,
    pub secret_key: ConfigKey,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct ConfigKey {
    pub founder_secret: String,
    pub follower_secret: String,
    pub consumer_secret: String,
    pub committer_secret: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Chain {
    pub name: String,
    pub chainId: String,
    pub endpoint: String,
    pub participant: String,
    pub opts: ChainOpts,
}
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct ChainOpts {
    pub randao: String,
    pub gasLimit: String,
    pub maxGasPrice: String,
    pub minGasReserve: String,
    pub maxDeposit: i32,
    pub minRateOfReturn: f32,
    pub minRevealWindow: i32,
    pub maxRevealDelay: i32,
    pub maxCampaigns: i32,
    pub startBlock: i32,
}

impl Config {
    pub fn parse_from_file(file: &PathBuf) -> Self {
        use std::fs::read_to_string;
        let confstr = read_to_string(file).expect("confile read");
        from_str(&confstr).expect("confile deser")
    }

    pub fn show() {
        let de: Self = Default::default();
        println!("{}", serde_json::to_string_pretty(&de).unwrap())
    }
}

impl ConfigKey {
    pub fn parse_from_file(file: &PathBuf) -> Self {
        use std::fs::read_to_string;
        let confstr = read_to_string(file).expect("ConfigKey confile read");
        from_str(&confstr).expect("ConfigKey confile deser")
    }

    pub fn show() {
        let de: Self = Default::default();
        println!("{}", serde_json::to_string_pretty(&de).unwrap())
    }
}
