#!/usr/bin/env python3
"""Verify retained M3 evidence without rendering or making a human decision."""

import hashlib
import json
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[4]
BASE = "e9dd03bb272a29de6243aec79a3450b47b90530e"
MATERIALS = ("glazed-ceramic", "leather", "wood")
REPORTS = Path("fixtures/materials/wood/reports")
VERIFIED = {}
HISTORICAL_INPUTS = {}


def require(condition, message):
    if not condition:
        raise ValueError(message)


def digest(data):
    return "sha256:" + hashlib.sha256(data).hexdigest()


def relative(path):
    return (ROOT / path).resolve().relative_to(ROOT).as_posix()


def base_bytes(name):
    return subprocess.run(
        ["git", "show", f"{BASE}:{name}"],
        cwd=ROOT, check=True, stdout=subprocess.PIPE,
    ).stdout


def committed(path, expected=None):
    name = relative(path)
    data = (ROOT / name).read_bytes()
    actual = digest(data)
    if name not in VERIFIED:
        original = base_bytes(name)
        require(data == original, f"{name}: differs from base revision")
        VERIFIED[name] = actual
    require(actual == VERIFIED[name], f"{name}: changed during verification")
    if expected is not None:
        require(actual == expected, f"{name}: recorded hash mismatch")
    return {"path": name, "sha256": actual}


def document(path):
    committed(path)
    return json.loads((ROOT / path).read_bytes())


def bound_map(directory, hashes):
    for name, expected in hashes.items():
        committed(directory / name, expected)
    return len(hashes)


def historical_inputs(hashes):
    """Bind captured candidate/trace source inputs to Git, not a later checkout."""
    for name, expected in hashes.items():
        path = Path(name)
        require(not path.is_absolute() and ".." not in path.parts,
                f"{name}: expected a repository-relative source path")
        actual = HISTORICAL_INPUTS.get(name) or VERIFIED.get(name)
        if actual is None:
            actual = digest(base_bytes(name))
        require(actual == expected, f"{name}: historical source hash mismatch")
        HISTORICAL_INPUTS[name] = actual
    return len(hashes)


def human_acceptance(material):
    directory = Path("fixtures/materials") / material
    review_path = directory / "reports/human-review.json"
    review = document(review_path)
    require(review["material"] == material, f"{material}: human review identity")
    require(review["status"] == "accepted", f"{material}: human review not accepted")
    require(review["humanDecision"]["source"] == "user", f"{material}: decision source")
    require(review["humanDecision"]["decision"] == "accepted", f"{material}: decision")
    require(bool(review["reviewer"]) and bool(review["reviewedAt"]), f"{material}: signer")
    manifest_path = directory / "expected/manifest.json"
    appearance_path = directory / "reports/pbr/review.json"
    comparison_path = directory / "reports/pbr/comparison.png"
    bindings = [
        committed(manifest_path, review["baselineManifestSha256"]),
        committed(appearance_path, review["appearanceReportSha256"]),
        committed(comparison_path, review["appearanceComparisonSha256"]),
    ]
    manifest = document(manifest_path)
    require(manifest["material"] == material, f"{material}: baseline identity")
    baseline_count = bound_map(directory / "expected", manifest["files"])
    expected_files = set(manifest["files"]) | {"manifest.json"}
    current_files = {
        p.relative_to(ROOT / directory / "expected").as_posix()
        for p in (ROOT / directory / "expected").rglob("*") if p.is_file()
    }
    require(current_files == expected_files, f"{material}: expected file inventory")
    appearance = document(appearance_path)
    require(appearance["material"] == material, f"{material}: appearance identity")
    require(
        appearance["baselineManifestSha256"] == review["baselineManifestSha256"],
        f"{material}: appearance baseline binding",
    )
    require(appearance["inputs"] == manifest["files"], f"{material}: PBR inputs")
    require(set(appearance["results"]) == set(manifest["planHashes"]),
            f"{material}: complete appearance case inventory")
    committed(directory / "review/render.py", appearance["sceneScriptSha256"])
    if "sceneHelperSha256" in appearance:
        committed(
            directory / "review" / appearance["sceneHelper"],
            appearance["sceneHelperSha256"],
        )
    for result in appearance["results"].values():
        committed(directory / "reports/pbr" / result["file"], result["sha256"])
    comparison = document(directory / "reports/pbr/comparison.json")
    require(
        comparison["reviewSha256"] == review["appearanceReportSha256"]
        and comparison["outputSha256"] == review["appearanceComparisonSha256"],
        f"{material}: comparison bindings",
    )
    committed(directory / "review/compare.py", comparison["scriptSha256"])
    return {
        "material": material,
        "review": committed(review_path),
        "status": review["status"],
        "reviewer": review["reviewer"],
        "reviewedAt": review["reviewedAt"],
        "decision": review["humanDecision"],
        "boundEvidence": bindings,
        "baselinePngCount": baseline_count,
        "appearanceFrameCount": len(appearance["results"]),
        "bindingsAndExpectedFilesMatchBase": True,
    }


