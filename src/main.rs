use reqwest;
use std::time::SystemTime;
use serde_json::Value;
use serde::Serialize;
use clap::Parser;
use std::u64;
use std::thread;
use std::time::Duration;
use std::error::Error;
use std::fmt::Display;

#[derive(Serialize)]
struct GetBlockRequest{
    params: (String, bool),
    jsonrpc: String,
    method: String,
    id: String
}

#[derive(Serialize)]
struct GetBlockNumberRequest{
    params: (),
    jsonrpc: String,
    method: String,
    id: String
}

#[derive(Parser)]
#[command(author, version, about)]
struct Cli{
    #[arg(short, long)]
    rpc_url: String,
}

fn format_generic<T: Into<f64> + Display>(value: T) -> String {
    const K: f64 = 1_000.0;
    const M: f64 = 1_000_000.0;
    const B: f64 = 1_000_000_000.0;

    let value = value.into();

    if value < K {
        return format!("{}", value);
    } else if value < M {
        return format!("{:.1}K", value / K);
    } else if value < B {
        return format!("{:.1}M", value / M);
    } else {
        return format!("{:.1}B", value / B);
    }
}

fn print_in_box(texts: Vec<String>){
    let max_len = texts.iter().map(|s| s.len()).max().unwrap_or(0);
    let horizontal_line = format!("#{:-<width$}#", "", width=max_len);
    println!("{}", horizontal_line);

    for text in texts{
        let line = format!("#{:-<width$}#", text, width=max_len);
        println!("{}", line);
    }
    println!("{}", horizontal_line);
}

