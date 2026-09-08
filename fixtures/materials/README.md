# Material fixtures

English | [简体中文](./README.zh-CN.md)

PR-008 adds [glazed ceramic](./glazed-ceramic/README.md), four 1K channels, two parameter variants, and protected golden tooling. PR-009 adds the [leather candidate](./leather/README.md), three node additions and controlled appearance evidence; human acceptance is recorded. PR-010 adds [directional wood](./wood/README.md), four 1K cases, and the measured 2K trace; wood human acceptance is recorded. Human review is tracked separately from machine checks; remote CI remains deferred.

Each material directory contains `material.mix`, `README.md`, `acceptance.json`, `variants/`, `expected/`, and `reports/`. Machine checks and human visual acceptance must agree before M3 closes.

Golden updates require an explicit acceptance flag, must refuse CI, and must produce reviewable before/after/difference evidence without staging or committing files. See [the implemented workflow](../../docs/material-goldens.md), [acceptance schema](./acceptance.schema.json), [golden policy](../../AGENTS.md), and [M3 roadmap](../../ROADMAP.md).
