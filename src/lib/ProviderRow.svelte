<script lang="ts">
  import type { Provider, QuotaKind, QuotaPayload } from "./types";
  import QuotaBar from "./QuotaBar.svelte";
  import Sparkline from "./Sparkline.svelte";
  import Logo from "./Logo.svelte";
  import { getQuota, fetchClaudeDailyTokens, openProviderUrl, type DailyEntry } from "./realData";

  let {
    provider,
    index = 0,
    snapshot = null,
  }: {
    provider: Provider;
    index?: number;
    snapshot?: QuotaPayload | null;
  } = $props();

  let daily = $state<DailyEntry[]>([]);

  // mini % shown next to chevron (5h utilization for Claude only)
  const miniPct = $derived.by(() => {
    if (provider.id !== "claude" || !snapshot?.segments?.length) return null;
    const seg = snapshot.segments.find((s) => s.label === "5h window");
    return seg ? Math.round(seg.used) : null;
  });
  const miniStatus = $derived(
    miniPct == null ? "" : miniPct > 90 ? "danger" : miniPct > 75 ? "warn" : "ok"
  );

  function openExternal(e: MouseEvent) {
    e.stopPropagation();
    openProviderUrl(provider.id);
  }

  let expanded = $state(false);
  let selectedKind = $state<QuotaKind | null>(null);
  let payload = $state<QuotaPayload | null>(null);
  let loading = $state(false);

  function toggle() {
    if (expanded && selectedKind) {
      selectedKind = null;
      payload = null;
      return;
    }
    expanded = !expanded;
    if (!expanded) {
      selectedKind = null;
      payload = null;
    }
  }

  async function pick(kind: QuotaKind) {
    if (!provider.available[kind]) return;
    selectedKind = kind;
    loading = true;
    payload = null;
    try {
      payload = await getQuota(provider.id, kind);
      if (provider.id === "claude" && kind === "plan" && daily.length === 0) {
        daily = await fetchClaudeDailyTokens(30);
      }
    } finally {
      loading = false;
    }
  }
  const idxStr = $derived(String(index + 1).padStart(2, "0"));
</script>