def material_gates(material, backend):
    filename = (
        f"{backend}.json" if material == "wood"
        else f"{material}-{backend}-regression.json"
    )
    report_path = REPORTS / filename
    report = document(report_path)
    directory = Path("fixtures/materials") / material
    acceptance = document(directory / "acceptance.json")
    manifest = document(directory / "expected/manifest.json")
    require(report["material"] == material, f"{report_path}: material")
    policy = "software" if backend == "software" else "hardware"
    require(report["policy"] == policy, f"{report_path}: policy")
    for flag in ("ok", "baselinePresent", "machineChecksPassed", "goldenComparisonsPassed"):
        require(report[flag] is True, f"{report_path}: {flag}")
    rules = {case["id"]: case for case in acceptance["cases"]}
    require(
        {case["id"] for case in report["cases"]} == set(rules)
        and len(report["cases"]) == len(rules),
        f"{report_path}: complete, unique case inventory",
    )
    require(report["size"] == acceptance["size"] == 1024, f"{report_path}: size")
    cases = []
    for case in report["cases"]:
        case_id = case["id"]
        rule = rules[case_id]
        label = f"{report_path}/{case_id}"
        require(case["planMatchesGolden"] is True, f"{label}: plan match")
        require(
            case["planHash"] == manifest["planHashes"][case_id]
            == case["execution"]["planHash"],
            f"{label}: baseline/execution plan hash",
        )
        expected_overrides = (
            document(directory / "variants" / f'{rule["variant"]}.json')["overrides"]
            if rule["variant"] is not None else {}
        )
        require(case["overrides"] == expected_overrides, f"{label}: overrides")
        require(set(case["channels"]) == set(acceptance["channels"]), f"{label}: channels")
        require(
            [item["channel"] for item in case["outputs"]] == list(acceptance["channels"]),
            f"{label}: output order",
        )
        counts = {"comparison": 0, "floatReadback": 0, "structure": 0, "causality": 0}
        for channel, evidence in case["channels"].items():
            for gate in ("comparison", "floatReadback", "structure"):
                require(evidence[gate]["ok"] is True, f"{label}/{channel}: {gate}")
                counts[gate] += 1
            require(evidence["comparison"]["status"] == "compared", f"{label}: comparison")
            tolerance = (
                {"maxAbsolute": 0, "meanAbsolute": 0.0, "pixelThreshold": 0,
                 "maxChangedPixelRatio": 0.0}
                if backend == "software" else acceptance["hardwareTolerance"]
            )
            require(evidence["comparison"]["tolerance"] == tolerance, f"{label}: tolerance")
            structure = evidence["structure"]
            structure_rule = rule["checks"][channel]
            require(structure["kind"] == structure_rule["kind"], f"{label}: structure kind")
            if "rule" in structure:
                require(structure["rule"] == structure_rule, f"{label}: structure rule")
            elif structure["kind"] == "uniform":
                require(
                    structure["expected"] == structure_rule["rgba"]
                    and structure["tolerance"] == structure_rule["tolerance"],
                    f"{label}: uniform rule",
                )
                require(structure["maxAbsolute"] <= structure_rule["tolerance"],
                        f"{label}: uniform measured error")
            elif structure["kind"] == "alternating":
                require(structure["expectedCells"] == structure_rule["cells"],
                        f"{label}: alternating cell rule")
                require(
                    structure["contrast"] >= structure_rule["minContrast"]
                    and structure["balanceError"] <= structure_rule["maxBalanceError"],
                    f"{label}: alternating contrast/balance",
                )
            else:
                raise ValueError(f"{label}: unrecognized structure record")
            if channel in rule["changes"]:
                require(evidence["causality"]["ok"] is True, f"{label}: causality")
                require(
                    evidence["causality"]["rule"] == rule["changes"][channel],
                    f"{label}: causality rule",
                )
                counts["causality"] += 1
            else:
                require(evidence["causality"] is None, f"{label}: absent causality")
            readback = evidence["floatReadback"]
            require(
                readback["nonFiniteComponents"] == readback["outOfRangeComponents"] == 0,
                f"{label}: finite bounded float readback",
            )
        require(
            [item["rule"] for item in case["relationships"]] == rule.get("relationships", []),
            f"{label}: relationship rules",
        )
        require(
            all(item["ok"] is True for item in case["relationships"]),
            f"{label}: relationship gates",
        )
        for output in case["outputs"]:
            channel_rule = acceptance["channels"][output["channel"]]
            require(output["size"] == [1024, 1024], f"{label}: output dimensions")
            require(output["encoding"] == channel_rule["encoding"], f"{label}: encoding")
            require(output["source"]["source"] == channel_rule["source"], f"{label}: source")
        cases.append({
            "id": case_id,
            "planHash": case["planHash"],
            "planMatchesGolden": True,
            "passedChannelGateCounts": counts,
            "passedRelationshipCount": len(case["relationships"]),
        })
    adapter = report["adapter"]
    require(
        adapter["backend"] == ("Vulkan" if backend == "software" else "Metal"),
        f"{report_path}: actual backend",
    )
    if backend == "software":
        require(report["softwareSource"]["revision"] == acceptance["softwareRevision"],
                f"{report_path}: pinned software revision")
        require("SwiftShader" in adapter["name"], f"{report_path}: software adapter")
    return {
        "material": material, "report": committed(report_path), "allGatesPassed": True,
        "capturedRunDirectory": str(Path(report["cases"][0]["outputs"][0]["path"]).parent.parent),
        "adapter": {"name": adapter["name"], "backend": adapter["backend"]},
        "cases": cases,
    }


