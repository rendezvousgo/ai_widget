<script lang="ts">
  import { onMount } from "svelte";

  let {
    label,
    used,
    limit,
    unit = "",
    resetAt,
    accent,
    delay = 0,
  }: {
    label: string;
    used: number;
    limit: number;
    unit?: string;
    resetAt?: string;
    accent: string;
    glow?: string;
    delay?: number;
  } = $props();

  let widthPct = $state(0);

  onMount(() => {
    const target = limit > 0 ? Math.min(1, used / limit) * 100 : 0;
    const id = setTimeout(() => (widthPct = target), delay + 30);
    return () => clearTimeout(id);
  });

  const fmt = (n: number): string => {
    if (n >= 1_000_000) return (n / 1_000_000).toFixed(2) + "M";
    if (n >= 1_000) return (n / 1_000).toFixed(1) + "k";
    if (Number.isInteger(n)) return n.toString();
    return n.toFixed(2);
  };

  const pctNum = $derived(Math.round((limit > 0 ? Math.min(1, used / limit) : 0) * 100));
  const status = $derived(pctNum > 90 ? "danger" : pctNum > 75 ? "warn" : "ok");
</script>

<div class="bar-row">
  <div class="bar-meta">
    <span class="bar-label">▸ {label}</span>
    {#if resetAt}<span class="bar-reset">↻ {resetAt}</span>{/if}
  </div>
  <div class="bar-track" style="--accent: {accent};">
    <div class="bar-segments"></div>
    <div class="bar-fill" style="width: {widthPct}%"></div>
  </div>
  <div class="bar-stats">
    <span class="bar-pct" data-status={status}>{pctNum}<span class="bar-pct-sym">%</span></span>
    {#if unit !== "%"}
      <span class="bar-num">
        <span class="bar-used">{fmt(used)}</span>
        <span class="bar-sep">/</span>
        <span class="bar-lim">{fmt(limit)}</span>
        {#if unit}<span class="bar-unit">{unit}</span>{/if}
      </span>
    {/if}
  </div>
</div>

<style>
  .bar-row {
    display: flex;
    flex-direction: column;
    gap: 4px;
    padding: 5px 0;
  }
  .bar-meta {
    display: flex;
    justify-content: space-between;
    align-items: center;
    font-family: var(--mono);
    font-size: 9px;
    text-transform: uppercase;
    letter-spacing: 0.14em;
  }
  .bar-label { color: var(--text-1); }
  .bar-reset { color: var(--text-3); }
  .bar-track {
    position: relative;
    height: 8px;
    background: var(--bg-0);
    border: 1px solid var(--line-strong);
    overflow: hidden;
  }
  .bar-segments {
    position: absolute;
    inset: 0;
    background-image: linear-gradient(
      90deg,
      transparent 0,
      transparent calc(10% - 1px),
      var(--bg-2) calc(10% - 1px),
      var(--bg-2) 10%
    );
    background-size: 10% 100%;
    background-repeat: repeat-x;
    pointer-events: none;
  }
  .bar-fill {
    position: absolute;
    inset: 0 auto 0 0;
    background: linear-gradient(
      90deg,
      color-mix(in oklab, var(--accent) 70%, var(--bg-0)) 0%,
      var(--accent) 100%
    );
    width: 0;
    transition: width 700ms cubic-bezier(0.2, 0.8, 0.2, 1);
    border-right: 1px solid var(--accent);
  }
  .bar-stats {
    display: flex;
    justify-content: space-between;
    align-items: baseline;
    font-family: var(--mono);
    font-size: 11px;
    margin-top: 1px;
  }
  .bar-pct {
    font-weight: 700;
    letter-spacing: 0.04em;
    font-size: 13px;
  }
  .bar-pct-sym { font-size: 9px; opacity: 0.7; margin-left: 1px; }
  .bar-pct[data-status="ok"] { color: var(--ok); }
  .bar-pct[data-status="warn"] { color: var(--warn); }
  .bar-pct[data-status="danger"] { color: var(--danger); }
  .bar-num { color: var(--text-2); font-size: 10px; }
  .bar-used { color: var(--text-0); font-weight: 600; }
  .bar-sep { color: var(--text-3); margin: 0 3px; }
  .bar-lim { color: var(--text-1); }
  .bar-unit { color: var(--text-3); margin-left: 4px; font-size: 9px; }
</style>
