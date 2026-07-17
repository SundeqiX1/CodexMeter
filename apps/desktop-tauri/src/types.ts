export type ConnectionStatus = "disconnected" | "connecting" | "connected" | "failed";

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

export interface CreditsSnapshot {
  hasCredits: boolean;
  unlimited: boolean;
  balance?: string | null;
}

export interface SpendControlLimitSnapshot {
  limit: string;
  used: string;
  remainingPercent: number;
  resetsAt: number;
}

export interface RateLimitSnapshot {
  limitId?: string | null;
  limitName?: string | null;
  primary?: RateLimitWindow | null;
  secondary?: RateLimitWindow | null;
  credits?: CreditsSnapshot | null;
  individualLimit?: SpendControlLimitSnapshot | null;
  spendControlReached?: boolean | null;
  planType?: string | null;
  rateLimitReachedType?: string | null;
}

export interface RateLimitResetCredit {
  id: string;
  resetType: string;
  status: string;
  grantedAt: number;
  expiresAt?: number | null;
  title?: string | null;
  description?: string | null;
}

export interface RateLimitResetCreditsSummary {
  availableCount: number;
  credits?: RateLimitResetCredit[] | null;
}

export interface RateLimitsEnvelope {
  rateLimits: RateLimitSnapshot;
  rateLimitsByLimitId?: Record<string, RateLimitSnapshot> | null;
  rateLimitResetCredits?: RateLimitResetCreditsSummary | null;
}

export interface FrontendState {
  connection: ConnectionState;
  snapshot?: RateLimitsEnvelope | null;
  lastUpdated?: number | null;
  platform: string;
}