def golden_check(backend):
    log_path = REPORTS / f"{backend}-golden-check.log"
    log = (ROOT / log_path).read_text()
    committed(log_path)
    require("xtask golden check" in log, f"{log_path}: command")
    require(
        log.count("Material machine checks: true; golden comparisons: true.") == 3,
        f"{log_path}: three successful material runs",
    )
    reports = [material_gates(material, backend) for material in MATERIALS]
    require(sum(len(report["cases"]) for report in reports) == 11, "eleven golden cases")
    for report in reports:
        require(
            f'Review: {report["capturedRunDirectory"]}/report.json' in log,
            f"{log_path}: material report belongs to this final run",
        )
    candidate_path = REPORTS / f"{backend}-candidate.json"
    candidate = document(candidate_path)
    input_count = historical_inputs(candidate["inputs"])
    baseline_count = bound_map(Path("fixtures/materials/wood/expected"),
                               candidate["previousBaseline"])
    committed(REPORTS / f"{backend}.json", candidate["artifacts"]["report.json"])
    return {
        "backend": backend, "log": committed(log_path), "caseCount": 11,
        "candidate": committed(candidate_path),
        "candidateInputHashCount": input_count,
        "candidateInputVerification": "Git source bytes at baseRevision",
        "candidatePreviousBaselineHashCount": baseline_count,
        "candidateFinalReportHashMatched": True,
        "materials": reports,
    }


