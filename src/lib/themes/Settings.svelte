<script lang="ts">
  import { onMount } from "svelte";
  import { listKeys, setKey, type KeyStatus } from "../realData";

  let { onClose, tz, onSetTz }: { onClose: () => void; tz: "utc" | "kst"; onSetTz: (v: "utc" | "kst") => void } = $props();

  let statuses = $state<KeyStatus[]>([]);
  let drafts = $state<Record<string, string>>({});
  let saving = $state<string | null>(null);

  const PROVIDER_LABELS: Record<string, string> = {
    openai: "OpenAI",
    gemini: "Google Gemini",
    perplexity: "Perplexity",
    xai: "xAI Grok",
    groq: "Groq",
    mistral: "Mistral",
    deepseek: "DeepSeek",
  };
  const PROVIDER_HINT: Record<string, string> = {
    openai: "starts with sk- · platform.openai.com",
    gemini: "AI Studio key · aistudio.google.com",
    perplexity: "pplx- key · perplexity.ai",
    xai: "xai- key · console.x.ai",
    groq: "gsk_ key · console.groq.com",
    mistral: "console.mistral.ai",
    deepseek: "platform.deepseek.com",
  };

  async function refresh() {
    statuses = await listKeys();
  }
  async function save(provider: string) {
    saving = provider;
    try {
      await setKey(provider, drafts[provider] ?? "");
      drafts[provider] = "";
      await refresh();
    } finally { saving = null; }
  }
  async function clear(provider: string) {
    saving = provider;
    try {
      await setKey(provider, "");
      drafts[provider] = "";
      await refresh();
    } finally { saving = null; }
  }
  onMount(refresh);
</script>

<div class="overlay">
  <div class="dialog">
    <header>
      <h2>API Keys</h2>
      <p>Stored locally at <code>~/.config/ai-quota-widget/keys.json</code></p>
      <button class="close" onclick={onClose} aria-label="Close">×</button>
    </header>
    <div class="list">
      {#each statuses as s (s.provider)}
        <div class="row">
          <div class="row-l">
            <div class="name">{PROVIDER_LABELS[s.provider] ?? s.provider}</div>
            <div class="hint">{PROVIDER_HINT[s.provider] ?? ""}</div>
            {#if s.set}<div class="cur">stored: {s.masked}</div>{/if}
          </div>
          <div class="row-r">
            <input
              type="password"
              placeholder={s.set ? "replace…" : "paste key"}
              bind:value={drafts[s.provider]}
              onkeydown={(e) => { if (e.key === "Enter") save(s.provider); }}
            />
            <button onclick={() => save(s.provider)} disabled={saving === s.provider}>
              {saving === s.provider ? "…" : "Save"}
            </button>
            {#if s.set}
              <button class="clear" onclick={() => clear(s.provider)}>Clear</button>
            {/if}
          </div>
        </div>
      {/each}
    </div>
    <div class="tz-row">
      <span class="tz-label">Timezone</span>
      <button class="tz-btn" class:active={tz === "kst"} onclick={() => onSetTz("kst")}>KST</button>
      <button class="tz-btn" class:active={tz === "utc"} onclick={() => onSetTz("utc")}>UTC</button>
    </div>
    <footer>
      <button class="done" onclick={onClose}>Done</button>
    </footer>
  </div>
</div>

<style>
  .overlay {
    position: absolute; inset: 0;
    background: rgba(0,0,0,0.45);
    z-index: 50;
    display: grid; place-items: center;
    backdrop-filter: blur(2px);
  }
  .dialog {
    width: min(560px, 92%);
    max-height: 92%;
    background: #f7f5ef;
    border: 1px solid #d8d3c5;
    color: #1a1a1a;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }
  header {
    padding: 14px 16px 10px 16px;
    border-bottom: 1px solid #d8d3c5;
    position: relative;
  }
  header h2 { font-size: 15px; font-weight: 500; letter-spacing: -0.01em; margin: 0 0 2px 0; }
  header p { margin: 0; font-size: 10px; color: #777; }
  header p code { font-family: "JetBrains Mono", monospace; font-size: 9px; background: #ece6d5; padding: 1px 4px; }
  .close {
    position: absolute; top: 12px; right: 12px;
    width: 24px; height: 24px;
    background: none; border: none;
    font-size: 18px; color: #999;
    cursor: pointer;
    border-radius: 50%;
  }
  .close:hover { background: #ece6d5; color: #1a1a1a; }
  .list {
    overflow-y: auto;
    padding: 6px 0;
  }
  .list::-webkit-scrollbar { width: 4px; }
  .list::-webkit-scrollbar-thumb { background: #d8d3c5; }
  .row {
    display: grid;
    grid-template-columns: 1.2fr 2fr;
    gap: 12px;
    padding: 12px 16px;
    border-bottom: 1px solid #ece6d5;
  }
  .row:last-child { border-bottom: none; }
  .name { font-size: 13px; font-weight: 500; }
  .hint { font-size: 10px; color: #999; padding-top: 1px; }
  .cur { font-size: 9px; color: #aaa; padding-top: 4px; font-family: "JetBrains Mono", monospace; }
  .row-r { display: flex; gap: 5px; align-items: center; }
  .row-r input {
    flex: 1; min-width: 0;
    padding: 6px 9px;
    border: 1px solid #d8d3c5;
    background: #fff;
    color: #1a1a1a;
    font-size: 11px;
    font-family: "JetBrains Mono", monospace;
    outline: none;
  }
  .row-r input:focus { border-color: #1a1a1a; }
  .row-r button {
    background: #1a1a1a;
    color: #f7f5ef;
    padding: 6px 12px;
    border: none;
    font-size: 11px;
    font-weight: 500;
    cursor: pointer;
    font-family: inherit;
  }
  .row-r button:hover:not(:disabled) { background: #444; }
  .row-r button:disabled { opacity: 0.4; cursor: not-allowed; }
  .row-r .clear { background: none; color: #999; padding: 6px 8px; }
  .row-r .clear:hover:not(:disabled) { color: #cc3333; background: none; }
  footer {
    padding: 12px 16px;
    border-top: 1px solid #d8d3c5;
    display: flex; justify-content: flex-end;
  }
  .done {
    background: #1a1a1a;
    color: #f7f5ef;
    padding: 7px 18px;
    border: none;
    font-size: 11px;
    font-weight: 500;
    cursor: pointer;
    font-family: inherit;
    letter-spacing: 0.04em;
  }
  .done:hover { background: #444; }
  .tz-row { display: flex; align-items: center; gap: 8px; padding: 10px 16px; border-top: 1px solid #d8d3c5; }
  .tz-label { font-size: 12px; color: #666; flex: 1; }
  .tz-btn { font-size: 11px; padding: 4px 12px; border: 1px solid #d8d3c5; background: #fff; color: #666; cursor: pointer; font-family: inherit; }
  .tz-btn.active { background: #1a1a1a; color: #f7f5ef; border-color: #1a1a1a; }
</style>
