"""Pure copy policy (K-1, STUB). decide(event, account, limits) -> Order | Reject. No I/O here."""
from __future__ import annotations

from dataclasses import dataclass


@dataclass(frozen=True)
class Limits:
    max_notional_usd: float
    max_leverage: float
    max_daily_loss_usd: float
    min_liq_distance_pct: float
    allowed_coins: frozenset[str]
    copy_ratio: float
    kill_switch: bool = False


@dataclass(frozen=True)
class Reject:
    reason: str


def precheck(coin: str, limits: Limits) -> Reject | None:
    """Example of the style: cheap, pure checks first. Full decide() lands in K-1."""
    if limits.kill_switch:
        return Reject("kill_switch")
    if coin not in limits.allowed_coins:
        return Reject(f"coin_not_allowed:{coin}")
    return None
