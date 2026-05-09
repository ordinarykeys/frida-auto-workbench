<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import Trash2 from "@lucide/svelte/icons/trash-2";
  import Terminal from "@lucide/svelte/icons/terminal";
  import { store } from "$lib/store.svelte";
  import { cn } from "$lib/utils";

  let autoScroll = $state(true);
  let scrollEl: HTMLDivElement | undefined = $state();

  $effect(() => {
    void store.logs;
    if (autoScroll && scrollEl) scrollEl.scrollTop = scrollEl.scrollHeight;
  });

  const LEVEL_COLOR: Record<string, string> = {
    info: "text-foreground/80",
    success: "text-emerald-600 dark:text-emerald-400",
    error: "text-red-600 dark:text-red-400",
    warn: "text-amber-600 dark:text-amber-400",
    ai: "text-primary",
  };
</script>

<div class="flex flex-col h-full bg-sidebar border-t-0">
  <div class="h-8 shrink-0 flex items-center px-3 gap-2 border-b border-border">
    <Terminal class="size-3.5 text-muted-foreground" />
    <span class="text-[11px] font-semibold uppercase tracking-[0.1em] text-muted-foreground">Output</span>
    <span class="text-[10px] text-muted-foreground/70 font-mono">{store.logs.length} 条</span>
    <div class="flex-1"></div>
    <label class="flex items-center gap-1.5 text-[11px] text-muted-foreground cursor-pointer select-none">
      <input type="checkbox" class="accent-primary scale-90" bind:checked={autoScroll} />
      自动滚动
    </label>
    <Button size="icon-sm" variant="ghost" class="h-6 w-6" onclick={() => store.clearLogs()} title="清空">
      <Trash2 class="size-3" />
    </Button>
  </div>
  <div bind:this={scrollEl} class="flex-1 overflow-auto px-3 py-2 font-mono text-[12px] leading-snug bg-background">
    {#if store.logs.length === 0}
      <div class="text-muted-foreground/50">_ 等待指令…</div>
    {:else}
      {#each store.logs as log (log.id)}
        <div class="flex gap-2">
          <span class="text-muted-foreground/50 shrink-0 select-none">[{log.time}]</span>
          <span class={cn("whitespace-pre-wrap break-all", LEVEL_COLOR[log.level])}>{log.msg}</span>
        </div>
      {/each}
    {/if}
  </div>
</div>
