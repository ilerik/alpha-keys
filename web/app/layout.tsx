import type { ReactNode } from "react";
import Link from "next/link";
import "./globals.css";
import { WalletButton } from "./wallet";
import { LoopbackProvider } from "./loopback";

export const metadata = {
  title: "Alpha Keys",
  description: "The account stays hidden. The track record is public. The feed is a key.",
};

export default function RootLayout({ children }: { children: ReactNode }) {
  return (
    <html lang="en">
      <body>
        <div className="scan" aria-hidden="true" />
        <LoopbackProvider>
        <div className="shell">
          <header className="top">
            <Link className="mark" href="/">ALPHA KEYS</Link>
            <span className="net">testnet</span>
            <nav>
              <Link href="/">Leaderboard</Link>
              <Link href="/t/alpha-1">Alpha One</Link>
              <Link href="/feed/alpha-1">Feed</Link>
              <Link href="/verify">Verify</Link>
            </nav>
            <span className="wallet">
              <WalletButton />
            </span>
          </header>
          <main>{children}</main>
          <footer>Token 0 is a Delayed60 key on alpha-keys.testnet. Attested means an OutLayer callback whose code hash matches the pinned WASM.</footer>
        </div>
        </LoopbackProvider>
      </body>
    </html>
  );
}
