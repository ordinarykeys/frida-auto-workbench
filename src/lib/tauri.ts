import { invoke } from "@tauri-apps/api/core";

export interface SoFile { name: string; arch: string; }

export interface ApkAnalysisResult {
  package_name: string;
  version_name: string;
  version_code: string;
  min_sdk: string;
  target_sdk: string;
  hardening: string;
  architectures: string[];
  preferred_arch: string;
  so_files: SoFile[];
  so_count: number;
  apk_size: number;
}

export interface AdbDevice {
  serial: string; state: string; model: string; abi: string;
}

export interface FridaStatus {
  running: boolean; port: number; binary: string; serial: string; message: string;
}

export interface AiProvider {
  id: string; name: string; base_url: string; api_key: string; model: string;
}

export interface ChatMessage {
  role: "system" | "user" | "assistant"; content: string;
}

export const tauri = {
  analyzeApk: (path: string) => invoke<ApkAnalysisResult>("analyze_apk", { path }),
  adbDevices: () => invoke<AdbDevice[]>("adb_devices"),
  adbListFridaFiles: (serial: string) => invoke<string[]>("adb_list_frida_files", { serial }),
  adbPushFrida: (serial: string, localPath: string, remoteName: string) =>
    invoke<string>("adb_push_frida", { serial, localPath, remoteName }),
  adbGetProcesses: (serial: string) => invoke<string[]>("adb_get_processes", { serial }),
  adbInstall: (serial: string, apkPath: string) =>
    invoke<string>("adb_install", { serial, apkPath }),
  fridaStart: (serial: string, binary: string, port: number) =>
    invoke<FridaStatus>("frida_start", { serial, binary, port }),
  fridaStop: (serial: string) => invoke<FridaStatus>("frida_stop", { serial }),
  fridaStatus: () => invoke<FridaStatus>("frida_status"),
  fridaForward: (serial: string, port: number) =>
    invoke<string>("frida_forward", { serial, port }),
  fridaPs: (serial: string, port: number) => invoke<string[]>("frida_ps", { serial, port }),
  aiListModels: (baseUrl: string, apiKey: string) =>
    invoke<string[]>("ai_list_models", { baseUrl, apiKey }),
  aiChat: (provider: AiProvider, messages: ChatMessage[], temperature?: number) =>
    invoke<string>("ai_chat", { req: { provider, messages, temperature } }),
  aiGenerateScript: (
    provider: AiProvider, packageName: string, soFile: string, hardening: string, intent: string,
  ) =>
    invoke<string>("ai_generate_script", { provider, packageName, soFile, hardening, intent }),
};
