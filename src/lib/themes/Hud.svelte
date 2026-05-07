<script lang="ts">
  import type { ThemeProps } from "./types";
  import type { QuotaPayload } from "../types";

  let p: ThemeProps = $props();

  let inflightLoad = $state<Set<string>>(new Set());

  // for HUD theme: always show all data inline (no expansion). Auto-load on mount.
  let allPayloads = $state<Record<string, QuotaPayload | null>>({});

  $effect(() => {
    // when usageCache updates for claude, mirror it
    if (p.usageCache.claude) allPayloads.claude = p.usageCache.claude;
  });

  async function loadProvider(id: string) {
    if (inflightLoad.has(id)) return;
    inflightLoad.add(id); inflightLoad = new Set(inflightLoad);
    const pld = await p.onLoadSegment(id, p.providers.find(x => x.id === id)?.available.plan ? "plan" : "api");
    allPayloads = { ...allPayloads, [id]: pld };
    inflightLoad.delete(id); inflightLoad = new Set(inflightLoad);
  }

  // auto-load all providers on mount
  $effect.root(() => {
    for (const prov of p.providers) {
      if (!allPayloads[prov.id]) loadProvider(prov.id);
    }
  });

  const ts = $derived(p.now.toLocaleTimeString("en-US", { hour12: false }));
  const dateStr = $derived(`${p.now.getFullYear()}.${String(p.now.getMonth()+1).padStart(2,"0")}.${String(p.now.getDate()).padStart(2,"0")}`);
</script>

