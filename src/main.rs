use reqwest;
use std::time::{SystemTime};
use serde_json::Value;
use serde::{Serialize};
use std::collections::HashMap;

#[derive(Serialize)]
struct Request{
    params: (String, bool),
    jsonrpc: String,
    method: String,
    id: String
}

#[tokio::main]
async fn main() {
   
    let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();

    let ETHERSCAN_API_KEY: &str = "UBI18HPD3AFGC3QH6FZCHX423MKMYXHVCP";
    let request_url = format!("https://api.etherscan.io/api?module=block&action=getblocknobytime&timestamp={timestamp}&closest=before&apikey={apiKey}",
        timestamp=now.to_string(),
        apiKey=ETHERSCAN_API_KEY);
    
    let response = reqwest::get(&request_url).await.unwrap().text().await.unwrap();
    let data: Value = serde_json::from_str(&response).unwrap();

    if let Some(result) = data["result"].as_str(){
        let block: u64 = result.parse().unwrap();
        println!("Currently at timestamp {}\nLast ethereum block was number {}", now, block);

        let ALCHEMY_URL: &str = "https://eth-mainnet.g.alchemy.com/v2/J4xKXWLsSFOdY3YFDyyuJLsAqHIbaDvC";
        
        let request = Request{
            params: (format!("0x{:x}", block), true),
            jsonrpc: String::from("2.0"),
            method: String::from("eth_getBlockByNumber"),
            id: String::from("1"),
        };

        let client = reqwest::Client::new();
        let res = client.post(ALCHEMY_URL).json(&request).send().await.unwrap().text().await.unwrap();
        println!("{:?}", res);
    }
    else{
        println!("Error when retrieving block number");
    }

    // Now json-rpc call to node to get block info



}
