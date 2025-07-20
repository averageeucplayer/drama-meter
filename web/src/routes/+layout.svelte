<script lang="ts">
	import { settings } from "$lib/stores.svelte";
	import { type Snippet } from "svelte";
	import "../app.css";
  	import AppProvider from "$lib/components/AppProvider.svelte";
  	import { preventDefault } from "$lib/utils";

	interface Props {
		children?: Snippet;
	}

	let { children }: Props = $props();
</script>

<svelte:window oncontextmenu={preventDefault} />
<AppProvider>
	<div class="{settings.app.general.accentColor} text-sm text-white">
		{@render children?.()}
	</div>
</AppProvider>
