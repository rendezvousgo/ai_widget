export type Lang = "en" | "ko";

type Dict = {
  title: string;
  plan: string;
  nowTab: string;
  trendTab: string;
  openDashboard: string;
  noData: (name: string) => string;
  loading: string;
  loadingDaily: string;
  noDailyData: string;
  sources: (n: number) => string;
  lastSync: string;
  monitoring: string;
  refresh: string;
  settings: string;
  darkMode: string;
  lightMode: string;
  close: string;
  pin: string;
  autostart: string;
  bucket: (label: string) => string;
  peak: string;
  avg: string;
  total: string;
  msgs: string;
  perDay: string;
  tokens: string;
  apiKeysTitle: string;
  apiKeysHint: string;
  done: string;
  save: string;
  clear: string;
  saving: string;
  refreshTitle: string;
  light: string;
  dark: string;
  langLabel: string;
  loginTitle: string;
  loginLead: string;
  loginStep1: string;
  loginStep2: string;
  loginStep3: string;
  loginInstallBtn: string;
  loginInstalled: string;
  loginCopy: string;
  loginCopied: string;
  loginRefresh: string;
  loginChecking: string;
  loginPlanNote: string;
};

const en: Dict = {
  title: "Quota",
  plan: "Plan",
  nowTab: "Now",
  trendTab: "Trend",
  openDashboard: "open dashboard ↗",
  noData: (n) => `no data — click ${n} to load`,
  loading: "loading…",
  loadingDaily: "loading 30d…",
  noDailyData: "no daily data found",
  sources: (n) => `${n} sources`,
  lastSync: "last sync ·",
  monitoring: "monitoring",
  refresh: "Refresh",
  settings: "Settings",
  darkMode: "Dark mode",
  lightMode: "Light mode",
  close: "Close",
  pin: "Pin",
  autostart: "Auto-start",
  bucket: (label) => label,
  peak: "▲ Peak",
  avg: "▬ Avg",
  total: "∑ Total",
  msgs: "✉ Msgs",
  perDay: "/d",
  tokens: "tok",
  apiKeysTitle: "API Keys",
  apiKeysHint: "Stored locally at",
  done: "Done",
  save: "Save",
  clear: "Clear",
  saving: "…",
  refreshTitle: "Refresh now",
  light: "Light mode",
  dark: "Dark mode",
  langLabel: "EN",
  loginTitle: "Sign in to view Pro / Max plan usage",
  loginLead: "This widget reads usage from Claude Code's local credentials. Install the Claude Code CLI and sign in once — the widget will pick it up automatically.",
  loginStep1: "Install Claude Code CLI",
  loginStep2: "Run claude, then type /login inside",
  loginStep3: "Refresh once you're logged in",
  loginInstallBtn: "Open install page",
  loginInstalled: "Already installed",
  loginCopy: "Copy",
  loginCopied: "Copied",
  loginRefresh: "I'm signed in — refresh",
  loginChecking: "Checking…",
  loginPlanNote: "Requires a Claude Pro or Max subscription. API key only? Plan usage isn't available — only OpenAI shows quota.",
};

const KO_BUCKETS: Record<string, string> = {
  "5h window": "5시간",
  "Weekly": "주간",
  "Weekly · Sonnet": "주간 · 소넷",
  "Weekly · Opus": "주간 · 오퍼스",
  "Weekly · Design": "주간 · 디자인",
  "Weekly · OAuth Apps": "주간 · OAuth 앱",
  "Weekly · Cowork": "주간 · 협업",
};

const ko: Dict = {
  title: "잔량",
  plan: "플랜",
  nowTab: "현재",
  trendTab: "추이",
  openDashboard: "대시보드 열기 ↗",
  noData: (n) => `데이터 없음 — ${n} 클릭`,
  loading: "불러오는 중…",
  loadingDaily: "30일 로드 중…",
  noDailyData: "일별 데이터 없음",
  sources: (n) => `${n}개 소스`,
  lastSync: "마지막 갱신 ·",
  monitoring: "모니터링",
  refresh: "새로고침",
  settings: "설정",
  darkMode: "다크 모드",
  lightMode: "라이트 모드",
  close: "닫기",
  pin: "고정",
  autostart: "자동 시작",
  bucket: (label) => KO_BUCKETS[label] ?? label,
  peak: "▲ 최고",
  avg: "▬ 평균",
  total: "∑ 총합",
  msgs: "✉ 메시지",
  perDay: "/일",
  tokens: "토큰",
  apiKeysTitle: "API 키",
  apiKeysHint: "로컬 저장 위치:",
  done: "완료",
  save: "저장",
  clear: "지우기",
  saving: "…",
  refreshTitle: "지금 새로고침",
  light: "라이트 모드",
  dark: "다크 모드",
  langLabel: "KO",
  loginTitle: "Pro / Max 플랜 사용량을 보려면 로그인하세요",
  loginLead: "이 위젯은 Claude Code의 로컬 인증 정보를 읽습니다. Claude Code CLI를 설치하고 한 번만 로그인하면 위젯이 자동으로 인식합니다.",
  loginStep1: "Claude Code CLI 설치",
  loginStep2: "claude 실행 후 안에서 /login 입력",
  loginStep3: "로그인 후 새로고침",
  loginInstallBtn: "설치 페이지 열기",
  loginInstalled: "설치됨",
  loginCopy: "복사",
  loginCopied: "복사됨",
  loginRefresh: "로그인 완료 — 새로고침",
  loginChecking: "확인 중…",
  loginPlanNote: "Claude Pro 또는 Max 구독이 필요합니다. API 키만 있다면 플랜 사용량은 볼 수 없고 OpenAI 사용량만 표시됩니다.",
};

const DICTS: Record<Lang, Dict> = { en, ko };

export function t(lang: Lang): Dict {
  return DICTS[lang] ?? en;
}

const KO_PROVIDER_NAMES: Record<string, string> = {
  claude: "클로드",
  openai: "오픈AI",
  gemini: "제미나이",
  perplexity: "퍼플렉시티",
  xai: "xAI",
  groq: "그록",
  mistral: "미스트랄",
  deepseek: "딥시크",
};

export function providerName(id: string, lang: Lang, fallback: string): string {
  if (lang === "ko" && KO_PROVIDER_NAMES[id]) return KO_PROVIDER_NAMES[id];
  // pretty-case fallback: "CLAUDE" → "Claude"
  if (!fallback) return "";
  return fallback.charAt(0).toUpperCase() + fallback.slice(1).toLowerCase();
}
