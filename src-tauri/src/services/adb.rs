use anyhow::{anyhow, Context, Result};
use std::{
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use crate::models::{DeviceFileEntry, FridaStatus};

fn fallback_adb_path(resource_dir: &Path) -> PathBuf {
    resource_dir.join("resources").join("android-tools").join("adb.exe")
}

pub fn resolve_adb_path(resource_dir: &Path) -> Result<PathBuf> {
    let bundled = fallback_adb_path(resource_dir);
    if bundled.exists() {
        return Ok(bundled);
    }

    if let Ok(android_home) = std::env::var("ANDROID_HOME") {
        let candidate = PathBuf::from(android_home)
            .join("platform-tools")
            .join("adb.exe");
        if candidate.exists() {
            return Ok(candidate);
        }
    }

    Ok(PathBuf::from("adb"))
}

pub fn run_adb(resource_dir: &Path, args: &[&str]) -> Result<String> {
    let adb = resolve_adb_path(resource_dir)?;
    let output = Command::new(adb)
        .args(args)
        .output()
        .with_context(|| format!("failed to execute adb with args {args:?}"))?;

    if !output.status.success() {
        return Err(anyhow!(
            "adb command failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

pub fn list_tmp_files(resource_dir: &Path) -> Result<Vec<DeviceFileEntry>> {
    let output = run_adb(resource_dir, &["shell", "su", "-c", "ls -la /data/local/tmp"])?;
    let entries = output
        .lines()
        .skip(1)
        .filter_map(|line| {
            let columns: Vec<&str> = line.split_whitespace().collect();
            if columns.len() < 8 {
                return None;
            }

            let name = columns[8..].join(" ");
            if name == "." || name == ".." {
                return None;
            }

            Some(DeviceFileEntry {
                permissions: columns[0].to_string(),
                owner: format!("{}:{}", columns[2], columns[3]),
                size: columns[4].to_string(),
                modified_at: format!(
                    "{} {} {}",
                    columns[5],
                    columns[6],
                    columns.get(7).copied().unwrap_or("")
                ),
                path: format!("/data/local/tmp/{name}"),
                name,
            })
        })
        .collect();

    Ok(entries)
}

pub fn start_frida_server(resource_dir: &Path, port: u16, binary_path: &str) -> Result<FridaStatus> {
    let kill_command = format!("pkill -f frida-server || true");
    let _ = run_adb(resource_dir, &["shell", "su", "-c", &kill_command]);

    let command = format!(
        "chmod 755 {binary_path} && nohup {binary_path} -D -l 0.0.0.0:{port} >/data/local/tmp/frida-mcp.log 2>&1 &"
    );
    let _ = run_adb(resource_dir, &["shell", "su", "-c", &command])?;

    let status = get_frida_status(resource_dir, port)?;
    Ok(FridaStatus {
        message: if status.running {
            format!("frida-server is running on port {port}.")
        } else {
            format!("frida-server launch command sent, but status check did not confirm port {port}.")
        },
        command,
        ..status
    })
}

pub fn stop_frida_server(resource_dir: &Path) -> Result<FridaStatus> {
    let command = "pkill -f frida-server || true";
    let _ = run_adb(resource_dir, &["shell", "su", "-c", command])?;
    Ok(FridaStatus {
        port: 0,
        running: false,
        pid: None,
        command: command.to_string(),
        message: "frida-server stop command executed.".to_string(),
    })
}

pub fn get_frida_status(resource_dir: &Path, port: u16) -> Result<FridaStatus> {
    let pid_lookup = run_adb(
        resource_dir,
        &[
            "shell",
            "su",
            "-c",
            &format!("pidof frida-server || ps -A | grep frida-server"),
        ],
    )
    .unwrap_or_default();

    let running = !pid_lookup.trim().is_empty();
    let pid = pid_lookup
        .split_whitespace()
        .find_map(|token| token.parse::<u32>().ok());

    Ok(FridaStatus {
        port,
        running,
        pid,
        command: format!("pidof frida-server @ {port}"),
        message: if running {
            format!("frida-server detected on device, port target {port}.")
        } else {
            "frida-server not detected on the device.".to_string()
        },
    })
}

pub fn run_host_process(executable: &str, args: &[&str]) -> Result<String> {
    let output = Command::new(executable)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .with_context(|| format!("failed to execute host process {executable}"))?;

    if !output.status.success() {
        return Err(anyhow!(
            "{executable} failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}
