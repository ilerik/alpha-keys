import json
from pathlib import Path

from copier.cli import load_events, render

ROOT = Path(__file__).resolve().parents[2]
FIXTURE = ROOT / "fixtures" / "feed" / "copier.jsonl"
GOLDEN = ROOT / "fixtures" / "feed" / "transcript.jsonl"


def test_fixture_transcript_matches_the_golden_file():
    lines = render(load_events(str(FIXTURE)))
    assert [json.loads(line)["type"] for line in lines] == ["order", "order", "reject"]
    assert json.loads(lines[2])["decision"]["reason"] == "coin_not_allowed:DOGE"
    assert lines == GOLDEN.read_text(encoding="utf-8").splitlines()
