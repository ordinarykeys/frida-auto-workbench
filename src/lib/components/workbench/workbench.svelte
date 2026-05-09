<script lang="ts">
  import ApkBlock from "./apk-block.svelte";
  import FridaBlock from "./frida-block.svelte";
  import AiBlock from "./ai-block.svelte";
</script>

<div class="px-3 py-3 space-y-3 max-w-[1600px] mx-auto">
  <div class="grid grid-cols-2 gap-3">
    <ApkBlock />
    <FridaBlock />
  </div>
  <AiBlock />
</div>
