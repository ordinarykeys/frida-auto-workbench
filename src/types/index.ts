export type ApkAnalysis = {
  filePath: string;
  fileName: string;
  packageName: string;
  appName?: string | null;
  packageVersion?: string | null;
  hardeningInfo: string;
  preferredAbi: string;
  availableAbis: string[];
  soFiles: SoFileEntry[];
  manifestSummary: string[];
};

export type SoFileEntry = {
  abi: string;
  path: string;
  name: string;
  size: number;
};

export type DeviceFileEntry = {
  name: string;
  path: string;
  permissions: string;
  owner: string;
  size: string;
  modifiedAt: string;
};

export type FridaStatus = {
  port: number;
  running: boolean;
  pid?: number | null;
  command: string;
  message: string;
};

export type ProviderModel = {
  id: string;
  label: string;
  contextWindow?: number | null;
};

export type AiProviderConfig = {
  providerLabel: string;
  baseUrl: string;
  apiKey?: string | null;
  modelsEndpoint?: string | null;
  modelId?: string | null;
  temperature: number;
  maxIterations: number;
  enableSelfCheck: boolean;
};

export type AiScriptResult = {
  script: string;
  summary: string;
  selfCheckNotes: string[];
};

export type TerminalSessionInfo = {
  sessionId: string;
  shell: string;
};

export type RunnerSessionInfo = {
  sessionId: string;
  command: string;
};
