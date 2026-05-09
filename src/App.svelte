<script lang="ts">
  import { onMount } from "svelte";
  import { Button } from "$lib/components/ui/button";
  import * as Select from "$lib/components/ui/select";
  import * as Sheet from "$lib/components/ui/sheet";
  import * as Tooltip from "$lib/components/ui/tooltip";
  import { Toaster } from "$lib/components/ui/sonner";
  import RefreshCw from "@lucide/svelte/icons/refresh-cw";
  import Settings from "@lucide/svelte/icons/settings";
  import Smartphone from "@lucide/svelte/icons/smartphone";
  import Cpu from "@lucide/svelte/icons/cpu";
  import Sun from "@lucide/svelte/icons/sun";
  import Moon from "@lucide/svelte/icons/moon";
  import { store } from "$lib/store.svelte";
  import { tauri } from "$lib/tauri";
  import Workbench from "$components/workbench/workbench.svelte";
  import OutputPanel from "$components/layout/output-panel.svelte";
  import SettingsPanel from "$components/panels/settings-panel.svelte";

  let outputHeight = $state(220);
  let settingsOpen = $state(false);
  let dark = $state(false);

  const refreshDevices = async () => {
    try {
      const list = await tauri.adbDevices();
      store.devices = list;
      if (list.length > 0 && !store.activeSerial) store.activeSerial = list[0].serial;
      store.pushLog(`检测到 ${list.length} 台设备`, "info");
    } catch (e) {
      store.pushLog(`adb 未就绪: ${e}`, "warn");
    }
  };

  onMount(() => {
    store.loadFromStorage();
    dark = localStorage.getItem("frida-workbench:theme") === "dark";
    if (dark) document.documentElement.classList.add("dark");
    refreshDevices();
    return () => store.saveToStorage();
  });

  $effect(() => {
    void store.activeSerial; void store.fridaPort; void store.customPort;
    void store.selectedFrida; void store.providers; void store.activeProviderId;
    store.saveToStorage();
  });

  const toggleDark = () => {
    dark = !dark;
    document.documentElement.classList.toggle("dark", dark);
    localStorage.setItem("frida-workbench:theme", dark ? "dark" : "light");
  };

  const startDrag = (e: MouseEvent) => {
    e.preventDefault();
    const startY = e.clientY;
    const startH = outputHeight;
    const onMove = (ev: MouseEvent) => {
      outputHeight = Math.max(60, Math.min(600, startH + (startY - ev.clientY)));
    };
    const onUp = () => {
      window.removeEventListener("mousemove", onMove);
      window.removeEventListener("mouseup", onUp);
    };
    window.addEventListener("mousemove", onMove);
    window.addEventListener("mouseup", onUp);
  };

  let activeDev = $derived(store.devices.find((d) => d.serial === store.activeSerial));
</script>

<Tooltip.Provider delayDuration={300}>
  <div class="flex h-screen flex-col overflow-hidden bg-background text-foreground select-none">
    <!-- Top bar -->
    <header class="h-10 shrink-0 border-b border-border bg-sidebar flex items-center px-3 gap-2">
      <!-- Device selector -->
      <Smartphone class="size-3.5 text-muted-foreground" />
      <Select.Root
        type="single"
        value={store.activeSerial}
        onValueChange={(v) => (store.activeSerial = v ?? "")}
      >
        <Select.Trigger class="h-7 w-[280px] text-xs font-mono">
          {activeDev ? `${activeDev.serial} · ${activeDev.model || "?"} · ${activeDev.abi || "?"}` : "未连接设备"}
        </Select.Trigger>
        <Select.Content>
          {#if store.devices.length === 0}
            <div class="px-2 py-1.5 text-xs text-muted-foreground">无设备 · 请连接 ADB</div>
          {/if}
          {#each store.devices as d}
            <Select.Item value={d.serial} class="text-xs font-mono">
              {d.serial} · {d.model || "?"} · {d.abi || "?"}
            </Select.Item>
          {/each}
        </Select.Content>
      </Select.Root>

      <Tooltip.Root>
        <Tooltip.Trigger>
          {#snippet child({ props })}
            <Button {...props} size="icon-sm" variant="ghost" class="h-7 w-7" onclick={refreshDevices}>
              <RefreshCw class="size-3.5" />
            </Button>
          {/snippet}
        </Tooltip.Trigger>
        <Tooltip.Content>刷新设备</Tooltip.Content>
      </Tooltip.Root>

      <div class="flex-1"></div>

      <!-- Status pills -->
      <div class="flex items-center gap-1.5">
        <span class="inline-flex items-center gap-1.5 h-6 px-2 rounded-md bg-background border border-border text-[11px] font-mono">
          <span class="tool-status-dot {activeDev ? 'bg-emerald-500' : 'bg-muted-foreground/40'}"></span>
          <span class="text-foreground/80">{activeDev ? activeDev.state : "offline"}</span>
        </span>
        <span class="inline-flex items-center gap-1.5 h-6 px-2 rounded-md bg-background border border-border text-[11px] font-mono">
          <Cpu class="size-3 text-muted-foreground" />
          <span class="tool-status-dot {store.fridaStatus?.running ? 'bg-emerald-500' : 'bg-muted-foreground/40'}"></span>
          <span class="text-foreground/80">{store.fridaStatus?.running ? `:${store.fridaStatus.port}` : "stopped"}</span>
        </span>
      </div>

      <div class="h-4 w-px bg-border mx-1"></div>

      <Tooltip.Root>
        <Tooltip.Trigger>
          {#snippet child({ props })}
            <Button {...props} size="icon-sm" variant="ghost" class="h-7 w-7" onclick={toggleDark}>
              {#if dark}<Sun class="size-3.5" />{:else}<Moon class="size-3.5" />{/if}
            </Button>
          {/snippet}
        </Tooltip.Trigger>
        <Tooltip.Content>切换主题</Tooltip.Content>
      </Tooltip.Root>

      <Sheet.Root bind:open={settingsOpen}>
        <Sheet.Trigger>
          {#snippet child({ props })}
            <Button {...props} size="icon-sm" variant="ghost" class="h-7 w-7">
              <Settings class="size-3.5" />
            </Button>
          {/snippet}
        </Sheet.Trigger>
        <Sheet.Content side="right" class="!max-w-[640px] sm:!max-w-[720px] p-0 w-[640px] sm:w-[720px]">
          <Sheet.Header class="px-4 py-3 border-b border-border">
            <Sheet.Title class="text-[13px]">AI Provider 设置</Sheet.Title>
          </Sheet.Header>
          <div class="h-[calc(100vh-49px)]">
            <SettingsPanel />
          </div>
        </Sheet.Content>
      </Sheet.Root>
    </header>

    <!-- Main -->
    <main class="flex-1 min-h-0 overflow-auto">
      <Workbench />
    </main>

    <!-- Drag handle + output -->
    <div
      role="separator"
      aria-orientation="horizontal"
      tabindex="-1"
      onmousedown={startDrag}
      class="group relative h-1 shrink-0 bg-border cursor-ns-resize hover:bg-primary/60 transition-colors"
    >
      <div class="absolute inset-x-0 -top-1 -bottom-1 z-10"></div>
    </div>
    <div style="height: {outputHeight}px" class="shrink-0">
      <OutputPanel />
    </div>

    <Toaster position="bottom-right" richColors />
  </div>
</Tooltip.Provider>
