<script lang="ts">
  type Point = { date: string; value: number };

  let {
    data,
    accent = "#6ee7ff",
    height = 80,
  }: { data: Point[]; accent?: string; height?: number } = $props();

  let hoverIdx = $state<number | null>(null);
  let svgEl: SVGSVGElement;

  const max = $derived(Math.max(1, ...data.map((d) => d.value)));
  const W = 800; // svg viewBox width (any value, scales)
  const H = $derived(height);

  function xAt(i: number): number {
    if (data.length <= 1) return 0;
    return (i / (data.length - 1)) * W;
  }
  function yAt(v: number): number {
    return H - (v / max) * (H - 8) - 4;
  }

  const linePath = $derived.by(() => {
    if (!data.length) return "";
    return data
      .map((d, i) => `${i === 0 ? "M" : "L"}${xAt(i).toFixed(1)},${yAt(d.value).toFixed(1)}`)
      .join(" ");
  });

  const areaPath = $derived.by(() => {
    if (!data.length) return "";
    let p = `M0,${H} `;
    for (let i = 0; i < data.length; i++) {
      p += `L${xAt(i).toFixed(1)},${yAt(data[i].value).toFixed(1)} `;
    }
    p += `L${xAt(data.length - 1)},${H} Z`;
    return p;
  });

  const fmt = (n: number): string => {
    if (n >= 1_000_000_000) return (n / 1_000_000_000).toFixed(2) + "B";
    if (n >= 1_000_000) return (n / 1_000_000).toFixed(1) + "M";
    if (n >= 1_000) return (n / 1_000).toFixed(1) + "k";
    return n.toString();
  };
  const MONTHS = ["Jan","Feb","Mar","Apr","May","Jun","Jul","Aug","Sep","Oct","Nov","Dec"];
  function fmtDate(s: string): string {
    const m = s.match(/^(\d{4})-(\d{2})-(\d{2})/);
    if (!m) return s;
    return `${MONTHS[+m[2] - 1]} ${+m[3]}`;
  }
  function fmtDateLong(s: string): string {
    const m = s.match(/^(\d{4})-(\d{2})-(\d{2})/);
    if (!m) return s;
    return `${m[1]}.${m[2]}.${m[3]}`;
  }

  function onMove(e: MouseEvent) {
    if (!svgEl || data.length === 0) return;
    const rect = svgEl.getBoundingClientRect();
    const ratio = Math.max(0, Math.min(1, (e.clientX - rect.left) / rect.width));
    hoverIdx = Math.round(ratio * (data.length - 1));
  }
  function onLeave() {
    hoverIdx = null;
  }

  const xLabelIdx = $derived(
    data.length === 0
      ? []
      : [0, Math.floor((data.length - 1) / 3), Math.floor(2 * (data.length - 1) / 3), data.length - 1]
  );

  const tooltipPctLeft = $derived(
    hoverIdx == null || data.length <= 1 ? 0 : (hoverIdx / (data.length - 1)) * 100
  );
</script>

<div class="spark" style="--accent: {accent}; --h: {H}px;">
  <div class="svg-wrap">
    <!-- svelte-ignore a11y_no_static_element_interactions a11y_mouse_events_have_key_events -->
    <svg
      bind:this={svgEl}
      viewBox="0 0 {W} {H}"
      preserveAspectRatio="none"
      class="spark-svg"
      onmousemove={onMove}
      onmouseleave={onLeave}
    >
      <!-- horizontal grid -->
      {#each [0.25, 0.5, 0.75] as t}
        <line x1="0" x2={W} y1={H * t} y2={H * t} stroke="rgba(0,0,0,0.04)" stroke-width="1" />
      {/each}
      <path d={areaPath} fill="var(--accent)" fill-opacity="0.18" />
      <path d={linePath} fill="none" stroke="var(--accent)" stroke-width="1.4" stroke-linejoin="round" />
      {#if data.length}
        <circle cx={xAt(data.length - 1)} cy={yAt(data[data.length - 1].value)} r="2.5" fill="var(--accent)" />
      {/if}
      {#if hoverIdx !== null}
        <line
          x1={xAt(hoverIdx)}
          x2={xAt(hoverIdx)}
          y1="0"
          y2={H}
          stroke="rgba(0,0,0,0.18)"
          stroke-width="1"
          stroke-dasharray="3 3"
        />
        <circle
          cx={xAt(hoverIdx)}
          cy={yAt(data[hoverIdx].value)}
          r="3.5"
          fill="var(--accent)"
          stroke="#fff"
          stroke-width="1.5"
        />
      {/if}
    </svg>
    {#if hoverIdx !== null}
      <div
        class="tooltip"
        class:tooltip-r={tooltipPctLeft > 65}
        style="left: {tooltipPctLeft}%;"
      >
        <span class="t-date">{fmtDateLong(data[hoverIdx].date)}</span>
        <span class="t-val">{fmt(data[hoverIdx].value)} <span class="t-unit">tok</span></span>
      </div>
    {/if}
  </div>
  <div class="x-axis">
    {#each xLabelIdx as i, k}
      <span class="x-tick" class:r={k === xLabelIdx.length - 1} style="left: {(i / Math.max(1, data.length - 1)) * 100}%;">
        {fmtDate(data[i]?.date ?? "")}
      </span>
    {/each}
  </div>
</div>

<style>
  .spark {
    margin-top: 4px;
  }
  .svg-wrap {
    position: relative;
    width: 100%;
  }
  .spark-svg {
    width: 100%;
    height: var(--h);
    display: block;
    cursor: crosshair;
  }
  .tooltip {
    position: absolute;
    top: 4px;
    transform: translateX(8px);
    background: #1a1a1a;
    color: #f7f5ef;
    font-family: "JetBrains Mono", monospace;
    font-size: 10px;
    padding: 4px 8px;
    display: flex;
    flex-direction: column;
    gap: 1px;
    pointer-events: none;
    white-space: nowrap;
    line-height: 1.3;
    z-index: 5;
  }
  .tooltip.tooltip-r { transform: translateX(calc(-100% - 8px)); }
  .t-date { color: #b8a994; font-size: 9px; letter-spacing: 0.04em; }
  .t-val { font-weight: 600; }
  .t-unit { color: #888; font-weight: 400; font-size: 9px; }

  .x-axis {
    position: relative;
    margin-top: 6px;
    height: 11px;
    font-family: "JetBrains Mono", monospace;
    font-size: 9px;
    color: #999;
    letter-spacing: 0.04em;
  }
  .x-tick {
    position: absolute;
    transform: translateX(-50%);
    white-space: nowrap;
  }
  .x-tick.r { transform: translateX(-100%); }
  .x-tick:first-child { transform: translateX(0); }
</style>
