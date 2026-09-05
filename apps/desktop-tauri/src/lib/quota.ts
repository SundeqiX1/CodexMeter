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
