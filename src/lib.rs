use serde::{Serialize,Deserialize};
use tokio::process::Command;
use std::error::Error;

use sys_info::*;

#[macro_use]
extern crate wei_log;

#[derive(Serialize, Debug)]
pub struct HardwareInfo {
    os_info: OsInfo,
    cpu_info: CpuInfo,
    gpu_info: Vec<GpuInfo>,
    mem_info: MemoryInfo,
    disks_info: Vec<DiskInfo>,
}

#[derive(Serialize, Debug)]
pub struct OsInfo {
    os_type: String,
    os_release: String,
}

#[derive(Serialize, Debug)]
pub struct CpuInfo {
    uuid: String,
    name: String,
    num: u32,
    speed: u64,
    core_num: u32
}


#[derive(Serialize, Debug)]
pub struct GpuInfo {
    index: String,
    name: String,
    uuid: String,
    gpu_bus_id: String,
    memory_used: String,
    memory_total: String,
    temperature: String,
    power_draw: String,
}

#[derive(Serialize, Debug)]
pub struct MemoryInfo {
    total: u64,
    free: u64,
    buffers: u64,
    cached: u64,
}

// 需要知道是什么类型的盘，ssd还是nvme，sata
#[derive(Serialize, Deserialize, Debug)]
pub struct DiskInfo {
    #[serde(rename = "MediaType")]
    media_type: String,
    #[serde(rename = "Model")]
    name: String,
    #[serde(rename = "Size")]
    size: u64,
}

pub async fn uuid() -> String {
    match std::fs::read_to_string(wei_env::dir_uuid()) {
        Ok(data) => data,
        Err(_) => {
            "".to_string()
        }
    }
}

pub async fn all() -> String {
    let uuid = uuid().await;

    if uuid == "" {
        return "{}".to_string();
    }

    let hardware: serde_json::Value = serde_json::from_str(&info().await).unwrap();
    let net: serde_json::Value = serde_json::from_str(&get_net_info().unwrap()).unwrap();
    let images: serde_json::Value = serde_json::from_str(&wei_run::run("wei-docker", vec!["image_list"]).unwrap()).unwrap();
    let containers: serde_json::Value = serde_json::from_str(&wei_run::run("wei-docker", vec!["container_ps"]).unwrap()).unwrap();

    let data = serde_json::json!({
        "code" : 200,
        "uuid" : uuid,
        "hardware" : hardware,
        "net" : net,
        "images" : images,
        "containers" : containers,
    });

    data.to_string()
    // serde_json::to_string_pretty(&data).unwrap()
}

pub async fn info() -> String {

    let os_info = OsInfo {
        os_type: os_type().unwrap(),
        os_release: os_release().unwrap(),
    };

    let cpu_info = get_cpu_info().await.unwrap();
    let gpu_info = get_gpu_info().await.unwrap();
    let mem_info = get_mem_info().unwrap(); 
    let disks_info = get_disk_info().unwrap();
    let hardware_info = HardwareInfo {
        os_info,
        cpu_info,
        gpu_info,
        mem_info,
        disks_info
    };

    let hardware_info_json = serde_json::to_string(&hardware_info).unwrap();

    hardware_info_json.to_string()
}

pub async fn get_cpu_info() -> Result<CpuInfo, Box<dyn Error>> {
    let cpu_serial_output = Command::new("cmd")
    .args(&["/C", "wmic cpu get ProcessorId"])
    .output().await
    .expect("执行命令失败");

    let cpu_serial_str = std::str::from_utf8(&cpu_serial_output.stdout).unwrap();

    // 解析并打印CPU序列号
    let cpu_serial = cpu_serial_str
        .lines()
        .find(|line| line.trim().len() > 0 && !line.contains("ProcessorId"))
        .unwrap_or("");

    let uuid = cpu_serial.trim();

    let output = match Command::new("cmd").args(&["/C", "wmic cpu get name"])
        .output().await
        {
            Ok(output) => output,
            Err(_) => {
                info!("wmic cpu get name 执行失败");
                return Err("执行命令失败".into());
            },
        };
    let output_str = std::str::from_utf8(&output.stdout).unwrap();
    let cpu_model = output_str.lines().find(|&line| !line.contains("Name")).unwrap_or("");
    let name = cpu_model.trim();

    let cpu_sockets_output = Command::new("cmd")
        .args(&["/C", "wmic cpu get SocketDesignation"])
        .output().await
        .expect("执行命令失败");
    let cpu_sockets_str = std::str::from_utf8(&cpu_sockets_output.stdout).unwrap();
    let num = cpu_sockets_str
        .lines()
        .filter(|line| line.trim().len() > 0 && !line.contains("SocketDesignation"))
        .count();

    let speed = match sys_info::cpu_speed() {
        Ok(speed) => speed,
        Err(_) => 0,
    };

    let core_num = match sys_info::cpu_num() {
        Ok(num) => num,
        Err(_) => 0,
    };

    Ok(CpuInfo {
        uuid: uuid.to_string(),
        name: name.to_string(),
        num: num as u32,
        speed,
        core_num
    })
}

