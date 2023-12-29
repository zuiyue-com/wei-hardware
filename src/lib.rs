use serde::{Serialize,Deserialize};
use serde_json::{json};
use serde_json::Value;
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
    version: String,
    bitness: String
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
    info!("check: hardware");
    let hardware_path = format!("{}cache/hardware.json",wei_env::home_dir().unwrap());
    let mut hardware = read_file_if_recent(hardware_path.clone(), 30 * 60).unwrap();
    if hardware == "" {
        hardware = info().await;
        write_to_file(hardware_path.clone(), &hardware).unwrap();
    }
    let hardware: serde_json::Value = match serde_json::from_str(&hardware) {
        Ok(data) => data,
        Err(_) => {
            let data:Value = serde_json::from_str(&info().await).unwrap();
            write_to_file(hardware_path, &data.to_string()).unwrap();
            data
        },
    };

    info!("check: net");
    let net_path = format!("{}cache/net.json",wei_env::home_dir().unwrap());
    let mut net = read_file_if_recent(net_path.clone(), 30 * 60).unwrap();
    if net == "" {
        net = get_net_info().unwrap();
        write_to_file(net_path, &net).unwrap();
    }
    let net: serde_json::Value = match serde_json::from_str(&net) {
        Ok(data) => data,
        Err(_) => serde_json::from_str(&get_net_info().unwrap()).unwrap()
    };

    info!("check: models");
    let models_path = format!("{}cache/models.json",wei_env::home_dir().unwrap());
    let mut models = read_file_if_recent(models_path.clone(), 10 * 60).unwrap();
    if models == "" {
        models = get_file_info("models".to_string());
        write_to_file(models_path, &models).unwrap();
    }
    let models: serde_json::Value = match serde_json::from_str(&models) {
        Ok(data) => data,
        Err(_) => serde_json::from_str(&get_file_info("models".to_string())).unwrap()
    };

    info!("check: ip");
    let ip_path = format!("{}cache/ip.json",wei_env::home_dir().unwrap());
    let mut ip = read_file_if_recent(ip_path.clone(), 30 * 60).unwrap();
    if ip == "" {
        ip = get_ip_info().await;
        write_to_file(ip_path, &ip).unwrap();
    }
    let ip: serde_json::Value = match serde_json::from_str(&ip) {
        Ok(data) => data,
        Err(_) => serde_json::from_str(&get_ip_info().await).unwrap()
    };
    

    info!("check: docker images");
    let images: serde_json::Value = serde_json::from_str(&wei_run::run("wei-docker", vec!["image_list_full"]).unwrap()).unwrap();
    info!("check: containers");
    let containers: serde_json::Value = serde_json::from_str(&wei_run::run("wei-docker", vec!["container_ps"]).unwrap()).unwrap();

    let data = serde_json::json!({
        "hardware" : hardware,
        "network" : net,
        "images" : images,
        "containers" : containers,
        "models" : models,
        "ip" : ip,
    });

    data.to_string()
}

pub async fn info() -> String {

    let info = os_info::get();

    let os_info = OsInfo {
        os_type: info.os_type().to_string(),
        version: info.version().to_string(),
        bitness: info.bitness().to_string(),
    };

    let cpu_info = match get_cpu_info().await {
        Ok(cpu_info) => cpu_info,
        Err(_) => {
            info!("获取CPU信息失败");
            CpuInfo {
                uuid: "".to_string(),
                name: "".to_string(),
                num: 0,
                speed: 0,
                core_num: 0
            }
        },
    };

    let gpu_info = get_gpu_info().await.unwrap();
    let mem_info = match get_mem_info() {
        Ok(mem_info) => mem_info,
        Err(_) => {
            info!("获取内存信息失败");
            MemoryInfo {
                total: 0,
                free: 0,
                buffers: 0,
                cached: 0,
            }
        },
    }; 
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
    let cpu_serial_output = match std::process::Command::new("cmd")
    .args(&["/C", "wmic cpu get ProcessorId"])
    .output() {
        Ok(output) => output,
        Err(_) => {
            info!("wmic cpu get ProcessorId 执行失败");
            return Err("执行命令失败".into());
        },
    };

    let cpu_serial_str = std::str::from_utf8(&cpu_serial_output.stdout).unwrap();

    // 解析并打印CPU序列号
    let cpu_serial = cpu_serial_str
        .lines()
        .find(|line| line.trim().len() > 0 && !line.contains("ProcessorId"))
        .unwrap_or("");

    let uuid = cpu_serial.trim();

    let output = match std::process::Command::new("cmd").args(&["/C", "wmic cpu get name"])
        .output()
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

    let cpu_sockets_output = match std::process::Command::new("cmd")
        .args(&["/C", "wmic cpu get SocketDesignation"])
        .output() {
            Ok(output) => output,
            Err(_) => {
                info!("wmic cpu get SocketDesignation 执行失败");
                return Err("执行命令失败".into());
            },
        };
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

    let gpu_info_output = match std::process::Command::new("cmd")
        .args(&["/C", "wmic path win32_videocontroller get name"])
        .output() {
            Ok(output) => output,
            Err(_) => {
                info!("wmic path win32_videocontroller get name 执行失败");
                return Err("执行命令失败".into());
            },
        };

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

    Ok(vec![])

    // Ok(vec![
    //     GpuInfo {
    //         index: "".to_string(),
    //         name: "".to_string(),
    //         uuid: "".to_string(),
    //         gpu_bus_id: "".to_string(),
    //         memory_used: "".to_string(),
    //         memory_total: "".to_string(),
    //         temperature: "".to_string(),
    //         power_draw: "".to_string(),
    //     }
    // ])
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

    let memory_output = match std::process::Command::new("cmd")
        .args(&["/C", "wmic ComputerSystem get TotalPhysicalMemory"])
        .output() {
            Ok(output) => output,
            Err(_) => {
                info!("wmic ComputerSystem get TotalPhysicalMemory 执行失败");
                return Err("执行命令失败".into());
            }
        };

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
    let output = match std::process::Command::new("powershell")
        .args(&[
            "Get-PhysicalDisk",
            "|",
            "Select-Object",
            "MediaType, Model, Size",
            "|",
            "ConvertTo-Json",
        ])
        .output() {
            Ok(output) => output,
            Err(_) => {
                info!("powershell 执行失败");
                return Err("执行命令失败".into());
            }
        };

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

    let data = match std::str::from_utf8(&output.stdout) {
        Ok(data) => data.to_string(),
        Err(_) => "".to_string()
    };

    if data.starts_with('{') {
        Ok(format!("[{}]", data))
    } else {
        Ok(data)
    }
}


