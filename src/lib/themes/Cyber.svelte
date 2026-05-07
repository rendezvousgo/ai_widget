<script lang="ts">
  import type { ThemeProps } from "./types";
  import type { QuotaPayload, QuotaKind } from "../types";
  import Logo from "../Logo.svelte";

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
  const dateStr = $derived(`${p.now.getFullYear()}.${String(p.now.getMonth()+1).padStart(2,"0")}.${String(p.now.getDate()).padStart(2,"0")}`);
  function snap(id: string) { return p.usageCache[id]; }
  function fiveh(id: string): number | null {
    const s = snap(id); if (!s?.segments?.length) return null;
    const seg = s.segments.find(x => x.label === "5h window"); return seg ? Math.round(seg.used) : null;
  }
  function status(pct: number | null): string {
    if (pct == null) return "";
    return pct > 90 ? "danger" : pct > 75 ? "warn" : "ok";
  }
</script>

<div class="frame" data-tauri-drag-region>
  <span class="brk brk-tl"></span><span class="brk brk-tr"></span>
  <span class="brk brk-bl"></span><span class="brk brk-br"></span>

  <header class="hd">
    <div class="brand">
      <span class="bl">[</span><span class="b1">AI</span><span class="b2">//</span><span class="b3">QUOTA</span><span class="bl">]</span>
    </div>
    <span class="clk">
      <span class="d">{dateStr}</span><span class="s">·</span><span class="t">{ts}</span>
    </span>
    <div class="ctrls" data-tauri-drag-region="false">
      <button class="ctrl ctrl-th" onclick={p.onCycleTheme} title={p.themeLabel}>{p.themeLabel}</button>
      <button class="ctrl" class:spin={p.refreshing} onclick={p.onRefresh} title="Refresh">
        <svg width="11" height="11" viewBox="0 0 16 16" fill="none"><path d="M14 8a6 6 0 11-2-4.5M14 2v4h-4" stroke="currentColor" stroke-width="1.4"/></svg>
      </button>
      <button class="ctrl" class:on={p.autostart} onclick={p.onToggleAutostart} title="Auto-start">
        <svg width="11" height="11" viewBox="0 0 16 16" fill="none"><path d="M8 1v7M4.5 3.5a6 6 0 107 0" stroke="currentColor" stroke-width="1.4"/></svg>
      </button>
      <button class="ctrl" class:on={p.pinned} onclick={p.onTogglePin} title="Pin">
        <svg width="11" height="11" viewBox="0 0 16 16" fill="none"><path d="M9 2l5 5-2 2-1-1-3 3 1 3-2 2-7-7 2-2 3 1 3-3-1-1z" stroke="currentColor" stroke-width="1.2"/></svg>
      </button>
      <button class="ctrl danger" onclick={p.onClose} title="Close">
        <svg width="11" height="11" viewBox="0 0 16 16" fill="none"><path d="M3 3l10 10M13 3L3 13" stroke="currentColor" stroke-width="1.5"/></svg>
      </button>
    </div>
  </header>
  <div class="hairline"></div>

  <div class="meta">
    <span><b>NODES</b> {p.providers.length.toString().padStart(2,"0")}</span><span class="sep">│</span>
    <span><b>STATE</b> <span class="ok">●ONLINE</span></span><span class="sep">│</span>
    <span><b>TH</b> {p.themeIdx + 1}/{p.themesCount}</span>
    <span class="trace">// trace::af72-90b3</span>
  </div>

  <section class="rows" data-tauri-drag-region="false">
    {#each p.providers as prov, i (prov.id)}
      {@const fp = fiveh(prov.id)}
      {@const st = status(fp)}
      <div class="row" class:expanded={expanded === prov.id} style="--accent: {prov.accent};">
        <button class="rh" onclick={() => toggle(prov.id)}>
          <span class="idx">N{String(i+1).padStart(2,"0")}</span>
          <!-- svelte-ignore a11y_no_static_element_interactions a11y_click_events_have_key_events -->
          <span class="lg" onclick={(e) => { e.stopPropagation(); p.onOpenProvider(prov.id); }}><Logo id={prov.id} size={13} /></span>
          <span class="nm">{prov.name}</span>
          {#if fp != null}<span class="pct" data-st={st}>{fp}%</span>{/if}
          <span class="tags">
            {#if prov.available.api}<span class="tag">API</span>{/if}
            {#if prov.available.plan}<span class="tag tp">PLAN</span>{/if}
          </span>
          <span class="chev" class:o={expanded === prov.id}>▶</span>
        </button>
        {#if expanded === prov.id}
          <div class="body">
            {#if !kind}
              <div class="choose">
                <button class="cb" disabled={!prov.available.api} onclick={() => pick(prov.id, "api")}>›API</button>
                <button class="cb" disabled={!prov.available.plan} onclick={() => pick(prov.id, "plan")}>›PLAN</button>
              </div>
            {:else if loading}
              <div class="ld">DECRYPTING...</div>
            {:else if payload?.error}
              <div class="er">▲ {payload.error}</div>
            {:else if payload?.segments?.length}
              <div class="bh">
                <button onclick={back}>◂ {kind.toUpperCase()}</button>
                {#if payload.plan}<span class="pl">{payload.plan}</span>{/if}<span class="lv">◆LIVE</span>
              </div>
              {#each payload.segments as seg (seg.label)}
                <div class="b">
                  <div class="b-meta"><span>▸ {seg.label}</span>{#if seg.resetAt}<span class="rs">↻{seg.resetAt}</span>{/if}</div>
                  <div class="b-tk"><div class="b-fl" style="width:{Math.min(100,(seg.used/seg.limit)*100)}%"></div></div>
                  <div class="b-st"><span class="b-pc" data-st={Math.round(seg.used/seg.limit*100) > 90 ? "danger" : Math.round(seg.used/seg.limit*100) > 75 ? "warn" : "ok"}>{Math.round((seg.used/seg.limit)*100)}%</span></div>
                </div>
              {/each}
            {/if}
          </div>
        {/if}
      </div>
    {/each}
  </section>

  <footer class="ft">
    <span class="dot"></span>
    <span class="msg">SYNC · {p.lastRefresh ? p.lastRefresh.toLocaleTimeString("en-US",{hour12:false}) : "—"}</span>
    <span class="ver">v0.1.0</span>
  </footer>
</div>

<style>
  .frame {
    position: absolute; inset: 0;
    background: linear-gradient(180deg,#0a0f1c 0%,#050810 100%);
    display: flex; flex-direction: column; overflow: hidden;
    border: 1px solid #1a2238; border-top: 2px solid #00ffe1;
    font-family: var(--mono); color: #f4f7ff;
  }
  .brk { position: absolute; width: 12px; height: 12px; z-index: 5; pointer-events: none; }
  .brk-tl { top: -1px; left: -1px; border-top: 2px solid #00ffe1; border-left: 2px solid #00ffe1; }
  .brk-tr { top: -1px; right: -1px; border-top: 2px solid #00ffe1; border-right: 2px solid #00ffe1; }
  .brk-bl { bottom: -1px; left: -1px; border-bottom: 1px solid #ff2acc; border-left: 1px solid #ff2acc; }
  .brk-br { bottom: -1px; right: -1px; border-bottom: 1px solid #ff2acc; border-right: 1px solid #ff2acc; }

  .hd { display: flex; align-items: center; gap: 8px; padding: 9px 10px 8px 12px; background: linear-gradient(180deg,#11182a 0%,#0a1020 100%); border-bottom: 1px solid #1a2238; cursor: grab; }
  .hd:active { cursor: grabbing; }
  .hairline { height: 1px; background: linear-gradient(90deg,transparent,#00ffe1 30%,#ff2acc 70%,transparent); opacity: 0.55; }
  .brand { display: flex; align-items: baseline; gap: 2px; pointer-events: none; font-size: 11px; font-weight: 700; letter-spacing: 0.18em; }
  .bl { color: #7e8cb0; }
  .b1 { color: #00ffe1; }
  .b2 { color: #ff2acc; padding: 0 1px; }
  .b3 { color: #f4f7ff; }
  .clk { margin-left: auto; padding: 2px 7px; border: 1px solid #2a365e; background: #050810; font-size: 10px; pointer-events: none; }
  .clk .d { color: #7e8cb0; font-size: 9px; margin-right: 3px; }
  .clk .s { color: #5a6688; margin: 0 3px; }
  .clk .t { color: #00ffe1; font-weight: 700; }
  .ctrls { display: flex; gap: 3px; }
  .ctrl { width: 22px; height: 20px; display: grid; place-items: center; color: #b6c3e2; border: 1px solid #2a365e; background: #0a0e1a; clip-path: polygon(3px 0,100% 0,100% calc(100% - 3px),calc(100% - 3px) 100%,0 100%,0 3px); cursor: pointer; }
  .ctrl-th { width: auto; padding: 0 7px; font-size: 8px; font-weight: 700; letter-spacing: 0.16em; color: #00ffe1; }
  .ctrl:hover { color: #00ffe1; border-color: #00ffe1; background: #0a1822; }
  .ctrl.on { color: #00ffe1; border-color: #00ffe1; background: #0c2230; }
  .ctrl.danger:hover { color: #ff3860; border-color: #ff3860; background: #1f0810; }
  .spin svg { animation: spin 0.8s linear infinite; }
  @keyframes spin { to { transform: rotate(360deg); } }

  .meta { display: flex; align-items: center; gap: 8px; padding: 4px 12px; background: #111728; border-bottom: 1px solid #1a2238; font-size: 9px; letter-spacing: 0.1em; color: #b6c3e2; flex-shrink: 0; }
  .meta b { color: #7e8cb0; font-weight: 600; margin-right: 3px; }
  .meta .sep { color: #2a365e; }
  .meta .ok { color: #36ffaa; }
  .meta .trace { margin-left: auto; color: #7e8cb0; font-size: 9px; }

  .rows { flex: 1; overflow-y: auto; background: #0a0e1a; }
  .rows::-webkit-scrollbar { width: 4px; }
  .rows::-webkit-scrollbar-thumb { background: #2a365e; }

  .row { border-bottom: 1px solid #1a2238; }
  .row:last-child { border-bottom: none; }
  .row.expanded { background: #111728; }
  .rh { width: 100%; display: flex; align-items: center; gap: 8px; padding: 7px 10px 7px 9px; cursor: pointer; border-left: 3px solid transparent; color: inherit; font-family: inherit; background: none; border-top: none; border-right: none; border-bottom: none; }
  .rh:hover { background: #0e1626; }
  .row.expanded .rh { border-left-color: var(--accent); background: #131a30; }
  .idx { font-size: 9px; color: #5a6688; min-width: 22px; }
  .lg { display: grid; place-items: center; width: 22px; height: 22px; border: 1px solid #2a365e; background: #050810; clip-path: polygon(3px 0,100% 0,100% calc(100% - 3px),calc(100% - 3px) 100%,0 100%,0 3px); color: var(--accent); cursor: pointer; }
  .lg:hover { border-color: var(--accent); background: color-mix(in oklab,var(--accent) 22%,#050810); }
  .nm { font-size: 11px; font-weight: 700; letter-spacing: 0.14em; color: #f4f7ff; }
  .row.expanded .nm { color: var(--accent); }
  .pct { font-size: 10px; font-weight: 700; padding: 1px 5px; border: 1px solid; }
  .pct[data-st="ok"]     { color: #36ffaa; border-color: rgba(54,255,170,0.5); background: rgba(54,255,170,0.1); }
  .pct[data-st="warn"]   { color: #ffd84d; border-color: rgba(255,216,77,0.5);  background: rgba(255,216,77,0.1); }
  .pct[data-st="danger"] { color: #ff3860; border-color: rgba(255,56,96,0.5);   background: rgba(255,56,96,0.1); }
  .tags { margin-left: auto; display: flex; gap: 3px; }
  .tag { font-size: 8px; font-weight: 700; letter-spacing: 0.14em; color: #7e8cb0; padding: 2px 5px; border: 1px solid #2a365e; background: #050810; }
  .tag.tp { color: #ff2acc; border-color: rgba(255,42,204,0.5); }
  .chev { font-size: 8px; color: #5a6688; width: 10px; text-align: center; }
  .chev.o { color: var(--accent); }

  .body { padding: 4px 12px 10px 32px; }
  .choose { display: grid; grid-template-columns: 1fr 1fr; gap: 6px; }
  .cb { padding: 7px 10px; border: 1px solid #2a365e; background: #050810; color: #f4f7ff; font-size: 11px; font-weight: 700; letter-spacing: 0.14em; cursor: pointer; clip-path: polygon(4px 0,100% 0,100% calc(100% - 4px),calc(100% - 4px) 100%,0 100%,0 4px); text-align: left; font-family: inherit; }
  .cb:not(:disabled):hover { border-color: var(--accent); background: color-mix(in oklab, var(--accent) 10%, #050810); }
  .cb:disabled { opacity: 0.3; cursor: not-allowed; }
  .ld { text-align: center; padding: 10px 0; font-size: 10px; color: #7e8cb0; letter-spacing: 0.22em; }
  .er { font-size: 10px; color: #ff3860; padding: 6px 0; }
  .bh { display: flex; justify-content: space-between; align-items: center; margin-bottom: 5px; gap: 5px; }
  .bh button { font-size: 10px; color: #b6c3e2; background: none; border: none; cursor: pointer; font-family: inherit; padding: 0; font-weight: 700; letter-spacing: 0.12em; }
  .bh button:hover { color: var(--accent); }
  .pl { font-size: 9px; letter-spacing: 0.16em; color: #00ffe1; padding: 1px 5px; border: 1px solid rgba(0,255,225,0.5); background: rgba(0,255,225,0.08); }
  .lv { font-size: 9px; letter-spacing: 0.16em; color: #36ffaa; padding: 1px 5px; border: 1px solid rgba(54,255,170,0.5); background: rgba(54,255,170,0.08); margin-left: auto; }
  .b { padding: 5px 0; }
  .b-meta { display: flex; justify-content: space-between; font-size: 9px; color: #b6c3e2; letter-spacing: 0.12em; margin-bottom: 3px; }
  .b-meta .rs { color: #5a6688; }
  .b-tk { height: 7px; background: #050810; border: 1px solid #2a365e; position: relative; overflow: hidden; }
  .b-fl { position: absolute; inset: 0 auto 0 0; background: linear-gradient(90deg, color-mix(in oklab, var(--accent) 70%, #050810), var(--accent)); transition: width 700ms cubic-bezier(0.2,0.8,0.2,1); border-right: 1px solid var(--accent); }
  .b-st { display: flex; justify-content: space-between; margin-top: 2px; }
  .b-pc { font-size: 12px; font-weight: 700; }
  .b-pc[data-st="ok"] { color: #36ffaa; }
  .b-pc[data-st="warn"] { color: #ffd84d; }
  .b-pc[data-st="danger"] { color: #ff3860; }

  .ft { display: flex; align-items: center; gap: 6px; padding: 5px 12px; border-top: 1px solid #1a2238; font-size: 9px; letter-spacing: 0.18em; color: #7e8cb0; background: #050810; }
  .ft .dot { width: 6px; height: 6px; background: #36ffaa; clip-path: polygon(50% 0,100% 50%,50% 100%,0 50%); }
  .ft .msg { flex: 1; color: #b6c3e2; }
  .ft .ver { color: #5a6688; }
</style>
