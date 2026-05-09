<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";
  import * as Select from "$lib/components/ui/select";
  import { toast } from "svelte-sonner";
  import Plus from "@lucide/svelte/icons/plus";
  import Trash2 from "@lucide/svelte/icons/trash-2";
  import Save from "@lucide/svelte/icons/save";
  import RefreshCw from "@lucide/svelte/icons/refresh-cw";
  import Check from "@lucide/svelte/icons/check";
  import { store, PROVIDER_PRESETS } from "$lib/store.svelte";
  import { tauri } from "$lib/tauri";
  import type { AiProvider } from "$lib/tauri";
  import { cn } from "$lib/utils";

  let editingId = $state(store.activeProviderId);
  let editing = $derived(store.providers.find((p) => p.id === editingId) ?? store.providers[0]);

  let draft = $state<AiProvider>({
    id: store.providers[0]?.id ?? "",
    name: store.providers[0]?.name ?? "",
    base_url: store.providers[0]?.base_url ?? "",
    api_key: store.providers[0]?.api_key ?? "",
    model: store.providers[0]?.model ?? "",
  });
  let models = $state<string[]>(store.providers[0]?.model ? [store.providers[0].model] : []);
  let fetching = $state(false);

  $effect(() => {
    const p = store.providers.find((x) => x.id === editingId);
    if (p) {
      draft = { ...p };
      models = p.model ? [p.model] : [];
    }
  });

  const newProvider = () => {
    const id = `p_${Date.now()}`;
    editingId = id;
    draft = { id, name: "新 Provider", base_url: "https://api.openai.com/v1", api_key: "", model: "" };
    models = [];
  };

  const save = () => {
    if (!draft.id || !draft.base_url) { toast.error("ID 与 Base URL 必填"); return; }
    store.upsertProvider({ ...draft });
    store.activeProviderId = draft.id;
    toast.success("Provider 已保存");
    store.pushLog(`已保存 Provider: ${draft.name} (${draft.model || "未选模型"})`, "success");
  };

  const del = () => {
    if (store.providers.length <= 1) { toast.error("至少保留一个 Provider"); return; }
    store.removeProvider(draft.id);
    const next = store.providers.find((p) => p.id !== draft.id);
    if (next) editingId = next.id;
  };

  const fetchModels = async () => {
    if (!draft.base_url) { toast.error("请先填写 Base URL"); return; }
    fetching = true;
    try {
      const list = await tauri.aiListModels(draft.base_url, draft.api_key);
      models = list;
      if (!draft.model && list[0]) draft = { ...draft, model: list[0] };
      store.pushLog(`从 ${draft.base_url} 拉到 ${list.length} 个模型`, "success");
      toast.success(`找到 ${list.length} 个模型`);
    } catch (e) {
      store.pushLog(`拉取模型失败: ${e}`, "error");
      toast.error(`拉取失败: ${e}`);
    } finally { fetching = false; }
  };

  const applyPreset = (presetId: string | null) => {
    if (!presetId) return;
    const preset = PROVIDER_PRESETS.find((p) => p.id === presetId);
    if (!preset) return;
    draft = { ...draft, name: preset.name, base_url: preset.base_url };
  };
</script>

