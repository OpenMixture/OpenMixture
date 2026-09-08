#!/usr/bin/env python3
"""Compare release public-Rust calls and CLI renders against accepted 1K pixels.

Requires Pillow and NumPy for output inspection only. Run on a quiet host with
explicit adapter policy; no baseline updates, image synthesis or GPU emulation.
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
    path.write_text(json.dumps(value, indent=2) + "\n")


def stats(values):
    return {"min": min(values), "median": statistics.median(values), "max": max(values)}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo", type=Path, required=True)
    parser.add_argument("--native-binary", type=Path, required=True)
    parser.add_argument("--cli-binary", type=Path, required=True)
    parser.add_argument("--out", type=Path, required=True)
    parser.add_argument("--backend", choices=["metal", "vulkan", "dx12"], required=True)
    parser.add_argument("--expect-adapter", required=True)
    parser.add_argument("--software", action="store_true")
    parser.add_argument("--trials", type=int, default=3)
    parser.add_argument("--materials", nargs="+", choices=["glazed-ceramic", "leather", "wood"], default=["glazed-ceramic", "leather", "wood"])
    args = parser.parse_args()
    if args.trials < 1 or (args.software and args.backend != "vulkan"):
        parser.error("positive trials and Vulkan for software policy are required")
    if len(args.materials) != len(set(args.materials)):
        parser.error("materials must be unique")
    if args.out.exists() or args.out.is_symlink():
        parser.error("output path must not already exist")
    repo, out = args.repo.resolve(), args.out.resolve()
    native, cli = args.native_binary.resolve(), args.cli_binary.resolve()
    if not native.is_file() or not cli.is_file():
        parser.error("both binaries must be built before measurement")
    out.mkdir(parents=True, exist_ok=False)
    (out / "sources").mkdir()
    source_names = subprocess.check_output([
        "git", "ls-files", "--cached", "--others", "--exclude-standard",
        "crates", "examples/native-consumer", "Cargo.toml", "Cargo.lock", "rust-toolchain.toml",
    ], cwd=repo, text=True).splitlines()
    summary = {
        "schemaVersion": 1, "ok": False,
        "sourceRevision": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=repo, text=True).strip(),
        "sourceStatus": subprocess.check_output(["git", "status", "--porcelain", "--untracked-files=all", "--", "crates", "examples/native-consumer"], cwd=repo, text=True).splitlines(),
        "sourceHashes": {p: sha(repo / p) for p in sorted(set(source_names)) if p.endswith((".rs", ".wgsl", ".mix", "Cargo.toml", "Cargo.lock", "rust-toolchain.toml"))},
        "nativeBinary": {"path": str(native), "sha256": sha(native)},
        "cliBinary": {"path": str(cli), "sha256": sha(cli)},
        "declaredProfile": "release --locked --all-features (build separately)",
        "helperSha256": sha(Path(__file__)),
        "host": platform.platform(), "python": platform.python_version(),
        "libraries": {"pillow": PIL.__version__, "numpy": np.__version__},
        "backend": args.backend, "software": args.software, "expectedAdapter": args.expect_adapter,
        "environment": {key: os.environ[key] for key in ["DYLD_LIBRARY_PATH", "VK_DRIVER_FILES", "VK_ICD_FILENAMES", "MIXTURE_SWIFTSHADER_SOURCE"] if key in os.environ},
        "method": "Sequential native process then CLI process per trial; no discarded warmups; each native process acquires one context and renders the same plan twice. Native reports separate compile/context/first/reused CPU wall times excluding file I/O and inspection. External CLI timer includes process lifetime, PNG encoding/writes and JSON. OS/driver caches uncontrolled. Comparisons run after all relevant timers.",
        "trials": args.trials, "invocations": [], "workloads": [],
    }
    flags = ["--backend", args.backend] + (["--software"] if args.software else [])

    def invoke(name, argv):
        start = time.perf_counter_ns()
        result = subprocess.run([str(v) for v in argv], cwd=out, capture_output=True, timeout=120)
        elapsed = (time.perf_counter_ns() - start) / 1e6
        (out / f"{name}.stdout").write_bytes(result.stdout)
        (out / f"{name}.stderr").write_bytes(result.stderr)
        receipt = {"id": name, "argv": [str(v) for v in argv], "cwd": str(out), "exitCode": result.returncode, "wallMs": elapsed, "stdoutSha256": sha(out / f"{name}.stdout"), "stderrSha256": sha(out / f"{name}.stderr")}
        summary["invocations"].append(receipt)
        save(out / "summary.json", summary)
        if result.returncode:
            raise RuntimeError(f"{name} failed; raw streams retained")
        report = json.loads(result.stdout)
        if report.get("ok") is not True:
            raise RuntimeError(f"{name} returned no completed result")
        return elapsed, report

    def adapter(context):
        actual = context["adapter"]
        if actual["backend"].lower() != args.backend or args.expect_adapter not in actual["name"]:
            raise RuntimeError("selected adapter does not match explicit policy")
        if args.software and actual["deviceType"] != "Cpu":
            raise RuntimeError("software policy did not select a CPU adapter")

    _, doctor = invoke("doctor", [cli, "doctor", "--json", *flags])
    adapter(doctor)
    if doctor["verdict"] != "healthy":
        raise RuntimeError("doctor did not verify compute and readback")
    for material in args.materials:
        source = repo / "fixtures/materials" / material
        contract = json.loads((source / "acceptance.json").read_bytes())
        manifest = json.loads((source / "expected/manifest.json").read_bytes())
        copied = out / "sources" / f"{material}.mix"
        shutil.copyfile(source / "material.mix", copied)
        for p in [source / "material.mix", source / "acceptance.json", source / "expected/manifest.json"]:
            summary["sourceHashes"][str(p.relative_to(repo))] = sha(p)
        if args.software:
            revision = subprocess.check_output(["git", "-C", os.environ["MIXTURE_SWIFTSHADER_SOURCE"], "rev-parse", "HEAD"], text=True).strip()
            if revision != contract["softwareRevision"]:
                raise RuntimeError("software source revision differs from the accepted policy")
            summary["softwareSourceRevision"] = revision
        workload = {"material": material, "runs": []}
        summary["workloads"].append(workload)
        for trial in range(1, args.trials + 1):
            name = f"{material}-{trial}"
            raw_dir, png_dir = out / f"{name}-raw", out / f"{name}-png"
            _, rust = invoke(f"{name}-native", [native, "measure", copied, raw_dir, args.backend, "software" if args.software else "hardware", args.expect_adapter])
            cli_ms, command = invoke(f"{name}-cli", [cli, "render", copied, "--size", "1024", "--output", "baseColor,normal,roughness,height", "--out", png_dir, *flags, "--json"])
            adapter(rust["context"])
            adapter(command["context"])
            expected_plan = manifest["planHashes"]["default"]
            if any(plan != expected_plan for plan in [rust["first"]["execution"]["planHash"], rust["reused"]["execution"]["planHash"], command["planHash"]]):
                raise RuntimeError("plan differs from the accepted default")
            if not rust["identicalPixels"] or not rust["pixelsCheckedAfterRendererDrop"]:
                raise RuntimeError("native call has no ownership/reuse evidence")
            comparisons = []
            for channel in ["baseColor", "normal", "roughness", "height"]:
                baseline = source / f"expected/default/{channel}.png"
                if sha(baseline) != manifest["files"][f"default/{channel}.png"]:
                    raise RuntimeError("accepted baseline differs from its manifest")
                first_path, reused_path = raw_dir / f"first-{channel}.rgba", raw_dir / f"reused-{channel}.rgba"
                first, reused = first_path.read_bytes(), reused_path.read_bytes()
                if len(first) != 1024 * 1024 * 4 or first != reused:
                    raise RuntimeError("native owned bytes have incorrect length or differ after reuse")
                native_pixels = np.frombuffer(first, dtype=np.uint8).reshape(1024, 1024, 4)
                with Image.open(png_dir / f"{channel}.png") as png, Image.open(baseline) as golden:
                    if png.size != (1024, 1024) or png.mode != "RGBA":
                        raise RuntimeError("CLI PNG has wrong dimensions/format")
                    if channel == "baseColor" and "srgb" not in png.info:
                        raise RuntimeError("CLI PNG lacks sRGB metadata")
                    if channel != "baseColor" and png.info.get("gamma") != 1.0:
                        raise RuntimeError("CLI PNG lacks linear metadata")
                    if not np.array_equal(native_pixels, np.asarray(png)):
                        raise RuntimeError("public Rust bytes and decoded CLI output differ on the same adapter")
                    delta = np.abs(native_pixels.astype(np.int16) - np.asarray(golden).astype(np.int16))
                tol = contract["hardwareTolerance"]
                maximum, mean = int(delta.max()), float(delta.mean())
                ratio = float((delta.max(axis=2) > tol["pixelThreshold"]).mean())
                passed = maximum == 0 if args.software else maximum <= tol["maxAbsolute"] and mean <= tol["meanAbsolute"] and ratio <= tol["maxChangedPixelRatio"]
                comparisons.append({"channel": channel, "nativeSha256": sha(first_path), "baselineSha256": sha(baseline), "maxAbsolute": maximum, "meanAbsolute": mean, "aboveThresholdPixelRatio": ratio, "pixelThreshold": tol["pixelThreshold"], "nativeMatchesCliExactly": True, "ok": passed})
                if not passed:
                    raise RuntimeError("accepted material tolerance exceeded")
            run = {"trial": trial, "compileMs": rust["compileMs"], "contextMs": rust["contextMs"], "firstCallMs": rust["firstCallMs"], "reusedCallMs": rust["reusedCallMs"], "cliWallMs": cli_ms, "firstExecution": rust["first"]["execution"], "reusedExecution": rust["reused"]["execution"], "cliExecution": command["execution"], "comparisons": comparisons}
            workload["runs"].append(run)
            save(out / "summary.json", summary)
            print(f"{name}: native first {run['firstCallMs']:.2f} ms, reused {run['reusedCallMs']:.2f} ms, CLI {cli_ms:.2f} ms; pixels pass", flush=True)
        workload["timingMs"] = {key: stats([run[key] for run in workload["runs"]]) for key in ["compileMs", "contextMs", "firstCallMs", "reusedCallMs", "cliWallMs"]}
    summary["ok"] = True
    save(out / "summary.json", summary)


if __name__ == "__main__":
    main()
