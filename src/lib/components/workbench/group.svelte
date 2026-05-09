<script lang="ts">
  import type { Snippet } from "svelte";

  let {
    title,
    icon,
    tabs,
    actions,
    children,
  }: {
    title: string;
    icon?: Snippet;
    tabs?: Snippet;
    actions?: Snippet;
    children: Snippet;
  } = $props();
</script>

<section class="bg-card border border-border rounded-lg shadow-[0_1px_2px_rgba(0,0,0,0.03)] overflow-hidden">
  <div class="flex items-center gap-2 px-3.5 pt-3 pb-1.5">
    {#if icon}
      <span class="text-muted-foreground inline-flex">{@render icon()}</span>
    {/if}
    <h3 class="text-[12.5px] font-semibold tracking-tight">{title}</h3>
    {#if tabs}
      <div class="ml-1 flex items-center gap-0.5">{@render tabs()}</div>
    {/if}
    <div class="flex-1"></div>
    {#if actions}
      <div class="flex items-center gap-1">{@render actions()}</div>
    {/if}
  </div>
  <div class="px-3.5 pb-3.5 pt-1.5 space-y-2.5">
    {@render children()}
  </div>
</section>
