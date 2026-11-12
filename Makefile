.PHONY: check test-rust test-ts test-py demo-seed leak-check

check: test-rust test-ts test-py leak-check

test-rust:
	cargo test --workspace

test-ts:
	pnpm -r test

test-py:
	cd copier && uv run --extra dev pytest -q && uv run --extra dev ruff check .

demo-seed:
	cargo run -p demo-seed --quiet
	cd copier && uv run --extra dev copier --fixture ../fixtures/feed/copier.jsonl

leak-check:
	python3 scripts/leak_check.py
