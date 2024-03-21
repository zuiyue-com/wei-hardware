use serde_json::{json};

#[macro_use]
extern crate wei_log;

#[tokio::main]
async fn main() {

    use sysinfo::{
        Components, Disks, Networks, System,
    };
    
    // Please note that we use "new_all" to ensure that all list of
    // components, network interfaces, disks and users are already
    // filled!
    let mut sys = System::new_all();
    
    // First we update all information of our `System` struct.
    sys.refresh_all();
    
    println!("=> system:");
    // RAM and swap information:
    println!("total memory: {} bytes", sys.total_memory());
    println!("used memory : {} bytes", sys.used_memory());
    println!("total swap  : {} bytes", sys.total_swap());
    println!("used swap   : {} bytes", sys.used_swap());
    
    // Display system information:
    println!("System name:             {:?}", System::name());
    println!("System kernel version:   {:?}", System::kernel_version());
    println!("System OS version:       {:?}", System::os_version());
    println!("System host name:        {:?}", System::host_name());
    
    // Number of CPUs:
    // println!("NB CPUs: {}", sys.cpus().len());
    
    // Display processes ID, name na disk usage:
    // for (pid, process) in sys.processes() {
    //     println!("[{pid}] {} {:?}", process.name(), process.disk_usage());
    // }
    
    
    // Network interfaces name, total data received and total data transmitted:
    // let networks = Networks::new_with_refreshed_list();
    // println!("=> networks:");
    // for (interface_name, data) in &networks {
    //     println!("{:?}", data);
        // println!(
        //     "{interface_name}: {} B (down) / {} B (up)",
        //     data.total_received(),
        //     data.total_transmitted(),
        // );
        // If you want the amount of data received/transmitted since last call
        // to `Networks::refresh`, use `received`/`transmitted`.
    // }
    
    



    wei_windows::init();
    wei_env::bin_init("wei-hardware");

    let instance = wei_single::SingleInstance::new("wei-hardware").unwrap();
    if !instance.is_single() { 
        std::process::exit(1);
    };

    // tokio::time::sleep(tokio::time::Duration::from_secs(15)).await;
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



