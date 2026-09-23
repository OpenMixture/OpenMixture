# MAT-02 existing-input downsample feasibility

English | [简体中文](./README.zh-CN.md)

On 2026-09-23, clean source `e744b5ea8586488c3e25aef9cdf9928b1e4f09da` generated the [input graph](./input.mix) through a freshly built public CLI on the recorded GT 1030 Vulkan and DX12 paths. This graph uses existing v2 value noise, levels, constants and color blend only. It does **not** implement or qualify morphology, subtraction, rust layers, final normals, browser parity or the complete material.

The [receipt](./receipt.json) records build/verifier/lock/design hashes, actual adapters, commands, plan hashes and all retained file hashes. [Design](./graph-design-at-measurement.json) and [plan](./qualification-plan-at-measurement.json) preserve the exact inputs before the later contract freeze. All sixteen PNGs, four raw CLI reports and the generated input are retained, so these measurements can be re-inspected without an expiring artifact.

At each backend, compare the 256² RGBA8 result against the arithmetic 4×4 box average of its 1024² result. Errors are in byte units (/255), use RGB for baseColor and red for Scalar channels. Maximum component mean absolute errors are **0.158619 baseColor** and **0.101737 height**, both below the selected 4/255 input feasibility target. The exposure mask (transported through metallic solely for inspection) is 0.238492; detail modulation (through roughness) is 0.332754. Both backends independently return the same listed metrics; this is not a substitute for a cross-runtime pixel test.

Agent inspection of the 256² baseColor input found connected blue coating and gray exposed areas with smooth transitions. This is input inspection, not a PBR review or human acceptance. Integer radius mapping is checked separately in the design review. Subsequent full-material default downsampling must still pass 4/255 after morphology/rust/normal implementation; it cannot be inferred from this simpler graph.

## Reproduce

Check out the measured revision cleanly, build `cargo build --locked -p mixture-cli`, and invoke [reproduce.py](./reproduce.py) with the resulting CLI path and a fresh output directory:

```text
python reproduce.py <path-to-mixture-executable> <fresh-output-directory>
```

Run from the repository root; the script uses `fixtures/materials/painted-metal/` design/defaults at that checkout, explicitly invokes Vulkan and DX12 without fallback, and rejects source/binary changes during execution. It requires Python, Pillow and NumPy; recorded versions are in the receipt. The development profile is adequate for pixel feasibility; no timing budget or performance acceptance is claimed. All pixels come from CLI/wgpu; Python only reads PNGs and computes comparisons. A reproduction on later source creates new evidence and must not overwrite this record.
