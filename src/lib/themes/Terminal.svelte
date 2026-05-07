<script lang="ts">
  import type { ThemeProps } from "./types";
  import type { QuotaPayload, QuotaKind } from "../types";

  let p: ThemeProps = $props();

  let expanded = $state<string | null>(null);
  let kind = $state<QuotaKind | null>(null);
  let payload = $state<QuotaPayload | null>(null);
  let loading = $state(false);

  async function pick(id: string, k: QuotaKind) {
    expanded = id; kind = k; loading = true; payload = null;
    payload = await p.onLoadSegment(id, k);
    loading = false;
  }
  function back() { kind = null; payload = null; }
  function toggle(id: string) {
    if (expanded === id) { expanded = null; kind = null; payload = null; }
    else { expanded = id; kind = null; payload = null; }
  }
  const ts = $derived(p.now.toLocaleTimeString("en-US", { hour12: false }));
  const dateStr = $derived(`${p.now.getFullYear()}-${String(p.now.getMonth()+1).padStart(2,"0")}-${String(p.now.getDate()).padStart(2,"0")}`);
  function fiveh(id: string): number | null {
    const s = p.usageCache[id]; if (!s?.segments?.length) return null;
    const seg = s.segments.find(x => x.label === "5h window"); return seg ? Math.round(seg.used) : null;
  }
  function bar(used: number, limit: number, width = 20): string {
    const pct = Math.min(1, used / Math.max(limit, 1));
    const fill = Math.round(pct * width);
    return "█".repeat(fill) + "░".repeat(width - fill);
  }
</script>

