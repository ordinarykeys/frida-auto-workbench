use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AdbDevice {
    pub serial: String,
    pub state: String,
    pub model: String,
    pub abi: String,
}

fn adb_path() -> String {
    std::env::var("ADB_PATH").unwrap_or_else(|_| "adb".to_string())
}

fn run_adb(args: &[&str]) -> Result<String, String> {
    let output = Command::new(adb_path())
        .args(args)
        .output()
        .map_err(|e| format!("执行 adb 失败 (检查 ADB 是否在 PATH): {}", e))?;
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    if !output.status.success() {
        return Err(format!("adb {} 失败:\n{}\n{}", args.join(" "), stdout, stderr));
    }
    Ok(stdout)
}

#[tauri::command]
pub async fn adb_devices() -> Result<Vec<AdbDevice>, String> {
    let out = run_adb(&["devices", "-l"])?;
    let mut devices = Vec::new();
    for line in out.lines().skip(1) {
        let line = line.trim();
        if line.is_empty() { continue; }
        let mut parts = line.split_whitespace();
        let serial = parts.next().unwrap_or("").to_string();
        if serial.is_empty() { continue; }
        let state = parts.next().unwrap_or("unknown").to_string();
        let mut model = String::new();
        for p in parts {
            if let Some(stripped) = p.strip_prefix("model:") {
                model = stripped.to_string();
            }
        }
        let abi = run_adb(&["-s", &serial, "shell", "getprop", "ro.product.cpu.abi"])
            .unwrap_or_default()
            .trim()
            .to_string();
        devices.push(AdbDevice { serial, state, model, abi });
    }
    Ok(devices)
}

#[tauri::command]
pub async fn adb_list_frida_files(serial: String) -> Result<Vec<String>, String> {
    let args: Vec<&str> = if serial.is_empty() {
        vec!["shell", "ls", "/data/local/tmp/"]
    } else {
        vec!["-s", &serial, "shell", "ls", "/data/local/tmp/"]
    };
    let out = run_adb(&args)?;
    let files: Vec<String> = out
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .filter(|l| {
            let lc = l.to_lowercase();
            lc.contains("frida") || lc.contains("re.frida") || lc.starts_with("fs")
        })
        .collect();
    Ok(files)
}

#[tauri::command]
pub async fn adb_push_frida(serial: String, local_path: String, remote_name: String) -> Result<String, String> {
    let remote = format!("/data/local/tmp/{}", remote_name);
    let serial_arg = format!("-s");
    let args: Vec<&str> = if serial.is_empty() {
        vec!["push", &local_path, &remote]
    } else {
        vec![&serial_arg, &serial, "push", &local_path, &remote]
    };
    let out = run_adb(&args)?;
    let chmod_args: Vec<&str> = if serial.is_empty() {
        vec!["shell", "chmod", "755", &remote]
    } else {
        vec![&serial_arg, &serial, "shell", "chmod", "755", &remote]
    };
    run_adb(&chmod_args)?;
    Ok(format!("已推送并赋权: {}\n{}", remote, out))
}

#[tauri::command]
pub async fn adb_get_processes(serial: String) -> Result<Vec<String>, String> {
    let args: Vec<&str> = if serial.is_empty() {
        vec!["shell", "ps", "-A"]
    } else {
        vec!["-s", &serial, "shell", "ps", "-A"]
    };
    let out = run_adb(&args)?;
    let mut procs: Vec<String> = out.lines().skip(1).map(|s| s.to_string()).collect();
    procs.sort();
    Ok(procs)
}

#[tauri::command]
pub async fn adb_install(serial: String, apk_path: String) -> Result<String, String> {
    let args: Vec<&str> = if serial.is_empty() {
        vec!["install", "-r", &apk_path]
    } else {
        vec!["-s", &serial, "install", "-r", &apk_path]
    };
    run_adb(&args)
}
