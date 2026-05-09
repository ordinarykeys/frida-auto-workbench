<script lang="ts">
  let {
    label,
    value,
    mono,
    danger,
    accent,
  }: {
    label: string;
    value?: string;
    mono?: boolean;
    danger?: boolean;
    accent?: string;
  } = $props();
</script>

<div class="flex items-baseline justify-between gap-3 min-w-0 py-1 border-b border-border/40 last:border-b-0 [&:nth-last-child(2)]:border-b-0">
  <span class="text-[11px] text-muted-foreground shrink-0 uppercase tracking-wider">{label}</span>
  <span
    class="text-[12.5px] truncate text-right text-foreground/90 {mono ? 'font-mono' : ''} {danger ? 'text-red-500' : ''}"
    title={value || ""}
  >
    {value || "—"}
    {#if accent}<span class="ml-1.5 text-[10px] text-primary font-mono">[{accent}]</span>{/if}
  </span>
</div>
