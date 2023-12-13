#[tokio::main]
async fn main() {
    let data: serde_json::Value = serde_json::from_str(&wei_hardware::all().await).unwrap();
    println!("{}", serde_json::to_string_pretty(&data).unwrap());
}

