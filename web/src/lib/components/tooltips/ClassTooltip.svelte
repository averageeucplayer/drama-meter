<script lang="ts">
  import QuickTooltip from "$lib/components/QuickTooltip.svelte";
  import type { Entity } from "$lib/types";
  const { entity: entity }: { entity: Entity } = $props();
</script>

{#snippet tooltip()}
  <div class="flex">
    {#if entity.arkPassiveActive}
      <div class="mr-1"><span class="text-purple-400">[Ark Passive]</span></div>
    {/if}
    {#if entity.spec}
      <span>{entity.spec} {entity.class}</span>
    {:else}
      <span>{entity.class}</span>
    {/if}
  </div>
{/snippet}

<QuickTooltip {tooltip}>
  <div class="">
    <div role="img" class={`table-cell class_${entity.classId}`} aria-label={entity.class}></div>
  </div>
</QuickTooltip>
