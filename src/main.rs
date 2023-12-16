#[tokio::main]
async fn main() {
    let data: serde_json::Value = serde_json::from_str(&wei_hardware::all().await).unwrap();
    println!("{}", serde_json::to_string_pretty(&data).unwrap());
}



// use os_info::{Type};

// fn main() -> std::io::Result<()> {
//     let info = os_info::get();

//     if info.os_type() == Type::Windows {
//         // 将版本号转换为字符串并分割
//         let version_str = info.version().to_string();
//         let parts: Vec<&str> = version_str.split('.').collect();

//         let version = parts[0].parse::<u32>().unwrap();

//         if version <= 6 {
//             println!("Windows 7");
//         } else if version == 10 {
//             println!("Windows 10");
//         } else {
//             println!("Windows {}", parts[0]);
//         }
//     }

//     Ok(())
// }
