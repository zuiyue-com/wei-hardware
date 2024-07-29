use serde_json::{json};

#[macro_use]
extern crate wei_log;

#[tokio::main]
async fn main() {

    // test code
    // match wei_hardware::get_gpu_info().await {
    //     Ok(data) => {
    //         println!("nvidia:{:?}", data);
    //     },
    //     Err(e) => {
    //         println!("nvidia error:{:?}", e);
    //     }
    // }


    wei_windows::init();
    wei_env::bin_init("wei-hardware");

    let instance = wei_single::SingleInstance::new("wei-hardware").unwrap();
    if !instance.is_single() { 
        std::process::exit(1);
    };

    loop {
        let config_data: serde_json::Value = serde_json::from_str(&wei_hardware::all().await).unwrap();
        let client = match reqwest::Client::builder()
        .timeout(tokio::time::Duration::from_secs(30))
        .build() {
            Ok(client) => client,
            Err(e) => {
                info!("hardware network error: {}", e);
                tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
                continue;
            }
        };

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



