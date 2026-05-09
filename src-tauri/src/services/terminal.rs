use anyhow::{anyhow, Context, Result};
use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use std::{
    collections::HashMap,
    io::{Read, Write},
    sync::{Arc, Mutex},
    thread,
};
use tauri::{AppHandle, Emitter};

use crate::models::TerminalSessionInfo;

type SharedWriter = Arc<Mutex<Box<dyn Write + Send>>>;

pub struct TerminalSession {
    pub writer: SharedWriter,
}

#[derive(Default)]
pub struct TerminalState {
    pub sessions: Mutex<HashMap<String, TerminalSession>>,
}

pub fn start_session(
    app: AppHandle,
    state: tauri::State<TerminalState>,
    mode: &str,
) -> Result<TerminalSessionInfo> {
    let (shell, args) = resolve_shell(mode);

    let pty_system = native_pty_system();
    let pair = pty_system
        .openpty(PtySize {
            rows: 30,
            cols: 120,
            pixel_width: 0,
            pixel_height: 0,
        })
        .context("failed to open PTY")?;

    let mut cmd = CommandBuilder::new(shell.clone());
    for arg in args {
        cmd.arg(arg);
    }
    let _child = pair.slave.spawn_command(cmd).context("failed to spawn shell")?;

    let mut reader = pair.master.try_clone_reader().context("failed to clone PTY reader")?;
    let writer = pair.master.take_writer().context("failed to take PTY writer")?;

    let session_id = format!("term-{}", uuid_like());
    let emit_session_id = session_id.clone();
    let app_for_thread = app.clone();

    thread::spawn(move || {
        let mut buffer = [0_u8; 8192];
        loop {
            match reader.read(&mut buffer) {
                Ok(0) => break,
                Ok(size) => {
                    let chunk = String::from_utf8_lossy(&buffer[..size]).to_string();
                    let _ = app_for_thread.emit(
                        "terminal-output",
                        serde_json::json!({
                            "sessionId": emit_session_id,
                            "data": chunk
                        }),
                    );
                }
                Err(_) => break,
            }
        }
    });

    state
        .sessions
        .lock()
        .map_err(|_| anyhow!("failed to lock terminal sessions"))?
        .insert(
            session_id.clone(),
            TerminalSession {
                writer: Arc::new(Mutex::new(writer)),
            },
        );

    Ok(TerminalSessionInfo { session_id, shell })
}

pub fn write_session(state: tauri::State<TerminalState>, session_id: &str, data: &str) -> Result<()> {
    let sessions = state
        .sessions
        .lock()
        .map_err(|_| anyhow!("failed to lock terminal sessions"))?;
    let session = sessions
        .get(session_id)
        .ok_or_else(|| anyhow!("terminal session not found"))?;

    let mut writer = session
        .writer
        .lock()
        .map_err(|_| anyhow!("failed to lock terminal writer"))?;
    writer.write_all(data.as_bytes()).context("failed to write terminal input")?;
    writer.flush().context("failed to flush terminal input")?;
    Ok(())
}

pub fn close_session(state: tauri::State<TerminalState>, session_id: &str) -> Result<()> {
    state
        .sessions
        .lock()
        .map_err(|_| anyhow!("failed to lock terminal sessions"))?
        .remove(session_id);
    Ok(())
}

fn uuid_like() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    format!("{now}")
}

fn resolve_shell(mode: &str) -> (String, Vec<String>) {
    match mode {
        "cmd" => ("cmd.exe".to_string(), vec![]),
        "powershell" => (
            "powershell.exe".to_string(),
            vec!["-NoLogo".to_string()],
        ),
        "adb" => ("cmd.exe".to_string(), vec!["/K".to_string(), "adb shell".to_string()]),
        _ => {
            if cfg!(target_os = "windows") {
                (
                    std::env::var("COMSPEC").unwrap_or_else(|_| "cmd.exe".to_string()),
                    vec![],
                )
            } else {
                (
                    std::env::var("SHELL").unwrap_or_else(|_| "bash".to_string()),
                    vec![],
                )
            }
        }
    }
}
