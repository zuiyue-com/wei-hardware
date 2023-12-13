#[tokio::main]
async fn main() {
    println!("{}", wei_hardware::info().await);
}
