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
  function fiveh(id: string): number | null {
    const s = p.usageCache[id]; if (!s?.segments?.length) return null;
    const seg = s.segments.find(x => x.label === "5h window"); return seg ? Math.round(seg.used) : null;
  }
  // neon glow uses stacked box-shadows (cheap, GPU accelerated)
  function glow(c: string): string {
    return `0 0 0 1px ${c}, 0 0 6px ${c}, 0 0 14px ${c}55, inset 0 0 4px ${c}33`;
  }
</script>

<div class="frame" data-tauri-drag-region>
  <header class="hd">
    <div class="brand">
      <span class="b1">AI</span><span class="dot"></span><span class="b2">QUOTA</span>
    </div>
    <span class="clk">{ts}</span>
    <div class="ctrls" data-tauri-drag-region="false">
      <button class="ct th" onclick={p.onCycleTheme}>{p.themeLabel}</button>
      <button class="ct" class:spin={p.refreshing} onclick={p.onRefresh} title="Refresh">↻</button>
      <button class="ct" class:on={p.autostart} onclick={p.onToggleAutostart} title="Auto">⏻</button>
      <button class="ct" class:on={p.pinned} onclick={p.onTogglePin} title="Pin">⌖</button>
      <button class="ct danger" onclick={p.onClose} title="Close">×</button>
    </div>
  </header>

  <section class="rows" data-tauri-drag-region="false">
    {#each p.providers as prov, i (prov.id)}
      {@const fp = fiveh(prov.id)}
      <div class="row" class:expanded={expanded === prov.id} style="--accent: {prov.accent}; --accent-glow: {glow(prov.accent)};">
        <button class="rh" onclick={() => toggle(prov.id)}>
          <!-- svelte-ignore a11y_no_static_element_interactions a11y_click_events_have_key_events -->
          <span class="orb" onclick={(e) => { e.stopPropagation(); p.onOpenProvider(prov.id); }}></span>
          <span class="nm">{prov.name}</span>
          {#if fp != null}
            <div class="ring" data-st={fp > 90 ? "danger" : fp > 75 ? "warn" : "ok"}>
              <svg width="36" height="36" viewBox="0 0 36 36">
                <circle cx="18" cy="18" r="15" stroke="rgba(255,255,255,0.08)" stroke-width="2" fill="none"/>
                <circle cx="18" cy="18" r="15" stroke="currentColor" stroke-width="2" fill="none"
                        stroke-dasharray={`${fp / 100 * 94.2} 94.2`} stroke-dashoffset="0" stroke-linecap="round" transform="rotate(-90 18 18)" />
              </svg>
              <span class="ring-num">{fp}</span>
            </div>
          {/if}
          <span class="arrow">{expanded === prov.id ? "−" : "+"}</span>
        </button>
        {#if expanded === prov.id}
          <div class="body">
            {#if !kind}
              <div class="picks">
                <button class="pk" disabled={!prov.available.api} onclick={() => pick(prov.id, "api")}>
                  <span class="pk-l">API</span><span class="pk-s">credits · rate</span>
                </button>
                <button class="pk" disabled={!prov.available.plan} onclick={() => pick(prov.id, "plan")}>
                  <span class="pk-l">PLAN</span><span class="pk-s">5h · weekly</span>
                </button>
              </div>
            {:else if loading}
              <div class="ld">⟶ loading</div>
            {:else if payload?.error}
              <div class="er">⚠ {payload.error}</div>
            {:else if payload?.segments?.length}
              <div class="bh">
                <button class="bk" onclick={back}>‹ back</button>
                {#if payload.plan}<span class="pl">{payload.plan}</span>{/if}
                <span class="lv">live</span>
              </div>
              {#each payload.segments as seg (seg.label)}
                {@const pct = Math.round((seg.used / seg.limit) * 100)}
                <div class="seg">
                  <div class="sg-h"><span>{seg.label}</span>{#if seg.resetAt}<span class="sg-r">{seg.resetAt}</span>{/if}</div>
                  <div class="sg-bar"><div class="sg-fl" style="width:{pct}%"></div></div>
                  <div class="sg-pc">{pct}<span>%</span></div>
                </div>
              {/each}
            {/if}
          </div>
        {/if}
      </div>
    {/each}
  </section>

  <footer class="ft">
    <span>SYNC</span>
    <span class="dotty"></span>
    <span>{p.lastRefresh ? p.lastRefresh.toLocaleTimeString("en-US",{hour12:false}) : "—"}</span>
    <span class="ver">0.1.0</span>
  </footer>
</div>

<style>
  .frame {
    position: absolute; inset: 0;
    background: radial-gradient(ellipse at 50% 0%, #1a0a2e 0%, #08051a 100%);
    color: #fff;
    font-family: "Inter", system-ui, sans-serif;
    overflow: hidden;
    display: flex;
    flex-direction: column;
    border-radius: 14px;
    border: 2px solid #ff2acc;
    box-shadow: 0 0 0 1px #ff2acc, 0 0 12px #ff2acc88, inset 0 0 24px rgba(255,42,204,0.08);
  }
  .hd {
    display: flex; align-items: center; gap: 10px;
    padding: 11px 13px;
    cursor: grab;
    border-bottom: 1px solid rgba(255,255,255,0.06);
    flex-shrink: 0;
  }
  .hd:active { cursor: grabbing; }
  .brand {
    display: flex; align-items: center; gap: 6px;
    pointer-events: none;
    font-size: 14px; font-weight: 800; letter-spacing: 0.06em;
  }
  .brand .b1 { color: #00ffe1; text-shadow: 0 0 4px #00ffe1, 0 0 12px #00ffe1aa; }
  .brand .b2 { color: #ff2acc; text-shadow: 0 0 4px #ff2acc, 0 0 12px #ff2acccc; }
  .brand .dot { width: 6px; height: 6px; border-radius: 50%; background: #fff; box-shadow: 0 0 6px #fff; }
  .clk {
    margin-left: auto;
    font-family: "JetBrains Mono", monospace;
    font-size: 11px; font-weight: 600;
    color: #00ffe1;
    text-shadow: 0 0 4px #00ffe1aa;
    pointer-events: none;
    letter-spacing: 0.06em;
  }
  .ctrls { display: flex; gap: 4px; }
  .ct {
    min-width: 24px; height: 22px; padding: 0 7px;
    background: rgba(255,255,255,0.04);
    border: 1px solid rgba(255,255,255,0.12);
    border-radius: 11px;
    color: #fff;
    cursor: pointer;
    font-size: 13px;
    font-family: inherit;
    display: grid; place-items: center;
  }
  .ct.th { font-size: 9px; font-weight: 700; letter-spacing: 0.14em; color: #00ffe1; border-color: #00ffe1; box-shadow: 0 0 6px #00ffe188; }
  .ct:hover { background: rgba(0,255,225,0.15); border-color: #00ffe1; box-shadow: 0 0 8px #00ffe1aa; color: #00ffe1; }
  .ct.on { background: #00ffe122; border-color: #00ffe1; color: #00ffe1; box-shadow: 0 0 8px #00ffe1aa; }
  .ct.danger:hover { background: #ff286622; border-color: #ff2866; color: #ff2866; box-shadow: 0 0 8px #ff2866aa; }
  .ct.spin { animation: spin 0.8s linear infinite; }
  @keyframes spin { to { transform: rotate(360deg); } }

  .rows { flex: 1; overflow-y: auto; padding: 8px; display: flex; flex-direction: column; gap: 8px; }
  .rows::-webkit-scrollbar { width: 4px; }
  .rows::-webkit-scrollbar-thumb { background: rgba(255,255,255,0.15); border-radius: 2px; }

  .row {
    border-radius: 12px;
    background: rgba(255,255,255,0.025);
    border: 1px solid rgba(255,255,255,0.08);
    transition: border-color 200ms;
  }
  .row.expanded {
    border-color: var(--accent);
    box-shadow: var(--accent-glow);
  }
  .rh {
    width: 100%;
    display: flex; align-items: center; gap: 10px;
    padding: 8px 12px;
    background: none;
    border: none;
    color: inherit;
    cursor: pointer;
    text-align: left;
    font-family: inherit;
  }
  .orb {
    width: 22px; height: 22px;
    border-radius: 50%;
    background: var(--accent);
    box-shadow: var(--accent-glow);
    cursor: pointer;
    flex-shrink: 0;
  }
  .orb:hover { transform: scale(1.1); transition: transform 120ms; }
  .nm { font-size: 13px; font-weight: 700; letter-spacing: 0.06em; color: #fff; flex: 1; }
  .row.expanded .nm { color: var(--accent); text-shadow: 0 0 6px var(--accent); }
  .ring { position: relative; width: 36px; height: 36px; }
  .ring[data-st="ok"]     { color: #36ffaa; }
  .ring[data-st="warn"]   { color: #ffd84d; }
  .ring[data-st="danger"] { color: #ff3860; }
  .ring svg { transform: rotate(0deg); filter: drop-shadow(0 0 3px currentColor); }
  .ring-num { position: absolute; inset: 0; display: grid; place-items: center; font-size: 11px; font-weight: 800; color: currentColor; font-family: "JetBrains Mono", monospace; }
  .arrow { width: 22px; height: 22px; display: grid; place-items: center; border: 1px solid rgba(255,255,255,0.2); border-radius: 50%; color: #fff; font-size: 14px; }
  .row.expanded .arrow { border-color: var(--accent); color: var(--accent); }

  .body { padding: 4px 12px 12px 44px; }
  .picks { display: grid; grid-template-columns: 1fr 1fr; gap: 8px; }
  .pk {
    background: rgba(0,0,0,0.3);
    border: 1px solid rgba(255,255,255,0.1);
    border-radius: 10px;
    padding: 9px 12px;
    color: #fff;
    cursor: pointer;
    font-family: inherit;
    text-align: left;
    display: flex; flex-direction: column; gap: 1px;
  }
  .pk:not(:disabled):hover { border-color: var(--accent); box-shadow: var(--accent-glow); }
  .pk:disabled { opacity: 0.3; cursor: not-allowed; }
  .pk-l { font-size: 12px; font-weight: 800; letter-spacing: 0.12em; color: var(--accent); }
  .pk-s { font-size: 9px; color: rgba(255,255,255,0.4); }

  .ld, .er { padding: 8px 0; font-size: 11px; }
  .ld { color: #00ffe1; }
  .er { color: #ff2866; }

  .bh { display: flex; align-items: center; gap: 6px; margin-bottom: 6px; }
  .bk { background: none; border: none; color: #fff; font-size: 11px; cursor: pointer; padding: 2px 0; font-family: inherit; }
  .bk:hover { color: var(--accent); }
  .pl { font-size: 9px; font-weight: 700; letter-spacing: 0.16em; padding: 2px 7px; border-radius: 8px; background: rgba(0,255,225,0.1); color: #00ffe1; border: 1px solid #00ffe1; }
  .lv { margin-left: auto; font-size: 9px; font-weight: 700; letter-spacing: 0.18em; color: #36ffaa; padding: 2px 7px; border-radius: 8px; background: rgba(54,255,170,0.1); border: 1px solid #36ffaa; box-shadow: 0 0 6px #36ffaaaa; }
  .seg { padding: 5px 0; }
  .sg-h { display: flex; justify-content: space-between; font-size: 10px; color: rgba(255,255,255,0.5); }
  .sg-r { color: rgba(255,255,255,0.3); }
  .sg-bar { height: 6px; background: rgba(255,255,255,0.05); border-radius: 3px; overflow: hidden; margin: 4px 0 2px; }
  .sg-fl { height: 100%; background: var(--accent); box-shadow: 0 0 6px var(--accent); border-radius: 3px; transition: width 700ms cubic-bezier(0.2,0.8,0.2,1); }
  .sg-pc { font-family: "JetBrains Mono", monospace; font-size: 14px; font-weight: 800; color: var(--accent); text-shadow: 0 0 6px var(--accent); }
  .sg-pc span { font-size: 10px; opacity: 0.7; }

  .ft {
    display: flex; align-items: center; gap: 8px;
    padding: 8px 14px;
    border-top: 1px solid rgba(255,255,255,0.06);
    font-family: "JetBrains Mono", monospace;
    font-size: 9px;
    letter-spacing: 0.18em;
    color: rgba(255,255,255,0.4);
    flex-shrink: 0;
  }
  .dotty {
    width: 5px; height: 5px; border-radius: 50%;
    background: #36ffaa;
    box-shadow: 0 0 5px #36ffaa, 0 0 10px #36ffaa66;
  }
  .ver { margin-left: auto; }
</style>
