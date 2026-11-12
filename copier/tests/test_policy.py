from dataclasses import replace

from copier.policy import Account, Limits, Order, Reject, decide, precheck

L = Limits(1000, 3, 100, 20, frozenset({"BTC", "ETH"}), 0.1)
A = Account(1000, 0, 0, 50)


def event(**overrides):
    base = {
        "traderId": "alpha-1",
        "seq": 1,
        "tsMs": 1,
        "coin": "ETH",
        "side": "B",
        "sizeBucket": "1",
        "px": "100",
        "reduceOnly": False,
        "kind": "open",
    }
    base.update(overrides)
    return base


def test_kill_switch():
    assert precheck("BTC", replace(L, kill_switch=True)) == Reject("kill_switch")
    assert decide(event(), A, replace(L, kill_switch=True)) == Reject("kill_switch")


def test_coin_allowlist():
    assert precheck("DOGE", L) == Reject("coin_not_allowed:DOGE")
    assert precheck("ETH", L) is None
    assert decide(event(coin="DOGE"), A, L) == Reject("coin_not_allowed:DOGE")


def test_notional_limit():
    account = Account(10_000, 950, 0, 50)
    decision = decide(event(sizeBucket="10", px="100"), account, L)
    assert decision == Reject("max_notional")


def test_leverage_limit():
    limits = replace(L, max_notional_usd=10_000)
    decision = decide(event(sizeBucket="10", px="400"), Account(100, 0, 0, 50), limits)
    assert decision == Reject("max_leverage")


def test_daily_loss_limit():
    assert decide(event(), Account(1000, 0, 100, 50), L) == Reject("daily_loss")


def test_liquidation_distance():
    assert decide(event(), Account(1000, 0, 0, 10), L) == Reject("liquidation_distance")


def test_lot_rounding_rejects_dust():
    decision = decide(event(sizeBucket="0.01"), A, replace(L, copy_ratio=0.01))
    assert decision == Reject("below_lot")


def test_accepts_a_sized_order_and_still_reduces_when_limits_are_hit():
    assert decide(event(), A, L) == Order("ETH", "B", 0.1, False)
    reduce = decide(
        event(kind="close", reduceOnly=True, coin="DOGE", side="A"),
        Account(1000, 5_000, 500, 1),
        L,
    )
    assert reduce == Order("DOGE", "A", 0.1, True)
