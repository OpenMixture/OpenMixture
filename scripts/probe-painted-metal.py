"""Bounded diagnostic for the frozen PERF-MAT graph, using the production CLI."""

import argparse
import hashlib
import json
from pathlib import Path
import subprocess


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--cli", required=True)
    parser.add_argument("--output", required=True)
    args = parser.parse_args()
    root = Path(__file__).resolve().parent.parent
    source = root / "docs/evidence/perf-mat-before/material.mix"
    original = source.read_bytes()
    cli = Path(args.cli).resolve(strict=True)
    output = Path(args.output)
    output.mkdir()  # Refuse to overwrite evidence from an earlier run.
    stages = ["macro", "exposure", "erodeX", "erodeY", "band", "rustSpread",
              "detailNoise", "detail", "rustMask", "coatingHeight", "height"]
    receipt = {
        "kind": "painted-metal-stage-diagnostic",
        "qualification": False,
        "sourceCommit": subprocess.check_output(
            ["git", "rev-parse", "HEAD"], cwd=root, text=True).strip(),
        "workingTreeStatus": subprocess.check_output(
            ["git", "status", "--porcelain"], cwd=root, text=True),
        "sourceSha256": hashlib.sha256(original).hexdigest(),
        "scriptSha256": hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
        "cliSha256": hashlib.sha256(cli.read_bytes()).hexdigest(),
        "size": [1024, 1024],
        "rows": [],
    }
    for stage in stages:
        graph = json.loads(original)
        replaced = 0
        for edge in graph["edges"]:
            if edge["to"] in ({"nodeId": "out", "portId": "height"},
                              {"nodeId": "normal", "portId": "in"}):
                edge["from"] = {"nodeId": stage, "portId": "value"}
                replaced += 1
        if replaced != 2:
            raise RuntimeError("Frozen material no longer has the two expected height edges")
        variant = output / (stage + ".mix")
        variant.write_text(json.dumps(graph, indent=2) + "\n", encoding="utf-8")
        command = [str(cli), "render", str(variant), "--size", "1024",
                   "--output", "height,normal", "--backend", "vulkan", "--software",
                   "--out", str(output / stage), "--json"]
        result = subprocess.run(command, capture_output=True, check=False)
        (output / (stage + ".json")).write_bytes(result.stdout)
        (output / (stage + ".stderr.log")).write_bytes(result.stderr)
        row = {"stage": stage, "command": command, "exitCode": result.returncode,
               "sourceSha256": hashlib.sha256(variant.read_bytes()).hexdigest()}
        receipt["rows"].append(row)
        if result.returncode == 0:
            row["planHash"] = json.loads(result.stdout)["planHash"]
        (output / "receipt.json").write_text(
            json.dumps(receipt, indent=2) + "\n", encoding="utf-8")
        if result.returncode:
            raise RuntimeError(f"{stage}: CLI failed; see retained structured report and stderr")
        print(stage, flush=True)


if __name__ == "__main__":
    main()
