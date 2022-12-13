<script lang="ts">
	import { invoke } from "@tauri-apps/api/tauri";

	import { onMount } from "svelte";
	let opacity = 0;

	let down = false;
	let startX = 0;
	let startY = 0;
	$: width = x - startX;
	$: height = y - startY;
	let x = 0;
	let y = 0;
	let canvasEl: HTMLCanvasElement;

	onMount(() => {
		setTimeout(() => (opacity = 0.5), 100);
		canvasEl.width = canvasEl.clientWidth;
		canvasEl.height = canvasEl.clientHeight;
		const ctx = canvasEl.getContext("2d");
		if (ctx) {
			ctx.fillStyle = "rgba(0, 0, 0, 1)";
			ctx.fillRect(0, 0, canvasEl.width, canvasEl.height);
		}
	});
</script>

<svelte:window
	on:keydown={(e) => {
		if (e.key === "Escape") {
			invoke("close_capture");
		}
	}}
	on:mousedown={(e) => {
		down = true;
		const ctx = canvasEl.getContext("2d");
		if (ctx) {
			ctx.fillStyle = "rgba(0, 0, 0, 1)";
			ctx.fillRect(0, 0, canvasEl.width, canvasEl.height);
		}
		startX = e.offsetX;
		startY = e.offsetY;
		x = startX;
		y = startY;
	}}
	on:mousemove={(e) => {
		if (down) {
			x = e.offsetX;
			y = e.offsetY;
			const ctx = canvasEl.getContext("2d");
			if (ctx) {
				ctx.fillStyle = "rgba(0, 0, 0, 1)";
				ctx.fillRect(0, 0, canvasEl.width, canvasEl.height);
				ctx.clearRect(startX, startY, width, height);
			}
		}
	}}
	on:mouseup={() => {
		down = false;
	}}
	on:click={(e) => {
		if (width >= 150 && height >= 150) {
			const ctx = canvasEl.getContext("2d");
			if (ctx) {
				ctx.clearRect(0, 0, canvasEl.width, canvasEl.height);
				ctx.strokeStyle = "rgba(255, 0, 0, 1)";
				ctx.strokeRect(startX - 2, startY - 2, width + 4, height + 4);
				opacity = 0.8;
			}
			invoke("capture", {
				x1: startX,
				y1: startY,
				x2: e.offsetX,
				y2: e.offsetY,
			});
		} else {
			const ctx = canvasEl.getContext("2d");
			if (ctx) {
				ctx.fillStyle = "rgba(0, 0, 0, 1)";
				ctx.fillRect(0, 0, canvasEl.width, canvasEl.height);
			}
		}
	}}
/>

<canvas
	class="w-screen h-screen top-0 left-0 absolute grid place-items-center text-white opacity-0 transition-opacity duration-500 select-none"
	style="opacity: {opacity};"
	bind:this={canvasEl}
/>
