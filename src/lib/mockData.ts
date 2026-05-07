import type { QuotaPayload } from "./types";

export function mockQuota(providerId: string, kind: "api" | "plan"): Promise<QuotaPayload> {
  return new Promise((resolve) => {
    setTimeout(() => {
      if (providerId === "claude" && kind === "plan") {
        resolve({
          kind: "plan",
          fetchedAt: Date.now(),
          segments: [
            { label: "5h window", used: 142_300, limit: 220_000, unit: "tok", resetAt: "in 2h 14m" },
            { label: "Weekly", used: 3_412_900, limit: 6_000_000, unit: "tok", resetAt: "in 4d 6h" },
          ],
        });
      } else if (providerId === "claude" && kind === "api") {
        resolve({
          kind: "api",
          fetchedAt: Date.now(),
          segments: [
            { label: "Credits", used: 12.47, limit: 50, unit: "USD" },
            { label: "Rate (1m)", used: 38, limit: 100, unit: "rpm" },
          ],
        });
      } else if (providerId === "openai" && kind === "api") {
        resolve({
          kind: "api",
          fetchedAt: Date.now(),
          segments: [
            { label: "Credits", used: 8.20, limit: 25, unit: "USD" },
            { label: "TPM", used: 145_000, limit: 500_000, unit: "tok/m" },
          ],
        });
      } else if (providerId === "gemini" && kind === "api") {
        resolve({
          kind: "api",
          fetchedAt: Date.now(),
          segments: [
            { label: "Daily", used: 312, limit: 1500, unit: "req" },
          ],
        });
      } else {
        resolve({
          kind,
          fetchedAt: Date.now(),
          segments: [],
          error: "Not available for this provider",
        });
      }
    }, 220);
  });
}
