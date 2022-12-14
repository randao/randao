use randao::error::Result;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct Configuration {
    // abicd, evm, profile
    component: String,
    //
    module: String,
    //
    submodule: String,
    //
    parameters: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug)]
struct ProfilerParam {
    enable: bool,
}

pub fn set_profiler(endpoint: &str, status: bool) -> Result<()> {
    let data = Configuration {
        component: "profiler".to_string(),
        module: String::new(),
        submodule: String::new(),
        parameters: serde_json::to_vec(&ProfilerParam { enable: status }).unwrap(),
    };
    let client = reqwest::blocking::Client::new();
    let res = client.post(endpoint).json(&data).send().unwrap();
    if res.status().is_success() {
        log::info!("{:?}", res.text().ok());
    } else {
        log::error!("{:?}", res.status())
    }

    Ok(())
}
