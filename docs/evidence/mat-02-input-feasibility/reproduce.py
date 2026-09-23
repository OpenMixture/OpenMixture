"""Measure existing-node inputs only; CLI/wgpu generates every pixel."""
import datetime
import hashlib
import json
from pathlib import Path
import subprocess
import sys

import numpy as np
from PIL import Image, __version__ as pillow_version

binary = Path(sys.argv[1]).resolve()
destination = Path(sys.argv[2]).resolve()
def git(*args):
    return subprocess.check_output(["git", *args], text=True).strip()
def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()
if git("status", "--porcelain"):
    raise RuntimeError("Use a clean checkout with a freshly built CLI")
revision = git("rev-parse", "HEAD")
destination.mkdir(exist_ok=False)
plan_path = Path("fixtures/materials/painted-metal/qualification-plan.json")
graph_path = Path("fixtures/materials/painted-metal/graph-design.json")
plan = json.loads(plan_path.read_text())
graph = json.loads(graph_path.read_text())
d = plan["defaults"]
substitutions = dict(d, exposureInputMin=0.8*(1-d["exposureAmount"]),
    exposureInputMax=0.8*(1-d["exposureAmount"])+0.2,
    exposureOutputMin=0, exposureOutputMax=1,
    detailMin=1-d["detailAmount"], paintHeight=0.2+d["paintThickness"])
selected = {"macro", "exposure", "detailNoise", "detail", "paintColor", "substrateColor", "coatingColor", "coatingHeight"}
document = {"version": 1, "nodes": [], "edges": [], "exposedParameters": []}
def connect(source, node, port):
    source_node, source_port = source.split(".")
    document["edges"].append({"from": {"nodeId": source_node, "portId": source_port}, "to": {"nodeId": node, "portId": port}})
for node in graph["nodes"]:
    if node["id"] not in selected:
        continue
    parameters = {k: substitutions[v[1:]] if isinstance(v,str) and v.startswith("$") else v for k,v in node.get("parameters",{}).items()}
    document["nodes"].append({k:node[k] for k in ["id","type","version"]} | {"parameters": parameters})
    for port, source in node.get("inputs",{}).items():
        connect(source,node["id"],port)
document["nodes"].append({"id":"out","type":"material-output","version":1,"parameters":{}})
for channel,source in {"baseColor":"coatingColor.color", "height":"coatingHeight.value", "metallic":"exposure.value", "roughness":"detail.value"}.items():
    connect(source,"out",channel)
input_path = destination / "input.mix"
input_path.write_text(json.dumps(document, indent=2)+"\n", encoding="utf-8")
receipt = dict(schemaVersion=1, sourceRevision=revision, sourceDirty=False,
    createdAt=datetime.datetime.now(datetime.timezone.utc).isoformat(),
    buildCommand="cargo build --locked -p mixture-cli", buildProfile="dev",
    cliSha256=sha(binary), verifierSha256=sha(__file__), lockSha256=sha("Cargo.lock"),
    graphDesignSha256=sha(graph_path), qualificationPlanSha256=sha(plan_path),
    python=sys.version, numpy=np.__version__, pillow=pillow_version,
    rust=subprocess.check_output(["rustc","--version"],text=True).strip(),
    scope="Existing-node W/detail/coating input feasibility only. No morphology, subtraction, rust layer, normal or full material qualification.",
    maxDefaultDownsampleMeanError=plan["maxDefaultDownsampleMeanError"],
    methodology="RGBA8 byte-domain absolute error against 4x4 arithmetic box average of 1024 output, evaluated at 256; RGB for baseColor and red for scalar channels.",
    ok=False,runs=[],comparisons=[],files={})
def save():
    (destination/"receipt.json").write_text(json.dumps(receipt,indent=2)+"\n",encoding="utf-8")
save()
for backend in ["vulkan","dx12"]:
    arrays = {}
    for size in [256,1024]:
        output = destination / f"{backend}-{size}"
        args=[str(binary),"render",str(input_path),"--size",str(size),"--output","baseColor,height,metallic,roughness","--backend",backend,"--out",str(output),"--json"]
        result=subprocess.run(args,capture_output=True,text=True)
        (destination/f"{backend}-{size}.json").write_text(result.stdout,encoding="utf-8")
        (destination/f"{backend}-{size}.stderr.log").write_text(result.stderr,encoding="utf-8")
        if result.returncode:
            raise RuntimeError(result.stdout+result.stderr)
        report=json.loads(result.stdout)
        if not report["ok"] or report["context"]["adapter"]["backend"].lower()!=backend:
            raise RuntimeError("Unexpected execution or adapter backend")
        receipt["runs"].append(dict(backend=backend,size=size,args=args[1:],adapter=report["context"]["adapter"],planHash=report.get("planHash")))
        arrays[size]={}
        for channel in ["baseColor","height","metallic","roughness"]:
            with Image.open(output/f"{channel}.png") as image:
                if image.mode!="RGBA" or image.size!=(size,size):
                    raise RuntimeError("Expected exact-size RGBA output")
                arrays[size][channel]=np.asarray(image,dtype=np.float64)
        save()
    errors={}
    for channel in arrays[256]:
        large=arrays[1024][channel].reshape(256,4,256,4,4).mean(axis=(1,3))
        components=3 if channel=="baseColor" else 1
        per_component=np.abs(arrays[256][channel]-large).mean(axis=(0,1))[:components]
        errors[channel]=dict(meanPerComponent=per_component.tolist(),maxMean=float(per_component.max()))
    passed=all(errors[c]["maxMean"]<=plan["maxDefaultDownsampleMeanError"] for c in ["baseColor","height"])
    receipt["comparisons"].append(dict(backend=backend,errors=errors,inputTargetPassed=passed))
    save()
if git("status","--porcelain") or git("rev-parse","HEAD")!=revision or sha(binary)!=receipt["cliSha256"]:
    raise RuntimeError("Source or binary changed during execution")
for path in sorted(destination.rglob("*")):
    if path.is_file() and path.name!="receipt.json":
        receipt["files"][path.relative_to(destination).as_posix()]=sha(path)
receipt["ok"]=all(item["inputTargetPassed"] for item in receipt["comparisons"])
save()
print(json.dumps({"ok":receipt["ok"],"sourceRevision":revision,"comparisons":receipt["comparisons"]},indent=2))
if not receipt["ok"]:
    sys.exit(1)
