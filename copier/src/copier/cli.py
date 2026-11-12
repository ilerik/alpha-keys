"""Dry-run copier. Live orders stay behind --live and are not part of the judge path."""

from __future__ import annotations

import argparse
import json
import os
import sys
from dataclasses import asdict

from copier.policy import Account, Limits, Order, decide

DEFAULT_LIMITS = Limits(
    max_notional_usd=1_000,
    max_leverage=3,
    max_daily_loss_usd=100,
    min_liq_distance_pct=20,
    allowed_coins=frozenset({"BTC", "ETH"}),
    copy_ratio=0.5,
)
DEFAULT_ACCOUNT = Account(equity_usd=10_000, notional_usd=0, daily_loss_usd=0, liq_distance_pct=40)


def render(events: list[dict], account: Account = DEFAULT_ACCOUNT, limits: Limits = DEFAULT_LIMITS) -> list[str]:
    lines: list[str] = []
    for event in events:
        decision = decide(event, account, limits)
        kind = "order" if isinstance(decision, Order) else "reject"
        lines.append(json.dumps({"event": event, "type": kind, "decision": asdict(decision)}, separators=(",", ":")))
    return lines


def load_events(path: str) -> list[dict]:
    events = []
    with open(path, encoding="utf-8") as handle:
        for line in handle:
            line = line.strip()
            if line:
                events.append(json.loads(line))
    return events


def main(argv: list[str] | None = None) -> None:
    parser = argparse.ArgumentParser(prog="copier")
    parser.add_argument("--live", action="store_true", help="place real orders on the follower account")
    parser.add_argument("--fixture", help="TradeEvent JSONL to replay")
    args = parser.parse_args(argv)
    if args.live:
        if os.environ.get("COPIER_LIVE") != "1" or not os.environ.get("FOLLOWER_HL_API_WALLET_KEY"):
            print("live trading requires COPIER_LIVE=1 and FOLLOWER_HL_API_WALLET_KEY", file=sys.stderr)
            raise SystemExit(2)
        print("live trading is not part of the judge path", file=sys.stderr)
        raise SystemExit(2)
    if not args.fixture:
        print("copier dry-run: pass --fixture <events.jsonl>", file=sys.stderr)
        raise SystemExit(2)
    for line in render(load_events(args.fixture)):
        print(line)


if __name__ == "__main__":
    main()
