import type { RateLimitSnapshot, RateLimitWindow, RateLimitsEnvelope } from "../types";

export const remainingPercent = (window: RateLimitWindow): number =>
  Math.min(100, Math.max(0, 100 - window.usedPercent));

export const windowLabel = (minutes?: number | null): string => {
  if (minutes == null) return "额度窗口";
  if (minutes === 60) return "每小时";
  if (minutes === 300) return "5 小时";
  if (minutes === 1_440) return "每日";
  if (minutes === 10_080) return "每周";
  if (minutes === 43_200 || minutes === 44_640) return "每月";
  if (minutes % 1_440 === 0) return `${minutes / 1_440} 天`;
  if (minutes % 60 === 0) return `${minutes / 60} 小时`;
  return `${minutes} 分钟`;
};

export const resetLabel = (timestamp?: number | null): string => {
  if (timestamp == null) return "重置时间未知";
  return new Intl.DateTimeFormat("zh-CN", {
    month: "short",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  }).format(new Date(timestamp * 1_000));
};

export const orderedLimits = (envelope?: RateLimitsEnvelope | null): RateLimitSnapshot[] => {
  if (!envelope) return [];
  const mapped = Object.values(envelope.rateLimitsByLimitId ?? {});
  if (mapped.length === 0) return [envelope.rateLimits];
  return mapped.sort((left, right) => {
    if (left.limitId === "codex") return -1;
    if (right.limitId === "codex") return 1;
    return displayName(left).localeCompare(displayName(right), "zh-CN");
  });
};

export const displayName = (limit: RateLimitSnapshot): string =>
  limit.limitName?.trim() || (limit.limitId === "codex" || !limit.limitId ? "Codex" : limit.limitId);

export const windowsFor = (limit?: RateLimitSnapshot | null): RateLimitWindow[] =>
  [limit?.primary, limit?.secondary]
    .filter((window): window is RateLimitWindow => Boolean(window))
    .sort(
      (left, right) =>
        (left.windowDurationMins ?? Number.MAX_SAFE_INTEGER) -
        (right.windowDurationMins ?? Number.MAX_SAFE_INTEGER),
    );

export const criticalWindow = (envelope?: RateLimitsEnvelope | null): RateLimitWindow | null => {
  let critical: RateLimitWindow | null = null;
  for (const limit of orderedLimits(envelope)) {
    for (const window of windowsFor(limit)) {
      if (!critical || remainingPercent(window) < remainingPercent(critical)) critical = window;
    }
  }
  return critical;
};

export const planName = (plan?: string | null): string | null => {
  if (!plan) return null;
  const names: Record<string, string> = {
    free: "Free",
    go: "Go",
    plus: "Plus",
    pro: "Pro",
    prolite: "Pro Lite",
    team: "Team",
    business: "Business",
    self_serve_business_usage_based: "Business",
    enterprise: "Enterprise",
    enterprise_cbp_usage_based: "Enterprise",
    edu: "Edu",
  };
  return names[plan] ?? plan;
};
