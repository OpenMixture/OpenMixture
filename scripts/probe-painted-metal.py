"""Bounded diagnostic for the frozen PERF-MAT graph, using the production CLI."""

import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import struct


def linear_midpoints(cli, output):
    """Literal dyadic inputs isolate gamma-one remapping from noise."""
    rows = []
    # Half-representable inputs whose affine result falls on a half midpoint.
    # Both rounding parities and exact endpoints are explicit numerical controls.
    bits = [0, 0x3c00, 0x3402, 0x3406, 0x3502, 0x3602, 0x3702,
            0x3801, 0x3803, 0x3901, 0x3a01, 0x3b01]
    for value_bits in bits:
        value = struct.unpack('<e', struct.pack('<H', value_bits))[0]
        expected = struct.unpack('<e', struct.pack('<e', 0.5 + 0.5 * value))[0]
        name = f"linear-{value_bits:04x}"
        nodes = []
        edges = []

        def node(node_id, kind, parameters, inputs=None):
            nodes.append({"id": node_id, "type": kind, "version": 1,
                          "parameters": parameters})
            for port, source in (inputs or {}).items():
                edges.append({"from": {"nodeId": source, "portId": "value"},
                              "to": {"nodeId": node_id, "portId": port}})

        node("input", "constant-scalar", {"value": value})
        node("actual", "levels", {"inputMin": 0, "inputMax": 1, "gamma": 1,
                                   "outputMin": 0.5, "outputMax": 1}, {"in": "input"})
        node("expected", "constant-scalar", {"value": expected})
        for sign, left, right in [("positive", "actual", "expected"),
                                  ("negative", "expected", "actual")]:
            node(sign, "scalar-subtract", {}, {"a": left, "b": right})
            node(sign + "Error", "levels", {"inputMin": 0, "inputMax": 1 / 2048,
                 "gamma": 1, "outputMin": 0, "outputMax": 1}, {"in": sign})
        node("out", "material-output", {},
             {"height": "positiveError", "roughness": "negativeError"})
        node("color", "constant-color", {"value": [0, 0, 0, 1]})
        edges.append({"from": {"nodeId": "color", "portId": "color"},
                      "to": {"nodeId": "out", "portId": "baseColor"}})
        graph = {"version": 1, "nodes": nodes, "edges": edges}
        source = output / (name + ".mix")
        source.write_text(json.dumps(graph, indent=2) + "\n", encoding="utf-8")
        command = [str(cli), "render", str(source), "--size", "1",
                   "--output", "height,roughness", "--backend", "vulkan", "--software",
                   "--out", str(output / name), "--json"]
        result = subprocess.run(command, capture_output=True, check=False)
        (output / (name + ".json")).write_bytes(result.stdout)
        (output / (name + ".stderr.log")).write_bytes(result.stderr)
        rows.append({"id": name, "inputHalfBits": value_bits, "input": value,
                     "expectedHalf": expected, "command": command,
                     "sourceSha256": hashlib.sha256(source.read_bytes()).hexdigest(),
                     "exitCode": result.returncode})
        (output / "linear-midpoints.json").write_text(json.dumps({
            "qualification": False, "expectedErrorPixel": [0, 0, 0, 255],
            "positiveError": "height", "negativeError": "roughness", "rows": rows,
        }, indent=2) + "\n", encoding="utf-8")
        if result.returncode:
            raise RuntimeError(f"{name}: CLI failed; see retained report")
        print(name, flush=True)


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
    linear_midpoints(cli, output)


if __name__ == "__main__":
    main()
