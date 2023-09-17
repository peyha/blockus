use reqwest;
use std::time::{SystemTime};
use serde_json::Value;
use serde::{Serialize};
use std::collections::HashMap;
use clap::Parser;
use std::u64;
use std::thread;
use std::time::Duration;

#[derive(Serialize)]
struct Request{
    params: (String, bool),
    jsonrpc: String,
    method: String,
    id: String
}

#[derive(Parser)]
#[command(author, version, about)]
struct Cli{
    #[arg(short, long)]
    etherscan_key: String,
    #[arg(short, long)]
    rpc_url: String,
}

#[tokio::main]
async fn main() {
    let args = Cli::parse();
    loop{
        
        let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();

        let request_url = format!("https://api.etherscan.io/api?module=block&action=getblocknobytime&timestamp={timestamp}&closest=before&apikey={apiKey}",
            timestamp=now.to_string(),
            apiKey=args.etherscan_key.as_str());
        
        let response = reqwest::get(&request_url).await.unwrap().text().await.unwrap();
        let data: Value = serde_json::from_str(&response).unwrap();

        if let Some(result) = data["result"].as_str(){
            let block: u64 = result.parse().unwrap();
            println!("Currently at timestamp {}\nLast ethereum block was number {}", now, block);

            let request = Request{
                params: (format!("0x{:x}", block), true),
                jsonrpc: String::from("2.0"),
                method: String::from("eth_getBlockByNumber"),
                id: String::from("1"),
            };

            let client = reqwest::Client::new();
            let res = client.post(args.rpc_url.as_str()).json(&request).send().await.unwrap().text().await.unwrap();
            let raw_data: Value = serde_json::from_str(&res).unwrap();
            let data = raw_data["result"].clone();
            let hash = data["hash"].as_str().unwrap();
            let validator = data["miner"].as_str().unwrap();
            let txs = data["transactions"].as_array().unwrap();
            println!("The hash of this block is {}", hash);
            println!("The block was generated by {}", validator);
            println!("Exactly {} transactions", txs.len());
            println!("{:?}", txs[0]);

            let mut min_gas = u64::MAX;
            let mut max_gas = u64::MIN;
            let mut avg_gas: u64= 0;
            let mut min_gas_price = u64::MAX;
            let mut max_gas_price = u64::MIN;
            let mut avg_gas_price = 0;
            for tx in txs{
                let gas = u64::from_str_radix(tx["gas"].as_str().unwrap().trim_start_matches("0x"), 16).unwrap();
                let gas_price = u64::from_str_radix(tx["gasPrice"].as_str().unwrap().trim_start_matches("0x"), 16).unwrap();
                let tx_id = u64::from_str_radix(tx["transactionIndex"].as_str().unwrap().trim_start_matches("0x"), 16).unwrap();
                
                min_gas = min_gas.min(gas);
                max_gas = max_gas.max(gas);
                avg_gas += gas;
                min_gas_price = min_gas_price.min(gas_price);
                max_gas_price = max_gas_price.max(gas_price);
                avg_gas_price += gas_price;
                println!("tx #{}: gas used {} priced at {}", tx_id, gas, gas_price);
            }
            avg_gas /= txs.len() as u64;
            avg_gas_price /= txs.len() as u64;
            println!("Gas used: min={}, max={}, avg={}", min_gas, max_gas, avg_gas);
            println!("Gas price: min={} Gwei, max={} Gwei, avg={} Gwei", (min_gas_price as f64) / 1e9, (max_gas_price as f64) / 1e9, (avg_gas_price as f64) / 1e9);
            
        }
        else{
            println!("Error when retrieving block number");
        }
        
        //sleeping until next block
        thread::sleep(Duration::from_secs(12));
    }



}
