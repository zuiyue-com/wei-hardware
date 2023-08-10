use serde::{Serialize,Deserialize};
use tokio::process::Command;
use std::error::Error;

use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use std::collections::HashSet;

use reqwest::Client;

use sysinfo::{SystemExt, DiskExt};

use sys_info::*;

#[derive(Serialize)]
struct HardwareInfo {
    uuid: String,
    user: String,
    os_info: OsInfo,
    cpu_info: CpuInfo,
    gpu_info: Vec<GpuInfo>,
    mem_info: MemoryInfo,
    disks_info: Vec<DiskInfo>,
}

#[derive(Serialize)]
struct OsInfo {
    os_type: String,
    os_release: String,
}

#[derive(Serialize)]
struct CpuInfo {
    model: Vec<String>,
    count: usize,
    cpu_num: u32,
    cpu_speed: u64,
    proc_total: u64,
    load_one: f64,
    load_five: f64,
    load_fifteen: f64,
}


#[derive(Serialize)]
struct GpuInfo {
    index: String,
    name: String,
    uuid: String,
    gpu_bus_id: String,
    memory_used: String,
    memory_total: String,
    temperature: String,
    power_draw: String,
}

#[derive(Serialize)]
struct MemoryInfo {
    free: u64,
    avail: u64,
    buffers: u64,
    cached: u64,
}

#[derive(Serialize, Deserialize)]
struct DiskInfo {
    name: String,
    total_space: u64,
    available_space: u64,
    file_system: String,
}

fn main() {
    println!("Hello, world!");
}

pub async fn info() {
    let uuid = uuid();
    let user = user_read().unwrap();
    let os_info = OsInfo {
        os_type: os_type().unwrap(),
        os_release: os_release().unwrap(),
    };
    let cpu_info = get_cpu_info().await.unwrap();
    let gpu_info = get_gpu_info().await.unwrap();
    let mem_info = get_mem_info().unwrap(); 
    let disks_info = get_disk_info().unwrap();

    let hardware_info = HardwareInfo {
        uuid,
        user,
        os_info,
        cpu_info,
        gpu_info,
        mem_info,
        disks_info
    };

    let hardware_info_json = serde_json::to_string(&hardware_info).unwrap();
    send_heartbeat(&hardware_info_json).await;
}

async fn send_heartbeat(hardware_info_json: &str) {
    let client = Client::new();
    info!("{}", hardware_info_json.to_string());
    let res = client.post("https://about.zuiyue.com/heartbeat/")
        .header("Content-Type", "application/json")
        .body(hardware_info_json.to_string())
        .send()
        .await.unwrap();
    if res.status().is_success() {
        info!("成功发送心跳");
    } else {
        info!("发送心跳失败: {}", res.status());
    }
}

fn uuid() -> std::string::String {
    // 添加token文件的名字
    let binding = crate::env::uuid_dir();
    let token_file_path = Path::new(&binding);

    // 检查token文件是否存在
    if !token_file_path.exists() {
        // 创建新的token文件
        let mut file = File::create(&token_file_path).unwrap();

        // 生成唯一的token。这里我们使用uuid crate生成一个UUID作为token
        let token = uuid::Uuid::new_v4().to_string();

        // 将token写入到文件中
        file.write_all(token.as_bytes()).unwrap();
    }

    // 读取并返回 token 文件的内容
    fs::read_to_string(&token_file_path).unwrap()
}

fn user_read() -> Result<String, Box<dyn std::error::Error>> {
    let current_file = Path::new("./user.dat");
    let home_file = Path::new(&crate::env::home_dir()?).join("user.dat");
    let root_file = Path::new("/root/user.dat");
    let binding = crate::env::user_dir();
    let user_dir = Path::new(&binding);

    if let Some(line) = get_first_line_from_file(current_file)? {
        return Ok(line.to_string());
    } else if let Some(line) = get_first_line_from_file(&home_file)? {
        return Ok(line.to_string());
    } else if let Some(line) = get_first_line_from_file(root_file)? {
        return Ok(line.to_string());
    } else if let Some(line) = get_first_line_from_file(user_dir)? {
        return Ok(line.to_string());
    } else {
        return Ok("".to_string());
    }
}

fn get_first_line_from_file(path: &Path) -> std::io::Result<Option<String>> {
    match fs::read_to_string(path) {
        Ok(content) => Ok(content.lines().next().map(|s| s.trim().to_string())),
        Err(e) => {
            if e.kind() == std::io::ErrorKind::NotFound {
                Ok(None)
            } else {
                Err(e)
            }
        },
    }
}

async fn get_cpu_info() -> Result<CpuInfo, Box<dyn Error>> {
    let mut models = HashSet::new();
    let mut physical_ids = HashSet::new();

    let cpuinfo = fs::read_to_string("/proc/cpuinfo")?;
    for line in cpuinfo.lines() {
        if line.starts_with("model name") {
            let model: Vec<&str> = line.split(": ").collect();
            if model.len() > 1 {
                models.insert(model[1].to_string());
            }
        } else if line.starts_with("physical id") {
            let id: Vec<&str> = line.split(": ").collect();
            if id.len() > 1 {
                physical_ids.insert(id[1].to_string());
            }
        }
    }

    let cpu_num_val = cpu_num().unwrap();
    let cpu_speed_val = cpu_speed().unwrap();
    let proc_total_val = proc_total().unwrap();

    let load = loadavg().unwrap();

    Ok(CpuInfo {
        model: models.into_iter().collect(),
        count: physical_ids.len(),
        cpu_num: cpu_num_val,
        cpu_speed: cpu_speed_val,
        proc_total: proc_total_val,
        load_one: load.one,
        load_five: load.five,
        load_fifteen: load.fifteen,
    })
}

async fn get_gpu_info() -> Result<Vec<GpuInfo>, Box<dyn Error>> {
    let output = Command::new("nvidia-smi")
        .arg("--query-gpu=index,name,uuid,gpu_bus_id,memory.used,memory.total,temperature.gpu,power.draw")
        .arg("--format=csv,noheader")
        .output()
        .await?;

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

fn get_mem_info() -> Result<MemoryInfo, Box<dyn Error>> {
    let mem = mem_info()?;

    Ok(MemoryInfo {
        free: mem.free,
        avail: mem.avail,
        buffers: mem.buffers,
        cached: mem.cached,
    })
}

fn get_disk_info() -> Result<Vec<DiskInfo>, Box<dyn std::error::Error>> {
    let system = sysinfo::System::new_all();
    
    let mut disks_info = Vec::new();
    for disk in system.disks() {
        disks_info.push(DiskInfo {
            name: disk.name().to_string_lossy().into(),
            total_space: disk.total_space(),
            available_space: disk.available_space(),
            file_system: String::from_utf8(disk.file_system().to_vec())?,
        });
    }
    Ok(disks_info)
}


