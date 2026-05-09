use serde::{Deserialize, Serialize};
use std::process::{Command, Stdio};
use std::sync::Mutex;
use tauri::State;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FridaStatus {
    pub running: bool,
    pub port: u16,
    pub binary: String,
    pub serial: String,
    pub message: String,
}

pub struct FridaState(pub Mutex<FridaStatus>);

impl Default for FridaState {
    fn default() -> Self {
        Self(Mutex::new(FridaStatus {
            running: false,
            port: 27042,
            binary: String::new(),
            serial: String::new(),
            message: String::new(),
        }))
    }
}

fn adb_path() -> String {
    std::env::var("ADB_PATH").unwrap_or_else(|_| "adb".to_string())
}

fn run_adb_capture(args: &[&str]) -> Result<String, String> {
    let output = Command::new(adb_path())
        .args(args)
        .output()
        .map_err(|e| format!("adb 执行失败: {}", e))?;
    if !output.status.success() {
        return Err(format!(
            "{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

#[tauri::command]
pub async fn frida_start(
    serial: String,
    binary: String,
    port: u16,
    state: State<'_, FridaState>,
) -> Result<FridaStatus, String> {
    if binary.is_empty() {
        return Err("请先选择 frida-server 二进制文件".into());
    }
    let remote = format!("/data/local/tmp/{}", binary);

    // chmod 755
    let chmod_args: Vec<String> = if serial.is_empty() {
        vec!["shell".into(), "chmod".into(), "755".into(), remote.clone()]
    } else {
        vec!["-s".into(), serial.clone(), "shell".into(), "chmod".into(), "755".into(), remote.clone()]
    };
    let chmod_refs: Vec<&str> = chmod_args.iter().map(|s| s.as_str()).collect();
    run_adb_capture(&chmod_refs)?;

    // Kill existing
    let _ = run_adb_capture(&{
        let mut v: Vec<&str> = Vec::new();
        if !serial.is_empty() { v.push("-s"); v.push(&serial); }
        v.extend_from_slice(&["shell", "su", "-c", "killall frida-server 2>/dev/null; killall frida-server-16 2>/dev/null; true"]);
        v
    });

    // Spawn frida-server in background via su (root). Use `nohup ... &` and disown stdin/stdout to avoid hanging adb.
    let cmd = format!("nohup {} -l 0.0.0.0:{} >/dev/null 2>&1 &", remote, port);
    let mut spawn_args: Vec<String> = Vec::new();
    if !serial.is_empty() { spawn_args.push("-s".into()); spawn_args.push(serial.clone()); }
    spawn_args.extend_from_slice(&["shell".into(), "su".into(), "-c".into(), cmd]);
    let spawn_refs: Vec<&str> = spawn_args.iter().map(|s| s.as_str()).collect();

    let mut child = Command::new(adb_path())
        .args(&spawn_refs)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .stdin(Stdio::null())
        .spawn()
        .map_err(|e| format!("启动 frida-server 失败: {}", e))?;

    // adb shell with `&` should return quickly; wait briefly
    std::thread::sleep(std::time::Duration::from_millis(800));
    let _ = child.try_wait();

    // Verify by checking pid
    let mut check_args: Vec<String> = Vec::new();
    if !serial.is_empty() { check_args.push("-s".into()); check_args.push(serial.clone()); }
    check_args.extend_from_slice(&["shell".into(), "pidof".into(), binary.clone()]);
    let check_refs: Vec<&str> = check_args.iter().map(|s| s.as_str()).collect();
    let pid_out = run_adb_capture(&check_refs).unwrap_or_default();
    let running = !pid_out.trim().is_empty();

    let status = FridaStatus {
        running,
        port,
        binary: binary.clone(),
        serial: serial.clone(),
        message: if running {
            format!("frida-server 已启动 (PID: {}, 端口: {})", pid_out.trim(), port)
        } else {
            "启动命令已下发，但 pidof 未发现进程；请确认设备已 root 并允许 su".into()
        },
    };
    *state.0.lock().unwrap() = status.clone();
    Ok(status)
}

#[tauri::command]
pub async fn frida_stop(serial: String, state: State<'_, FridaState>) -> Result<FridaStatus, String> {
    let mut kill_args: Vec<String> = Vec::new();
    if !serial.is_empty() { kill_args.push("-s".into()); kill_args.push(serial.clone()); }
    kill_args.extend_from_slice(&[
        "shell".into(), "su".into(), "-c".into(),
        "killall frida-server 2>/dev/null; killall frida-server-16 2>/dev/null; true".into(),
    ]);
    let kill_refs: Vec<&str> = kill_args.iter().map(|s| s.as_str()).collect();
    let _ = run_adb_capture(&kill_refs);

    let mut s = state.0.lock().unwrap();
    s.running = false;
    s.message = "frida-server 已停止".into();
    Ok(s.clone())
}

#[tauri::command]
pub async fn frida_status(state: State<'_, FridaState>) -> Result<FridaStatus, String> {
    Ok(state.0.lock().unwrap().clone())
}

#[tauri::command]
pub async fn frida_forward(serial: String, port: u16) -> Result<String, String> {
    let port_str = format!("tcp:{}", port);
    let mut args: Vec<&str> = Vec::new();
    if !serial.is_empty() { args.push("-s"); args.push(&serial); }
    args.extend_from_slice(&["forward", &port_str, &port_str]);
    run_adb_capture(&args)?;
    Ok(format!("已转发本机 127.0.0.1:{} ⇄ 设备 :{}", port, port))
}

#[tauri::command]
pub async fn frida_ps(serial: String, port: u16) -> Result<Vec<String>, String> {
    // Use frida-ps via shell if available locally; otherwise list pids on device.
    let port_str = port.to_string();
    let frida_ps = Command::new("frida-ps")
        .args(["-H", &format!("127.0.0.1:{}", port), "-a"])
        .output();
    if let Ok(out) = frida_ps {
        if out.status.success() {
            let s = String::from_utf8_lossy(&out.stdout);
            return Ok(s.lines().map(|l| l.to_string()).collect());
        }
    }
    // Fallback: adb shell ps
    let _ = port_str;
    let mut args: Vec<&str> = Vec::new();
    if !serial.is_empty() { args.push("-s"); args.push(&serial); }
    args.extend_from_slice(&["shell", "ps", "-A"]);
    let out = run_adb_capture(&args)?;
    Ok(out.lines().map(|l| l.to_string()).collect())
}
