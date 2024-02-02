use serde_json::{json};

#[macro_use]
extern crate wei_log;

#[tokio::main]
async fn main() {
    wei_windows::init();
    wei_env::bin_init("wei-hardware");

    use single_instance::SingleInstance;
    let instance = SingleInstance::new("wei-hardware").unwrap();
    if !instance.is_single() { 
        std::process::exit(1);
    };

    tokio::time::sleep(tokio::time::Duration::from_secs(15)).await;
    loop {
        let config_data: serde_json::Value = serde_json::from_str(&wei_hardware::all().await).unwrap();
        let client = reqwest::Client::new();

        let server_url = wei_api::server_url().unwrap();
    
        let mut body = wei_api::server_data("").unwrap();
        body["modac"] = json!("iam");
        body["tech_type"] = json!("docker");
        body["config"] = config_data;

        let body = body.to_string();

        info!("hardware url:{}", server_url);
        info!("hardware body:{}", body);
        
        let response = match client.post(server_url).body(body).send().await {
            Ok(response) => response,
            Err(e) => {
                info!("hardware network error: {}", e);
                tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
                continue;
            }
        };

        info!("hardware response:{}", &response.text().await.unwrap());
        tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
    }
}


