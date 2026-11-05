"""copier CLI (K-2, STUB). Dry-run is the default; --live requires follower's own key from env."""
import argparse


def main() -> None:
    p = argparse.ArgumentParser(prog="copier")
    p.add_argument("--live", action="store_true", help="place real orders (follower's own account)")
    p.add_argument("--fixture", help="replay a recorded TradeEvent JSONL instead of the gateway")
    args = p.parse_args()
    mode = "LIVE" if args.live else "dry-run"
    print(f"copier stub ({mode}); see docs/TASKS.md K-1..K-6")


if __name__ == "__main__":
    main()
