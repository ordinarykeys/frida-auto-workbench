use anyhow::{anyhow, Context, Result};
use std::{
    collections::HashMap,
    io::{BufRead, BufReader},
    process::{Child, Command, Stdio},
    sync::Mutex,
    thread,
};
use tauri::{AppHandle, Emitter};

use crate::models::{FridaRunRequest, RunnerSessionInfo};

pub struct RunnerSession {
    pub child: Child,
}

#[derive(Default)]
pub struct RunnerState {
    pub sessions: Mutex<HashMap<String, RunnerSession>>,
}

pub fn start_frida_run(
    app: AppHandle,
    state: tauri::State<RunnerState>,
    request: FridaRunRequest,
) -> Result<RunnerSessionInfo> {
    let mut command = Command::new(&request.runtime_path);
    let args = build_frida_args(&request);
    command
        .args(&args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .stdin(Stdio::null());

    let mut child = command.spawn().context("failed to spawn frida runtime")?;
    let stdout = child.stdout.take().ok_or_else(|| anyhow!("failed to capture stdout"))?;
    let stderr = child.stderr.take().ok_or_else(|| anyhow!("failed to capture stderr"))?;

    let session_id = format!("run-{}", uuid_like());
    let emit_stdout_id = session_id.clone();
    let emit_stderr_id = session_id.clone();
    let app_stdout = app.clone();
    let app_stderr = app.clone();

    thread::spawn(move || {
        let reader = BufReader::new(stdout);
        for line in reader.lines().map_while(Result::ok) {
            let _ = app_stdout.emit(
                "runner-output",
                serde_json::json!({
                    "sessionId": emit_stdout_id,
                    "stream": "stdout",
                    "data": line
                }),
            );
        }
    });

    thread::spawn(move || {
        let reader = BufReader::new(stderr);
        for line in reader.lines().map_while(Result::ok) {
            let _ = app_stderr.emit(
                "runner-output",
                serde_json::json!({
                    "sessionId": emit_stderr_id,
                    "stream": "stderr",
                    "data": line
                }),
            );
        }
    });

    let printable = format!("{} {}", request.runtime_path, args.join(" "));

    state
        .sessions
        .lock()
        .map_err(|_| anyhow!("failed to lock runner sessions"))?
        .insert(session_id.clone(), RunnerSession { child });

    Ok(RunnerSessionInfo {
        session_id,
        command: printable,
    })
}

pub fn stop_run(state: tauri::State<RunnerState>, session_id: &str) -> Result<()> {
    let mut sessions = state
        .sessions
        .lock()
        .map_err(|_| anyhow!("failed to lock runner sessions"))?;
    if let Some(mut session) = sessions.remove(session_id) {
        let _ = session.child.kill();
    }
    Ok(())
}

fn build_frida_args(request: &FridaRunRequest) -> Vec<String> {
    let mut args = vec![
        "-H".to_string(),
        format!("127.0.0.1:{}", request.port),
        "-f".to_string(),
        request.package_name.clone(),
        "-l".to_string(),
        request.script_path.clone(),
    ];

    if request.mode == "attach" {
        args = vec![
            "-H".to_string(),
            format!("127.0.0.1:{}", request.port),
            "-n".to_string(),
            request.package_name.clone(),
            "-l".to_string(),
            request.script_path.clone(),
        ];
    } else {
        args.push("--no-pause".to_string());
    }

    args
}

fn uuid_like() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    format!("{now}")
}
