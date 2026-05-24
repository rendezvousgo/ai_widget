import { invoke } from "@tauri-apps/api/core";
import { openUrl } from "@tauri-apps/plugin-opener";
import type { QuotaPayload } from "./types";
import { mockQuota } from "./mockData";

type OAuthUsageRaw = {
  items: Array<{ label: string; utilization: number; resets_at: string; reset_label: string; }>;
  fetched_at: number;
  cached: boolean;
  plan: string;
  plan_raw: string;
  renews_at_utc: string | null;
  renews_at_kst: string | null;
  renews_days: number | null;
};

type ProbeRaw = {
  provider: string;
  valid: boolean;
  status: string;
  detail: string;
  items: Array<{ label: string; used_pct: number; raw: string; reset_label: string; }>;
  fetched_at: number;
  cached: boolean;
};

export type KeyStatus = { provider: string; set: boolean; masked: string };

export async function listKeys(): Promise<KeyStatus[]> {
  try { return await invoke<KeyStatus[]>("list_api_keys"); } catch { return []; }
}
export async function setKey(provider: string, key: string): Promise<void> {
  await invoke("set_api_key", { provider, key });
}

async function fetchClaudePlan(force = false): Promise<QuotaPayload> {
  try {
    const data = await invoke<OAuthUsageRaw>("get_oauth_usage", { force });
    if (!data.items?.length) {
      return { kind: "plan", fetchedAt: Date.now(), segments: [], error: "no usage data", plan: data.plan };
    }
    const planLabel = data.cached ? `${data.plan} · cached` : data.plan;
    return {
      kind: "plan",
      fetchedAt: data.fetched_at * 1000,
      plan: planLabel,
      renewsUtc: data.renews_at_utc,
      renewsKst: data.renews_at_kst,
      renewsDays: data.renews_days,
      segments: data.items.map((it) => ({
        label: it.label, used: it.utilization, limit: 100, unit: "%", resetAt: it.reset_label,
      })),
    };
  } catch (e) {
    return { kind: "plan", fetchedAt: Date.now(), segments: [], error: typeof e === "string" ? e : (e as Error)?.message ?? "fetch failed" };
  }
}

async function fetchProviderProbe(providerId: string, force = false): Promise<QuotaPayload> {
  try {
    const data = await invoke<ProbeRaw>("probe_provider", { provider: providerId, force });
    if (!data.valid) {
      return {
        kind: "api",
        fetchedAt: data.fetched_at * 1000,
        segments: [],
        error: data.status === "missing-key" ? "no API key — open settings" : `${data.status}: ${data.detail}`,
        plan: undefined,
      };
    }
    if (data.items.length === 0) {
      return {
        kind: "api",
        fetchedAt: data.fetched_at * 1000,
        plan: data.cached ? "connected · cached" : "connected",
        segments: [{ label: "status", used: 0, limit: 100, unit: "", resetAt: data.detail }],
      };
    }
    return {
      kind: "api",
      fetchedAt: data.fetched_at * 1000,
      plan: data.cached ? `${data.detail} · cached` : data.detail,
      segments: data.items.map((it) => ({
        label: it.label, used: it.used_pct, limit: 100, unit: "%", resetAt: it.reset_label,
      })),
    };
  } catch (e) {
    return { kind: "api", fetchedAt: Date.now(), segments: [], error: typeof e === "string" ? e : (e as Error)?.message ?? "fetch failed" };
  }
}

type ChatGptUsageRaw = {
  items: Array<{ label: string; utilization: number; resets_at: string; reset_label: string; }>;
  fetched_at: number;
  cached: boolean;
  plan: string;
  renews_at_utc: string | null;
  renews_at_kst: string | null;
  renews_days: number | null;
};

async function fetchChatGptPlan(force = false): Promise<QuotaPayload> {
  try {
    const data = await invoke<ChatGptUsageRaw>("get_chatgpt_usage", { force });
    if (!data.items?.length) {
      return { kind: "plan", fetchedAt: Date.now(), segments: [], error: "no usage data", plan: data.plan };
    }
    const planLabel = data.cached ? `${data.plan} · cached` : data.plan;
    return {
      kind: "plan",
      fetchedAt: data.fetched_at * 1000,
      plan: planLabel,
      renewsUtc: data.renews_at_utc,
      renewsKst: data.renews_at_kst,
      renewsDays: data.renews_days,
      segments: data.items.map((it) => ({
        label: it.label, used: it.utilization, limit: 100, unit: "%", resetAt: it.reset_label,
      })),
    };
  } catch (e) {
    return { kind: "plan", fetchedAt: Date.now(), segments: [], error: typeof e === "string" ? e : (e as Error)?.message ?? "fetch failed" };
  }
}

export async function getQuota(providerId: string, kind: "api" | "plan", force = false): Promise<QuotaPayload> {
  if (providerId === "claude" && kind === "plan") return fetchClaudePlan(force);
  if (providerId === "claude" && kind === "api") return mockQuota(providerId, kind);
  if (providerId === "openai" && kind === "plan") return fetchChatGptPlan(force);
  // Other providers — use probe
  if (kind === "api") return fetchProviderProbe(providerId, force);
  return mockQuota(providerId, kind);
}

export async function openProviderUrl(providerId: string): Promise<void> {
  const urls: Record<string, string> = {
    claude: "https://claude.ai/settings/usage",
    openai: "https://chatgpt.com/codex/cloud/settings/analytics#usage",
    gemini: "https://aistudio.google.com/app/apikey",
    perplexity: "https://perplexity.ai/settings/api",
    xai: "https://console.x.ai/",
    groq: "https://console.groq.com/keys",
    mistral: "https://console.mistral.ai/usage",
    deepseek: "https://platform.deepseek.com/usage",
  };
  const url = urls[providerId];
  if (url) {
    try { await openUrl(url); } catch (e) { console.error("openUrl failed:", e); }
  }
}

export type DailyEntry = { date: string; tokens: number; messages: number };
export async function fetchClaudeDailyTokens(days = 30): Promise<DailyEntry[]> {
  try { return await invoke<DailyEntry[]>("get_claude_daily_tokens", { days }); } catch { return []; }
}

export type UsageHistoryPoint = { ts: number; five_hour: number | null; seven_day: number | null };
export async function fetchUsageHistory(days = 30): Promise<UsageHistoryPoint[]> {
  try { return await invoke<UsageHistoryPoint[]>("get_usage_history", { days }); } catch { return []; }
}

export async function fetchChatGptUsageHistory(days = 30): Promise<UsageHistoryPoint[]> {
  try { return await invoke<UsageHistoryPoint[]>("get_chatgpt_usage_history", { days }); } catch { return []; }
}

export async function fetchCodexDailyTokens(days = 30): Promise<DailyEntry[]> {
  try { return await invoke<DailyEntry[]>("get_codex_daily_tokens", { days }); } catch { return []; }
}

export type ComponentStatus = { name: string; status: string };
export type IncidentInfo = { name: string; status: string; created_at: string };
export type ServiceStatus = {
  indicator: string;
  description: string;
  components: ComponentStatus[];
  incidents: IncidentInfo[];
  fetched_at: number;
  cached: boolean;
};
export async function fetchServiceStatus(provider: string): Promise<ServiceStatus | null> {
  try { return await invoke<ServiceStatus>("get_service_status", { provider }); } catch { return null; }
}
