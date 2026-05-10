<script lang="ts">
  import type { ThemeProps } from "./types";
  import type { QuotaPayload } from "../types";
  import Settings from "./Settings.svelte";
  import Logo from "../Logo.svelte";
  import Sparkline from "../Sparkline.svelte";
  import { fetchClaudeDailyTokens, fetchUsageHistory, type DailyEntry, type UsageHistoryPoint } from "../realData";
  import { t as tr, providerName } from "../i18n";

  let p: ThemeProps = $props();
  let showSettings = $state(false);
  const T = $derived(tr(p.lang));

  let active = $state<string>("claude");
  let activePayload = $state<QuotaPayload | null>(null);
  let loading = $state(false);
  let sidebarOpen = $state(true);
  function toggleSidebar() {
    sidebarOpen = !sidebarOpen;
    try { localStorage.setItem("sidebarOpen", sidebarOpen ? "1" : "0"); } catch {}
  }
  $effect(() => {
    try {
      const saved = localStorage.getItem("sidebarOpen");
      if (saved === "0") sidebarOpen = false;
    } catch {}
  });

  let view = $state<"now" | "trend">("now");
  let daily = $state<DailyEntry[]>([]);
  let history = $state<UsageHistoryPoint[]>([]);
  let loadingDaily = $state(false);
  type TrendDiag = { path: string; path_exists: boolean; jsonl_count: number; total_bytes: number };
  let trendDiag = $state<TrendDiag | null>(null);
  async function loadDaily() {
    if ((daily.length > 0 || history.length > 0) || loadingDaily) return;
    loadingDaily = true;
    try {
      const [d, h, diag] = await Promise.all([
        fetchClaudeDailyTokens(30),
        fetchUsageHistory(30),
        (async () => {
          try { const { invoke } = await import("@tauri-apps/api/core"); return await invoke<TrendDiag>("claude_trend_diag"); } catch { return null; }
        })(),
      ]);
      daily = d;
      history = h;
      trendDiag = diag;
    } finally { loadingDaily = false; }
  }
  async function setView(v: "now" | "trend") {
    view = v;
    if (v === "trend") await loadDaily();
  }
  $effect(() => {
    if (active === "claude" && view === "trend") loadDaily();
  });

  async function focus(id: string) {
    active = id;
    if (id === "claude") {
      activePayload = p.usageCache[id] ?? null;
    } else {
      // mock for now
      loading = true;
      activePayload = await p.onLoadSegment(id, "api");
      loading = false;
    }
  }
  $effect(() => {
    if (active === "claude" && p.usageCache.claude) activePayload = p.usageCache.claude;
  });

  const ts = $derived(p.now.toLocaleTimeString("en-US", { hour12: false }));
  const dateStr = $derived(`${p.now.getFullYear()}.${String(p.now.getMonth()+1).padStart(2,"0")}.${String(p.now.getDate()).padStart(2,"0")}`);
  function fiveh(id: string): number | null {
    const s = p.usageCache[id]; if (!s?.segments?.length) return null;
    const seg = s.segments.find(x => x.label === "5h window"); return seg ? Math.round(seg.used) : null;
  }
  const activeProvider = $derived(p.providers.find((x) => x.id === active));
</script>

