# Examples

English | [简体中文](./README.zh-CN.md)

PR-007 provides three readable `.mix v1` documents that validate, compile, and render through the same `wgpu` executor:

| Example | Graph and useful outputs | Exposed controls |
| --- | --- | --- |
| [checker.mix](./checker.mix) | Checker → baseColor; optional channels use versioned defaults | `frequency` → cellsX |
| [levels.mix](./levels.mix) | Scalar → levels → roughness, with a white baseColor | `input`, `gamma`, input/output bounds |
| [blend.mix](./blend.mix) | Checker + tint, scalar → levels → blend mask, blend → baseColor; scalar → roughness | `frequency`, `contrast`, `strength` |

```bash
cargo run --locked -p mixture-cli -- validate examples/checker.mix --json
cargo run --locked -p mixture-cli -- inspect examples/blend.mix --plan --json
cargo run --locked -p mixture-cli -- render examples/checker.mix --size 256 --out ./tmp/checker
cargo run --locked -p mixture-cli -- render examples/levels.mix --size 256 \
  --output baseColor,roughness --set 'gamma=2' --out ./tmp/levels --json
cargo run --locked -p mixture-cli -- render examples/blend.mix --size 256 \
  --output baseColor,roughness,normal --out ./tmp/blend --json
```

Files use deterministic `<channel>.png` names. Color output is sRGB; scalar and encoded-normal output is linear. Requesting only roughness in blend prunes the color/levels branch and executes one constant pass. The examples establish graph execution, not realistic material quality.

The separate `render-builtin checker`/doctor probe does not read a `.mix`; it shares the checker shader and dispatch path. See [graph rendering](../docs/graph-rendering.md), [format](../docs/file-format.md), [contracts](../docs/node-contracts.md), and [development](../docs/development.md).