def trace_2k(backend):
    directory = REPORTS / "2k" / backend
    trace_path = directory / "trace.json"
    trace = document(trace_path)
    selection = document(directory / "selection.json")
    render = document(directory / "render.json")
    require(trace["ok"] is True and render["ok"] is True, f"{trace_path}: success")
    require(trace["baselinesUnchanged"] is True, f"{trace_path}: baseline preservation")
    require(trace["size"] == render["execution"]["size"] == [2048, 2048], "2K size")
    require(trace["ranking"] == selection["ranking"], f"{trace_path}: retained selection")
    known_cases = {
        (material, case["id"])
        for material in MATERIALS
        for case in document(Path("fixtures/materials") / material / "acceptance.json")["cases"]
    }
    ranking = trace["ranking"]
    require(len(ranking) == 11, f"{trace_path}: ranking count")
    require({(r["material"], r["case"]) for r in ranking} == known_cases,
            f"{trace_path}: complete ranking")
    ordered = sorted(ranking, key=lambda r: (
        -r["estimates"]["peakBytes"], -r["passCount"], r["material"],
        r["case"] != "default", r["case"],
    ))
    require(ranking == ordered, f"{trace_path}: ranking order")
    for rank in ranking:
        inspection = document(
            directory / f'inspect-{rank["material"]}-{rank["case"]}.json'
        )
        plan = inspection["plan"]
        require(inspection["ok"] is True, f"{trace_path}: inspection success")
        require(plan["hash"] == rank["planHash"], f"{trace_path}: inspected plan hash")
        require(plan["estimates"] == rank["estimates"], f"{trace_path}: inspected estimates")
        require(len(plan["passes"]) == rank["passCount"], f"{trace_path}: pass count")
    require(
        trace["selected"] == {"material": ranking[0]["material"], "case": ranking[0]["case"],
                              "overrides": ranking[0]["overrides"]},
        f"{trace_path}: selected largest workload",
    )
    require(trace["selected"] == {"material": "wood", "case": "default", "overrides": {}},
            f"{trace_path}: recorded workload")
    execution = trace["execution"]
    adapter = execution["adapter"]
    require(
        adapter["backend"] == ("Vulkan" if backend == "software" else "Metal")
        and trace["policy"]["expectedAdapter"] in adapter["name"]
        and trace["policy"]["software"] is (backend == "software"),
        f"{trace_path}: selected adapter and explicit policy",
    )
    if backend == "software":
        require(trace["softwareSource"]["revision"] == document(
            Path("fixtures/materials/wood/expected/manifest.json")
        )["softwareRevision"], f"{trace_path}: pinned software revision")
    require(
        trace["requestedOutputs"] == [item["channel"] for item in render["outputs"]],
        f"{trace_path}: all requested output channels",
    )
    for key in ("allocations", "estimates", "planHash", "passCount", "size",
                "readbackBytes", "mappedBytes", "rgbaBytes", "pipelineCache"):
        require(execution[key] == render["execution"][key], f"{trace_path}: render {key}")
    require(trace["planHash"] == execution["planHash"] == ranking[0]["planHash"],
            f"{trace_path}: plan binding")
    allocation = execution["allocations"]
    estimate = execution["estimates"]
    budget = trace["transientBudgetBytes"]
    require(budget == 512 * 1024 * 1024, f"{trace_path}: documented budget")
    require(allocation["peakBytes"] == estimate["peakBytes"] <= budget,
            f"{trace_path}: bounded peak")
    require(allocation["cumulativeBytes"] == estimate["cumulativeBytes"],
            f"{trace_path}: cumulative allocation")
    for allocation_key, estimate_key in (
        ("textureBytes", "textureBytes"), ("uniformBytes", "uniformBytes"),
        ("stagingBytes", "cumulativeReadbackBytes"),
    ):
        require(allocation[allocation_key] == estimate[estimate_key],
                f"{trace_path}: {allocation_key}")
    require(allocation["liveBytes"] == allocation["reusedBytes"] == 0,
            f"{trace_path}: final live/reuse counts")
    require(allocation["releasedBytes"] == allocation["cumulativeBytes"],
            f"{trace_path}: release count")
    require(allocation["textureCount"] == allocation["uniformCount"] == execution["passCount"],
            f"{trace_path}: pass allocation count")
    require(allocation["stagingCount"] == len(trace["requestedOutputs"]) == 4,
            f"{trace_path}: sequential output count")
    require(allocation["peakStagingBytes"] == estimate["readbackBufferBytes"],
            f"{trace_path}: one live staging buffer")
    input_count = historical_inputs(trace["inputs"])
    artifact_count = bound_map(directory, trace["artifacts"])
    return {
        "backend": backend, "trace": committed(trace_path),
        "adapter": {"name": adapter["name"], "backend": adapter["backend"]},
        "rankedCaseCount": len(ranking), "selected": trace["selected"],
        "planHash": trace["planHash"], "size": trace["size"],
        "inputHashCount": input_count, "outputPngHashCount": artifact_count,
        "inputVerification": "Git source bytes at baseRevision",
        "transientBudgetBytes": budget, "allocations": allocation,
        "lifetimeDecision": trace["lifetimeDecision"],
        "allocationScope": trace["allocationScope"], "limits": trace["limits"],
        "allRecordedChecksPassed": True,
    }


