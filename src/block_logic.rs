use std::{fmt::Display, num::ParseIntError};
use json::parse;
use serde_json::Value;
use serde::Serialize;
use std::error::Error;
use reqwest;
use crate::utils::{format_generic, parse_hexa_value};


#[derive(Serialize)]
struct GetBlockRequest{
    params: (String, bool),
    jsonrpc: String,
    method: String,
    id: String
}

// Struct for GetBlockNumber RPC call
#[derive(Serialize)]
struct GetBlockNumberRequest{
    params: (),
    jsonrpc: String,
    method: String,
    id: String
}

pub enum BlockRequestError{
    RequestError(reqwest::Error),
    ConversionError(&'static str),
    IntConversionError(ParseIntError),
    JsonConversionError(serde_json::Error),
}

// Retrieves the info of the block
pub async fn get_block_info(block: u64, now: u64, url: &str) -> Result<Vec<String>, BlockRequestError>{
    let mut texts = Vec::new();
    texts.push(String::from("System info:"));
    texts.push(String::from(format!("---Timestamp: {}", now)));

    // Performs GetBlockRequest

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
                            .await
                            .map_err(BlockRequestError::RequestError)?
                            .text()
                            .await
                            .map_err(BlockRequestError::RequestError)?;


    let raw_data: Value = serde_json::from_str(&res)
        .map_err(BlockRequestError::JsonConversionError)?;
    let data = raw_data["result"].clone();
    let hash = data["hash"].as_str().ok_or(BlockRequestError::ConversionError("hash to str"))?;
    let validator = data["miner"].as_str().ok_or(BlockRequestError::ConversionError("miner address to str"))?;
    let txs = data["transactions"].as_array().ok_or(BlockRequestError::ConversionError("transactions to array"))?;

    let block_size = parse_hexa_value(&data["size"])? / 1000;
    let block_timestamp = parse_hexa_value(&data["timestamp"])?;

    // Creates block info section
    texts.push(String::from(""));
    texts.push(String::from("Block info:"));
    texts.push(String::from(format!("---Block timestamp: {}", block_timestamp)));
    texts.push(String::from(format!("---Block number: {}", block)));
    texts.push(String::from(format!("---Block hash: {}", hash)));
    texts.push(String::from(format!("---Block validator: {}", validator)));
    texts.push(String::from(format!("---Block size: {} kb", block_size)));
    texts.push(String::from(""));

    // Creates transaction info section
    texts.push(String::from("Txs info:"));
    texts.push(String::from(format!("---Tx nb: {}", txs.len())));
    

    let mut min_gas = u64::MAX;
    let mut max_gas = u64::MIN;
    let mut avg_gas: u64 = 0;
    let mut min_gas_price = u64::MAX;
    let mut max_gas_price = u64::MIN;
    let mut avg_gas_price = 0;
    let mut type_count: (u64, u64, u64, u64) = (0, 0, 0, 0);
    
    for tx in txs{

        let gas = parse_hexa_value(&tx["gas"])?;
        let gas_price = parse_hexa_value(&tx["gasPrice"])?;
        let tx_type = parse_hexa_value(&tx["type"])?;
        

        min_gas = min_gas.min(gas);
        max_gas = max_gas.max(gas);
        avg_gas += gas;
        min_gas_price = min_gas_price.min(gas_price);
        max_gas_price = max_gas_price.max(gas_price);
        avg_gas_price += gas_price;

        match tx_type {
            0 => type_count.0 += 1,
            1 => type_count.1 += 1,
            2 => type_count.2 += 1,
            3 => type_count.3 += 1,
            _ => (),
        };

    }

    if txs.len() > 0 {   
        avg_gas /= txs.len() as u64;
        avg_gas_price /= txs.len() as u64;
    }

    texts.push(String::from(format!("---transfer: {}, deployment: {}, execution: {}, blob: {}", type_count.0, type_count.1, type_count.2, type_count.3)));

    // Creates gas info section
    texts.push(String::from(""));
    texts.push(String::from("Gas info:"));

    let gas_used = parse_hexa_value(&data["gasUsed"])?;
    let gas_max = parse_hexa_value(&data["gasLimit"])?;
    let gas_target = gas_max / 2;
    
    // EIP-1559 Feature
    let target_diff = (100 as f64)*((gas_used as f64) - (gas_target as f64)) / (gas_target as f64);
    let max_diff = (100 as f64)* (gas_used as f64) / (gas_max as f64);
    texts.push(String::from(format!("---Gas target: {}, Gas total usage {}", format_generic(gas_target as u32), format_generic(gas_used as u32))));
    texts.push(String::from(format!("---Gas objective {:.2}% from target, {:.2}% of maximum", target_diff, max_diff)));
    if target_diff < 0. {
        texts.push(String::from("---Base fee will decrease"))
    }
    else{
        texts.push(String::from("---Base fee will increase"));
    }

    texts.push(String::from(format!("---Gas usage: min={}, max={}, avg={}", format_generic(min_gas as u32), format_generic(max_gas as u32), format_generic(avg_gas as u32))));
    texts.push(String::from(format!("---Gas price: min={:.2} Gwei, max={:.2} Gwei, avg={:.2} Gwei", 
                                    (min_gas_price as f64) / 1e9, 
                                    (max_gas_price as f64) / 1e9, 
                                    (avg_gas_price as f64) / 1e9)));
    

    let base_fee = parse_hexa_value(&data["baseFeePerGas"])?;
    texts.push(String::from(format!("---Base fee: {:.2} gwei", (base_fee as f64) * 1e-9)));
    texts.push(String::from(format!("---Priority fee: min={:.2} Gwei, max={:.2} Gwei, avg {:.2} gwei", (min_gas_price as f64) * 1e-9 - (base_fee as f64) * 1e-9,
                                                (max_gas_price as f64) * 1e-9 - (base_fee as f64) * 1e-9,
                                                (avg_gas_price as f64) * 1e-9 - (base_fee as f64) * 1e-9)));
    
    //  blobversionedhashes (in tx) maxfeeperblobgas (in tx), 
    // Creates blob info section
    // EIP-4844 Feature
    if data.get("blobGasUsed").is_some() {
        texts.push(String::from(""));
        texts.push(String::from("Blob info:"));

        let blob_gas_used = parse_hexa_value(&data["blobGasUsed"])?;
        let excess_blob_gas = parse_hexa_value(&data["excessBlobGas"])?;
        texts.push(String::from(format!("---Blob gas used: {}", format_generic(blob_gas_used as u32))));
        texts.push(String::from(format!("---Excess blob gas: {}", format_generic(excess_blob_gas as u32))));
        let blob_gas_target: u64 = 393216;
        let blob_gas_max: u64 = 786432;
        let blob_target_diff = (100 as f64)*((blob_gas_used as f64) - (blob_gas_target as f64)) / (blob_gas_target as f64);
        let blob_max_diff = (100 as f64)* (blob_gas_used as f64) / (blob_gas_max as f64);      
        texts.push(String::from(format!("---Blob gas target: {}, Blob gas total usage {}, Blob excess gas {}", format_generic(blob_gas_target as u32), format_generic(blob_gas_used as u32), format_generic(excess_blob_gas as u32))));
        texts.push(String::from(format!("---Blob gas objective {:.2}% from target, {:.2}% of maximum", blob_target_diff, blob_max_diff)));
        if blob_target_diff < 0. {
            texts.push(String::from("---Blob base fee will decrease"));
        }
        else {
            texts.push(String::from("---Blob gas fee will increase"));
        }

    }
    Ok(texts)
}

// Gets the last block number 
pub async fn get_block_number(url: &str) -> Result<u64, BlockRequestError>{
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

    let block = u64::from_str_radix(data["result"].as_str().ok_or(BlockRequestError::ConversionError("block number result to str"))?
        .trim_start_matches("0x"), 16).map_err(BlockRequestError::IntConversionError)?;    
    
    Ok(block)
}


