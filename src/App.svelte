<script lang="ts">
  import { onMount } from "svelte";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { LogicalPosition, LogicalSize } from "@tauri-apps/api/dpi";
  import {
    enable as enableAutostart,
    disable as disableAutostart,
    isEnabled as isAutostartEnabled,
  } from "@tauri-apps/plugin-autostart";
  import { PROVIDERS } from "./lib/providers";
  import { getQuota, openProviderUrl } from "./lib/realData";
  import type { QuotaPayload } from "./lib/types";

  import Minimal from "./lib/themes/Minimal.svelte";
  import Dark from "./lib/themes/Dark.svelte";
  import Login from "./lib/Login.svelte";
  import { invoke } from "@tauri-apps/api/core";
  // Other themes archived (still on disk under src/lib/themes/) — uncomment to re-enable.
  // import Cyber from "./lib/themes/Cyber.svelte";
  // import Terminal from "./lib/themes/Terminal.svelte";
  // import Neon from "./lib/themes/Neon.svelte";
  // import Hud from "./lib/themes/Hud.svelte";

  const win = getCurrentWindow();
  const RATIO = 360 / 220;
  const MIN_W = 320;
  const AUTO_REFRESH_MS = 45 * 1000;

  const THEMES = ["minimal", "dark"] as const;
  type Theme = (typeof THEMES)[number];
  const THEME_LABELS: Record<Theme, string> = {
    minimal: "MINIMAL",
    dark: "DARK",
  };

  let pinned = $state(false);
  let autostart = $state(false);
  let now = $state(new Date());
  let lastRefresh = $state<Date | null>(null);
  let refreshing = $state(false);
  let usageCache = $state<Record<string, QuotaPayload | null>>({});
  let theme = $state<Theme>("minimal");
  let lang = $state<"en" | "ko">("en");
  let loggedIn = $state<boolean | null>(null); // null = not yet checked

  async function checkLogin() {
    try {
      const s = await invoke<{ logged_in: boolean }>("claude_login_status");
      loggedIn = s.logged_in;
    } catch {
      loggedIn = false;
    }
  }

  function toggleLang() {
    lang = lang === "en" ? "ko" : "en";
    try { localStorage.setItem("lang", lang); } catch {}
  }

  function cycleTheme() {
    const i = THEMES.indexOf(theme);
    theme = THEMES[(i + 1) % THEMES.length];
    try { localStorage.setItem("theme", theme); } catch {}
  }
  function toggleDark() {
    if (theme === "minimal") theme = "dark";
    else if (theme === "dark") theme = "minimal";
    else theme = "dark";
    try { localStorage.setItem("theme", theme); } catch {}
  }

  async function refreshClaude(force = false) {
    refreshing = true;
    try {
      const result = await getQuota("claude", "plan", force);
      usageCache = { ...usageCache, claude: result };
      lastRefresh = new Date();
    } finally {
      refreshing = false;
    }
  }

  async function loadSegment(id: string, kind: "api" | "plan"): Promise<QuotaPayload | null> {
    const result = await getQuota(id, kind);
    if (id === "claude" && kind === "plan") {
      usageCache = { ...usageCache, claude: result };
    }
    return result;
  }

  onMount(() => {
    try {
      const saved = localStorage.getItem("theme") as Theme | null;
      if (saved && THEMES.includes(saved)) theme = saved;
      const savedLang = localStorage.getItem("lang") as "en" | "ko" | null;
      if (savedLang === "en" || savedLang === "ko") lang = savedLang;
    } catch {}
    checkLogin();
    const tickClock = setInterval(() => (now = new Date()), 1000);
    refreshClaude(false);
    const tickRefresh = setInterval(() => refreshClaude(false), AUTO_REFRESH_MS);
    isAutostartEnabled().then((v) => (autostart = v)).catch(() => {});
    return () => {
      clearInterval(tickClock);
      clearInterval(tickRefresh);
    };
  });

  function togglePin() {
    pinned = !pinned;
    win.setAlwaysOnTop(pinned).catch(() => {});
  }
  function close() {
    win.close().catch(() => {});
  }
  async function toggleAutostart() {
    try {
      if (autostart) { await disableAutostart(); autostart = false; }
      else { await enableAutostart(); autostart = true; }
    } catch (e) { console.error(e); }
  }

  type Dir = "E" | "W" | "N" | "S" | "NE" | "NW" | "SE" | "SW";

  function startResize(dir: Dir) {
    return async (e: PointerEvent) => {
      if (e.button !== 0) return;
      e.preventDefault();
      e.stopPropagation();
      (e.target as HTMLElement).setPointerCapture(e.pointerId);

      const factor = await win.scaleFactor();
      const initSize = (await win.outerSize()).toLogical(factor);
      const initPos = (await win.outerPosition()).toLogical(factor);
      const startX = e.screenX;
      const startY = e.screenY;

      let raf: number | null = null;
      let pending: { w: number; h: number; x: number; y: number } | null = null;
      const east = dir.includes("E");
      const west = dir.includes("W");
      const north = dir.includes("N");
      const south = dir.includes("S");
      const horiz = east || west;
      const vert = north || south;
      const target = e.target as HTMLElement;

      const onMove = (ev: PointerEvent) => {
        const dx = ev.screenX - startX;
        const dy = ev.screenY - startY;
        let w = initSize.width, h = initSize.height, x = initPos.x, y = initPos.y;
        if (east) w = initSize.width + dx;
        if (west) { w = initSize.width - dx; x = initPos.x + dx; }
        if (south) h = initSize.height + dy;
        if (north) { h = initSize.height - dy; y = initPos.y + dy; }
        if (horiz && !vert) { h = w / RATIO; }
        else if (vert && !horiz) { w = h * RATIO; }
        else {
          if (Math.abs(dx) >= Math.abs(dy)) {
            const newH = w / RATIO;
            if (north) y = initPos.y + (initSize.height - newH);
            h = newH;
          } else {
            const newW = h * RATIO;
            if (west) x = initPos.x + (initSize.width - newW);
            w = newW;
          }
        }
        if (w < MIN_W) {
          const adjW = MIN_W, adjH = MIN_W / RATIO;
          if (west) x = initPos.x + (initSize.width - adjW);
          if (north) y = initPos.y + (initSize.height - adjH);
          w = adjW; h = adjH;
        }
        pending = { w, h, x, y };
        if (raf == null) {
          raf = requestAnimationFrame(async () => {
            raf = null;
            if (!pending) return;
            const p = pending; pending = null;
            if (west || north) await win.setPosition(new LogicalPosition(p.x, p.y)).catch(() => {});
            await win.setSize(new LogicalSize(p.w, p.h)).catch(() => {});
          });
        }
      };
      const cleanup = () => {
        target.removeEventListener("pointermove", onMove);
        target.removeEventListener("pointerup", cleanup);
        target.removeEventListener("pointercancel", cleanup);
        try { target.releasePointerCapture(e.pointerId); } catch {}
        if (raf != null) cancelAnimationFrame(raf);
      };
      target.addEventListener("pointermove", onMove);
      target.addEventListener("pointerup", cleanup);
      target.addEventListener("pointercancel", cleanup);
    };
  }

  const themeIdx = $derived(THEMES.indexOf(theme));
  const themeLabel = $derived(THEME_LABELS[theme]);

  const themeProps = $derived({
    providers: PROVIDERS,
    usageCache,
    pinned,
    autostart,
    refreshing,
    lastRefresh,
    now,
    themeLabel,
    themeIdx,
    themesCount: THEMES.length,
    onTogglePin: togglePin,
    onClose: close,
    onRefresh: () => refreshClaude(true),
    onToggleAutostart: toggleAutostart,
    onCycleTheme: cycleTheme,
    onToggleDark: toggleDark,
    isDark: theme === "dark",
    lang,
    onToggleLang: toggleLang,
    onOpenProvider: openProviderUrl,
    onLoadSegment: loadSegment,
  });
