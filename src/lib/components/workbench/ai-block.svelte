<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { Textarea } from "$lib/components/ui/textarea";
  import * as Select from "$lib/components/ui/select";
  import * as Dialog from "$lib/components/ui/dialog";
  import { toast } from "svelte-sonner";
  import Bot from "@lucide/svelte/icons/bot";
  import Zap from "@lucide/svelte/icons/zap";
  import Eye from "@lucide/svelte/icons/eye";
  import Copy from "@lucide/svelte/icons/copy";
  import Code from "@lucide/svelte/icons/code";
  import Sparkles from "@lucide/svelte/icons/sparkles";
  import { store, type ChatMessage } from "$lib/store.svelte";
  import { tauri } from "$lib/tauri";
  import Group from "./group.svelte";

  const INTENT_PRESETS = [
    "hook 关键 native 函数并打印参数 / 返回值",
    "绕过 SSL Pinning (OkHttp / X509TrustManager)",
    "Java 层 dump 加密参数",
    "反调试与反 Frida 检测绕过",
    "Trace 所有 RegisterNatives 注册的 JNI 方法",
  ];

  let intent = $state(INTENT_PRESETS[0]);
  let script = $state("");
  let analysis = $state("");
  let phase: "idle" | "plan" | "code" | "verify" = $state("idle");
  let dlgOpen = $state(false);
  let dlgTab: "script" | "analysis" = $state("script");

  let provider = $derived(store.activeProvider);

  const ensureProvider = (): boolean => {
    const p = store.activeProvider;
    if (!p || !p.api_key || !p.model) {
      toast.error("请先打开右上角 ⚙ 配置 AI Provider 并选择模型");
      return false;
    }
    return true;
  };

  function extractCode(text: string): string | null {
    const m = text.match(/```(?:javascript|js)?\n([\s\S]*?)```/);
    return m ? m[1].trim() : null;
  }

  const runFlow = async () => {
    if (!ensureProvider() || !provider) return;
    if (!store.apk) { toast.error("请先在 [APK] 区分析目标"); return; }
    store.aiBusy = true;
    script = "";
    analysis = "";
    try {
      phase = "plan";
      store.pushLog(`[AI] 阶段 1/3 · 目标分析 → ${provider.model}`, "ai");
      const planPrompt = `分析下列 Android 目标，输出 hook 思路 (≤6 条要点，中文)：
- 包名: ${store.apk.package_name}
- 加固: ${store.apk.hardening}
- 架构: ${store.apk.architectures.join("/")} (优先 ${store.apk.preferred_arch})
- 目标 SO: ${store.selectedSo || "(未选)"}
- 用户意图: ${intent}
仅输出 markdown 列表，不要代码。`;
      const plan = await tauri.aiChat(provider, [
        { role: "system", content: "你是资深 Android 逆向工程师，回答简洁专业。" },
        { role: "user", content: planPrompt },
      ] as ChatMessage[], 0.2);
      analysis = plan;
      store.pushLog(`[AI] 分析完成 (${plan.length} 字符)`, "ai");

      phase = "code";
      store.pushLog(`[AI] 阶段 2/3 · 生成 Frida 脚本`, "ai");
      const code = await tauri.aiGenerateScript(
        provider, store.apk.package_name, store.selectedSo, store.apk.hardening,
        `${intent}\n\n参考分析:\n${plan}`,
      );
      script = code;
      store.pushLog(`[AI] 脚本生成完成 (${code.length} 字符)`, "ai");

      phase = "verify";
      store.pushLog(`[AI] 阶段 3/3 · 自检改进`, "ai");
      const verifyPrompt = `请审查这段 Frida 脚本，列出 ≤3 条潜在问题并给出最终改进版（仅输出一个 \`\`\`javascript 代码块）。

\`\`\`javascript
${code}
\`\`\``;
      const final = await tauri.aiChat(provider, [
        { role: "system", content: "你是 Frida 资深用户，仅输出最终的 ```javascript 代码块。" },
        { role: "user", content: verifyPrompt },
      ] as ChatMessage[], 0.2);
      const extracted = extractCode(final) || code;
      script = extracted;
      store.pushLog(`[AI] 自检完成，最终脚本 ${extracted.length} 字符`, "success");
      toast.success("脚本已生成");
      dlgOpen = true;
    } catch (e) {
      store.pushLog(`[AI] 失败: ${e}`, "error");
      toast.error(`AI 失败: ${e}`);
    } finally {
      store.aiBusy = false;
      phase = "idle";
    }
  };

  const copy = async () => {
    if (!script) return;
    try {
      await navigator.clipboard.writeText(script);
      toast.success("脚本已复制");
    } catch { toast.error("复制失败"); }
  };

  const PHASE_LABEL = { idle: "", plan: "分析目标…", code: "生成脚本…", verify: "自检改进…" } as const;
