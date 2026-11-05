# copier/

The opt-in copier agent (Python, uv). It subscribes to the gateway feed and mirrors trades into the **follower's own** Hyperliquid account, within limits: max notional, max leverage, daily loss cap, liquidation-distance guard, allowed coins, copy ratio, and a kill switch.

`--dry-run` is the default. Baseline fleet code goes in `src/copier/vendor/` with `# baseline:` headers. Tasks are K-1 to K-6.

```bash
cd copier && uv run pytest
uv run copier --dry-run --fixture ../fixtures/feed/sample.jsonl
```