</script>

<main class="widget" data-theme={theme}>
  {#if theme === "dark"}<Dark {...themeProps} />
  {:else}<Minimal {...themeProps} />
  {/if}

  {#if loggedIn === false}
    <Login {lang} isDark={theme === "dark"} onLoggedIn={() => { loggedIn = true; refreshClaude(true); }} />
  {/if}
</main>

<style>
  .widget {
    position: relative;
    width: 100vw;
    height: 100vh;
    background: var(--bg-0);
  }
  .rz { position: absolute; z-index: 99; }
  .rz-n  { top: 0;    left: 14px;  right: 14px;  height: 6px;  cursor: ns-resize; }
  .rz-s  { bottom: 0; left: 14px;  right: 14px;  height: 6px;  cursor: ns-resize; }
  .rz-e  { right: 0;  top: 14px;   bottom: 14px; width: 6px;   cursor: ew-resize; }
  .rz-w  { left: 0;   top: 14px;   bottom: 14px; width: 6px;   cursor: ew-resize; }
  .rz-nw { top: 0;    left: 0;     width: 14px;  height: 14px; cursor: nwse-resize; }
  .rz-ne { top: 0;    right: 0;    width: 14px;  height: 14px; cursor: nesw-resize; }
  .rz-sw { bottom: 0; left: 0;     width: 14px;  height: 14px; cursor: nesw-resize; }
  .rz-se { bottom: 0; right: 0;    width: 14px;  height: 14px; cursor: nwse-resize; }
</style>
