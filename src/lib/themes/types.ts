import type { Provider, QuotaPayload } from "../types";

export type ThemeProps = {
  providers: Provider[];
  usageCache: Record<string, QuotaPayload | null>;
  pinned: boolean;
  autostart: boolean;
  refreshing: boolean;
  lastRefresh: Date | null;
  now: Date;
  themeLabel: string;
  themeIdx: number;
  themesCount: number;
  onTogglePin: () => void;
  onClose: () => void;
  onMinimize: () => void;
  onRefresh: () => void;
  onToggleAutostart: () => void;
  onCycleTheme: () => void;
  onToggleDark: () => void;
  isDark: boolean;
  lang: "en" | "ko";
  onToggleLang: () => void;
  onOpenProvider: (id: string) => void;
  onLoadSegment: (id: string, kind: "api" | "plan") => Promise<QuotaPayload | null>;
};