<div class="frame" data-tauri-drag-region>
<header class="hd" data-tauri-drag-region>
    <div class="title" data-tauri-drag-region>Quota</div>
    <button onclick={toggleSidebar} class="ct ct-side" data-tauri-drag-region="false" title={sidebarOpen ? "Hide sidebar" : "Show sidebar"} aria-label="Toggle sidebar">
      <svg width="14" height="14" viewBox="0 0 16 16" fill="none">
        <rect x="2" y="3" width="12" height="10" rx="1.5" stroke="currentColor" stroke-width="1.3"/>
        <line x1="6.2" y1="3.4" x2="6.2" y2="12.6" stroke="currentColor" stroke-width="1.3"/>
      </svg>
    </button>
    <div class="date">{dateStr}<br/>{ts}</div>
    <div class="ctrls" data-tauri-drag-region="false">
      <button onclick={p.onToggleLang} class="ct ct-lang" title={p.lang === "ko" ? "한국어" : "English"} aria-label="Toggle language">
        {p.lang === "ko" ? "KO" : "EN"}
      </button>
      <button onclick={p.onRefresh} class="ct" class:spin={p.refreshing} title={T.refreshTitle} aria-label="Refresh">
        <svg width="14" height="14" viewBox="0 0 16 16" fill="none">
          <path d="M14 8a6 6 0 11-2-4.5M14 2v4h-4" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
        </svg>
      </button>
      <button onclick={() => (showSettings = true)} class="ct" title={T.settings} aria-label="Settings">
        <svg width="14" height="14" viewBox="0 0 16 16" fill="none">
          <path d="M9.6 1.5l.3 1.6a5.5 5.5 0 011.4 0.8l1.5-.5 1.4 2.4-1.2 1.1a5.5 5.5 0 010 1.6l1.2 1.1-1.4 2.4-1.5-.5a5.5 5.5 0 01-1.4 .8l-.3 1.6h-3.2l-.3-1.6a5.5 5.5 0 01-1.4-.8l-1.5 .5-1.4-2.4 1.2-1.1a5.5 5.5 0 010-1.6l-1.2-1.1 1.4-2.4 1.5 .5a5.5 5.5 0 011.4-.8l.3-1.6h3.2z" stroke="currentColor" stroke-width="1.2" stroke-linejoin="round"/>
          <circle cx="8" cy="8" r="2" stroke="currentColor" stroke-width="1.2"/>
        </svg>
      </button>
      <button onclick={p.onToggleDark} class="ct" class:on={p.isDark} title={p.isDark ? T.light : T.dark} aria-label="Toggle dark mode">
        {#if p.isDark}
          <svg width="14" height="14" viewBox="0 0 16 16" fill="none">
            <circle cx="8" cy="8" r="3" stroke="currentColor" stroke-width="1.4"/>
            <path d="M8 1.5v1.5M8 13v1.5M1.5 8h1.5M13 8h1.5M3.5 3.5l1.1 1.1M11.4 11.4l1.1 1.1M3.5 12.5l1.1-1.1M11.4 4.6l1.1-1.1" stroke="currentColor" stroke-width="1.3" stroke-linecap="round"/>
          </svg>
        {:else}
          <svg width="14" height="14" viewBox="0 0 16 16" fill="none">
            <path d="M14.5 7.5A6 6 0 118 1A4.7 4.7 0 0014.5 7.5z" stroke="currentColor" stroke-width="1.4" stroke-linejoin="round"/>
          </svg>
        {/if}
      </button>
      <button onclick={p.onClose} class="ct danger" title={T.close} aria-label="Close">
        <svg width="14" height="14" viewBox="0 0 16 16" fill="none">
          <path d="M3.5 3.5l9 9M12.5 3.5l-9 9" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
        </svg>
      </button>
    </div>
  </header>

  <div class="rule"></div>

  <section class="content" class:rail={!sidebarOpen} data-tauri-drag-region="false">
    <aside class="picker">
      {#each p.providers as prov (prov.id)}
        {@const fp = fiveh(prov.id)}
        <button class="px" class:active={active === prov.id} onclick={() => focus(prov.id)} style="--accent: {prov.accent};" title={providerName(prov.id, p.lang, prov.name)}>
          <span class="px-logo"><Logo id={prov.id} size={14} /></span>
          <span class="nm">{providerName(prov.id, p.lang, prov.name)}</span>
          {#if fp != null}<span class="pc">{fp}<span class="pcs">%</span></span>{/if}
        </button>
      {/each}
    </aside>

    <div class="detail" style="--accent: {activeProvider?.accent ?? '#999'};">
      <div class="big-row">
        <span class="big-logo"><Logo id={active} size={20} /></span>
        <div class="big-name">{providerName(activeProvider?.id ?? "", p.lang, activeProvider?.name ?? "")}</div>
        {#if active === "claude"}
          <div class="tabs">
            <button class:active={view === "now"} onclick={() => setView("now")}>{T.nowTab}</button>
            <span class="tab-sep">·</span>
            <button class:active={view === "trend"} onclick={() => setView("trend")}>{T.trendTab}</button>
          </div>
        {/if}
        <button class="link" onclick={() => p.onOpenProvider(active)}>{T.openDashboard}</button>
      </div>

      {#if active === "claude" && view === "trend"}
        {#if loadingDaily}
          <div class="hint">{T.loadingDaily}</div>
        {:else if daily.length && daily.some((d) => d.tokens > 0)}
          {@const peak = Math.max(...daily.map(d=>d.tokens))}
          {@const total = daily.reduce((a,d)=>a+d.tokens,0)}
          {@const avg = Math.round(total/daily.length)}
          {@const totalMsgs = daily.reduce((a,d)=>a+d.messages,0)}
          {@const fmtNum = (n: number) => n >= 1e9 ? (n/1e9).toFixed(2)+"B" : n >= 1e6 ? (n/1e6).toFixed(1)+"M" : n >= 1e3 ? (n/1e3).toFixed(1)+"k" : String(n)}
          <div class="trend-wrap">
            <Sparkline
              data={daily.map((d) => ({ date: d.date, value: d.tokens }))}
              accent={activeProvider?.accent ?? "#d97757"}
              height={110}
            />
            <div class="trend-stats">
              <div class="ts-cell"><div class="ts-k">{T.peak}</div><div class="ts-v">{fmtNum(peak)}<span class="ts-u">{T.tokens}</span></div></div>
              <div class="ts-cell"><div class="ts-k">{T.avg}</div><div class="ts-v">{fmtNum(avg)}<span class="ts-u">{T.perDay}</span></div></div>
              <div class="ts-cell"><div class="ts-k">{T.total}</div><div class="ts-v">{fmtNum(total)}<span class="ts-u">{T.tokens}</span></div></div>
              <div class="ts-cell"><div class="ts-k">{T.msgs}</div><div class="ts-v">{totalMsgs.toLocaleString()}</div></div>
            </div>
          </div>
        {:else if history.length >= 2}
          {@const fmtTime = (ts: number) => { const d = new Date(ts*1000); return `${String(d.getMonth()+1).padStart(2,"0")}.${String(d.getDate()).padStart(2,"0")} ${String(d.getHours()).padStart(2,"0")}:${String(d.getMinutes()).padStart(2,"0")}`; }}
          {@const sevenSeries = history.filter(p => p.seven_day != null).map(p => ({ date: fmtTime(p.ts), value: (p.seven_day ?? 0) * 100 }))}
          {@const fiveSeries = history.filter(p => p.five_hour != null).map(p => ({ date: fmtTime(p.ts), value: (p.five_hour ?? 0) * 100 }))}
          <div class="trend-wrap">
            <div class="trend-h">{T.trendUtilHeader}</div>
            <Sparkline data={sevenSeries} accent={activeProvider?.accent ?? "#d97757"} height={90} />
            <div class="trend-stats">
              <div class="ts-cell"><div class="ts-k">{T.trendWeeklyNow}</div><div class="ts-v">{Math.round((history[history.length-1].seven_day ?? 0)*100)}<span class="ts-u">%</span></div></div>
              <div class="ts-cell"><div class="ts-k">{T.trend5hNow}</div><div class="ts-v">{Math.round((history[history.length-1].five_hour ?? 0)*100)}<span class="ts-u">%</span></div></div>
              <div class="ts-cell"><div class="ts-k">{T.trendPoints}</div><div class="ts-v">{history.length}</div></div>
              <div class="ts-cell"><div class="ts-k">{T.trendSpan}</div><div class="ts-v">{Math.round((history[history.length-1].ts - history[0].ts) / 3600)}<span class="ts-u">h</span></div></div>
            </div>
          </div>
        {:else}
          <div class="trend-empty">
            <div class="trend-empty-icon">
              <svg width="32" height="32" viewBox="0 0 32 32" fill="none">
                <path d="M4 24L11 18L17 22L28 10" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" stroke-dasharray="2 3"/>
                <circle cx="11" cy="18" r="1.5" fill="currentColor"/>
                <circle cx="17" cy="22" r="1.5" fill="currentColor"/>
                <circle cx="28" cy="10" r="1.5" fill="currentColor"/>
              </svg>
            </div>
            <div class="trend-empty-h">{T.noDailyData}</div>
            <div class="trend-empty-sub">{T.noDailyDataHint}</div>
          </div>
        {/if}
      {:else if loading}
        <div class="hint">{T.loading}</div>
      {:else if activePayload?.error}
        <div class="hint err">{activePayload.error}</div>
      {:else if activePayload?.segments?.length}
        {#if activePayload.plan}<div class="plan-line">{T.plan} · <strong>{activePayload.plan}</strong></div>{/if}
        {#each activePayload.segments as seg (seg.label)}
          {@const pct = Math.round((seg.used / seg.limit) * 100)}
          <div class="seg">
            <div class="seg-h">
              <span class="seg-l">{T.bucket(seg.label)}</span>
              {#if seg.resetAt}<span class="seg-r">{seg.resetAt}</span>{/if}
            </div>
            <div class="big-pc"><span class="big-num">{pct}</span><span class="big-sym">%</span></div>
            <div class="bar"><div class="fill" style="width: {pct}%"></div></div>
          </div>
        {/each}
      {:else}
        <div class="hint">{T.noData(activeProvider?.name.toLowerCase() ?? "")}</div>
      {/if}
    </div>
  </section>

  <footer class="ft" data-tauri-drag-region>
    <span class="ft-l" data-tauri-drag-region>{T.sources(p.providers.length)}</span>
    <span class="ft-c" data-tauri-drag-region>{T.lastSync} {p.lastRefresh ? p.lastRefresh.toLocaleTimeString(p.lang === "ko" ? "ko-KR" : "en-US",{hour12:false}) : "—"}</span>
    <span class="ft-r" data-tauri-drag-region>v0.1.2</span>
  </footer>

  {#if showSettings}
    <Settings onClose={() => (showSettings = false)} />
  {/if}
</div>

<style>
  .frame {
    position: absolute; inset: 0;
    background: #f7f5ef;
    color: #1a1a1a;
    font-family: "Inter", -apple-system, "Segoe UI", system-ui, sans-serif;
    overflow: hidden;
    display: flex;
    flex-direction: column;
    border: 1px solid #d8d3c5;
    isolation: isolate;
  }

  /* ==================== BACKGROUND VARIANTS (v2) ==================== */
  .bg {
    position: absolute;
    inset: 0;
    z-index: 0;
    pointer-events: none;
    overflow: hidden;
  }
  .frame > :not(.bg) { position: relative; z-index: 1; }

  /* === NEBULA: 5 vivid color clouds slowly morphing position+scale === */
  .bg-nebula .neb {
    position: absolute;
    width: 65%;
    aspect-ratio: 1;
    border-radius: 50%;
    will-change: transform;
  }
  .bg-nebula .n1 { top: -20%; left: -15%; background: radial-gradient(circle, rgba(217, 119, 87, 0.55), transparent 60%); animation: neb1 17s ease-in-out infinite alternate; }
  .bg-nebula .n2 { bottom: -25%; right: -10%; background: radial-gradient(circle, rgba(78, 134, 211, 0.42), transparent 60%); animation: neb2 21s ease-in-out infinite alternate; }
  .bg-nebula .n3 { top: 25%; right: -20%; width: 50%; background: radial-gradient(circle, rgba(195, 95, 195, 0.38), transparent 60%); animation: neb3 19s ease-in-out infinite alternate; }
  .bg-nebula .n4 { bottom: 15%; left: 10%; width: 45%; background: radial-gradient(circle, rgba(120, 200, 160, 0.32), transparent 60%); animation: neb4 23s ease-in-out infinite alternate; }
  .bg-nebula .n5 { top: 40%; left: -10%; width: 38%; background: radial-gradient(circle, rgba(240, 180, 100, 0.30), transparent 60%); animation: neb5 16s ease-in-out infinite alternate; }
  @keyframes neb1 { to { transform: translate(22%, 18%) scale(1.25); } }
  @keyframes neb2 { to { transform: translate(-18%, -22%) scale(1.30); } }
  @keyframes neb3 { to { transform: translate(-28%, 22%) scale(1.18); } }
  @keyframes neb4 { to { transform: translate(20%, -15%) scale(1.40); } }
  @keyframes neb5 { to { transform: translate(30%, 25%) scale(1.25); } }

  /* === STARFIELD: 38 stars at 3 size tiers, twinkle (opacity stagger) === */
  .bg-starfield .star {
    position: absolute;
    width: var(--sz, 1px);
    height: var(--sz, 1px);
    background: rgba(217, 119, 87, 0.85);
    border-radius: 50%;
    box-shadow: 0 0 var(--sz) rgba(217, 119, 87, 0.4);
    animation: twinkle 3.5s ease-in-out infinite;
    animation-delay: var(--d);
  }
  @keyframes twinkle {
    0%, 100% { opacity: 0.18; transform: scale(0.8); }
    50%      { opacity: 1;    transform: scale(1.1); }
  }

  /* === CONSTELLATION: 22 connected dots, lines fade in/out, dots twinkle === */
  .bg-constellation svg {
    position: absolute; inset: 0;
    width: 100%; height: 100%;
  }
  .bg-constellation .lines line {
    stroke: rgba(217, 119, 87, 0.35);
    stroke-width: 1;
    animation: con-line 4s ease-in-out infinite;
    animation-delay: var(--d);
  }
  @keyframes con-line {
    0%, 100% { opacity: 0.1; }
    50%      { opacity: 0.7; }
  }
  .bg-constellation .dots circle {
    fill: rgba(217, 119, 87, 0.85);
    animation: con-dot 3s ease-in-out infinite;
    animation-delay: var(--d);
  }
  @keyframes con-dot {
    0%, 100% { opacity: 0.5; r: 1.5; }
    50%      { opacity: 1;   r: 3; }
  }

  /* === AURORA: 4 colorful ribbons sweeping with vertical curve === */
  .bg-aurora { display: block; }
  .bg-aurora .band {
    position: absolute;
    left: -60%;
    width: 220%;
    height: 70%;
    will-change: transform;
    border-radius: 50%;
    filter: none;
  }
  .bg-aurora .b1 {
    top: -25%;
    background: radial-gradient(ellipse 50% 100% at 50% 50%, rgba(217, 119, 87, 0.50) 0%, rgba(217, 119, 87, 0.20) 35%, transparent 70%);
    animation: aurora-flow 13s ease-in-out infinite alternate;
  }
  .bg-aurora .b2 {
    top: 5%;
    background: radial-gradient(ellipse 60% 100% at 50% 50%, rgba(195, 95, 195, 0.45) 0%, rgba(195, 95, 195, 0.18) 40%, transparent 75%);
    animation: aurora-flow 17s ease-in-out infinite alternate-reverse;
  }
  .bg-aurora .b3 {
    top: 35%;
    background: radial-gradient(ellipse 55% 100% at 50% 50%, rgba(78, 134, 211, 0.45) 0%, rgba(78, 134, 211, 0.18) 40%, transparent 75%);
    animation: aurora-flow 15s ease-in-out infinite alternate;
  }
  .bg-aurora .b4 {
    top: 65%;
    background: radial-gradient(ellipse 60% 100% at 50% 50%, rgba(170, 220, 130, 0.42) 0%, rgba(170, 220, 130, 0.16) 40%, transparent 75%);
    animation: aurora-flow 19s ease-in-out infinite alternate-reverse;
  }
  @keyframes aurora-flow {
    0%   { transform: translateX(-15%) translateY(0%); }
    100% { transform: translateX(15%)  translateY(-3%); }
  }

  /* === WAVES: 6 layered sine waves, parallax — stroke-dashoffset chase === */
  .bg-waves svg {
    position: absolute; inset: 0;
    width: 100%; height: 100%;
    fill: none;
  }
  .bg-waves path {
    stroke-width: 1.4;
    stroke-linecap: round;
    stroke-dasharray: 12 18;
  }
  .bg-waves path:nth-child(1) { stroke: rgba(217, 119, 87, 0.55); animation: wave-chase 5s linear infinite; }
  .bg-waves path:nth-child(2) { stroke: rgba(78, 134, 211, 0.45); animation: wave-chase 6s linear infinite reverse; }
  .bg-waves path:nth-child(3) { stroke: rgba(195, 95, 195, 0.40); animation: wave-chase 7s linear infinite; }
  .bg-waves path:nth-child(4) { stroke: rgba(170, 220, 130, 0.40); animation: wave-chase 8s linear infinite reverse; }
  .bg-waves path:nth-child(5) { stroke: rgba(217, 119, 87, 0.32); animation: wave-chase 9s linear infinite; stroke-width: 1; }
  .bg-waves path:nth-child(6) { stroke: rgba(78, 134, 211, 0.28); animation: wave-chase 10s linear infinite reverse; stroke-width: 1; }
  @keyframes wave-chase {
    from { stroke-dashoffset: 0; }
    to   { stroke-dashoffset: -150; }
  }

  /* === PULSE: 3 emitter points each emit 3 expanding rings === */
  .bg-pulse .emit {
    position: absolute;
    width: 10px; height: 10px;
  }
  .bg-pulse .emit::before {
    content: "";
    position: absolute;
    inset: 0;
    width: 8px; height: 8px;
    border-radius: 50%;
    background: rgba(217, 119, 87, 0.7);
    box-shadow: 0 0 8px rgba(217, 119, 87, 0.5);
  }
  .bg-pulse .e1 { top: 25%; left: 18%; }
  .bg-pulse .e2 { top: 60%; left: 65%; }
  .bg-pulse .e3 { top: 80%; left: 35%; }
  .bg-pulse .e1 ::before { background: rgba(217, 119, 87, 0.7); }
  .bg-pulse .emit > span {
    position: absolute;
    inset: -2px;
    width: 14px; height: 14px;
    border: 1.5px solid rgba(217, 119, 87, 0.7);
    border-radius: 50%;
    opacity: 0;
    will-change: transform, opacity;
  }
  .bg-pulse .e1 > .r1 { animation: emit-ring 4s ease-out infinite; }
  .bg-pulse .e1 > .r2 { animation: emit-ring 4s ease-out infinite 1.3s; }
  .bg-pulse .e1 > .r3 { animation: emit-ring 4s ease-out infinite 2.6s; }
  .bg-pulse .e2 > span { border-color: rgba(78, 134, 211, 0.7); }
  .bg-pulse .e2 > .r1 { animation: emit-ring 5s ease-out infinite 0.5s; }
  .bg-pulse .e2 > .r2 { animation: emit-ring 5s ease-out infinite 2s; }
  .bg-pulse .e2 > .r3 { animation: emit-ring 5s ease-out infinite 3.5s; }
  .bg-pulse .e3 > span { border-color: rgba(195, 95, 195, 0.7); }
  .bg-pulse .e3 > .r1 { animation: emit-ring 4.5s ease-out infinite 0.2s; }
  .bg-pulse .e3 > .r2 { animation: emit-ring 4.5s ease-out infinite 1.7s; }
  .bg-pulse .e3 > .r3 { animation: emit-ring 4.5s ease-out infinite 3.2s; }
  @keyframes emit-ring {
    0%   { transform: scale(0.5); opacity: 0; }
    20%  { opacity: 0.8; }
    100% { transform: scale(20); opacity: 0; }
  }

  /* === TUNNEL: rings receding to a vanishing point at center === */
  .bg-tunnel {
    display: grid;
    place-items: center;
    perspective: 800px;
  }
  .bg-tunnel .t-ring {
    position: absolute;
    width: 200%; aspect-ratio: 1;
    border: 2px solid rgba(217, 119, 87, 0.55);
    border-radius: 50%;
    opacity: 0;
    transform: scale(0.05);
    animation: tunnel-out 4s linear infinite;
    will-change: transform, opacity;
  }
  .bg-tunnel .t-ring:nth-child(1) { animation-delay: 0s; }
  .bg-tunnel .t-ring:nth-child(2) { animation-delay: 0.5s; border-color: rgba(78, 134, 211, 0.55); }
  .bg-tunnel .t-ring:nth-child(3) { animation-delay: 1s; }
  .bg-tunnel .t-ring:nth-child(4) { animation-delay: 1.5s; border-color: rgba(195, 95, 195, 0.55); }
  .bg-tunnel .t-ring:nth-child(5) { animation-delay: 2s; }
  .bg-tunnel .t-ring:nth-child(6) { animation-delay: 2.5s; border-color: rgba(78, 134, 211, 0.55); }
  .bg-tunnel .t-ring:nth-child(7) { animation-delay: 3s; }
  .bg-tunnel .t-ring:nth-child(8) { animation-delay: 3.5s; border-color: rgba(195, 95, 195, 0.55); }
  @keyframes tunnel-out {
    0%   { transform: scale(0.04); opacity: 0; }
    8%   { opacity: 0.9; }
    100% { transform: scale(1.4); opacity: 0; }
  }

  /* === STREAMS: 11 vertical light streams falling, varying speed/opacity === */
  .bg-streams .stream {
    position: absolute;
    top: -30%;
    width: 1px;
    height: 130%;
    background: linear-gradient(180deg,
      transparent 0%,
      rgba(217, 119, 87, var(--opacity, 0.4)) 30%,
      rgba(255, 175, 130, var(--opacity, 0.7)) 50%,
      rgba(217, 119, 87, var(--opacity, 0.4)) 70%,
      transparent 100%);
    animation: stream-fall var(--dur, 5s) linear infinite;
    animation-delay: var(--delay, 0s);
    will-change: transform;
  }
  @keyframes stream-fall {
    from { transform: translateY(-50%); }
    to   { transform: translateY(75%); }
  }

  /* === MESH: vivid multi-color radial mesh slowly shifting === */
  .bg-mesh {
    background:
      radial-gradient(circle at 20% 25%, rgba(217, 119, 87, 0.40), transparent 35%),
      radial-gradient(circle at 75% 15%, rgba(78, 134, 211, 0.35), transparent 38%),
      radial-gradient(circle at 50% 80%, rgba(195, 95, 195, 0.30), transparent 40%),
      radial-gradient(circle at 85% 70%, rgba(170, 220, 130, 0.28), transparent 38%),
      radial-gradient(circle at 10% 70%, rgba(240, 180, 100, 0.22), transparent 35%);
    animation: mesh-shift 20s ease-in-out infinite alternate;
    will-change: background-position;
  }
  @keyframes mesh-shift {
    0%   { background-position: 0% 0%, 0% 0%, 0% 0%, 0% 0%, 0% 0%; }
    100% { background-position: 8% 6%, -10% 8%, 6% -8%, -8% 4%, 10% -6%; }
  }

  .hd {
    display: flex; align-items: center; padding: 14px 14px 12px 26px;
    cursor: grab;
    flex-shrink: 0;
  }
  .ct-side { margin-left: 10px; }
  .hd:active { cursor: grabbing; }
  .title { font-size: 20px; font-weight: 400; letter-spacing: -0.015em; color: #1a1a1a; pointer-events: none; line-height: 1; }
  .date { margin-left: 14px; padding-left: 14px; border-left: 1px solid #d8d3c5; font-size: 10px; line-height: 1.4; color: #777; pointer-events: none; font-feature-settings: "tnum"; }
  .ctrls { margin-left: auto; display: flex; gap: 3px; align-items: center; }
  .ct {
    width: 28px;
    height: 28px;
    background: none;
    border: none;
    color: #777;
    cursor: pointer;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    border-radius: 50%;
    padding: 0;
    font-family: inherit;
    flex-shrink: 0;
    line-height: 1;
  }
  .ct svg { display: block; }
  .ct-lang {
    font-size: 10px;
    font-weight: 600;
    letter-spacing: 0.06em;
    color: #777;
  }
  .ct:hover { background: #ece6d5; color: #1a1a1a; }
  .ct.on { background: #1a1a1a; color: #f7f5ef; }
  .ct.danger:hover { background: #1a1a1a; color: #f7f5ef; }
  .ct.spin svg { animation: spin 0.8s linear infinite; }
  @keyframes spin { to { transform: rotate(360deg); } }
  .rule { height: 1px; background: #1a1a1a; margin: 0 18px; flex-shrink: 0; }

  .content {
    flex: 1;
    display: grid;
    grid-template-columns: 121px 1fr;
    gap: 0;
    overflow: hidden;
    padding: 14px 0 14px 26px;
    transition: grid-template-columns 240ms cubic-bezier(0.2,0.8,0.2,1);
  }
  .content.rail { grid-template-columns: 38px 1fr; }
  .content.rail .picker { padding-right: 8px; }
  .content.rail .px { padding-left: 0; padding-right: 0; gap: 0; }
  .content.rail .px .nm { display: none; }
  .content.rail .px .pc { display: none; }
  .picker { display: flex; flex-direction: column; gap: 4px; padding-right: 10px; border-right: 1px solid #d8d3c5; }
  .px {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 10px 8px 0;
    background: none;
    border: none;
    border-bottom: 1px solid transparent;
    cursor: pointer;
    text-align: left;
    color: #999;
    font-family: inherit;
  }
  .px .px-logo {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 16px; height: 16px;
    flex-shrink: 0;
    line-height: 0;
  }
  .px .nm { font-size: 13px; font-weight: 500; }
  .px .pc { margin-left: auto; font-size: 12px; font-weight: 600; color: var(--accent); font-feature-settings: "tnum"; }
  .px .pcs { font-size: 9px; color: #aaa; margin-left: 1px; }
  .px:hover { color: #1a1a1a; }
  .px.active { color: #1a1a1a; border-bottom-color: var(--accent); }
  .px.active .nm { font-weight: 600; }

  .detail {
    padding: 0 22px 26px 24px;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .detail > .seg:last-child { margin-bottom: 8px; }
  .detail::-webkit-scrollbar { width: 3px; }
  .detail::-webkit-scrollbar-thumb { background: #d8d3c5; }
  .big-row {
    display: flex;
    align-items: baseline;
    gap: 9px;
    padding-bottom: 4px;
  }
  .big-logo {
    width: 22px; height: 22px;
    color: var(--accent);
    flex-shrink: 0;
    line-height: 0;
    align-self: center;
    transform: translateY(2px);
  }
  .big-name { font-size: 22px; font-weight: 200; letter-spacing: -0.02em; color: #1a1a1a; line-height: 1; }
  .link {
    margin-left: auto;
    background: none; border: none; padding: 0;
    color: var(--accent); font-size: 10px; font-family: inherit;
    cursor: pointer; letter-spacing: 0.04em;
  }
  .tabs {
    display: inline-flex;
    align-items: baseline;
    gap: 6px;
    margin-left: 14px;
    padding-left: 14px;
    border-left: 1px solid #d8d3c5;
  }
  .tabs button {
    background: none; border: none; padding: 0;
    color: #aaa; font-size: 10px;
    font-family: inherit;
    cursor: pointer;
    letter-spacing: 0.04em;
    text-transform: lowercase;
  }
  .tabs button.active { color: #1a1a1a; font-weight: 500; }
  .tabs button:hover { color: #1a1a1a; }
  .tab-sep { color: #ccc; }
  .trend-wrap { padding-top: 10px; }
  .trend-h { font-size: 10px; letter-spacing: 0.06em; text-transform: uppercase; color: #999; padding-bottom: 4px; }
  .trend-empty {
    display: flex; flex-direction: column; align-items: center; justify-content: center;
    text-align: center;
    padding: 40px 24px 24px 24px;
    gap: 8px;
  }
  .trend-empty-icon { color: #c8c2b1; margin-bottom: 6px; }
  .trend-empty-h { font-size: 13px; font-weight: 500; color: #1a1a1a; }
  .trend-empty-sub { font-size: 11px; color: #999; line-height: 1.5; max-width: 320px; }
  .trend-stats {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 12px;
    margin-top: 14px;
    padding-top: 12px;
    border-top: 1px solid #ece6d5;
  }
  .ts-cell { display: flex; flex-direction: column; gap: 3px; }
  .ts-k {
    font-size: 9px;
    letter-spacing: 0.12em;
    color: #999;
    text-transform: uppercase;
    font-weight: 500;
  }
  .ts-v {
    font-family: "JetBrains Mono", monospace;
    font-size: 14px;
    font-weight: 600;
    color: #1a1a1a;
    font-feature-settings: "tnum";
    display: inline-flex;
    align-items: baseline;
    gap: 3px;
  }
  .ts-u { font-size: 9px; color: #999; font-weight: 400; }
  .trend-meta {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 12px;
    padding-top: 10px;
    margin-top: 6px;
    border-top: 1px solid #ece6d5;
    font-feature-settings: "tnum";
  }
  .trend-meta > div { display: flex; flex-direction: column; gap: 2px; }
  .tm-k { font-size: 8px; letter-spacing: 0.16em; color: #999; text-transform: uppercase; }
  .tm-v { font-size: 13px; font-weight: 500; color: #1a1a1a; }
  .link:hover { text-decoration: underline; }
  .plan-line { font-size: 10px; color: #777; padding-top: 14px; }
  .plan-line strong { color: #1a1a1a; font-weight: 600; }
  .hint { color: #999; font-size: 11px; padding: 8px 0; }
  .hint.err { color: #cc3333; }
  .seg { padding: 8px 0 0 0; border-top: 1px solid #ece6d5; margin-top: 6px; }
  .seg:first-of-type { border-top: none; }
  .seg-h { display: flex; justify-content: space-between; font-size: 9px; letter-spacing: 0.06em; color: #777; text-transform: uppercase; }
  .seg-r { color: #aaa; }
  .big-pc { display: flex; align-items: baseline; gap: 1px; padding: 2px 0; }
  .big-num { font-size: 28px; font-weight: 200; letter-spacing: -0.04em; color: #1a1a1a; line-height: 1; font-feature-settings: "tnum"; }
  .big-sym { font-size: 12px; color: #777; font-weight: 400; }
  .bar { height: 2px; background: #ece6d5; position: relative; overflow: hidden; }
  .fill {
    position: absolute; inset: 0 auto 0 0;
    background: var(--accent);
    transition: width 700ms cubic-bezier(0.2,0.8,0.2,1);
    overflow: hidden;
  }
  .fill::after {
    content: "";
    position: absolute;
    top: 0; bottom: 0; left: 0;
    width: 35%;
    background: linear-gradient(
      90deg,
      transparent 0%,
      rgba(255, 255, 255, 0.55) 50%,
      transparent 100%
    );
    animation: sheen 3.4s ease-in-out infinite;
    will-change: transform;
  }
  @keyframes sheen {
    0%   { transform: translateX(-100%); }
    55%  { transform: translateX(420%); }
    100% { transform: translateX(420%); }
  }

  .ft {
    display: flex; align-items: center; gap: 12px;
    padding: 10px 18px;
    border-top: 1px solid #d8d3c5;
    font-size: 9px;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: #999;
    flex-shrink: 0;
  }
  .ft-l, .ft-r { color: #aaa; }
  .ft-c {
    flex: 1; text-align: center;
    display: inline-flex; align-items: center; justify-content: center; gap: 6px;
  }
  .ft-c::before {
    content: "";
    width: 5px; height: 5px;
    border-radius: 50%;
    background: var(--accent, #c66);
    --accent: #d97757;
    animation: livepulse 2.2s ease-in-out infinite;
  }
  @keyframes livepulse {
    0%, 100% { opacity: 0.25; transform: scale(0.85); }
    50%      { opacity: 1;    transform: scale(1.05); }
  }
</style>