<div class="row" class:expanded style="--accent: {provider.accent};">
  <button class="head" onclick={toggle} aria-expanded={expanded}>
    <span class="idx">N{idxStr}</span>
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <span class="logo-wrap" onclick={openExternal} title="Open {provider.name} usage page">
      <span class="logo"><Logo id={provider.id} size={14} /></span>
    </span>
    <span class="name">{provider.name}</span>
    {#if miniPct !== null}
      <span class="mini-pct" data-status={miniStatus}>{miniPct}%</span>
    {/if}
    <span class="meta">
      {#if provider.available.api}<span class="tag">API</span>{/if}
      {#if provider.available.plan}<span class="tag tag-plan">PLAN</span>{/if}
    </span>
    <span class="chev" class:open={expanded}>▶</span>
  </button>

  {#if expanded}
    <div class="body">
      {#if !selectedKind}
        <div class="choice">
          <button
            class="choice-btn"
            class:disabled={!provider.available.api}
            disabled={!provider.available.api}
            onclick={() => pick("api")}
          >
            <span class="ck-glyph">›</span>
            <span class="ck-stack">
              <span class="ck-label">API</span>
              <span class="ck-sub">credits · rate</span>
            </span>
          </button>
          <button
            class="choice-btn"
            class:disabled={!provider.available.plan}
            disabled={!provider.available.plan}
            onclick={() => pick("plan")}
          >
            <span class="ck-glyph">›</span>
            <span class="ck-stack">
              <span class="ck-label">PLAN</span>
              <span class="ck-sub">5h · weekly</span>
            </span>
          </button>
        </div>
      {:else}
        <div class="bars">
          <div class="bars-head">
            <button class="back" onclick={() => { selectedKind = null; payload = null; }}>
              ◂ {selectedKind === "api" ? "API" : "PLAN"}
            </button>
            <span class="bars-info">
              {#if payload?.plan}
                <span class="plan-tag">{payload.plan}</span>
              {/if}
              {#if payload?.fetchedAt}
                <span class="live">◆ LIVE</span>
              {/if}
            </span>
          </div>
          {#if loading}
            <div class="loading">DECRYPTING...</div>
          {:else if payload?.error}
            <div class="err">▲ {payload.error}</div>
          {:else if payload}
            {#each payload.segments as seg, i (seg.label)}
              <QuotaBar
                label={seg.label}
                used={seg.used}
                limit={seg.limit}
                unit={seg.unit}
                resetAt={seg.resetAt}
                accent={provider.accent}
                glow={provider.accent}
                delay={i * 80}
              />
            {/each}
            {#if provider.id === "claude" && selectedKind === "plan" && daily.length > 0}
              <Sparkline
                data={daily.map((d) => ({ date: d.date, value: d.tokens }))}
                accent={provider.accent}
              />
            {/if}
          {/if}
        </div>
      {/if}
    </div>
  {/if}
</div>

<style>
  .row {
    border-bottom: 1px solid var(--line);
    position: relative;
  }
  .row:last-child { border-bottom: none; }
  .row.expanded {
    background: var(--bg-2);
  }
  .head {
    width: 100%;
    display: flex;
    align-items: center;
    gap: 9px;
    padding: 8px 10px 8px 0;
    text-align: left;
    cursor: pointer;
    border-left: 3px solid transparent;
    padding-left: 9px;
  }
  .head:hover {
    background: var(--bg-2);
  }
  .row.expanded .head {
    border-left-color: var(--accent);
    background: var(--bg-3);
  }
  .idx {
    font-family: var(--mono);
    font-size: 9px;
    letter-spacing: 0.1em;
    color: var(--text-3);
    min-width: 22px;
  }
  .logo-wrap {
    display: grid;
    place-items: center;
    width: 22px;
    height: 22px;
    border: 1px solid var(--line-strong);
    background: var(--bg-0);
    clip-path: polygon(var(--cut-size) 0, 100% 0, 100% calc(100% - var(--cut-size)), calc(100% - var(--cut-size)) 100%, 0 100%, 0 var(--cut-size));
    border-radius: var(--radius);
    flex-shrink: 0;
  }
  .row.expanded .logo-wrap {
    border-color: var(--accent);
    background: color-mix(in oklab, var(--accent) 15%, var(--bg-0));
  }
  .logo-wrap { cursor: pointer; }
  .logo-wrap:hover {
    border-color: var(--accent);
    background: color-mix(in oklab, var(--accent) 22%, var(--bg-0));
  }
  .logo {
    color: var(--accent);
    line-height: 0;
  }
  .mini-pct {
    font-family: var(--mono);
    font-size: 10px;
    font-weight: 700;
    letter-spacing: 0.04em;
    padding: 1px 5px;
    border: 1px solid;
    margin-left: 4px;
  }
  .mini-pct[data-status="ok"]     { color: var(--ok);     border-color: color-mix(in oklab, var(--ok) 50%, transparent);     background: color-mix(in oklab, var(--ok) 10%, var(--bg-0)); }
  .mini-pct[data-status="warn"]   { color: var(--warn);   border-color: color-mix(in oklab, var(--warn) 50%, transparent);   background: color-mix(in oklab, var(--warn) 10%, var(--bg-0)); }
  .mini-pct[data-status="danger"] { color: var(--danger); border-color: color-mix(in oklab, var(--danger) 50%, transparent); background: color-mix(in oklab, var(--danger) 10%, var(--bg-0)); }
  .name {
    font-family: var(--mono);
    font-weight: 600;
    color: var(--text-0);
    font-size: 11px;
    letter-spacing: 0.14em;
  }
  .row.expanded .name { color: var(--accent); }
  .meta {
    margin-left: auto;
    display: flex;
    gap: 4px;
  }
  .tag {
    font-family: var(--mono);
    font-size: 8px;
    font-weight: 600;
    letter-spacing: 0.14em;
    color: var(--text-2);
    padding: 2px 5px;
    border: 1px solid var(--line-strong);
    background: var(--bg-0);
  }
  .tag-plan {
    color: var(--accent-2);
    border-color: color-mix(in oklab, var(--accent-2) 50%, var(--line-strong));
  }
  .chev {
    font-size: 8px;
    color: var(--text-3);
    line-height: 1;
    width: 10px;
    text-align: center;
  }
  .chev.open {
    color: var(--accent);
  }

  .body {
    padding: 4px 14px 12px 32px;
  }
  .choice {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 6px;
  }
  .choice-btn {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 10px;
    border: 1px solid var(--line-strong);
    background: var(--bg-0);
    cursor: pointer;
    text-align: left;
    clip-path: polygon(var(--cut-size) 0, 100% 0, 100% calc(100% - var(--cut-size)), calc(100% - var(--cut-size)) 100%, 0 100%, 0 var(--cut-size));
    border-radius: var(--radius);
  }
  .choice-btn:hover:not(.disabled) {
    border-color: var(--accent);
    background: color-mix(in oklab, var(--accent) 10%, var(--bg-0));
  }
  .choice-btn.disabled {
    opacity: 0.3;
    cursor: not-allowed;
  }
  .ck-glyph {
    font-family: var(--mono);
    color: var(--accent);
    font-size: 14px;
    font-weight: 600;
  }
  .ck-stack {
    display: flex;
    flex-direction: column;
    gap: 1px;
  }
  .ck-label {
    font-family: var(--mono);
    font-size: 11px;
    font-weight: 700;
    letter-spacing: 0.16em;
    color: var(--text-0);
  }
  .ck-sub {
    font-family: var(--mono);
    font-size: 8px;
    letter-spacing: 0.1em;
    color: var(--text-3);
  }

  .bars-head {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 6px;
  }
  .back {
    font-family: var(--mono);
    font-size: 10px;
    font-weight: 600;
    letter-spacing: 0.14em;
    color: var(--text-1);
    cursor: pointer;
    padding: 2px 0;
  }
  .back:hover { color: var(--accent); }
  .bars-info {
    display: inline-flex;
    align-items: center;
    gap: 5px;
  }
  .plan-tag {
    font-family: var(--mono);
    font-size: 9px;
    letter-spacing: var(--tracking);
    color: var(--accent);
    padding: 1px 6px;
    border: 1px solid color-mix(in oklab, var(--accent) 50%, transparent);
    background: color-mix(in oklab, var(--accent) 10%, var(--bg-0));
  }
  .live {
    font-family: var(--mono);
    font-size: 9px;
    letter-spacing: 0.18em;
    color: var(--ok);
    padding: 1px 6px;
    border: 1px solid color-mix(in oklab, var(--ok) 50%, transparent);
    background: color-mix(in oklab, var(--ok) 10%, var(--bg-0));
  }
  .loading {
    text-align: center;
    padding: 12px 0;
    font-family: var(--mono);
    font-size: 10px;
    color: var(--text-2);
    letter-spacing: 0.22em;
  }
  .err {
    font-family: var(--mono);
    font-size: 10px;
    color: var(--danger);
    padding: 6px 0;
    letter-spacing: 0.1em;
  }
</style>
