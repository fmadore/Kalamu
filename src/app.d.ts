// See https://svelte.dev/docs/kit/types#app
declare global {
	interface Window {
		__TAURI_INTERNALS__?: unknown;
	}
}

export {};