use std::fs::{self, DirEntry};
use std::path::Path;
use std::io;
use std::time::SystemTime;

#[derive(Serialize)]
pub struct FileInfo {
    path: String,
    size: u64,
    creation_time: u64,
}

pub fn get_file_info(path: String) -> String {
    let path = Path::new(&path);
    let mut files_info = Vec::new();

    visit_dirs(path, &mut files_info).unwrap();
    let json = serde_json::to_string_pretty(&files_info).unwrap();
    json
}

pub fn visit_dirs(dir: &Path, files_info: &mut Vec<FileInfo>) -> io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                visit_dirs(&path, files_info)?;
            } else {
                let info = file_info(&entry);
                files_info.push(info);
            }
        }
    }
    Ok(())
}

pub fn file_info(entry: &DirEntry) -> FileInfo {
    let path = entry.path();
    let metadata = entry.metadata().unwrap_or_else(|_| panic!("无法获取元数据"));

    let file_size = metadata.len();
    let creation_time = metadata.created()
        .unwrap_or(SystemTime::UNIX_EPOCH)
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    FileInfo {
        path: path.to_string_lossy().into_owned(),
        size: file_size,
        creation_time: creation_time,
    }
}


use std::fs::{OpenOptions};
use std::io::{Read, Write};
use std::time::{Duration};

fn read_file_if_recent<P: AsRef<Path>>(file_path: P, max_age_secs: u64) -> io::Result<String> {
    let path = file_path.as_ref();

    // 检查并创建文件夹（如果需要）
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)?;
        }
    }

    create_file_if_not_exists(path)?;

    let last_modified = fs::metadata(&path)
        .and_then(|metadata| metadata.modified())
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    // 检查文件是否在指定时间内被修改
    if SystemTime::now().duration_since(last_modified)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))? < Duration::from_secs(max_age_secs) {
        // 文件在指定时间内被修改，读取并返回内容
        let mut file = fs::File::open(&path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        Ok(contents)
    } else {
        // 文件超出指定时间，返回空字符串
        Ok(String::new())
    }
}

fn create_file_if_not_exists<P: AsRef<Path>>(file_path: P) -> io::Result<()> {
    let path = file_path.as_ref();

    if !path.exists() {
        // 如果文件不存在，则创建一个新文件
        File::create(path)?;
    }

    Ok(())
}

use std::fs::File;
fn write_to_file<P: AsRef<Path>>(file_path: P, content: &str) -> io::Result<()> {
    // 删除文件
    if file_path.as_ref().exists() {
        fs::remove_file(&file_path)?;
    }

    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .open(file_path)?;


    file.write_all(content.as_bytes())?;

    Ok(())
}

pub async fn get_ip_info() -> String {

    match ip_chaxun().await {
        Ok(data) => {
            return data;
        },
        Err(_) => {}
    }

    match ip_pconline().await {
        Ok(data) => {
            return data;
        },
        Err(_) => {}
    }

    match ip_csdn().await {
        Ok(data) => {
            return data;
        },
        Err(_) => {
            return "{}".to_string();
        }
    }
}

async fn ip_chaxun() -> Result<String, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let mut header = reqwest::header::HeaderMap::new();
    header.insert("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 \
    (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36".parse().unwrap());
    let res = client.get("https://2023.ipchaxun.com").headers(header).send().await?;
    let body = res.text().await?;

    let body: Value = match serde_json::from_str(&body) {
        Ok(data) => data,
        Err(_) => {
            return Ok("".to_string());
        }
    };

    Ok(json!({
        "ipsite" : "ipchaxun.com",
        "data" : body
    }).to_string())
}

pub async fn ip_pconline() -> Result<String, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let mut header = reqwest::header::HeaderMap::new();
    header.insert("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 \
    (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36".parse().unwrap());
    let res = client.get("https://whois.pconline.com.cn/ipJson.jsp?ip=&json=true").headers(header).send().await?;
    let body = res.text().await?;
    let body: Value = match serde_json::from_str(&body) {
        Ok(data) => data,
        Err(_) => {
            return Ok("".to_string());
        }
    };

    Ok(json!({
        "ipsite" : "pconline.com.cn",
        "data" : body
    }).to_string())
}

pub async fn ip_csdn() -> Result<String, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let mut header = reqwest::header::HeaderMap::new();
    header.insert("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 \
    (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36".parse().unwrap());
    let res = client.get("https://searchplugin.csdn.net/api/v1/ip/get?ip").headers(header).send().await?;
    let body = res.text().await?;

    let body: Value = match serde_json::from_str(&body) {
        Ok(data) => data,
        Err(_) => {
            return Ok("".to_string());
        }
    };

    Ok(json!({
        "ipsite" : "csdn.net",
        "data" : body
    }).to_string())
}