<div class="frame" data-tauri-drag-region>
  <div class="scan"></div>

  <header class="hd">
    <span class="prompt">$&nbsp;</span><span class="cmd">claude-quota --watch</span>
    <span class="caret">▌</span>
    <span class="hd-r">{dateStr} {ts}</span>
    <div class="ctrls" data-tauri-drag-region="false">
      <button class="ctrl" onclick={p.onCycleTheme} title={p.themeLabel}>[{p.themeLabel}]</button>
      <button class="ctrl" class:spin={p.refreshing} onclick={p.onRefresh} title="reload">[R]</button>
      <button class="ctrl" class:on={p.autostart} onclick={p.onToggleAutostart} title="autostart">[A]</button>
      <button class="ctrl" class:on={p.pinned} onclick={p.onTogglePin} title="pin">[P]</button>
      <button class="ctrl" onclick={p.onClose} title="quit">[X]</button>
    </div>
  </header>

  <div class="box">
    <div class="bx-tl">┌</div><div class="bx-tr">┐</div>
    <div class="bx-bl">└</div><div class="bx-br">┘</div>
    <div class="bx-line bx-top"></div>
    <div class="bx-line bx-bot"></div>
    <div class="bx-line bx-l"></div>
    <div class="bx-line bx-r"></div>

    <div class="banner">[ AI · QUOTA · MONITOR ]</div>

    <pre class="meta">  uptime: <em>OK</em>   nodes: <em>{p.providers.length}</em>   tier: <em>MAX5x</em>   pid: <em>0x{Math.floor(Math.random()*9999).toString(16).padStart(4,'0')}</em></pre>

    <section class="rows" data-tauri-drag-region="false">
      {#each p.providers as prov, i (prov.id)}
        {@const fp = fiveh(prov.id)}
        <div class="row" class:expanded={expanded === prov.id}>
          <button class="rh" onclick={() => toggle(prov.id)}>
            <span class="ix">{String(i+1).padStart(2,"0")}.</span>
            <span class="nm">{prov.name.toLowerCase()}</span>
            {#if fp != null}
              <span class="bar">[{bar(fp, 100, 14)}]</span>
              <span class="pct">{String(fp).padStart(3, " ")}%</span>
            {:else}
              <span class="bar dim">[{"░".repeat(14)}]</span>
              <span class="pct dim">  ?%</span>
            {/if}
            <!-- svelte-ignore a11y_no_static_element_interactions a11y_click_events_have_key_events -->
            <span class="ext" onclick={(e) => { e.stopPropagation(); p.onOpenProvider(prov.id); }} title="open">↗</span>
            <span class="ar">{expanded === prov.id ? "▼" : "▸"}</span>
          </button>
          {#if expanded === prov.id}
            <div class="body">
              {#if !kind}
                <pre class="prompt-line">  &gt; select metric:</pre>
                <div class="choose">
                  <button class="cb" disabled={!prov.available.api} onclick={() => pick(prov.id, "api")}>[1] api &nbsp;&nbsp;&nbsp; credits / rate</button>
                  <button class="cb" disabled={!prov.available.plan} onclick={() => pick(prov.id, "plan")}>[2] plan &nbsp;&nbsp; 5h / weekly</button>
                </div>
              {:else if loading}
                <pre class="ld">  $ fetching... <span class="caret">▌</span></pre>
              {:else if payload?.error}
                <pre class="er">  ! ERR: {payload.error}</pre>
              {:else if payload?.segments?.length}
                <pre class="prompt-line">  &gt; {kind} {payload.plan ? `[plan=${payload.plan}]` : ""} <span class="lv">[LIVE]</span></pre>
                {#each payload.segments as seg (seg.label)}
                  {@const pct = Math.round((seg.used/seg.limit)*100)}
                  <pre class="b">  {seg.label.padEnd(18, " ")} [{bar(seg.used, seg.limit, 22)}] {String(pct).padStart(3, " ")}%{seg.resetAt ? `  ↻ ${seg.resetAt}` : ""}</pre>
                {/each}
                <pre class="back-line">  &gt; <button class="back" onclick={back}>back</button></pre>
              {/if}
            </div>
          {/if}
        </div>
      {/each}
    </section>

    <div class="ft">
      <pre>  sync: {p.lastRefresh ? p.lastRefresh.toLocaleTimeString("en-US",{hour12:false}) : "—"}    [build v0.1.0]</pre>
    </div>
  </div>
</div>

<style>
  .frame {
    position: absolute; inset: 0;
    background: #02060a;
    color: #6cd690;
    font-family: "JetBrains Mono", ui-monospace, Menlo, Consolas, monospace;
    font-size: 11px;
    overflow: hidden;
    border: 1px solid #0d3018;
    display: flex;
    flex-direction: column;
  }
  .scan {
    position: absolute; inset: 0;
    background: repeating-linear-gradient(180deg, rgba(0,255,102,0.04) 0, rgba(0,255,102,0.04) 1px, transparent 1px, transparent 3px);
    pointer-events: none;
    z-index: 1;
  }
  .hd {
    display: flex; align-items: center; gap: 6px;
    padding: 6px 9px;
    background: #04140a;
    border-bottom: 1px solid #0d3018;
    cursor: grab;
    z-index: 2;
    position: relative;
  }
  .hd:active { cursor: grabbing; }
  .prompt { color: #00ff66; font-weight: 700; }
  .cmd { color: #b6ffd1; pointer-events: none; }
  .caret { color: #00ff66; animation: blink 1s steps(2) infinite; }
  @keyframes blink { 50% { opacity: 0; } }
  .hd-r { margin-left: auto; color: #33aa55; pointer-events: none; font-size: 10px; }
  .ctrls { display: flex; gap: 2px; }
  .ctrl { color: #33aa55; background: none; border: none; padding: 0 2px; cursor: pointer; font-family: inherit; font-size: 10px; }
  .ctrl:hover { color: #00ff66; background: #0d3018; }
  .ctrl.on { color: #00ff66; }
  .ctrl.spin { color: #aaff00; }

  .box { flex: 1; position: relative; padding: 14px 12px 8px 12px; z-index: 2; display: flex; flex-direction: column; overflow: hidden; }
  .bx-tl, .bx-tr, .bx-bl, .bx-br { position: absolute; color: #1a5028; font-size: 12px; line-height: 1; pointer-events: none; }
  .bx-tl { top: 4px; left: 6px; }
  .bx-tr { top: 4px; right: 6px; }
  .bx-bl { bottom: 4px; left: 6px; }
  .bx-br { bottom: 4px; right: 6px; }
  .bx-line { position: absolute; pointer-events: none; }
  .bx-top { top: 8px; left: 14px; right: 14px; height: 1px; background: #1a5028; }
  .bx-bot { bottom: 8px; left: 14px; right: 14px; height: 1px; background: #1a5028; }
  .bx-l { left: 8px; top: 14px; bottom: 14px; width: 1px; background: #1a5028; }
  .bx-r { right: 8px; top: 14px; bottom: 14px; width: 1px; background: #1a5028; }

  .banner {
    text-align: center;
    color: #00ff66;
    font-weight: 700;
    letter-spacing: 0.2em;
    padding: 4px 0 6px;
    text-shadow: 0 0 1px currentColor;
  }
  pre.meta {
    color: #6cd690;
    margin: 0;
    font-size: 10px;
    padding-bottom: 6px;
    border-bottom: 1px dashed #1a5028;
    margin-bottom: 4px;
  }
  pre.meta em { color: #00ff66; font-style: normal; }

  .rows { flex: 1; overflow-y: auto; }
  .rows::-webkit-scrollbar { width: 4px; }
  .rows::-webkit-scrollbar-thumb { background: #1a5028; }

  .row { padding: 0; }
  .row.expanded { background: rgba(0,255,102,0.025); }
  .rh {
    width: 100%;
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 4px 4px;
    background: none;
    border: none;
    color: inherit;
    font-family: inherit;
    font-size: 11px;
    cursor: pointer;
    text-align: left;
  }
  .rh:hover { background: rgba(0,255,102,0.05); }
  .ix { color: #33aa55; }
  .nm { color: #b6ffd1; min-width: 70px; font-weight: 700; }
  .row.expanded .nm { color: #00ff66; }
  .bar { color: #00ff66; letter-spacing: -1px; }
  .bar.dim { color: #1a5028; }
  .pct { color: #b6ffd1; font-weight: 700; }
  .pct.dim { color: #33aa55; }
  .ext { margin-left: auto; color: #33aa55; cursor: pointer; padding: 0 4px; }
  .ext:hover { color: #00ff66; }
  .ar { color: #33aa55; font-size: 9px; }

  .body { padding: 0 4px 6px 24px; }
  pre.prompt-line { color: #6cd690; margin: 4px 0; }
  pre.prompt-line .lv { color: #00ff66; margin-left: 6px; }
  .choose { display: flex; flex-direction: column; gap: 1px; padding-left: 2px; }
  .cb { background: none; border: none; color: #b6ffd1; font-family: inherit; font-size: 11px; cursor: pointer; text-align: left; padding: 2px 8px; }
  .cb:not(:disabled):hover { background: rgba(0,255,102,0.08); color: #00ff66; }
  .cb:disabled { opacity: 0.3; cursor: not-allowed; }
  pre.ld { color: #aaff00; margin: 4px 0; }
  pre.er { color: #ff4444; margin: 4px 0; }
  pre.b { color: #b6ffd1; margin: 1px 0; font-size: 10px; letter-spacing: -0.5px; }
  pre.back-line { margin: 4px 0; color: #6cd690; }
  pre.back-line .back { background: none; border: 1px solid #1a5028; padding: 1px 8px; color: #00ff66; cursor: pointer; font-family: inherit; font-size: 10px; }
  pre.back-line .back:hover { background: rgba(0,255,102,0.1); }

  .ft { padding: 2px 0 0; border-top: 1px dashed #1a5028; margin-top: 2px; }
  .ft pre { color: #33aa55; font-size: 10px; margin: 0; }
</style>