async fn get_block_info(block: u64, now: u64, url: &str) -> Result<Vec<String>, Box<dyn Error>>{
    let mut texts = Vec::new();
    texts.push(String::from("System info:"));
    texts.push(String::from(format!("---Timestamp: {}", now)));

    let request = GetBlockRequest{
        params: (format!("0x{:x}", block), true),
        jsonrpc: String::from("2.0"),
        method: String::from("eth_getBlockByNumber"),
        id: String::from("1"),
    };

    let client = reqwest::Client::new();
    let res = client.post(url)
                            .json(&request)
                            .send()
                            .await?
                            .text()
                            .await?;

    let raw_data: Value = serde_json::from_str(&res)?;
    let data = raw_data["result"].clone();
    let hash = data["hash"].as_str().ok_or("fail convert")?;
    let validator = data["miner"].as_str().ok_or("fail convert")?;
    let txs = data["transactions"].as_array().ok_or("fail convert")?;

    let block_size = u64::from_str_radix(
                                data["size"].as_str().ok_or("fail convert")?
                                                .trim_start_matches("0x"), 
                                16)? / 1000;
    let block_timestamp = u64::from_str_radix(data["timestamp"].as_str().ok_or("fail convert")?.trim_start_matches("0x"), 16)?;
    //println!("{:?}", txs[0]);
    texts.push(String::from(""));
    texts.push(String::from("Block info:"));
    texts.push(String::from(format!("---Block timestamp: {}", block_timestamp)));
    texts.push(String::from(format!("---Block number: {}", block)));
    texts.push(String::from(format!("---Block hash: {}", hash)));
    texts.push(String::from(format!("---Block validator: {}", validator)));
    texts.push(String::from(format!("---Block size: {} kb", block_size)));
    texts.push(String::from(""));
    texts.push(String::from("Txs info:"));
    texts.push(String::from(format!("---Tx nb: {}", txs.len())));
    

    let mut min_gas = u64::MAX;
    let mut max_gas = u64::MIN;
    let mut avg_gas: u64 = 0;
    let mut sum_gas: u64 = 0;
    let mut min_gas_price = u64::MAX;
    let mut max_gas_price = u64::MIN;
    let mut avg_gas_price = 0;
    let mut type_count: (u64, u64, u64) = (0, 0, 0);
    //println!("{}", data);
    for tx in txs{
        let gas = u64::from_str_radix(tx["gas"].as_str().ok_or("fail convert")?.trim_start_matches("0x"), 16)?;
        let gas_price = u64::from_str_radix(tx["gasPrice"].as_str().ok_or("fail convert")?.trim_start_matches("0x"), 16)?;
        let tx_type: u64 = u64::from_str_radix(tx["type"].as_str().ok_or("fail convert")?.trim_start_matches("0x"), 16)?;

        min_gas = min_gas.min(gas);
        max_gas = max_gas.max(gas);
        avg_gas += gas;
        sum_gas += gas;
        min_gas_price = min_gas_price.min(gas_price);
        max_gas_price = max_gas_price.max(gas_price);
        avg_gas_price += gas_price;

        match tx_type {
            0 => type_count.0 += 1,
            1 => type_count.1 += 1,
            2 => type_count.2 += 1,
            _ => (),
        };

    }

    if txs.len() > 0 {   
        avg_gas /= txs.len() as u64;
        avg_gas_price /= txs.len() as u64;
    }

    texts.push(String::from(format!("---transfer: {}, deployment: {}, execution: {}", type_count.0, type_count.1, type_count.2)));

    texts.push(String::from(""));
    texts.push(String::from("Gas info:"));

    let gas_used = u64::from_str_radix(data["gasUsed"].as_str().ok_or("fail convert")?.trim_start_matches("0x"), 16)?;

    let gas_max = u64::from_str_radix(data["gasLimit"].as_str().ok_or("fail convert")?.trim_start_matches("0x"), 16)?;
    let gas_target = gas_max / 2;
     
    let target_diff = (100 as f64)*((gas_used as f64) - (gas_target as f64)) / (gas_target as f64);
    let max_diff = (100 as f64)* (gas_used as f64) / (gas_max as f64);
    texts.push(String::from(format!("---Gas target: {}, Gas total usage {}", format_generic(gas_target as u32), format_generic(gas_used as u32))));
    texts.push(String::from(format!("---Gas objective {:.2}% from target, {:.2}% of maximum", target_diff, max_diff)));
    if target_diff < 0. {
        texts.push(String::from("---Block size will increase"))
    }
    else{
        texts.push(String::from("---Block size will decrease"));
    }

    texts.push(String::from(format!("---Gas usage: min={}, max={}, avg={}", format_generic(min_gas as u32), format_generic(max_gas as u32), format_generic(avg_gas as u32))));

    //println!("Gas target: sum={}, {}% from target, {}% from max", sum_gas, target_diff, max_diff);
    texts.push(String::from(format!("---Gas price: min={:.2} Gwei, max={:.2} Gwei, avg={:.2} Gwei", 
                                    (min_gas_price as f64) / 1e9, 
                                    (max_gas_price as f64) / 1e9, 
                                    (avg_gas_price as f64) / 1e9)));
    

    let base_fee = u64::from_str_radix(
        data["baseFeePerGas"].as_str().ok_or("fail_convert")?
                                .trim_start_matches("0x"),
            16)?;
                
    texts.push(String::from(format!("---Base fee: {:.2} gwei", (base_fee as f64) * 1e-9)));
    texts.push(String::from(format!("---Priority fee: min={:.2} Gwei, max={:.2} Gwei, avg {:.2} gwei", (min_gas_price as f64) * 1e-9 - (base_fee as f64) * 1e-9,
                                                (max_gas_price as f64) * 1e-9 - (base_fee as f64) * 1e-9,
                                                (avg_gas_price as f64) * 1e-9 - (base_fee as f64) * 1e-9)));
    Ok(texts)
}

async fn get_block_number(url: &str) -> Result<u64, Box<dyn Error>>{
    let request = GetBlockNumberRequest{
        params: (),
        jsonrpc: String::from("2.0"),
        method: String::from("eth_blockNumber"),
        id: String::from("1")
    };
    let client = reqwest::Client::new();
    let res = client.post(url)
                            .json(&request)
                            .send()
                            .await
                            .unwrap()
                            .text()
                            .await.
                            unwrap();
    let data: Value = serde_json::from_str(&res).unwrap();

    println!("Catching up to the last block");
    let block = u64::from_str_radix(data["result"].as_str().ok_or("fail convert")?.trim_start_matches("0x"), 16)?;    
    
    Ok(block)
}

#[tokio::main]
async fn main() {
    let args = Cli::parse();
    let mut block: u64 = 0;

    loop{
        let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();       
        if block % 20 == 0 {

            match get_block_number(args.rpc_url.as_str()).await{
                Ok(block_number) => block = block_number,
                _ => {
                    println!("Error at time {}", now);
                    continue;
                }
            }
        }
        match get_block_info(block, now, args.rpc_url.as_str()).await{
            Ok(texts) => {
                print_in_box(texts);
                //sleeping until next block
                thread::sleep(Duration::from_secs(12));
                print!("{}[2J", 27 as char);
                block += 1;
            }
            _ => println!("Error on block {}", block),
        };  
        

    }



}
