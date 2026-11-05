from copier.policy import Limits, Reject, precheck

L = Limits(1000, 3, 100, 20, frozenset({"BTC", "ETH"}), 0.1)


def test_kill_switch():
    from dataclasses import replace
    assert precheck("BTC", replace(L, kill_switch=True)) == Reject("kill_switch")


def test_coin_allowlist():
    assert precheck("DOGE", L) == Reject("coin_not_allowed:DOGE")
    assert precheck("ETH", L) is None