</script>

<Group title="AI 自动生成 Frida 脚本">
  {#snippet icon()}<Sparkles class="size-3.5" />{/snippet}

  <div class="form-row">
    <span class="label">意图预设</span>
    <Select.Root type="single" value={intent} onValueChange={(v) => (intent = v ?? "")}>
      <Select.Trigger class="h-7 text-xs flex-1 rounded-md">{intent}</Select.Trigger>
      <Select.Content>
        {#each INTENT_PRESETS as p}
          <Select.Item value={p} class="text-xs">{p}</Select.Item>
        {/each}
      </Select.Content>
    </Select.Root>
  </div>

  <div class="form-row">
    <span class="label">自定义</span>
    <Input
      bind:value={intent}
      placeholder="可直接编辑意图…"
      class="h-7 text-xs flex-1 rounded-md"
    />
    <Button onclick={runFlow} disabled={store.aiBusy} size="sm">
      {#if store.aiBusy}
        <Zap class="size-3.5 animate-pulse" />
      {:else}
        <Bot class="size-3.5" />
      {/if}
      {store.aiBusy ? PHASE_LABEL[phase] : "AI 一键生成"}
    </Button>
    <Button onclick={() => (dlgOpen = true)} disabled={!script} variant="outline" size="sm">
      <Eye class="size-3.5" /> 查看脚本
    </Button>
  </div>

  <div class="form-row">
    <span class="label">上下文</span>
    <span class="text-[11.5px] text-muted-foreground truncate flex-1">
      {provider ? `${provider.name}${provider.model ? " · " + provider.model : ""}` : "(未配置 Provider)"}
      {#if store.apk}{" · "}{store.apk.package_name}{/if}
      {#if store.selectedSo}{" · "}{store.selectedSo}{/if}
      {#if store.apk}{" · "}{store.apk.hardening}{/if}
      {#if script}<span class="ml-2 text-emerald-600 font-mono">脚本就绪 ({script.length} chars)</span>{/if}
    </span>
  </div>
</Group>

<Dialog.Root bind:open={dlgOpen}>
  <Dialog.Content class="!max-w-[1000px] w-[90vw] max-h-[90vh] flex flex-col p-0 gap-0">
    <Dialog.Header class="px-4 py-3 border-b border-border flex flex-row items-center justify-between space-y-0">
      <Dialog.Title class="text-[13px] flex items-center gap-2">
        <Code class="size-4" /> 生成结果
      </Dialog.Title>
      <Button size="sm" variant="outline" onclick={copy} disabled={!script}>
        <Copy class="size-3.5" /> 复制脚本
      </Button>
    </Dialog.Header>
    <div class="flex border-b border-border px-4">
      <button
        class="px-3 py-2 text-[11.5px] {dlgTab === 'script' ? 'text-primary border-b-2 border-primary -mb-px' : 'text-muted-foreground hover:text-foreground'}"
        onclick={() => (dlgTab = "script")}
      >Script</button>
      <button
        class="px-3 py-2 text-[11.5px] {dlgTab === 'analysis' ? 'text-primary border-b-2 border-primary -mb-px' : 'text-muted-foreground hover:text-foreground'}"
        onclick={() => (dlgTab = "analysis")}
      >Analysis</button>
    </div>
    <div class="flex-1 min-h-0 overflow-auto p-4">
      {#if dlgTab === "script"}
        <Textarea
          bind:value={script}
          placeholder="脚本将显示在此处。可手动微调，再点 [复制] 拷贝。"
          class="font-mono text-[11.5px] min-h-[60vh] resize-none leading-snug"
        />
      {:else}
        <div class="text-[12.5px] whitespace-pre-wrap leading-relaxed">{analysis || "_ 暂无分析"}</div>
      {/if}
    </div>
  </Dialog.Content>
</Dialog.Root>
