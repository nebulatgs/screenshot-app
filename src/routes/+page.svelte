<script lang="ts">
	import { bucket } from "$lib/stores";
	import { invoke } from "@tauri-apps/api/tauri";
	import { onMount } from "svelte";

	onMount(() => {
		invoke("set_bucket", {
			name: $bucket.name,
			region: $bucket.region,
			access: $bucket.accessKey,
			secret: $bucket.secretKey,
			endpoint: $bucket.endpoint,
			custom: $bucket.customDomain,
		});
	});
</script>

<div class="flex flex-col gap-3">
	<input
		class="border-slate-600 border-2 outline-none p-3 rounded-lg"
		type="text"
		bind:value={$bucket.name}
	/>
	<input
		class="border-slate-600 border-2 outline-none p-3 rounded-lg"
		type="text"
		bind:value={$bucket.region}
	/>
	<input
		class="border-slate-600 border-2 outline-none p-3 rounded-lg"
		type="text"
		bind:value={$bucket.accessKey}
	/>
	<input
		class="border-slate-600 border-2 outline-none p-3 rounded-lg"
		type="text"
		bind:value={$bucket.secretKey}
	/>
	<input
		class="border-slate-600 border-2 outline-none p-3 rounded-lg"
		type="text"
		bind:value={$bucket.endpoint}
	/>
	<input
		class="border-slate-600 border-2 outline-none p-3 rounded-lg"
		type="text"
		bind:value={$bucket.customDomain}
	/>
</div>

<button
	on:click={() => {
		invoke("set_bucket", {
			name: $bucket.name,
			region: $bucket.region,
			access: $bucket.accessKey,
			secret: $bucket.secretKey,
			endpoint: $bucket.endpoint,
			custom: $bucket.customDomain,
		});
	}}>Save</button
>
