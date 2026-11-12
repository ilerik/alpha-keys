"""Pure copy policy. decide(event, account, limits) -> Order | Reject. No I/O."""

from __future__ import annotations

import math
from dataclasses import dataclass


@dataclass(frozen=True)
class Limits:
    max_notional_usd: float
    max_leverage: float
    max_daily_loss_usd: float
    min_liq_distance_pct: float
    allowed_coins: frozenset[str]
    copy_ratio: float
    lot_size: float = 0.001
    kill_switch: bool = False


@dataclass(frozen=True)
class Account:
    equity_usd: float
    notional_usd: float
    daily_loss_usd: float
    liq_distance_pct: float | None


@dataclass(frozen=True)
class Order:
    coin: str
    side: str
    size: float
    reduce_only: bool


@dataclass(frozen=True)
class Reject:
    reason: str


def bucket_size(bucket: str) -> float:
    text = bucket.removesuffix("+")
    return float(text)


def round_lot(size: float, lot: float) -> float:
    if lot <= 0:
        raise ValueError("lot_size")
    steps = math.floor(size / lot + 1e-9)
    return round(steps * lot, 10)


def decide(event: dict, account: Account, limits: Limits) -> Order | Reject:
    """Size a mirrored order or reject it. Reduces stay allowed when risk limits are already hit."""
    if limits.kill_switch:
        return Reject("kill_switch")
    coin = str(event["coin"])
    reduce_only = bool(event.get("reduceOnly") or event.get("kind") in {"reduce", "close"})
    if not reduce_only and coin not in limits.allowed_coins:
        return Reject(f"coin_not_allowed:{coin}")
    raw = bucket_size(str(event["sizeBucket"])) * limits.copy_ratio
    size = round_lot(raw, limits.lot_size)
    if size <= 0:
        return Reject("below_lot")
    if reduce_only:
        return Order(coin, str(event["side"]), size, True)
    if account.daily_loss_usd >= limits.max_daily_loss_usd:
        return Reject("daily_loss")
    price = float(event["px"])
    added = size * price
    if account.notional_usd + added > limits.max_notional_usd:
        return Reject("max_notional")
    equity = account.equity_usd
    leverage = (account.notional_usd + added) / equity if equity > 0 else math.inf
    if leverage > limits.max_leverage:
        return Reject("max_leverage")
    if account.liq_distance_pct is not None and account.liq_distance_pct < limits.min_liq_distance_pct:
        return Reject("liquidation_distance")
    return Order(coin, str(event["side"]), size, False)


def precheck(coin: str, limits: Limits) -> Reject | None:
    """Cheap checks kept for the original stub tests."""
    if limits.kill_switch:
        return Reject("kill_switch")
    if coin not in limits.allowed_coins:
        return Reject(f"coin_not_allowed:{coin}")
    return None
