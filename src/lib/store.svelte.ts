import type { AdbDevice, ApkAnalysisResult, FridaStatus, AiProvider, ChatMessage } from "./tauri";

export type LogLevel = "info" | "success" | "error" | "warn" | "ai";
export interface LogEntry { id: number; time: string; msg: string; level: LogLevel; }

let counter = 0;

class AppStore {
  // device
  devices = $state<AdbDevice[]>([]);
  activeSerial = $state<string>("");

  // apk
  apkPath = $state<string>("");
  apk = $state<ApkAnalysisResult | null>(null);
  selectedSo = $state<string>("");

  // frida
  fridaFiles = $state<string[]>([]);
  selectedFrida = $state<string>("");
  fridaPort = $state<number>(27042);
  customPort = $state<boolean>(false);
  fridaStatus = $state<FridaStatus | null>(null);

  // logs
  logs = $state<LogEntry[]>([]);

  // ai
  providers = $state<AiProvider[]>([
    { id: "default", name: "OpenAI", base_url: "https://api.openai.com/v1", api_key: "", model: "" },
  ]);
  activeProviderId = $state<string>("default");
  aiBusy = $state<boolean>(false);

  pushLog(msg: string, level: LogLevel = "info") {
    this.logs = [
      ...this.logs.slice(-499),
      { id: ++counter, time: new Date().toLocaleTimeString(), msg, level },
    ];
  }
  clearLogs() { this.logs = []; }

  upsertProvider(p: AiProvider) {
    const idx = this.providers.findIndex((x) => x.id === p.id);
    if (idx >= 0) {
      const next = [...this.providers]; next[idx] = p;
      this.providers = next;
    } else {
      this.providers = [...this.providers, p];
    }
  }
  removeProvider(id: string) {
    this.providers = this.providers.filter((p) => p.id !== id);
    if (this.activeProviderId === id && this.providers[0]) {
      this.activeProviderId = this.providers[0].id;
    }
  }
  get activeProvider(): AiProvider | null {
    return this.providers.find((p) => p.id === this.activeProviderId) ?? null;
  }

  // persist a subset to localStorage
  saveToStorage() {
    try {
      localStorage.setItem("frida-workbench:state", JSON.stringify({
        activeSerial: this.activeSerial,
        fridaPort: this.fridaPort,
        customPort: this.customPort,
        selectedFrida: this.selectedFrida,
        providers: this.providers,
        activeProviderId: this.activeProviderId,
      }));
    } catch {}
  }
  loadFromStorage() {
    try {
      const raw = localStorage.getItem("frida-workbench:state");
      if (!raw) return;
      const s = JSON.parse(raw);
      if (s.activeSerial != null) this.activeSerial = s.activeSerial;
      if (typeof s.fridaPort === "number") this.fridaPort = s.fridaPort;
      if (typeof s.customPort === "boolean") this.customPort = s.customPort;
      if (typeof s.selectedFrida === "string") this.selectedFrida = s.selectedFrida;
      if (Array.isArray(s.providers) && s.providers.length) this.providers = s.providers;
      if (typeof s.activeProviderId === "string") this.activeProviderId = s.activeProviderId;
    } catch {}
  }
}

export const store = new AppStore();

export const PROVIDER_PRESETS: { id: string; name: string; base_url: string }[] = [
  { id: "openai", name: "OpenAI", base_url: "https://api.openai.com/v1" },
  { id: "anthropic", name: "Claude (兼容 OpenAI)", base_url: "https://api.anthropic.com/v1" },
  { id: "deepseek", name: "DeepSeek", base_url: "https://api.deepseek.com/v1" },
  { id: "moonshot", name: "Moonshot (Kimi)", base_url: "https://api.moonshot.cn/v1" },
  { id: "qwen", name: "通义千问", base_url: "https://dashscope.aliyuncs.com/compatible-mode/v1" },
  { id: "siliconflow", name: "硅基流动", base_url: "https://api.siliconflow.cn/v1" },
  { id: "ollama", name: "Ollama 本地", base_url: "http://localhost:11434/v1" },
];

export type { ChatMessage };
