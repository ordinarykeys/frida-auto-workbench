import { invoke } from "@tauri-apps/api/core";
import type {
  AiProviderConfig,
  AiScriptResult,
  ApkAnalysis,
  DeviceFileEntry,
  FridaStatus,
  ProviderModel,
  TerminalSessionInfo,
} from "@/types";

export async function analyzeApk(filePath: string) {
  return invoke<ApkAnalysis>("analyze_apk", { filePath });
}

export async function refreshTmpFiles(port: number) {
  return invoke<DeviceFileEntry[]>("list_tmp_files", { port });
}

export async function startFridaServer(port: number, binaryPath?: string | null) {
  return invoke<FridaStatus>("start_frida_server", { port, binaryPath });
}

export async function stopFridaServer() {
  return invoke<FridaStatus>("stop_frida_server");
}

export async function getFridaStatus(port: number) {
  return invoke<FridaStatus>("get_frida_status", { port });
}

export async function fetchProviderModels(config: AiProviderConfig) {
  return invoke<ProviderModel[]>("fetch_provider_models", { config });
}

export async function generateFridaScript(payload: {
  config: AiProviderConfig;
  packageName: string;
  soName?: string | null;
  hardeningInfo?: string | null;
  analysisNotes?: string[];
}) {
  return invoke<AiScriptResult>("generate_frida_script", payload);
}

export async function startTerminalSession(mode: string) {
  return invoke<TerminalSessionInfo>("start_terminal_session", { request: { mode } });
}

export async function writeTerminalInput(sessionId: string, data: string) {
  return invoke<void>("write_terminal_input", { sessionId, data });
}

export async function closeTerminalSession(sessionId: string) {
  return invoke<void>("close_terminal_session", { sessionId });
}
