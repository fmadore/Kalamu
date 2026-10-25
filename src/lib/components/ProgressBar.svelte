<script lang="ts">
	// A determinate bar: finished in ink, failed in red, the rest the track.
	let {
		done,
		failed = 0,
		total,
		label
	}: { done: number; failed?: number; total: number; label: string } = $props();

	const pct = (n: number) => (total > 0 ? (100 * n) / total : 0);
</script>

<div
	class="bar"
	role="progressbar"
	aria-label={label}
	aria-valuemin={0}
	aria-valuemax={total}
	aria-valuenow={done}
>
	<span class="done" style:width="{pct(done)}%"></span>
	<span class="failed" style:width="{pct(failed)}%"></span>
</div>

<style>
	.bar {
		display: flex;
		height: 6px;
		border-radius: 3px;
		background: var(--line);
		overflow: hidden;
	}
	.done {
		background: var(--ink);
		transition: width 0.3s ease;
	}
	.failed {
		background: var(--danger);
	}
</style>
