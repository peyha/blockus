use reqwest;
use std::time::{SystemTime};


fn main() {
   
    let request_url = format!("https://api.etherscan.io/api?module=block&action=getblocknobytime&timestamp={timestamp}&closest=before&apikey={apiKey}",
        timestamp=SystemTime.now(),
        apiKey='UBI18HPD3AFGC3QH6FZCHX423MKMYXHVCP');
    
    
}
