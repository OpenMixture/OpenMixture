#!/usr/bin/env python3
"""Capture public CLI diagnostics without building or requesting a GPU."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def write_json(path, value):
    with path.open("x", encoding="utf-8") as stream:
        json.dump(value, stream, indent=2, ensure_ascii=False)
        stream.write("\n")


def git(root, *args):
    return subprocess.check_output(["git", "-C", str(root), *args], text=True).strip()


def probes(out):
    warp = "fixtures/nodes/warp/missing-input.mix"
    checker = "examples/checker.mix"
    invalid = "fixtures/format/invalid/"
    return [
        ("help", ["--help"], 0),
        ("unknown-port-json", ["validate", invalid + "unknown-port.mix", "--json"], 2),
        ("unknown-port-human", ["inspect", invalid + "unknown-port.mix", "--plan"], 2),
        ("missing-warp-json", ["validate", warp, "--json"], 2),
        ("missing-warp-human", ["inspect", warp, "--plan"], 2),
        ("invalid-parameters", ["validate", invalid + "invalid-parameters.mix", "--json"], 2),
        ("cycle", ["validate", invalid + "cycle.mix", "--json"], 2),
        ("duplicate-key", ["validate", invalid + "duplicate-key.mix", "--json"], 2),
        ("invalid-exposed", ["validate", invalid + "invalid-exposed.mix", "--json"], 2),
        ("unknown-override", ["inspect", checker, "--plan", "--json", "--set", "frequncy=16"], 2),
        ("budget", ["inspect", checker, "--plan", "--json", "--size", "4096"], 2),
        ("doctor-none", ["doctor", "--backend", "none", "--json"], 1),
        ("missing-file", ["validate", str(out / "not-present.mix"), "--json"], 1),
        ("usage-json", ["inspect", checker, "--json"], 2),
        ("missing-warp-render-human", ["render", warp, "--out", str(out / "unused-render-output"), "--backend", "none"], 2),
        ("missing-warp-validate-human", ["validate", warp], 2),
    ]


def capture(root, binary, profile, build_command, out):
    cases = probes(out)
    source_paths = sorted({arg for _, args, _ in cases for arg in args
                           if arg.endswith(".mix") and not Path(arg).is_absolute()})
    source_hashes = {path: digest(root / path) for path in source_paths}
    binary_hash = digest(binary)
    revision = git(root, "rev-parse", "HEAD")
    source_status = git(root, "status", "--porcelain", "--", "Cargo.toml", "Cargo.lock",
                        "rust-toolchain.toml", ".cargo", "crates", *source_paths)
    metadata = {
        "schemaVersion": 1,
        "sourceRevision": revision,
        "sourceRoot": str(root),
        "workingDirectory": str(root),
        "sourceStatus": source_status,
        "binary": str(binary),
        "binaryProfile": profile,
        "declaredBuildCommand": build_command,
        "binaryProfileEvidence": "Caller declaration; the helper does not build or inspect compiler flags.",
        "binarySha256": binary_hash,
        "binarySourceEvidence": "Checkout revision and clean/dirty source status are recorded separately; the binary has no embedded source-revision attestation.",
        "helperSha256": digest(Path(__file__)),
        "fixtureSha256": source_hashes,
        "gpuAcquisitionRequested": False,
        "gpuPolicy": "Only doctor/render could acquire a GPU; both explicitly use --backend none. Other probes use CPU-only validate/inspect/help paths.",
        "probes": [],
        "runCompleted": False,
    }
    write_json(out / "started.json", metadata)
    captured = {}
    for name, args, expected in cases:
        command = [str(binary), *args]
        result = subprocess.run(command, cwd=root, capture_output=True, timeout=60, check=False)
        stdout_path = out / (name + ".stdout")
        stderr_path = out / (name + ".stderr")
        with stdout_path.open("xb") as stream:
            stream.write(result.stdout)
        with stderr_path.open("xb") as stream:
            stream.write(result.stderr)
        probe = {
            "name": name, "command": command, "exitCode": result.returncode,
            "expectedExitCode": expected, "expectedExitCodeMatched": result.returncode == expected,
            "stdout": stdout_path.name, "stderr": stderr_path.name,
            "stdoutSha256": digest(stdout_path), "stderrSha256": digest(stderr_path),
        }
        write_json(out / (name + ".json"), probe)
        metadata["probes"].append(probe)
        captured[name] = result.stdout

    doctor = json.loads(captured["doctor-none"])
    missing = json.loads(captured["missing-warp-json"])
    expected_port = any(d.get("code") == "MIX_PORT_REQUIRED_CONNECTION"
                        and d.get("nodeId") == "sample" and d.get("portId") == "displacement"
                        for d in missing.get("diagnostics", []))
    metadata["observations"] = {
        "missingWarpJsonIdentifiesPort": expected_port,
        "missingWarpValidateHumanIdentifiesPort": b"port: displacement" in captured["missing-warp-validate-human"],
        "missingWarpInspectHumanIdentifiesPort": b"port: displacement" in captured["missing-warp-human"],
        "missingWarpRenderHumanIdentifiesPort": b"port: displacement" in captured["missing-warp-render-human"],
        "doctorNoneHasNoAdapterOrDevice": doctor.get("adapter") is None and doctor.get("device") is None,
        "doctorNoneHasNoEffectiveBackend": doctor.get("requested", {}).get("effectiveBackends") == [],
        "doctorNoneProbesNotRun": doctor.get("computeProbe") == "notRun" and doctor.get("readbackProbe") == "notRun",
        "renderOutputDirectoryAbsent": not (out / "unused-render-output").exists(),
    }
    metadata["binaryUnchanged"] = digest(binary) == binary_hash
    metadata["fixturesUnchanged"] = all(digest(root / path) == value for path, value in source_hashes.items())
    metadata["sourceRevisionAfter"] = git(root, "rev-parse", "HEAD")
    metadata["sourceRevisionUnchanged"] = metadata["sourceRevisionAfter"] == revision
    metadata["expectedExitCodesMatched"] = all(p["expectedExitCodeMatched"] for p in metadata["probes"])
    metadata["runCompleted"] = True
    write_json(out / "summary.json", metadata)
    observations = metadata["observations"]
    capture_ok = all(metadata[key] for key in ("binaryUnchanged", "fixturesUnchanged", "sourceRevisionUnchanged", "expectedExitCodesMatched"))
    capture_ok &= all(observations[key] for key in (
        "doctorNoneHasNoAdapterOrDevice", "doctorNoneHasNoEffectiveBackend",
        "doctorNoneProbesNotRun", "renderOutputDirectoryAbsent"))
    print(out / "summary.json")
    # Human-context observations are findings, not a golden asserting that bugs must persist.
    return 0 if capture_ok else 1


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", required=True, type=Path, help="Already-built mixture executable; never built by this script.")
    parser.add_argument("--profile", required=True, choices=("debug", "release"), help="Declared Cargo build profile, recorded as provenance.")
    parser.add_argument("--build-command", help="Optional recorded build command; this string is never executed.")
    parser.add_argument("--out", required=True, type=Path, help="New output directory outside this durable evidence tree; existing paths are rejected.")
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[4]
    binary = args.binary.resolve()
    if not binary.is_file() or not os.access(binary, os.X_OK):
        parser.error("--binary must identify an existing executable; build it separately before running this helper.")
    if os.path.lexists(args.out):
        parser.error("--out already exists; evidence is never overwritten. Choose a new directory.")
    out = args.out.resolve()
    if out.is_relative_to(Path(__file__).resolve().parent):
        parser.error("--out must be outside the durable evidence directory.")
    out.parent.mkdir(parents=True, exist_ok=True)
    out.mkdir(exist_ok=False)
    try:
        return capture(root, binary, args.profile, args.build_command, out)
    except (OSError, ValueError, subprocess.SubprocessError) as error:
        failure = {"runCompleted": False, "errorType": type(error).__name__, "message": str(error)}
        if isinstance(error, subprocess.TimeoutExpired):
            failure["command"] = error.cmd
            for channel, value in (("stdout", error.stdout), ("stderr", error.stderr)):
                with (out / ("timeout." + channel)).open("xb") as stream:
                    stream.write(value or b"")
        write_json(out / "failure.json", failure)
        print(f"Probe capture failed; partial evidence retained at {out}: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
