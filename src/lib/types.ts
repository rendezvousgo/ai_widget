export type QuotaKind = "api" | "plan";

export type QuotaSegment = {
  label: string;
  used: number;
  limit: number;
  unit?: string;
  resetAt?: string;
};

export type QuotaPayload = {
  kind: QuotaKind;
  segments: QuotaSegment[];
  fetchedAt: number;
  error?: string;
  plan?: string;
};

export type Provider = {
  id: string;
  name: string;
  accent: string;
  glow: string;
  available: { api: boolean; plan: boolean };
};