<div class="frame" data-tauri-drag-region>
  <!-- corner targets -->
  <span class="tg tg-tl">
    <svg viewBox="0 0 16 16"><path d="M0 0v6M0 0h6M0 8h2M8 0v2" stroke="#ffaa33" stroke-width="2" fill="none"/></svg>
  </span>
  <span class="tg tg-tr">
    <svg viewBox="0 0 16 16"><path d="M16 0v6M16 0h-6M16 8h-2M8 0v2" stroke="#ffaa33" stroke-width="2" fill="none"/></svg>
  </span>
  <span class="tg tg-bl">
    <svg viewBox="0 0 16 16"><path d="M0 16v-6M0 16h6M0 8h2M8 16v-2" stroke="#ff5533" stroke-width="2" fill="none"/></svg>
  </span>
  <span class="tg tg-br">
    <svg viewBox="0 0 16 16"><path d="M16 16v-6M16 16h-6M16 8h-2M8 16v-2" stroke="#ff5533" stroke-width="2" fill="none"/></svg>
  </span>

  <header class="hd">
    <div class="brand-stack">
      <div class="b1">— TACTICAL DISPLAY —</div>
      <div class="b2">AI QUOTA SYS / REV.A</div>
    </div>
    <div class="hud-clock">
      <div class="hc-d">{dateStr}</div>
      <div class="hc-t">{ts}<span class="hc-z">UTC{Intl.DateTimeFormat().resolvedOptions().timeZone}</span></div>
    </div>
    <div class="ctrls" data-tauri-drag-region="false">
      <button class="ct" onclick={p.onCycleTheme} title="theme">[{p.themeLabel}]</button>
      <button class="ct" class:spin={p.refreshing} onclick={p.onRefresh} title="reload">⟲</button>
      <button class="ct" class:on={p.autostart} onclick={p.onToggleAutostart}>⏻</button>
      <button class="ct" class:on={p.pinned} onclick={p.onTogglePin}>⌖</button>
      <button class="ct danger" onclick={p.onClose}>✕</button>
    </div>
  </header>

  <!-- horizontal hairlines -->
  <div class="rule"></div>

  <div class="status-bar">
    <span class="sb-cell"><b>STATUS</b><span class="sb-v ok">▲ ONLINE</span></span>
    <span class="sb-cell"><b>NODES</b><span class="sb-v">{p.providers.length}</span></span>
    <span class="sb-cell"><b>SYNC</b><span class="sb-v">{p.lastRefresh ? p.lastRefresh.toLocaleTimeString("en-US",{hour12:false,second:undefined}) : "—"}</span></span>
    <span class="sb-cell"><b>TIER</b><span class="sb-v ok">MAX 5×</span></span>
  </div>

  <div class="rule"></div>

  <section class="targets" data-tauri-drag-region="false">
    {#each p.providers as prov, i (prov.id)}
      {@const pld = allPayloads[prov.id]}
      <div class="target" style="--accent: {prov.accent};">
        <div class="t-head">
          <span class="t-no">T-{String(i+1).padStart(3, "0")}</span>
          <span class="t-name">{prov.name}</span>
          <span class="t-tag">{prov.id.toUpperCase()}-CHANNEL</span>
          <!-- svelte-ignore a11y_no_static_element_interactions a11y_click_events_have_key_events -->
          <span class="t-link" onclick={() => p.onOpenProvider(prov.id)} title="open">↗</span>
        </div>
        {#if inflightLoad.has(prov.id)}
          <div class="t-empty">acquiring signal…</div>
        {:else if pld?.error}
          <div class="t-err">▲ {pld.error}</div>
        {:else if pld?.segments?.length}
          {#each pld.segments.slice(0, 2) as seg (seg.label)}
            {@const pct = Math.round((seg.used / seg.limit) * 100)}
            <div class="seg">
              <div class="seg-h">
                <span class="sh-l">{seg.label.toUpperCase()}</span>
                <span class="sh-pc" data-st={pct > 90 ? "danger" : pct > 75 ? "warn" : "ok"}>{pct}%</span>
              </div>
              <div class="seg-bar">
                {#each Array.from({length: 16}) as _, k}
                  <span class="seg-tick" class:on={k < Math.round((pct/100)*16)} class:warn={pct > 75 && k < Math.round((pct/100)*16)} class:danger={pct > 90 && k < Math.round((pct/100)*16)}></span>
                {/each}
              </div>
            </div>
          {/each}
        {:else}
          <div class="t-empty">no signal</div>
        {/if}
      </div>
    {/each}
  </section>

  <footer class="ft">
    <span class="ft-l">CLASSIFICATION: PUBLIC</span>
    <span class="ft-c">— ALL SYSTEMS NOMINAL —</span>
    <span class="ft-r">v0.1.0</span>
  </footer>
</div>

<style>
  .frame {
    position: absolute; inset: 0;
    background:
      radial-gradient(ellipse at 50% 100%, rgba(255,170,51,0.06) 0%, transparent 60%),
      #0a0805;
    color: #ffd9a0;
    font-family: "JetBrains Mono", ui-monospace, monospace;
    overflow: hidden;
    display: flex;
    flex-direction: column;
    border: 1px solid #4a3a20;
    font-size: 11px;
  }
  .tg { position: absolute; width: 16px; height: 16px; z-index: 5; pointer-events: none; }
  .tg-tl { top: 4px; left: 4px; }
  .tg-tr { top: 4px; right: 4px; }
  .tg-bl { bottom: 4px; left: 4px; }
  .tg-br { bottom: 4px; right: 4px; }

  .hd {
    display: flex; align-items: center; gap: 12px;
    padding: 10px 14px 8px 14px;
    cursor: grab;
    flex-shrink: 0;
  }
  .hd:active { cursor: grabbing; }
  .brand-stack { pointer-events: none; }
  .b1 { font-size: 8px; letter-spacing: 0.32em; color: #b88a44; }
  .b2 { font-size: 11px; font-weight: 700; letter-spacing: 0.24em; color: #ffaa33; text-shadow: 0 0 2px currentColor; }
  .hud-clock {
    margin-left: auto;
    text-align: right;
    pointer-events: none;
    font-feature-settings: "tnum";
  }
  .hc-d { font-size: 9px; color: #b88a44; letter-spacing: 0.18em; }
  .hc-t { font-size: 13px; font-weight: 700; color: #ffaa33; letter-spacing: 0.12em; }
  .hc-z { font-size: 8px; margin-left: 6px; color: #6a5028; }

  .ctrls { display: flex; gap: 2px; }
  .ct { background: none; border: 1px solid #4a3a20; color: #b88a44; padding: 3px 6px; font-family: inherit; font-size: 10px; cursor: pointer; }
  .ct:hover { color: #ffaa33; border-color: #ffaa33; background: rgba(255,170,51,0.08); }
  .ct.on { color: #ffaa33; border-color: #ffaa33; background: rgba(255,170,51,0.12); }
  .ct.danger:hover { color: #ff3322; border-color: #ff3322; background: rgba(255,51,34,0.1); }
  .ct.spin { animation: spin 0.8s linear infinite; }
  @keyframes spin { to { transform: rotate(360deg); } }

  .rule {
    height: 1px;
    margin: 0 14px;
    background: repeating-linear-gradient(90deg, #4a3a20 0, #4a3a20 4px, transparent 4px, transparent 8px);
    flex-shrink: 0;
  }

  .status-bar {
    display: flex; gap: 12px;
    padding: 6px 14px;
    flex-shrink: 0;
  }
  .sb-cell { display: flex; align-items: baseline; gap: 6px; font-size: 10px; }
  .sb-cell b { font-weight: 600; color: #6a5028; letter-spacing: 0.18em; }
  .sb-v { color: #ffd9a0; font-weight: 700; letter-spacing: 0.12em; }
  .sb-v.ok { color: #88dd44; }

  .targets { flex: 1; overflow-y: auto; padding: 8px 14px; display: flex; flex-direction: column; gap: 8px; }
  .targets::-webkit-scrollbar { width: 4px; }
  .targets::-webkit-scrollbar-thumb { background: #4a3a20; }

  .target {
    border: 1px solid #2a2218;
    background: #0a0805;
    padding: 7px 10px;
    position: relative;
  }
  .target::before, .target::after {
    content: "";
    position: absolute;
    width: 8px; height: 8px;
    border-color: var(--accent);
    border-style: solid;
  }
  .target::before { top: -1px; left: -1px; border-width: 1px 0 0 1px; }
  .target::after { bottom: -1px; right: -1px; border-width: 0 1px 1px 0; }
  .t-head { display: flex; align-items: baseline; gap: 8px; padding-bottom: 5px; border-bottom: 1px dashed #2a2218; margin-bottom: 6px; }
  .t-no { font-size: 9px; color: #6a5028; letter-spacing: 0.16em; }
  .t-name { font-size: 12px; font-weight: 700; color: var(--accent); letter-spacing: 0.16em; }
  .t-tag { margin-left: auto; font-size: 8px; letter-spacing: 0.18em; color: #b88a44; padding: 1px 5px; border: 1px solid #4a3a20; }
  .t-link { color: #ffaa33; cursor: pointer; padding: 0 4px; }
  .t-link:hover { color: #fff2dd; }
  .t-empty, .t-err { font-size: 10px; color: #6a5028; padding: 6px 0; letter-spacing: 0.12em; }
  .t-err { color: #ff3322; }

  .seg { padding: 3px 0; }
  .seg-h { display: flex; justify-content: space-between; font-size: 9px; letter-spacing: 0.16em; }
  .sh-l { color: #b88a44; }
  .sh-pc { font-weight: 700; }
  .sh-pc[data-st="ok"] { color: #88dd44; }
  .sh-pc[data-st="warn"] { color: #ffd84d; }
  .sh-pc[data-st="danger"] { color: #ff3322; }
  .seg-bar { display: flex; gap: 1px; padding: 3px 0; }
  .seg-tick { flex: 1; height: 8px; background: #1c1810; border-left: 1px solid #2a2218; }
  .seg-tick.on { background: #88dd44; }
  .seg-tick.on.warn { background: #ffd84d; }
  .seg-tick.on.danger { background: #ff3322; }

  .ft {
    display: flex; align-items: center;
    padding: 6px 14px;
    border-top: 1px dashed #2a2218;
    font-size: 9px;
    letter-spacing: 0.2em;
    color: #6a5028;
    flex-shrink: 0;
  }
  .ft-l { color: #b88a44; }
  .ft-c { flex: 1; text-align: center; color: #ffaa33; }
  .ft-r { color: #6a5028; font-feature-settings: "tnum"; }
</style>
