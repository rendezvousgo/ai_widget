<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { openUrl } from "@tauri-apps/plugin-opener";
  import { t as tr, type Lang } from "./i18n";

  type LoginStatus = { logged_in: boolean; cli_installed: boolean; credentials_path: string };
  let { lang = "en" as Lang, isDark = false, onLoggedIn }: {
    lang?: Lang;
    isDark?: boolean;
    onLoggedIn: () => void;
  } = $props();

  const T = $derived(tr(lang));
  let status = $state<LoginStatus | null>(null);
  let checking = $state(false);
  let copied = $state<string>("");

  async function check() {
    checking = true;
    try {
      status = await invoke<LoginStatus>("claude_login_status");
      if (status.logged_in) onLoggedIn();
    } finally {
      checking = false;
    }
  }

  async function openInstallPage() {
    try { await openUrl("https://claude.ai/install"); } catch (e) { console.error(e); }
  }
  async function copyCmd(cmd: string) {
    try {
      await navigator.clipboard.writeText(cmd);
      copied = cmd;
      setTimeout(() => { if (copied === cmd) copied = ""; }, 1500);
    } catch (e) { console.error(e); }
  }

  $effect(() => {
    check();
    // poll every 4s while login screen is shown — auto-detect once user signs in
    const id = setInterval(check, 4000);
    return () => clearInterval(id);
  });
</script>

<div class="login" class:dark={isDark}>
  <div class="card">
    <div class="brand">Quota</div>
    <h1>{T.loginTitle}</h1>
    <p class="lead">{T.loginLead}</p>

    <ol class="steps">
      <li>
        <span class="num">1</span>
        <div class="body">
          <div class="step-h">{T.loginStep1}</div>
          {#if status && !status.cli_installed}
            <button class="btn" onclick={openInstallPage}>{T.loginInstallBtn} ↗</button>
          {:else if status && status.cli_installed}
            <div class="ok">✓ {T.loginInstalled}</div>
          {/if}
        </div>
      </li>
      <li>
        <span class="num">2</span>
        <div class="body">
          <div class="step-h">{T.loginStep2}</div>
          <div class="cmd-row">
            <span class="prompt">$</span>
            <code>claude</code>
            <button class="cmd-copy" onclick={() => copyCmd("claude")}>{copied === "claude" ? T.loginCopied : T.loginCopy}</button>
          </div>
          <div class="cmd-row">
            <span class="prompt">›</span>
            <code>/login</code>
            <button class="cmd-copy" onclick={() => copyCmd("/login")}>{copied === "/login" ? T.loginCopied : T.loginCopy}</button>
          </div>
        </div>
      </li>
      <li>
        <span class="num">3</span>
        <div class="body">
          <div class="step-h">{T.loginStep3}</div>
          <button class="btn primary" onclick={check} disabled={checking}>
            {checking ? T.loginChecking : T.loginRefresh}
          </button>
        </div>
      </li>
    </ol>

    <p class="footnote">{T.loginPlanNote}</p>
  </div>
</div>

<style>
  .login {
    position: absolute; inset: 0;
    display: flex; align-items: center; justify-content: center;
    background: #f7f5ef;
    color: #1a1a1a;
    font-family: "Inter", -apple-system, "Segoe UI", system-ui, sans-serif;
    z-index: 100;
  }
  .login.dark { background: #1d1e26; color: #ebecf0; }
  .card {
    width: 100%; max-width: 480px;
    padding: 26px 32px 24px 32px;
  }
  .brand {
    font-size: 13px; font-weight: 600; letter-spacing: -0.01em;
    color: #d97757;
    margin-bottom: 6px;
  }
  h1 {
    margin: 0 0 6px 0;
    font-size: 19px; font-weight: 500; letter-spacing: -0.015em;
    line-height: 1.3;
  }
  .lead {
    margin: 0 0 18px 0;
    font-size: 12px; color: #777; line-height: 1.5;
  }
  .login.dark .lead { color: #9ca0ac; }

  .steps { list-style: none; padding: 0; margin: 0; display: flex; flex-direction: column; gap: 12px; }
  .steps li { display: flex; gap: 12px; align-items: flex-start; }
  .num {
    flex-shrink: 0;
    width: 22px; height: 22px;
    border-radius: 50%;
    background: #ece6d5;
    color: #1a1a1a;
    font-size: 11px; font-weight: 600;
    display: inline-flex; align-items: center; justify-content: center;
    font-feature-settings: "tnum";
  }
  .login.dark .num { background: #262731; color: #ebecf0; }
  .body { flex: 1; min-width: 0; display: flex; flex-direction: column; gap: 6px; }
  .step-h { font-size: 12px; font-weight: 500; }
  .ok { font-size: 11px; color: #6b8e5a; }
  .login.dark .ok { color: #80b06a; }

  .btn {
    align-self: flex-start;
    background: transparent;
    border: 1px solid #d8d3c5;
    color: inherit;
    padding: 5px 10px;
    border-radius: 4px;
    font-family: inherit;
    font-size: 11px;
    cursor: pointer;
    line-height: 1.2;
  }
  .login.dark .btn { border-color: #34363f; }
  .btn:hover { background: #ece6d5; }
  .login.dark .btn:hover { background: #262731; }
  .btn.primary {
    background: #1a1a1a;
    color: #f7f5ef;
    border-color: #1a1a1a;
  }
  .login.dark .btn.primary { background: #ebecf0; color: #1d1e26; border-color: #ebecf0; }
  .btn.primary:hover { opacity: 0.85; }
  .btn:disabled { opacity: 0.5; cursor: default; }

  .cmd-row {
    display: inline-flex; align-items: center;
    background: #ece6d5;
    border-radius: 4px;
    padding: 3px 4px 3px 8px;
    gap: 6px;
    align-self: flex-start;
    min-width: 140px;
  }
  .login.dark .cmd-row { background: #262731; }
  .prompt {
    color: #aaa;
    font-family: "JetBrains Mono", "SF Mono", Menlo, monospace;
    font-size: 10px;
    user-select: none;
  }
  .login.dark .prompt { color: #5f6470; }
  code { flex: 1; }
  code {
    font-family: "JetBrains Mono", "SF Mono", Menlo, monospace;
    font-size: 11px;
    color: inherit;
  }
  .cmd-copy {
    background: transparent;
    border: none;
    color: #777;
    font-family: inherit;
    font-size: 9px; font-weight: 600;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    padding: 2px 6px;
    border-radius: 3px;
    cursor: pointer;
  }
  .cmd-copy:hover { background: rgba(0,0,0,0.06); color: #1a1a1a; }
  .login.dark .cmd-copy:hover { background: rgba(255,255,255,0.1); color: #ebecf0; }

  .footnote {
    margin: 18px 0 0 0;
    font-size: 10px; color: #aaa;
    line-height: 1.5;
  }
  .login.dark .footnote { color: #5f6470; }
</style>
