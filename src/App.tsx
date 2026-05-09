import { useMemo, useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { toast, Toaster } from "sonner";
import { Bot, BrainCircuit, FolderUp, LoaderCircle, Play, RefreshCcw, ScanSearch, Square, Usb } from "lucide-react";
import { AppShell } from "@/components/layout/app-shell";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Switch } from "@/components/ui/switch";
import { Textarea } from "@/components/ui/textarea";
import { analyzeApk, fetchProviderModels, generateFridaScript, getFridaStatus, refreshTmpFiles, startFridaServer, stopFridaServer } from "@/lib/tauri";
import type { AiProviderConfig, AiScriptResult, ApkAnalysis, DeviceFileEntry, FridaStatus, ProviderModel } from "@/types";

const defaultAiConfig: AiProviderConfig = {
  providerLabel: "OpenAI Compatible",
  baseUrl: "https://api.openai.com/v1",
  apiKey: "",
  modelsEndpoint: "/models",
  modelId: "",
  temperature: 0.2,
  maxIterations: 3,
  enableSelfCheck: true,
};

type LogLevel = "info" | "success" | "warning" | "error";

type LogEntry = {
  id: number;
  level: LogLevel;
  time: string;
  message: string;
};

function App() {
  const [apkPath, setApkPath] = useState("");
  const [analysis, setAnalysis] = useState<ApkAnalysis | null>(null);
  const [selectedSo, setSelectedSo] = useState("");
  const [tmpFiles, setTmpFiles] = useState<DeviceFileEntry[]>([]);
  const [fridaPort, setFridaPort] = useState("27042");
  const [customPort, setCustomPort] = useState(false);
  const [fridaBinaryPath, setFridaBinaryPath] = useState("/data/local/tmp/frida-server");
  const [fridaStatus, setFridaStatus] = useState<FridaStatus | null>(null);
  const [aiConfig, setAiConfig] = useState<AiProviderConfig>(defaultAiConfig);
  const [availableModels, setAvailableModels] = useState<ProviderModel[]>([]);
  const [generatedScript, setGeneratedScript] = useState<AiScriptResult | null>(null);
  const [isBusy, setIsBusy] = useState<Record<string, boolean>>({});
  const [logs, setLogs] = useState<LogEntry[]>([
    { id: 1, level: "info", time: nowTime(), message: "工作台已就绪" },
  ]);

  const soOptions = analysis?.soFiles ?? [];
  const selectedSoEntry = useMemo(
    () => soOptions.find((entry) => entry.path === selectedSo) ?? soOptions[0] ?? null,
    [selectedSo, soOptions],
  );
  const visibleTmpFiles = tmpFiles.slice(0, 8);

  function nowTime() {
    return new Date().toLocaleTimeString("zh-CN", { hour12: false });
  }

  function appendLog(level: LogLevel, message: string) {
    setLogs((current) => [{ id: Date.now() + Math.random(), level, time: nowTime(), message }, ...current].slice(0, 200));
  }

  function setBusy(key: string, value: boolean) {
    setIsBusy((current) => ({ ...current, [key]: value }));
  }

  async function handlePickApk() {
    const selected = await open({
      title: "选择 APK",
      filters: [{ name: "Android Package", extensions: ["apk"] }],
      multiple: false,
    });

    if (!selected || Array.isArray(selected)) {
      return;
    }

    setApkPath(selected);
    appendLog("info", `已选择 APK: ${selected}`);
  }

  async function handleAnalyzeApk() {
    if (!apkPath) {
      toast.error("请先选择 APK");
      return;
    }

    try {
      setBusy("apk", true);
      appendLog("info", "开始分析 APK");
      const result = await analyzeApk(apkPath);
      setAnalysis(result);
      setSelectedSo(result.soFiles.find((entry) => entry.abi === "arm64-v8a")?.path ?? result.soFiles[0]?.path ?? "");
      appendLog("success", `分析完成: ${result.packageName}`);
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      toast.error(message);
      appendLog("error", `分析失败: ${message}`);
    } finally {
      setBusy("apk", false);
    }
  }

  async function handleRefreshTmp() {
    try {
      setBusy("tmp", true);
      appendLog("info", "刷新 /data/local/tmp");
      const result = await refreshTmpFiles(Number(fridaPort));
      setTmpFiles(result);
      appendLog("success", `读取到 ${result.length} 个文件`);
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      toast.error(message);
      appendLog("error", `刷新失败: ${message}`);
    } finally {
      setBusy("tmp", false);
    }
  }

  async function handleStartFrida() {
    try {
      setBusy("frida", true);
      appendLog("info", `启动 frida-server, 端口 ${fridaPort}`);
      const result = await startFridaServer(Number(fridaPort), fridaBinaryPath);
      setFridaStatus(result);
      appendLog(result.running ? "success" : "warning", result.message);
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      toast.error(message);
      appendLog("error", `启动失败: ${message}`);
    } finally {
      setBusy("frida", false);
    }
  }

  async function handleStopFrida() {
    try {
      setBusy("stop", true);
      appendLog("info", "停止 frida-server");
      const result = await stopFridaServer();
      setFridaStatus(result);
      appendLog("warning", result.message);
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      toast.error(message);
      appendLog("error", `停止失败: ${message}`);
    } finally {
      setBusy("stop", false);
    }
  }

  async function handleStatusCheck() {
    try {
      setBusy("status", true);
      const result = await getFridaStatus(Number(fridaPort));
      setFridaStatus(result);
      appendLog(result.running ? "success" : "warning", result.message);
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      toast.error(message);
      appendLog("error", `状态检查失败: ${message}`);
    } finally {
      setBusy("status", false);
    }
  }

  async function handleFetchModels() {
    try {
      setBusy("models", true);
      appendLog("info", "获取模型列表");
      const models = await fetchProviderModels(aiConfig);
      setAvailableModels(models);
      setAiConfig((current) => ({ ...current, modelId: current.modelId || models[0]?.id || "" }));
      appendLog("success", `已获取 ${models.length} 个模型`);
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      toast.error(message);
      appendLog("error", `模型获取失败: ${message}`);
    } finally {
      setBusy("models", false);
    }
  }

  async function handleGenerateScript() {
    if (!analysis) {
      toast.error("请先分析 APK");
      return;
    }

    if (!aiConfig.modelId) {
      toast.error("请先选择模型");
      return;
    }

    try {
      setBusy("ai", true);
      appendLog("info", "开始生成脚本");
      const result = await generateFridaScript({
        config: aiConfig,
        packageName: analysis.packageName,
        soName: selectedSoEntry?.name ?? null,
        hardeningInfo: analysis.hardeningInfo,
        analysisNotes: analysis.manifestSummary,
      });
      setGeneratedScript(result);
      appendLog("success", "脚本生成完成");
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      toast.error(message);
      appendLog("error", `脚本生成失败: ${message}`);
    } finally {
      setBusy("ai", false);
    }
  }

  const apkGroup = (
    <fieldset className="group-box">
      <legend>APK 信息</legend>
      <div className="group-rows">
        <FormRow label="APK">
          <Input value={apkPath} readOnly placeholder="选择 APK" className="h-7 w-[560px] text-[12px]" />
          <Button variant="secondary" className="h-7 px-2" onClick={handlePickApk}>
            <FolderUp className="size-3.5" />
            上传
          </Button>
          <Button className="h-7 px-2" onClick={handleAnalyzeApk} disabled={isBusy.apk}>
            {isBusy.apk ? <LoaderCircle className="size-3.5 animate-spin" /> : <ScanSearch className="size-3.5" />}
            分析
          </Button>
        </FormRow>

        <FormRow label="包名">
          <Input value={analysis?.packageName ?? ""} readOnly placeholder="-" className="h-7 w-[160px] text-[12px]" />
          <Button variant="secondary" className="h-7 px-2">
            复制
          </Button>
          <span className="field-tag">加固</span>
          <Input value={analysis?.hardeningInfo ?? ""} readOnly placeholder="未识别" className="h-7 w-[110px] text-[12px]" />
          <span className="field-tag">SO架构</span>
          <Input value={analysis?.preferredAbi ?? ""} readOnly placeholder="arm64-v8a" className="h-7 w-[110px] text-[12px]" />
        </FormRow>

        <FormRow label="SO文件">
          <Select value={selectedSo} onValueChange={setSelectedSo} disabled={!soOptions.length}>
            <SelectTrigger className="h-7 w-[150px]">
              <SelectValue placeholder="选择 so" />
            </SelectTrigger>
            <SelectContent>
              {soOptions.map((entry) => (
                <SelectItem key={entry.path} value={entry.path}>
                  {entry.name}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
          <Button variant="secondary" className="h-7 px-2">
            复制SO
          </Button>
          <span className="field-tag">Frida文件</span>
          <Select value={fridaBinaryPath} onValueChange={setFridaBinaryPath}>
            <SelectTrigger className="h-7 w-[150px]">
              <SelectValue placeholder="-" />
            </SelectTrigger>
            <SelectContent>
              {visibleTmpFiles.length ? (
                visibleTmpFiles.map((entry) => (
                  <SelectItem key={entry.path} value={entry.path}>
                    {entry.name}
                  </SelectItem>
                ))
              ) : (
                <SelectItem value="/data/local/tmp/frida-server">frida-server</SelectItem>
              )}
            </SelectContent>
          </Select>
          <Button variant="secondary" className="h-7 px-2" onClick={handleRefreshTmp} disabled={isBusy.tmp}>
            {isBusy.tmp ? <LoaderCircle className="size-3.5 animate-spin" /> : <RefreshCcw className="size-3.5" />}
            刷新
          </Button>
          <label className="flex items-center gap-1 text-[12px] text-[#1f1f1f]">
            <Switch checked={customPort} onCheckedChange={setCustomPort} />
            自定义端口
          </label>
          <Input value={fridaPort} onChange={(event) => setFridaPort(event.target.value)} className="h-7 w-[70px] text-[12px]" disabled={!customPort} />
        </FormRow>
      </div>
    </fieldset>
  );

  const center = (
    <fieldset className="group-box">
      <legend>Frida 控制</legend>
      <div className="group-rows">
        <div className="flex flex-wrap items-center gap-2">
          <Button className="h-7 px-2" onClick={handleStartFrida} disabled={isBusy.frida}>
            {isBusy.frida ? <LoaderCircle className="size-3.5 animate-spin" /> : <Play className="size-3.5" />}
            运行Frida
          </Button>
          <Button variant="destructive" className="h-7 px-2" onClick={handleStopFrida} disabled={isBusy.stop}>
            {isBusy.stop ? <LoaderCircle className="size-3.5 animate-spin" /> : <Square className="size-3.5" />}
            停止Frida
          </Button>
          <Button variant="secondary" className="h-7 px-2" onClick={handleStatusCheck} disabled={isBusy.status}>
            {isBusy.status ? <LoaderCircle className="size-3.5 animate-spin" /> : <Usb className="size-3.5" />}
            状态
          </Button>
          <Badge variant={fridaStatus?.running ? "default" : "warning"}>{fridaStatus?.running ? "运行中" : "未运行"}</Badge>
          <span className="text-[12px] text-[#1f1f1f]">PID {fridaStatus?.pid ?? "-"}</span>
        </div>
        <div className="status-box">{fridaStatus?.message ?? "等待操作"}</div>
      </div>
    </fieldset>
  );

  const lower = (
    <div className="grid h-full grid-cols-[1.2fr_1fr] gap-3">
      <fieldset className="group-box">
        <legend>日志</legend>
        <div className="log-box">
          {logs.map((entry) => (
            <div key={entry.id} className="log-row-lite">
              <span className="text-slate-400">{entry.time}</span>
              <span
                className={
                  entry.level === "error"
                    ? "text-rose-400"
                    : entry.level === "success"
                      ? "text-emerald-400"
                      : entry.level === "warning"
                        ? "text-amber-300"
                        : "text-sky-400"
                }
              >
                {entry.level.toUpperCase()}
              </span>
              <span className="text-slate-100">{entry.message}</span>
            </div>
          ))}
        </div>
      </fieldset>

      <div className="grid h-full grid-rows-[auto_1fr] gap-3">
        <fieldset className="group-box">
          <legend>AI 配置</legend>
          <div className="group-rows">
            <div className="flex items-center gap-2">
              <Input className="h-7 text-[12px]" value={aiConfig.providerLabel} onChange={(event) => setAiConfig((current) => ({ ...current, providerLabel: event.target.value }))} placeholder="提供方" />
              <Input className="h-7 text-[12px]" value={aiConfig.baseUrl} onChange={(event) => setAiConfig((current) => ({ ...current, baseUrl: event.target.value }))} placeholder="Base URL" />
            </div>
            <div className="flex items-center gap-2">
              <Input className="h-7 flex-1 text-[12px]" type="password" value={aiConfig.apiKey ?? ""} onChange={(event) => setAiConfig((current) => ({ ...current, apiKey: event.target.value }))} placeholder="API Key" />
              <Input className="h-7 w-[88px] text-[12px]" value={aiConfig.modelsEndpoint ?? ""} onChange={(event) => setAiConfig((current) => ({ ...current, modelsEndpoint: event.target.value }))} placeholder="/models" />
              <Button variant="secondary" className="h-7 px-2" onClick={handleFetchModels} disabled={isBusy.models}>
                {isBusy.models ? <LoaderCircle className="size-3.5 animate-spin" /> : <Bot className="size-3.5" />}
                获取
              </Button>
            </div>
            <div className="flex items-center gap-2">
              <Select value={aiConfig.modelId ?? ""} onValueChange={(value) => setAiConfig((current) => ({ ...current, modelId: value }))}>
                <SelectTrigger className="h-7 w-[180px]">
                  <SelectValue placeholder="选择模型" />
                </SelectTrigger>
                <SelectContent>
                  {availableModels.map((model) => (
                    <SelectItem key={model.id} value={model.id}>
                      {model.label}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
              <Input
                className="h-7 w-[64px] text-[12px]"
                type="number"
                min={1}
                max={6}
                value={aiConfig.maxIterations}
                onChange={(event) => setAiConfig((current) => ({ ...current, maxIterations: Number(event.target.value) || 1 }))}
              />
              <label className="flex items-center gap-1 text-[12px] text-[#1f1f1f]">
                <Switch checked={aiConfig.enableSelfCheck} onCheckedChange={(checked) => setAiConfig((current) => ({ ...current, enableSelfCheck: checked }))} />
                自检
              </label>
              <Button className="h-7 px-2" onClick={handleGenerateScript} disabled={isBusy.ai}>
                {isBusy.ai ? <LoaderCircle className="size-3.5 animate-spin" /> : <BrainCircuit className="size-3.5" />}
                写脚本
              </Button>
            </div>
          </div>
        </fieldset>

        <fieldset className="group-box">
          <legend>脚本</legend>
          <div className="group-rows h-full">
            <div className="status-box truncate">{generatedScript?.summary ?? "未生成"}</div>
            <Textarea className="min-h-[160px] flex-1 resize-none bg-white font-mono text-[11px] leading-4 text-[#1f1f1f]" value={generatedScript?.script ?? ""} readOnly placeholder="// 脚本" />
            <div className="grid grid-cols-2 gap-2">
              {(generatedScript?.selfCheckNotes.slice(0, 2) ?? ["等待生成", "等待自检"]).map((note) => (
                <div key={note} className="status-box truncate text-[11px]">
                  {note}
                </div>
              ))}
            </div>
          </div>
        </fieldset>
      </div>
    </div>
  );

  return (
    <>
      <AppShell sidebar={apkGroup} rightPanel={lower}>
        {center}
      </AppShell>
      <Toaster position="top-right" richColors />
    </>
  );
}

function FormRow({
  label,
  children,
}: {
  label: string;
  children: React.ReactNode;
}) {
  return (
    <div className="grid grid-cols-[42px_1fr] items-center gap-2">
      <span className="text-[12px] text-[#1f1f1f]">{label}</span>
      <div className="flex items-center gap-2 overflow-hidden">{children}</div>
    </div>
  );
}

export default App;
