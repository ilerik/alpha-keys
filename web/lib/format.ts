export function usdFromE6(value: number): string {
  const sign = value < 0 ? "-" : "";
  const abs = Math.abs(value);
  const whole = Math.floor(abs / 1_000_000);
  const frac = String(abs % 1_000_000).padStart(6, "0").replace(/0+$/, "");
  return frac ? `${sign}${whole}.${frac}` : `${sign}${whole}`;
}

export function accessMessage(status: number): string {
  if (status === 401) return "Sign in to open the feed.";
  if (status === 403) return "403 no viewing key";
  if (status === 200) return "Trades are streaming.";
  return `Feed error ${status}`;
}
