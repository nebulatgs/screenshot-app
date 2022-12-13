import { browser } from "$app/environment";
import { writable, type Writable } from "svelte/store";

function savedWritable<T>(key: string, initial: T): Writable<T> {
	const { subscribe, set, update } = writable<T>(initial);

	if (browser) {
		const json = localStorage.getItem(key);
		if (json) set(JSON.parse(json));
	}

	return {
		subscribe,
		set(value: T) {
			if (browser) localStorage.setItem(key, JSON.stringify(value));
			set(value);
		},
		update,
	};
}

export const bucket = savedWritable("bucket", {
	name: "",
	region: "",
	accessKey: "",
	secretKey: "",
	endpoint: "",
	customDomain: "",
});