<div class="grid grid-cols-[220px_1fr] h-full">
  <!-- Provider list -->
  <div class="border-r border-border flex flex-col">
    <div class="h-9 flex items-center justify-between px-3 border-b border-border bg-sidebar">
      <span class="text-[11px] uppercase tracking-[0.1em] font-semibold text-muted-foreground">Providers</span>
      <Button size="icon-sm" variant="ghost" class="h-6 w-6" onclick={newProvider}>
        <Plus class="size-3" />
      </Button>
    </div>
    <div class="flex-1 overflow-auto">
      {#each store.providers as p (p.id)}
        <button
          onclick={() => (editingId = p.id)}
          class={cn(
            "w-full text-left px-3 py-2 text-xs flex items-center gap-2 hover:bg-accent border-b border-border",
            editingId === p.id && "bg-accent",
          )}
        >
          <span class="flex-1 truncate font-medium">{p.name}</span>
          {#if store.activeProviderId === p.id}
            <span class="text-[9px] font-mono px-1 py-0.5 border border-primary text-primary rounded">ACTIVE</span>
          {/if}
        </button>
      {/each}
    </div>
  </div>

  <!-- Editor -->
  <div class="flex flex-col">
    <div class="h-9 flex items-center justify-end px-3 border-b border-border bg-sidebar gap-2">
      {#if store.activeProviderId !== draft.id}
        <Button size="sm" variant="outline" class="h-7" onclick={() => (store.activeProviderId = draft.id)}>
          <Check class="size-3.5" /> 设为当前
        </Button>
      {/if}
      <Button size="sm" variant="outline" class="h-7" onclick={del}>
        <Trash2 class="size-3.5" /> 删除
      </Button>
      <Button size="sm" class="h-7" onclick={save}>
        <Save class="size-3.5" /> 保存
      </Button>
    </div>

    <div class="flex-1 overflow-auto px-4 py-3 space-y-3">
      <div class="grid grid-cols-2 gap-3">
        <div class="space-y-1">
          <Label for="ai-name" class="text-[10.5px] text-muted-foreground uppercase tracking-wider">名称</Label>
          <Input id="ai-name" bind:value={draft.name} class="h-8 text-xs" />
        </div>
        <div class="space-y-1">
          <Label for="ai-preset" class="text-[10.5px] text-muted-foreground uppercase tracking-wider">预设</Label>
          <Select.Root type="single" onValueChange={applyPreset}>
            <Select.Trigger id="ai-preset" class="h-8 text-xs">选择预设</Select.Trigger>
            <Select.Content>
              {#each PROVIDER_PRESETS as p}
                <Select.Item value={p.id} class="text-xs">{p.name}</Select.Item>
              {/each}
            </Select.Content>
          </Select.Root>
        </div>
      </div>

      <div class="space-y-1">
        <Label for="ai-base" class="text-[10.5px] text-muted-foreground uppercase tracking-wider">Base URL (兼容 OpenAI · 含 /v1)</Label>
        <Input id="ai-base" bind:value={draft.base_url} class="h-8 font-mono text-xs" />
      </div>

      <div class="space-y-1">
        <Label for="ai-key" class="text-[10.5px] text-muted-foreground uppercase tracking-wider">API Key</Label>
        <Input id="ai-key" type="password" bind:value={draft.api_key} placeholder="sk-..." class="h-8 font-mono text-xs" />
      </div>

      <div class="space-y-1">
        <Label for="ai-model" class="text-[10.5px] text-muted-foreground uppercase tracking-wider">模型 (动态获取)</Label>
        <div class="flex gap-2">
          <Select.Root type="single" value={draft.model} onValueChange={(v) => (draft = { ...draft, model: v ?? "" })}>
            <Select.Trigger id="ai-model" class="h-8 font-mono text-xs flex-1">
              {draft.model || "先点 [获取模型] 拉取列表"}
            </Select.Trigger>
            <Select.Content>
              {#each models as m}
                <Select.Item value={m} class="font-mono text-xs">{m}</Select.Item>
              {/each}
            </Select.Content>
          </Select.Root>
          <Button onclick={fetchModels} disabled={fetching} variant="outline" size="sm" class="h-8">
            <RefreshCw class="size-3.5 {fetching ? 'animate-spin' : ''}" /> 获取模型
          </Button>
        </div>
      </div>

      <div class="text-[11px] text-muted-foreground border-t border-border pt-3 mt-2 space-y-1">
        <ul class="list-disc list-inside space-y-0.5">
          <li>OpenAI 兼容协议: Base URL 以 <span class="font-mono">/v1</span> 结尾。</li>
          <li>本地 Ollama: <span class="font-mono">http://localhost:11434/v1</span>，无需 Key。</li>
          <li>模型列表通过 <span class="font-mono">GET /v1/models</span> 实时获取。</li>
          <li>Key 仅保存到本机 <span class="font-mono">localStorage</span>。</li>
        </ul>
      </div>
    </div>
  </div>
</div>
