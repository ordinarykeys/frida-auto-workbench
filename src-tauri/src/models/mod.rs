use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApkAnalysis {
    pub file_path: String,
    pub file_name: String,
    pub package_name: String,
    pub app_name: Option<String>,
    pub package_version: Option<String>,
    pub hardening_info: String,
    pub preferred_abi: String,
    pub available_abis: Vec<String>,
    pub so_files: Vec<SoFileEntry>,
    pub manifest_summary: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SoFileEntry {
    pub abi: String,
    pub path: String,
    pub name: String,
    pub size: u64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceFileEntry {
    pub name: String,
    pub path: String,
    pub permissions: String,
    pub owner: String,
    pub size: String,
    pub modified_at: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FridaStatus {
    pub port: u16,
    pub running: bool,
    pub pid: Option<u32>,
    pub command: String,
    pub message: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiProviderConfig {
    pub provider_label: String,
    pub base_url: String,
    pub api_key: Option<String>,
    pub models_endpoint: Option<String>,
    pub model_id: Option<String>,
    pub temperature: f32,
    pub max_iterations: u8,
    pub enable_self_check: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderModel {
    pub id: String,
    pub label: String,
    pub context_window: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiScriptResult {
    pub script: String,
    pub summary: String,
    pub self_check_notes: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct ModelListResponse {
    pub data: Vec<ModelItem>,
}

#[derive(Debug, Deserialize)]
pub struct ModelItem {
    pub id: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TerminalSessionInfo {
    pub session_id: String,
    pub shell: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TerminalSessionRequest {
    pub mode: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RunnerSessionInfo {
    pub session_id: String,
    pub command: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FridaRunRequest {
    pub runtime_path: String,
    pub script_path: String,
    pub package_name: String,
    pub port: u16,
    pub mode: String,
}
