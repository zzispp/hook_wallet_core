use actix_web::{get, web::Json};
use serde::Serialize;
use tracing::instrument;
use std::time::{SystemTime, Instant};
use sysinfo::System;

#[derive(Serialize)]
struct ServerStatus {
    timestamp: u64,
    uptime_seconds: u64,
    os: OsInfo,
    cpu: CpuInfo,
    memory: MemoryInfo,
}

#[derive(Serialize,Clone)]
struct OsInfo {
    name: String,
    version: String,
    arch: String,
}

#[derive(Serialize)]
struct CpuInfo {
    cpu_count: usize,
    cpu_usage: f32,
}

#[derive(Serialize)]
struct MemoryInfo {
    total_memory: u64,
    used_memory: u64,
    available_memory: u64,
    memory_usage_percent: f32,
}

lazy_static::lazy_static! {
    static ref START_TIME: Instant = Instant::now();
    static ref OS_INFO: OsInfo = OsInfo {
        name: System::name().unwrap_or_else(|| "Unknown".to_string()),
        version: System::os_version().unwrap_or_else(|| "Unknown".to_string()),
        arch: std::env::consts::ARCH.to_string(),
    };
}

#[get("/")]
pub async fn server_status() -> Json<ServerStatus> {
    let now = SystemTime::now();
    let timestamp = now
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let uptime = START_TIME.elapsed().as_secs();

    // 获取系统信息
    let mut sys = System::new_all();

    // 刷新CPU信息以获取使用率
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    sys.refresh_cpu_all();

    // CPU信息
    let cpu_count = sys.cpus().len();
    let cpu_usage = sys.global_cpu_usage();

    let cpu_info = CpuInfo {
        cpu_count,
        cpu_usage,
    };

    // 内存信息
    let total_memory = sys.total_memory();
    let used_memory = sys.used_memory();
    let available_memory = sys.available_memory();
    let memory_usage_percent = if total_memory > 0 {
        (used_memory as f32 / total_memory as f32) * 100.0
    } else {
        0.0
    };

    let memory_info = MemoryInfo {
        total_memory,
        used_memory,
        available_memory,
        memory_usage_percent,
    };

    Json(ServerStatus {
        timestamp,
        uptime_seconds: uptime,
        os: OS_INFO.clone(),
        cpu: cpu_info,
        memory: memory_info,
    })
}
