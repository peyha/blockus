use reqwest;
use std::time::{SystemTime};
use serde_json::Value;
use std::collections::HashMap;

#[tokio::main]
async fn main() {
   
    let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();

    let request_url = format!("https://api.etherscan.io/api?module=block&action=getblocknobytime&timestamp={timestamp}&closest=before&apikey={apiKey}",
        timestamp=now.to_string(),
        apiKey="UBI18HPD3AFGC3QH6FZCHX423MKMYXHVCP");
    
    let response = reqwest::get(&request_url).await.unwrap().text().await.unwrap();
    let data: Value = serde_json::from_str(&response).unwrap();

    if let Some(result) = data["result"].as_str(){
        let block: u64 = result.parse().unwrap();
        println!("Currently at timestamp {}\nLast ethereum block was number {}", now, block);
    }
    else{
        println!("Error when retrieving block number");
    }


}