pub async fn get_gpu_info() -> Result<Vec<GpuInfo>, Box<dyn Error>> {
    // 需要先区分是N卡还是A卡，还是国产显卡，再使用不同的命令来获取信息

    let gpu_info_output = std::process::Command::new("cmd")
        .args(&["/C", "wmic path win32_videocontroller get name"])
        .output()
        .expect("执行命令失败");

    let gpu_info_str = std::str::from_utf8(&gpu_info_output.stdout).unwrap();

    // 解析并打印显卡信息
    for line in gpu_info_str.lines().skip(1) {
        let name = line.trim();
        if name.is_empty() {
            continue;
        }

        // 根据显卡名称区分不同品牌
        if name.contains("NVIDIA") {
            return Ok(nvidia().await?);
        } else if name.contains("AMD") {
            // println!("AMD显卡: {}", name);
        } else if name.contains("华为") {
            // println!("华为显卡: {}", name);
        } else {
            // println!("其他显卡: {}", name);
        }
    }

    Ok(vec![
        GpuInfo {
            index: "".to_string(),
            name: "".to_string(),
            uuid: "".to_string(),
            gpu_bus_id: "".to_string(),
            memory_used: "".to_string(),
            memory_total: "".to_string(),
            temperature: "".to_string(),
            power_draw: "".to_string(),
        }
    ])
}

async fn nvidia() -> Result<Vec<GpuInfo>, Box<dyn Error>> {
    let output = match Command::new("nvidia-smi")
    .arg("--query-gpu=index,name,uuid,gpu_bus_id,memory.used,memory.total,temperature.gpu,power.draw")
    .arg("--format=csv,noheader")
    .output()
    .await {
        Ok(output) => output,
        Err(_) => {
            info!("nvidia-smi 执行失败");
            return Err("执行命令失败".into());
        },
    };

    if output.status.success() {
        let output = String::from_utf8_lossy(&output.stdout);

        let gpu_info: Vec<GpuInfo> = split_gpu_info(&output)
            .into_iter()
            .map(|info| GpuInfo {
                index: info[0].clone(),
                name: info[1].clone(),
                uuid: info[2].clone(),
                gpu_bus_id: info[3].clone(),
                memory_used: info[4].clone(),
                memory_total: info[5].clone(),
                temperature: info[6].clone(),
                power_draw: info[7].clone(),
            })
            .collect();

        Ok(gpu_info)
    } else {
        Err(format!("Command failed with error: {:?}", output.stderr).into())
    }
}

fn split_gpu_info(gpu_info: &str) -> Vec<Vec<String>> {
    gpu_info
        .lines()
        .map(|line| {
            line.split(',')
                .map(|s| s.trim().to_string())
                .collect()
        })
        .collect()
}

pub async fn enable_gpu_persistence_mode() -> Result<(), Box<dyn Error>> {
    let output = Command::new("nvidia-smi")
        .arg("-pm")
        .arg("1")
        .output()
        .await?;

    if output.status.success() {
        Ok(())
    } else {
        Err(format!("Failed to enable GPU persistence mode: {:?}", output.stderr).into())
    }
}

pub fn get_mem_info() -> Result<MemoryInfo, Box<dyn Error>> {
    let mem = mem_info()?;

    let memory_output = std::process::Command::new("cmd")
        .args(&["/C", "wmic ComputerSystem get TotalPhysicalMemory"])
        .output()
        .expect("执行命令失败");

    let mut memory_str = std::str::from_utf8(&memory_output.stdout).unwrap();

    // 解析并打印内存总量
    if let Some(line) = memory_str.lines().nth(1) {
        memory_str = line.trim();
    }

    Ok(MemoryInfo {
        total: memory_str.parse::<u64>().unwrap(),
        free: mem.free,
        buffers: mem.buffers,
        cached: mem.cached,
    })
}

pub fn get_disk_info() -> Result<Vec<DiskInfo>, Box<dyn std::error::Error>> {
    let output = std::process::Command::new("powershell")
        .args(&[
            "Get-PhysicalDisk",
            "|",
            "Select-Object",
            "MediaType, Model, Size",
            "|",
            "ConvertTo-Json",
        ])
        .output()
        .expect("执行命令失败");

    let output_str = std::str::from_utf8(&output.stdout).unwrap();

    // 解析JSON格式的输出
    let disks: Vec<DiskInfo> = serde_json::from_str(output_str).unwrap_or_else(|_| vec![]);


    Ok(disks)
}


pub fn get_net_info() -> Result<String, Box<dyn std::error::Error>> {
    let output = match std::process::Command::new("powershell")
    .args(&[
        "Get-NetAdapter | Where-Object { $_.Status -eq 'Up' } | ForEach-Object {
            $adapter = $_
            $stats = Get-NetAdapterStatistics -Name $adapter.Name
            $ip = Get-NetIPAddress -InterfaceIndex $adapter.ifIndex | Where-Object { $_.AddressFamily -eq 'IPv4' }
            
            [PSCustomObject]@{
                name = $adapter.InterfaceDescription
                status = $adapter.Status
                mac = $adapter.MacAddress
                ip = $ip.IPAddress
                received = $stats.ReceivedBytes
                sent = $stats.SentBytes
            }
        } | ConvertTo-Json"
    ])
    .output() {
        Ok(data) => data,
        Err(_) => return Ok("".to_string())
    };

    match std::str::from_utf8(&output.stdout) {
        Ok(data) => Ok(data.to_string()),
        Err(_) => Ok("".to_string())
    }
}

