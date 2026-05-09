<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { Switch } from "$lib/components/ui/switch";
  import * as Select from "$lib/components/ui/select";
  import { open } from "@tauri-apps/plugin-dialog";
  import { toast } from "svelte-sonner";
  import Play from "@lucide/svelte/icons/play";
  import Square from "@lucide/svelte/icons/square";
  import RefreshCw from "@lucide/svelte/icons/refresh-cw";
  import Upload from "@lucide/svelte/icons/upload";
  import ArrowRight from "@lucide/svelte/icons/arrow-right";
  import Search from "@lucide/svelte/icons/search";
  import Cpu from "@lucide/svelte/icons/cpu";
  import { store } from "$lib/store.svelte";
  import { tauri } from "$lib/tauri";
  import Group from "./group.svelte";
  import Kv from "./kv.svelte";

  const DEFAULT_PORT = 27042;

  let busy = $state(false);
  let tab: "server" | "process" = $state("server");
  let filter = $state("");
  let procItems = $state<string[]>([]);
  let procSource: "frida-ps" | "adb-ps" = $state("frida-ps");
  let procBusy = $state(false);

  let effectivePort = $derived(store.customPort ? store.fridaPort : DEFAULT_PORT);

  let lastSerial = $state("");
  $effect(() => {
    if (store.activeSerial && store.activeSerial !== lastSerial) {
      lastSerial = store.activeSerial;
      refreshFiles();
    }
  });

  const refreshFiles = async () => {
    if (!store.activeSerial) { store.pushLog("未选择设备", "warn"); return; }
    busy = true;
    try {
      const files = await tauri.adbListFridaFiles(store.activeSerial);
      store.fridaFiles = files;
      if (files.length > 0 && !store.selectedFrida) store.selectedFrida = files[0];
      store.pushLog(`/data/local/tmp 中发现 ${files.length} 个 frida 文件`, "success");
    } catch (e) { store.pushLog(`读取失败: ${e}`, "error"); }
    finally { busy = false; }
  };

  const pushFromLocal = async () => {
    if (!store.activeSerial) { toast.error("请先连接设备"); return; }
    try {
      const file = await open({ multiple: false });
      if (typeof file !== "string") return;
      const remoteName = file.split(/[\\/]/).pop() || "frida-server";
      store.pushLog(`正在推送 ${file} → /data/local/tmp/${remoteName}`, "info");
      const out = await tauri.adbPushFrida(store.activeSerial, file, remoteName);
      store.pushLog(out, "success");
      await refreshFiles();
    } catch (e) { store.pushLog(`推送失败: ${e}`, "error"); }
  };

  const handleStart = async () => {
    if (!store.selectedFrida) { toast.error("请先选择 frida-server"); return; }
    if (!store.activeSerial) { toast.error("请先连接设备"); return; }
    busy = true;
    store.pushLog(`启动 frida-server (${store.selectedFrida}) :${effectivePort}`, "info");
    try {
      const status = await tauri.fridaStart(store.activeSerial, store.selectedFrida, effectivePort);
      store.fridaStatus = status;
      store.pushLog(status.message, status.running ? "success" : "warn");
      if (status.running) toast.success("frida-server 启动成功");
    } catch (e) {
      store.pushLog(`启动失败: ${e}`, "error");
      toast.error(`启动失败: ${e}`);
    } finally { busy = false; }
  };

  const handleStop = async () => {
    busy = true;
    try {
      const status = await tauri.fridaStop(store.activeSerial);
      store.fridaStatus = status;
      store.pushLog("frida-server 已停止", "info");
    } catch (e) { store.pushLog(`停止失败: ${e}`, "error"); }
    finally { busy = false; }
  };

  const handleForward = async () => {
    if (!store.activeSerial) return;
    try {
      const out = await tauri.fridaForward(store.activeSerial, effectivePort);
      store.pushLog(out, "success");
    } catch (e) { store.pushLog(`端口转发失败: ${e}`, "error"); }
  };

  const refreshProcs = async () => {
    if (!store.activeSerial) { store.pushLog("未选择设备", "warn"); return; }
    procBusy = true;
    try {
      if (procSource === "frida-ps") {
        const list = await tauri.fridaPs(store.activeSerial, effectivePort);
        procItems = list;
        store.pushLog(`frida-ps -H 127.0.0.1:${effectivePort} ⇒ ${list.length} 条`, "success");
      } else {
        const list = await tauri.adbGetProcesses(store.activeSerial);
        procItems = list;
        store.pushLog(`adb shell ps -A ⇒ ${list.length} 条`, "success");
      }
    } catch (e) { store.pushLog(`获取进程失败: ${e}`, "error"); }
    finally { procBusy = false; }
  };

  let filteredProcs = $derived(
    procItems.filter((line) => !filter || line.toLowerCase().includes(filter.toLowerCase())),
  );