def receipt():
    materials = [human_acceptance(material) for material in MATERIALS]
    goldens = [golden_check(backend) for backend in ("software", "metal")]
    traces = [trace_2k(backend) for backend in ("software", "metal")]
    all_files = VERIFIED | HISTORICAL_INPUTS
    return {
        "schemaVersion": 1,
        "kind": "m3-committed-acceptance-evidence-audit",
        "baseRevision": BASE,
        "ok": True,
        "scope": "Local verification of existing committed evidence; no new GPU run or human decision.",
        "remoteCi": {"status": "deferred", "remoteAcceptanceClaimed": False},
        "historicalCapturePolicy": (
            "Current reports/human-review.json status and humanDecision govern acceptance. "
            "Earlier pending captions and automated humanAcceptanceClaimed=false records "
            "remain historical capture evidence; they are not new rejection decisions."
        ),
        "humanAcceptance": materials,
        "goldenChecks": goldens,
        "allocationTraces": traces,
        "verifiedCommittedFileCount": len(all_files),
        "verifiedCommittedFiles": dict(sorted(all_files.items())),
        "currentEvidenceIntegrity": {
            "method": "Current retained evidence bytes must equal baseRevision and recorded hashes.",
            "workingTreeEqualsBaseRequired": True,
            "fileCount": len(VERIFIED),
            "files": sorted(VERIFIED),
        },
        "historicalSourceBindings": {
            "method": "Candidate and trace inputs hashes are checked against git show baseRevision:path.",
            "workingTreeEqualsBaseRequired": False,
            "fileCount": len(HISTORICAL_INPUTS),
            "files": sorted(HISTORICAL_INPUTS),
        },
        "filesVerifiedByBothMethods": len(VERIFIED.keys() & HISTORICAL_INPUTS.keys()),
        "verificationCommand": (
            "python3 docs/reviews/m3/native-consumer/verify_acceptance.py --check"
        ),
        "limits": [
            "Reported machine outcomes and their declared rules are verified, not recomputed from pixels.",
            "Human approval is read from the existing user decision records, not inferred from machine gates.",
            "The 2K evidence covers the selected workload and descriptor bytes, not all graphs or process RSS.",
            "Original temporary paths in capture reports are historical; this audit uses committed paths.",
            "Recorded GPU runs describe baseRevision sources; newer source changes require their own validation.",
            "This receipt does not claim M4 SDK, package publication, or remote CI acceptance.",
        ],
    }


def main():
    require(sys.argv[1:] in ([], ["--check"]), "usage: verify_acceptance.py [--check]")
    result = receipt()
    if sys.argv[1:]:
        saved = json.loads((ROOT / "docs/reviews/m3/acceptance.json").read_bytes())
        require(saved == result, "saved acceptance receipt differs from verified evidence")
        print(json.dumps({
            "ok": True, "baseRevision": BASE,
            "verifiedCommittedFileCount": len(VERIFIED | HISTORICAL_INPUTS),
            "currentEvidenceFileCount": len(VERIFIED),
            "historicalSourceFileCount": len(HISTORICAL_INPUTS),
            "humanAcceptedMaterials": 3, "goldenCasesPerBackend": 11, "allocationTraces": 2,
        }, sort_keys=True))
    else:
        print(json.dumps(result, indent=2, ensure_ascii=False, sort_keys=True))


if __name__ == "__main__":
    try:
        main()
    except (OSError, ValueError, KeyError, subprocess.CalledProcessError) as error:
        print(f"M3 acceptance verification failed: {error}", file=sys.stderr)
        sys.exit(1)
