.PHONY: check test-rust test-ts test-py demo-seed

check: test-rust test-ts test-py

test-rust:
	cargo test --workspace

test-ts:
	pnpm -r test

test-py:
	cd copier && uv run --extra dev pytest -q

demo-seed:
	@echo "TODO(H-3): offline replay fixtures -> pnl-core -> feed -> copier --dry-run"
