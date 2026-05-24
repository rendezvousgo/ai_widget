import type { Provider } from "./types";

export const PROVIDERS: Provider[] = [
  {
    id: "claude",
    name: "CLAUDE",
    accent: "#ff9f3f",
    glow: "#ff9f3f",
    available: { api: true, plan: true },
  },
  {
    id: "openai",
    name: "OPENAI",
    accent: "#36ffaa",
    glow: "#36ffaa",
    available: { api: false, plan: true },
  },
];
