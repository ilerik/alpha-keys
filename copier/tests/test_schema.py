import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


def test_trade_event_fixture_matches_schema_keys():
    schema = json.loads((ROOT / "docs/schemas/trade-event.json").read_text())
    required = schema["required"]
    event = json.loads((ROOT / "fixtures/feed/alpha-1.jsonl").read_text().splitlines()[0])
    assert set(required) <= set(event)
    assert event["side"] in schema["properties"]["side"]["enum"]
    assert event["kind"] in schema["properties"]["kind"]["enum"]
    assert event["sizeBucket"] in schema["properties"]["sizeBucket"]["enum"]
