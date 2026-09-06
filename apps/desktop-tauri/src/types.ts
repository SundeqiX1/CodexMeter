export type ConnectionStatus = "disconnected" | "connecting" | "connected" | "stale" | "failed";

export interface ConnectionState {
  status: ConnectionStatus;
  message?: string | null;
  executable?: string | null;
}

export interface RateLimitWindow {
  usedPercent: number;
  windowDurationMins?: number | null;
  resetsAt?: number | null;
}

export interface RateLimitSnapshot {
  planType?: string | null;
  primary?: RateLimitWindow | null;
  secondary?: RateLimitWindow | null;
}

export interface RateLimitResetCreditsSummary {
  availableCount: number;
}

export interface RateLimitsEnvelope {
  rateLimits?: RateLimitSnapshot | null;
  rateLimitsByLimitId?: Record<string, RateLimitSnapshot> | null;
  rateLimitResetCredits?: RateLimitResetCreditsSummary | null;
}

export interface SavedPosition {
  x: number;
  y: number;
}

export type AppLanguage = "system" | "en" | "zh-CN";
export type ResolvedLanguage = Exclude<AppLanguage, "system">;

export interface AppSettings {
  language: AppLanguage;
  refreshIntervalSecs: number;
  compactMenuBar: boolean;
  hideMissingWindows: boolean;
  widgetVisible: boolean;
  widgetPosition?: SavedPosition | null;
  codexBinaryPath?: string | null;
}

export interface FrontendState {
  connection: ConnectionState;
  snapshot?: RateLimitsEnvelope | null;
  lastUpdated?: number | null;
  platform: string;
  settings: AppSettings;
}