</script>

<Group title="Frida 引擎">
  {#snippet icon()}<Cpu class="size-3.5" />{/snippet}

  {#snippet tabs()}
    <button
      class="px-2 py-0.5 text-[11.5px] rounded-md transition-colors {tab === 'server' ? 'bg-primary/10 text-primary' : 'text-muted-foreground hover:text-foreground hover:bg-muted'}"
      onclick={() => (tab = "server")}
    >Server</button>
    <button
      class="px-2 py-0.5 text-[11.5px] rounded-md transition-colors {tab === 'process' ? 'bg-primary/10 text-primary' : 'text-muted-foreground hover:text-foreground hover:bg-muted'}"
      onclick={() => (tab = "process")}
    >Process</button>
  {/snippet}

  {#if tab === "server"}
    <div class="form-row">
      <span class="label">Server</span>
      <Select.Root
        type="single"
        value={store.selectedFrida}
        onValueChange={(v) => (store.selectedFrida = v ?? "")}
      >
        <Select.Trigger class="h-7 font-mono text-xs flex-1 rounded-md">
          {store.selectedFrida || "未发现 frida-server"}
        </Select.Trigger>
        <Select.Content>
          {#each store.fridaFiles as f}
            <Select.Item value={f} class="font-mono text-xs">{f}</Select.Item>
          {/each}
        </Select.Content>
      </Select.Root>
      <Button size="sm" variant="outline" onclick={refreshFiles} disabled={busy}>
        <RefreshCw class="size-3.5 {busy ? 'animate-spin' : ''}" /> 刷新
      </Button>
      <Button size="sm" variant="outline" onclick={pushFromLocal}>
        <Upload class="size-3.5" /> 上传
      </Button>
    </div>

    <div class="form-row">
      <span class="label">端口</span>
      <Input
        type="number"
        class="h-7 w-24 font-mono text-xs rounded-md"
        value={store.customPort ? store.fridaPort : DEFAULT_PORT}
        disabled={!store.customPort}
        oninput={(e) => (store.fridaPort = Number((e.target as HTMLInputElement).value) || DEFAULT_PORT)}
      />
      <label class="flex items-center gap-1.5 text-[11.5px] text-muted-foreground">
        <Switch bind:checked={store.customPort} /> 自定义
      </label>
      <div class="flex-1"></div>
      <Button variant="outline" size="sm" onclick={handleForward}>
        <ArrowRight class="size-3.5" /> forward
      </Button>
      <Button onclick={handleStart} disabled={busy || store.fridaStatus?.running} size="sm">
        <Play class="size-3.5" /> 启动
      </Button>
      <Button
        onclick={handleStop}
        disabled={busy || !store.fridaStatus?.running}
        variant="destructive"
        size="sm"
      >
        <Square class="size-3.5" /> 停止
      </Button>
    </div>

    <div class="grid grid-cols-2 gap-x-6">
      <Kv label="device" value={store.activeSerial || "—"} mono />
      <Kv label="binary" value={store.selectedFrida || "—"} mono />
      <Kv label="port" value={String(effectivePort)} mono />
      <Kv label="status" value={store.fridaStatus?.running ? "alive" : "halted"} mono />
    </div>
  {:else}
    <div class="flex items-center gap-2">
      <div class="relative flex-1">
        <Search class="absolute left-2.5 top-1/2 -translate-y-1/2 size-3.5 text-muted-foreground" />
        <Input
          bind:value={filter}
          placeholder="过滤包名 / pid"
          class="h-7 pl-8 font-mono text-xs rounded-md"
        />
      </div>
      <div class="flex rounded-md border border-border overflow-hidden h-7">
        <button
          onclick={() => (procSource = "frida-ps")}
          class="px-2 text-[11px] font-mono {procSource === 'frida-ps' ? 'bg-primary text-primary-foreground' : 'bg-card text-muted-foreground hover:bg-muted'}"
        >frida-ps</button>
        <button
          onclick={() => (procSource = "adb-ps")}
          class="px-2 text-[11px] font-mono border-l border-border {procSource === 'adb-ps' ? 'bg-primary text-primary-foreground' : 'bg-card text-muted-foreground hover:bg-muted'}"
        >adb ps</button>
      </div>
      <Button onclick={refreshProcs} disabled={procBusy} size="sm">
        <RefreshCw class="size-3.5 {procBusy ? 'animate-spin' : ''}" /> 刷新
      </Button>
    </div>

    <div class="border border-border rounded-md bg-muted/30 overflow-auto h-[160px]">
      <pre class="p-2 font-mono text-[11px] leading-snug text-foreground/80 whitespace-pre">{filteredProcs.length === 0 ? "暂无数据，点 [刷新]" : filteredProcs.join("\n")}</pre>
    </div>
  {/if}
</Group>
