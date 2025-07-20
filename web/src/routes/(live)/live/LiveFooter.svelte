<script lang="ts">
  import { MeterTab } from "$lib/types";
  import { getAppContext } from "$lib/context";

  let { tab = $bindable() }: { tab: MeterTab } = $props();
  const appContext = getAppContext();
</script>

{#snippet meterTab(name: string, t: MeterTab)}
  <button class="rounded-xs shrink-0 px-1.5 transition {tab === t ? 'bg-accent-500/40' : ''}" onclick={() => (tab = t)}>
    {name}
  </button>
{/snippet}
<div class="flex h-6 select-none items-center justify-between bg-neutral-800/70 px-1 text-neutral-300">
  <div class="flex h-full items-center overflow-x-scroll text-xs">
    {@render meterTab("DPS", MeterTab.DAMAGE)}
    {@render meterTab("PARTY", MeterTab.PARTY_BUFFS)}
    {@render meterTab("SELF", MeterTab.SELF_BUFFS)}
    {@render meterTab("TANK", MeterTab.TANK)}
    {@render meterTab("BOSS", MeterTab.BOSS)}
  </div>
  <div class="flex items-center gap-1 px-1 tracking-tighter">
    <div class="text-xs">{$appContext.appName}</div>
    <div class="text-xs text-neutral-500">
      {$appContext.version}
    </div>
  </div>
</div>
