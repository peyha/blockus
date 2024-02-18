use std::time::SystemTime;
use clap::Parser;
use std::u64;
use std::thread;
use std::time::Duration;

mod utils;
use utils::print_in_box;
mod block_logic;
use block_logic::{get_block_info, get_block_number};

// Struct for GetBlock RPC call

// CLI arguments
#[derive(Parser)]
#[command(author, version, about)]
struct Cli{
    #[arg(short, long, default_value_t=String::from("https://eth.merkle.io"))]
    rpc_url: String,
}

#[tokio::main]
async fn main() {
    let args = Cli::parse();
    let mut block: u64 = 0;

    loop{
        let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();       
        
        // Catching up to the last block every 20 blocks
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
