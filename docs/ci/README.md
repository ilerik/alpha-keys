# CI template

The bot token has no `workflow` scope, so this file is stored here instead of in `.github/workflows/`. To enable CI from your machine, run:

```bash
mkdir -p .github/workflows && git mv docs/ci/ci.yml.example .github/workflows/ci.yml
```
