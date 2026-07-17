import type { CSSProperties } from "react";

interface GaugeProps {
  percent: number | null;
  label: string;
  detail: string;
}

export function Gauge({ percent, label, detail }: GaugeProps) {
  const normalized = percent ?? 0;
  const style = { "--quota": normalized } as CSSProperties;

  return (
    <section className="gauge-block" aria-label={`${label}剩余额度`}>
      <div className="gauge" style={style}>
        <div className="gauge__inner">
          <span className="gauge__value">{percent == null ? "—" : `${Math.round(percent)}%`}</span>
          <span className="gauge__caption">剩余</span>
        </div>
      </div>
      <div className="gauge-copy">
        <span className="eyebrow">CURRENT WINDOW</span>
        <h2>{label}</h2>
        <p>{detail}</p>
      </div>
    </section>
  );
}
