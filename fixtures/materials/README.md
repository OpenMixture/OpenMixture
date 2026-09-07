# Material fixtures

Reserved for glazed ceramic, leather-like, and directional wood golden materials introduced by PR-008 through PR-010. M0 contains no synthetic baselines or acceptance claims.

Each future material directory will contain `material.mix`, `README.md`, `acceptance.json`, `variants/`, `expected/`, and `reports/`. Machine checks and human visual acceptance must agree before M3 closes.

Golden updates require an explicit acceptance flag, must refuse CI, and must produce reviewable before/after/difference evidence without staging or committing files. See [the golden policy](../../AGENTS.md) and [M3 roadmap](../../ROADMAP.md).
