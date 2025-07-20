<script lang="ts">
  	import { load } from "$lib/api";
  	import { setAppContext } from "$lib/context";
  	import type { AppContext } from "$lib/types";
  	import { onMount, setContext, type Snippet } from "svelte";
  	import { writable } from "svelte/store";
  	import Loader from "./Loader.svelte";
  
	interface Props {
		children?: Snippet;
	}

	let { children }: Props = $props();
	let appContext = writable<AppContext>({
		loadedOn: "",
		esthers: [],
		estherNameToIcon: {},
		arkPassiveIdToSpec: {},
		arkPassives: {},
		bossHpMap: {},
        encounterMap: {},
        difficultyMap: [],
        raidGates: {},
        guardianRaidBosses: [],
        classesMap: {},
        classNameToClassId: {},
        classes: [],
        cardMap: {},
        cardIds: [],
		supportClassIds: [],
	})

	onMount(() => {
		onload()
	})

	async function onload() {
		const result = await load();
		appContext.set(result);
	}

	setAppContext(appContext)

</script>

{#if $appContext.loadedOn }
	{@render children?.()}	
{:else}
	<Loader/>
{/if}