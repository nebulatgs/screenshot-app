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

	let settings = [
		"System",
		"Bluetooth & devices",
		"Network & internet",
		"Personalization",
		"Apps",
		"Accounts",
		"Time & language",
		"Gaming",
		"Accessibility",
		"Privacy & security",
		"Windows Update",
	];

	let active = settings[7];
</script>

<!-- <div class="flex flex-col gap-3">
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
> -->

<div class="flex text-white w-screen h-screen p-6 gap-6">
	<div class="w-72 flex-shrink-0 flex flex-col gap-1">
		{#each settings as setting}
			<div
				class="bg-white rounded-md p-2 px-3 flex items-center gap-5"
				class:hover:bg-opacity-5={setting === active}
				class:hover:bg-opacity-10={setting !== active}
				class:bg-opacity-0={setting !== active}
				class:bg-opacity-10={setting === active}
			>
				<div class="w-6 h-6 bg-black grid place-items-center" />
				<h2 class="text-sm">{setting}</h2>
			</div>
		{/each}
	</div>
	<div class="w-full flex flex-col gap-6">
		<h1 class="text-3xl font-semibold">Gaming</h1>
		<div class="flex flex-col gap-1">
			<div
				class="bg-white bg-opacity-5 hover:bg-opacity-10 rounded-sm p-4 flex items-center gap-5"
			>
				<div class="w-10 h-10 bg-black grid place-items-center" />
				<div class="flex flex-col">
					<h2 class="text-sm">Xbox Game Bar</h2>
					<p class="text-xs text-secondary">
						Controller and keyboard shortcuts
					</p>
				</div>
			</div>
			<div
				class="bg-white bg-opacity-5 hover:bg-opacity-10 rounded-sm p-4 flex items-center gap-5"
			>
				<div class="w-10 h-10 bg-black grid place-items-center" />
				<div class="flex flex-col">
					<h2 class="text-sm">Captures</h2>
					<p class="text-xs text-secondary">
						Save location, recording preferences
					</p>
				</div>
			</div>
			<div
				class="bg-white bg-opacity-5 hover:bg-opacity-10 rounded-sm p-4 flex items-center gap-5"
			>
				<div class="w-10 h-10 bg-black grid place-items-center" />
				<div class="flex flex-col">
					<h2 class="text-sm">Game Mode</h2>
					<p class="text-xs text-secondary">Optimize your PC for play</p>
				</div>
			</div>
		</div>
	</div>
</div>
