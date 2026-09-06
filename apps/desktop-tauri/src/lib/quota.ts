import type { RateLimitSnapshot, RateLimitWindow, RateLimitsEnvelope } from "../types";

export const remainingPercent = (window: RateLimitWindow): number =>
  Math.min(100, Math.max(0, 100 - window.usedPercent));

export const resetLabel = (timestamp?: number | null, locale?: string, unknownLabel = "Unknown"): string => {
  if (timestamp == null) return unknownLabel;
  return new Intl.DateTimeFormat(locale, {
    month: "short",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  }).format(new Date(timestamp * 1_000));
};

export const codexLimit = (envelope?: RateLimitsEnvelope | null): RateLimitSnapshot | null => {
  if (!envelope) return null;
  return envelope.rateLimitsByLimitId?.codex ?? envelope.rateLimits ?? null;
};

const planLabels: Record<string, string> = {
  free: "Free",
  go: "Go",
  plus: "Plus",
  prolite: "Pro 5×",
  pro: "Pro 20×",
  team: "Team",
  self_serve_business_prolite: "Business · Pro 5×",
  self_serve_business_usage_based: "Business · Usage-based",
  business: "Business",
  ent26: "Enterprise",
  enterprise_cbp_automation: "Enterprise",
  enterprise_cbp_usage_based: "Enterprise · Usage-based",
  enterprise: "Enterprise",
  edu: "Edu",
  edu_plus: "Edu Plus",
  edu_pro: "Edu Pro",
};

export const subscriptionLabel = (envelope?: RateLimitsEnvelope | null): string | null => {
  const raw = codexLimit(envelope)?.planType?.trim().toLowerCase();
  if (!raw || raw === "unknown") return null;
  return planLabels[raw] ?? raw
    .split(/[_-]+/)
    .filter(Boolean)
    .map((part) => part[0]?.toUpperCase() + part.slice(1))
    .join(" ");
};

export const windowForDuration = (
  envelope: RateLimitsEnvelope | null | undefined,
  minutes: number,
): RateLimitWindow | null =>
  windowsFor(codexLimit(envelope)).find((window) => window.windowDurationMins === minutes) ?? null;

const windowsFor = (limit?: RateLimitSnapshot | null): RateLimitWindow[] =>
  [limit?.primary, limit?.secondary]
    .filter((window): window is RateLimitWindow => Boolean(window))
    .sort(
      (left, right) =>
        (left.windowDurationMins ?? Number.MAX_SAFE_INTEGER) -
        (right.windowDurationMins ?? Number.MAX_SAFE_INTEGER),
    );
