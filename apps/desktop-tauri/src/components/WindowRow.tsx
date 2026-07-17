import { remainingPercent, resetLabel, windowLabel } from "../lib/quota";
import type { RateLimitWindow } from "../types";

interface WindowRowProps {
  window: RateLimitWindow;
}

export function WindowRow({ window }: WindowRowProps) {
  const remaining = remainingPercent(window);

  return (
    <div className="window-row">
      <div className="window-row__label">
        <strong>{windowLabel(window.windowDurationMins)}</strong>
        <span>{resetLabel(window.resetsAt)}</span>
      </div>
      <div className="window-row__meter" aria-hidden="true">
        <span style={{ width: `${remaining}%` }} />
      </div>
      <output>{Math.round(remaining)}%</output>
    </div>
  );
}
