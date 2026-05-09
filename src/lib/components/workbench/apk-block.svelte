<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import * as Select from "$lib/components/ui/select";
  import { open } from "@tauri-apps/plugin-dialog";
  import { toast } from "svelte-sonner";
  import Upload from "@lucide/svelte/icons/upload";
  import Play from "@lucide/svelte/icons/play";
  import Package from "@lucide/svelte/icons/package";
  import { store } from "$lib/store.svelte";
  import { tauri } from "$lib/tauri";
  import Group from "./group.svelte";
  import Kv from "./kv.svelte";

  let busy = $state(false);

  function formatSize(bytes: number) {
    if (!bytes) return "—";
    const units = ["B", "KB", "MB", "GB"];
    let v = bytes;
    let i = 0;
    while (v >= 1024 && i < units.length - 1) { v /= 1024; i++; }
    return `${v.toFixed(2)} ${units[i]}`;
  }

  let filteredSo = $derived.by(() => {
    if (!store.apk) return [];
    const preferred = store.apk.preferred_arch || "arm64-v8a";
    const rest = store.apk.so_files.filter((s) => s.arch === preferred);
    return rest.length > 0 ? rest : store.apk.so_files;
  });

  const pick = async () => {
    try {
      const f = await open({ multiple: false, filters: [{ name: "Android Package", extensions: ["apk", "aab"] }] });
      if (typeof f === "string") { store.apkPath = f; store.pushLog(`已选择 APK: ${f}`, "info"); }
    } catch (e) { store.pushLog(`选择失败: ${e}`, "error"); }
  };

  const analyze = async () => {
    if (!store.apkPath) { toast.error("请先选择 APK 文件"); return; }
    busy = true;
    store.pushLog(`开始分析 APK: ${store.apkPath}`, "info");
    try {
      const result = await tauri.analyzeApk(store.apkPath);
      store.apk = result;
      const preferred = result.so_files.find((s) => s.arch === (result.preferred_arch || "arm64-v8a"));
      store.selectedSo = preferred?.name || result.so_files[0]?.name || "";
      store.pushLog(
        `完成: ${result.package_name} · ${result.architectures.join("/")} · ${result.so_count} so · ${result.hardening}`,
        "success",
      );
      toast.success("APK 分析完成");
    } catch (e) {
      store.pushLog(`分析失败: ${e}`, "error");
      toast.error(`分析失败: ${e}`);
    } finally { busy = false; }
  };
</script>

<Group title="APK 目标">
  {#snippet icon()}<Package class="size-3.5" />{/snippet}

  <div class="flex items-center gap-2">
    <Input
      bind:value={store.apkPath}
      placeholder="选择 .apk / .aab"
      readonly
      class="font-mono text-xs h-7 flex-1 rounded-md"
    />
    <Button onclick={pick} variant="outline" size="sm">
      <Upload class="size-3.5" /> 浏览
    </Button>
    <Button onclick={analyze} disabled={!store.apkPath || busy} size="sm">
      <Play class="size-3.5" /> {busy ? "分析中" : "分析"}
    </Button>
  </div>

  <div class="grid grid-cols-2 gap-x-6 -mt-0.5">
    <Kv label="包名" value={store.apk?.package_name} mono />
    <Kv label="加固" value={store.apk?.hardening} danger={store.apk?.hardening !== "未加固" && !!store.apk?.hardening} />
    <Kv label="架构" value={store.apk?.architectures.length ? store.apk.architectures.join("/") : undefined} mono accent={store.apk?.preferred_arch} />
    <Kv label="SO 数量" value={store.apk ? `${store.apk.so_count}` : undefined} mono />
    <Kv label="version" value={store.apk ? `${store.apk.version_name || "—"} (${store.apk.version_code || "—"})` : undefined} mono />
    <Kv label="size" value={formatSize(store.apk?.apk_size || 0)} mono />
  </div>

  <div class="form-row pt-1">
    <span class="label">目标 SO</span>
    <Select.Root
      type="single"
      value={store.selectedSo}
      onValueChange={(v) => (store.selectedSo = v ?? "")}
    >
      <Select.Trigger class="h-7 font-mono text-xs flex-1 rounded-md">
        {store.selectedSo || "先点 [分析] 提取 .so 列表"}
      </Select.Trigger>
      <Select.Content>
        {#each store.apk?.so_files ?? [] as s}
          <Select.Item value={s.name} class="font-mono text-xs">
            <span class="tool-pill mr-2 {s.arch === store.apk?.preferred_arch ? 'border-primary text-primary' : 'border-border text-muted-foreground'}">{s.arch}</span>
            {s.name}
          </Select.Item>
        {/each}
      </Select.Content>
    </Select.Root>
    <span class="text-[11px] font-mono text-muted-foreground shrink-0 w-14 text-right">
      {filteredSo.length}/{store.apk?.so_count ?? 0}
    </span>
  </div>
</Group>
