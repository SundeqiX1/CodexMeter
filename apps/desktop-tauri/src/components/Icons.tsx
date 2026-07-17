interface IconProps {
  size?: number;
}

export function RefreshIcon({ size = 16 }: IconProps) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" fill="none" aria-hidden="true">
      <path d="M20 7v5h-5" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round" />
      <path d="M18.1 16.2a8 8 0 1 1 .4-8.9L20 12" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round" />
    </svg>
  );
}

export function LinkIcon({ size = 16 }: IconProps) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" fill="none" aria-hidden="true">
      <path d="M10.2 13.8a4.2 4.2 0 0 0 5.9 0l2.2-2.2a4.2 4.2 0 0 0-5.9-5.9l-1.3 1.3" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" />
      <path d="M13.8 10.2a4.2 4.2 0 0 0-5.9 0l-2.2 2.2a4.2 4.2 0 0 0 5.9 5.9l1.3-1.3" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" />
    </svg>
  );
}

export function CloseIcon({ size = 15 }: IconProps) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" fill="none" aria-hidden="true">
      <path d="m7 7 10 10M17 7 7 17" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" />
    </svg>
  );
}
