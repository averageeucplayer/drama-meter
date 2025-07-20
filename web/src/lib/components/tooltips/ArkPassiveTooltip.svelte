<script lang="ts">
  import QuickTooltip from "$lib/components/QuickTooltip.svelte";
  import { AP, type APTypes } from "$lib/constants/classes";
  import { getAppContext } from "$lib/context";
  import type { EntityState } from "$lib/entity.svelte";
  import type { ArkPassiveNode } from "$lib/types";

  const { state }: { state: EntityState } = $props();
  const appContext = getAppContext();
  
  let arkPassiveSpec = $derived.by(() => {
    if (!state.entity.arkPassiveData || !state.entity.arkPassiveData.enlightenment) return "";
    for (const node of state.entity.arkPassiveData.enlightenment) {
      const specName = $appContext.arkPassiveIdToSpec[node.id] || "Unknown";
      if (specName !== "Unknown") {
        return specName;
      }
    }
  });

</script>

{#snippet renderTree(name: string, tree?: ArkPassiveNode[])}
  {@const [text, color] = AP[name as APTypes]}
  {#if tree && tree.length}
    <div class="text-purple-400">[{text}]</div>
    <div class="flex flex-col">
      {#each tree as node}
        {@const data = $appContext.arkPassives[node.id]}
        {#if data}
          <div class="flex items-center gap-1">
            <div class={color}>T{data[3] + 1} {data[0]}</div>
            <span class="text-white">Lv. {node.lv}</span>
          </div>
        {/if}
      {/each}
    </div>
  {/if}
{/snippet}

{#snippet tooltip()}
  <div class="flex flex-col">
    <div>
      {state.name}
    </div>
    <div class="text-xs">
      {#if state.entity.arkPassiveData && state.entity.spec}
        {#if arkPassiveSpec == state.entity.spec}
          {@render renderTree("evolution", state.entity.arkPassiveData.evolution)}
          {@render renderTree("enlightenment", state.entity.arkPassiveData.enlightenment)}
          {@render renderTree("leap", state.entity.arkPassiveData.leap)}
        {:else}
          <div class="text-violet-400">Mismatched Ark Passive Data</div>
        {/if}
      {/if}
    </div>
  </div>
{/snippet}

<QuickTooltip {tooltip} delay={500} class="truncate">
  {state.name}
</QuickTooltip>
