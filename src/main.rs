// #[cfg(target_os = "windows")]
// static DATA_1: &'static [u8] = include_bytes!("../../wei-release/windows/qbittorrent/qbittorrent.exe");

// use serde_json::{json};

// #[macro_use]
// extern crate wei_log;

#[tokio::main]
async fn main() {
    // #[cfg(target_os = "windows")]
    // if std::env::args().collect::<Vec<_>>().len() > 1000 {
    //     println!("{:?}", DATA_1);
    // }   
    // wei_env::bin_init("wei-hardware");

    // tokio::time::sleep(tokio::time::Duration::from_secs(15)).await;
    // loop {
    //     let config_data: serde_json::Value = serde_json::from_str(&wei_hardware::all().await).unwrap();
    //     let client = reqwest::Client::new();

    //     let server_url = crate::api::server_url().unwrap();
    
    //     let mut body = crate::api::server_data("").unwrap();
    //     body["modac"] = json!("iam");
    //     body["tech_type"] = json!("docker");
    //     body["config"] = config_data;

    //     let body = body.to_string();

    //     info!("hardware url:{}", server_url);
    //     info!("hardware body:{}", body);
        
    //     let response = match client.post(server_url).body(body).send().await {
    //         Ok(response) => response,
    //         Err(e) => {
    //             info!("hardware network error: {}", e);
    //             tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
    //             continue;
    //         }
    //     };

    //     info!("hardware response:{}", &response.text().await.unwrap());
    //     tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
    // }
}


