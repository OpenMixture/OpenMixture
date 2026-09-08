#!/usr/bin/env python3
"""Capture sequential release-CLI latency and compare outputs to accepted pixels.

This is an optional review helper, not a renderer or a benchmark SLO gate.
Requires Pillow and NumPy. Never writes fixtures or accepts golden updates.
"""

import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import shutil
import statistics
import subprocess
import time

import numpy as np
import PIL
from PIL import Image


def sha(path):
    return "sha256:" + hashlib.sha256(path.read_bytes()).hexdigest()


def save(path, value):
    path.write_text(json.dumps(value, indent=2, ensure_ascii=False) + "\n")


def stats(values):
    return {"min": min(values), "median": statistics.median(values), "max": max(values)}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo", type=Path, required=True)
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--out", type=Path, required=True)
    parser.add_argument("--backend", choices=["metal", "vulkan"], required=True)
    parser.add_argument("--software", action="store_true")
    parser.add_argument("--expect-adapter", required=True)
    parser.add_argument("--trials", type=int, default=3)
    parser.add_argument("--wood-only", action="store_true")
    args = parser.parse_args()
    if args.trials < 1:
        parser.error("--trials must be positive")
    repo, binary, out = args.repo.resolve(), args.binary.resolve(), args.out.resolve()
    if not binary.is_file():
        parser.error("--binary must be an already built executable")
    out.mkdir(parents=True, exist_ok=False)
    (out / "sources").mkdir()
    flags = ["--backend", args.backend] + (["--software"] if args.software else [])
    summary = {
        "schemaVersion": 1, "ok": False,
        "sourceRevision": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=repo, text=True).strip(),
        "binary": {"path": str(binary), "sha256": sha(binary), "profile": "release (caller must build --release --locked --all-features)"},
        "host": platform.platform(), "python": platform.python_version(),
        "libraries": {"pillow": PIL.__version__, "numpy": np.__version__},
        "helperSha256": sha(Path(__file__)), "backend": args.backend, "software": args.software,
        "expectedAdapter": args.expect_adapter, "trials": args.trials,
        "environment": {key: os.environ[key] for key in ["DYLD_LIBRARY_PATH", "MIXTURE_SWIFTSHADER_SOURCE"] if key in os.environ},
        "method": "Sequential fresh CLI processes; no discarded warmup; host wall timer surrounds spawn through process exit including GPU acquisition, rendering, PNG encoding/writes and JSON; PNG comparison runs after timer. OS/driver disk caches may be warm. No GPU timestamp measurements.",
        "invocations": [], "workloads": [], "sourceHashes": {},
    }
    tracked = subprocess.check_output(["git", "ls-files", "crates", "Cargo.toml", "Cargo.lock", "rust-toolchain.toml"], cwd=repo, text=True).splitlines()
    summary["runtimeSourceHashes"] = {p: sha(repo / p) for p in tracked if p.endswith((".rs", ".wgsl", "Cargo.toml", "Cargo.lock", "rust-toolchain.toml"))}

    def run(name, argv, json_report=True):
        started = time.perf_counter_ns()
        result = subprocess.run([str(binary), *argv], cwd=out, capture_output=True, timeout=120)
        elapsed = (time.perf_counter_ns() - started) / 1e6
        (out / f"{name}.stdout").write_bytes(result.stdout)
        (out / f"{name}.stderr").write_bytes(result.stderr)
        receipt = {"id": name, "argv": [str(binary), *argv], "cwd": str(out), "exitCode": result.returncode, "wallMs": elapsed, "stdout": f"{name}.stdout", "stderr": f"{name}.stderr"}
        summary["invocations"].append(receipt)
        save(out / "summary.json", summary)
        if result.returncode:
            raise RuntimeError(f"{name} failed; see captured stdout/stderr")
        report = json.loads(result.stdout) if json_report else None
        if report is not None and not report.get("ok"):
            raise RuntimeError(f"{name} reported ok=false")
        return receipt, report

    for trial in range(args.trials):
        run(f"version-{trial + 1}", ["--version"], json_report=False)
    _, doctor = run("doctor", ["doctor", "--json", *flags])
    if args.expect_adapter not in doctor["adapter"]["name"]:
        raise RuntimeError("doctor selected an unexpected adapter")

    channels = ["baseColor", "normal", "roughness", "height"]
    workloads = [("wood", "default", channels, [])]
    if not args.wood_only:
        workloads = [("glazed-ceramic", "default", channels, []), ("leather", "default", channels, []), *workloads,
                     ("wood", "coarse-grain", channels, ["--set", "grainRepeat=16"]),
                     ("wood", "baseColor-only", ["baseColor"], [])]
    for material, case, requested, overrides in workloads:
        source = repo / "fixtures" / "materials" / material
        copied = out / "sources" / f"{material}.mix"
        if not copied.exists():
            shutil.copyfile(source / "material.mix", copied)
        summary["sourceHashes"][str(source.relative_to(repo) / "material.mix")] = sha(copied)
        contract = json.loads((source / "acceptance.json").read_text())
        manifest = json.loads((source / "expected" / "manifest.json").read_text())
        summary["sourceHashes"][str(source.relative_to(repo) / "acceptance.json")] = sha(source / "acceptance.json")
        summary["sourceHashes"][str(source.relative_to(repo) / "expected" / "manifest.json")] = sha(source / "expected" / "manifest.json")
        item = {"material": material, "case": case, "channels": requested, "runs": []}
        summary["workloads"].append(item)
        for trial in range(args.trials):
            name = f"{material}-{case}-{trial + 1}"
            png_dir = out / "png" / name
            receipt, report = run(name, ["render", str(copied), "--size", "1024", "--output", ",".join(requested), "--out", str(png_dir), *overrides, *flags, "--json"])
            execution = report["execution"]
            if args.expect_adapter not in execution["adapter"]["name"]:
                raise RuntimeError(f"{name}: unexpected adapter")
            if [o["channel"] for o in report["outputs"]] != requested:
                raise RuntimeError(f"{name}: wrong output channels")
            expected_case = "default" if case == "baseColor-only" else case
            if case != "baseColor-only" and report["planHash"] != manifest["planHashes"][expected_case]:
                raise RuntimeError(f"{name}: unexpected plan hash")
            comparisons = []
            for output in report["outputs"]:
                channel = output["channel"]
                relative = f"{expected_case}/{channel}.png"
                baseline = source / "expected" / relative
                if sha(baseline) != manifest["files"][relative]:
                    raise RuntimeError("baseline differs from accepted manifest")
                actual = Path(output["path"])
                with Image.open(actual) as im, Image.open(baseline) as reference:
                    if im.size != (1024, 1024) or im.mode != "RGBA" or output["encoding"] != contract["channels"][channel]["encoding"]:
                        raise RuntimeError(f"{name}: wrong size/mode/encoding")
                    if channel == "baseColor":
                        if "srgb" not in im.info:
                            raise RuntimeError("missing sRGB PNG metadata")
                    elif im.info.get("gamma") != 1.0:
                        raise RuntimeError("missing linear PNG metadata")
                    delta = np.abs(np.asarray(im).astype(np.int16) - np.asarray(reference).astype(np.int16))
                tolerance = contract["hardwareTolerance"]
                maximum = int(delta.max())
                mean = float(delta.mean())
                changed = float((delta.max(axis=2) > tolerance["pixelThreshold"]).mean())
                passed = maximum == 0 if args.software else maximum <= tolerance["maxAbsolute"] and mean <= tolerance["meanAbsolute"] and changed <= tolerance["maxChangedPixelRatio"]
                comparisons.append({"channel": channel, "pngSha256": sha(actual), "baseline": str(baseline.relative_to(repo)), "baselineSha256": sha(baseline), "maxAbsolute": maximum, "meanAbsolute": mean, "changedPixelRatio": changed, "ok": passed})
                if not passed:
                    raise RuntimeError(f"{name}/{channel}: accepted pixel tolerance exceeded")
            item["runs"].append({"receipt": name, "wallMs": receipt["wallMs"], "planHash": report["planHash"], "passCount": execution["passCount"], "timings": execution["timings"], "pipelineCache": execution["pipelineCache"], "allocations": execution["allocations"], "comparisons": comparisons})
            save(out / "summary.json", summary)
            print(f"{name}: {receipt['wallMs']:.2f} ms, pixels pass", flush=True)
        item["wallMs"] = stats([r["wallMs"] for r in item["runs"]])
        item["timingMs"] = {key: stats([r["timings"][key] for r in item["runs"]]) for key in item["runs"][0]["timings"]}
    summary["versionWallMs"] = stats([r["wallMs"] for r in summary["invocations"] if r["id"].startswith("version-")])
    summary["ok"] = True
    save(out / "summary.json", summary)


if __name__ == "__main__":
    main()
