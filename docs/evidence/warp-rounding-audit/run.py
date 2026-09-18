"""Run the predeclared diagnostic plans with the sole wgpu executor."""
import subprocess, sys, json, platform, os
from pathlib import Path
root=Path(__file__).resolve().parents[3]
here=Path(__file__).resolve().parent
out=root/'tmp/warp-rounding-audit'
subprocess.run([sys.executable,str(here/'generate.py')],check=True)
runner=root/'target/release/examples/rounding_texture_probe'
if os.name=='nt': runner=runner.with_suffix('.exe')
for plan in sorted(out.glob('*.json')):
    if plan.name in ['cases.json','plans-sha256.json','execution.json']: continue
    subprocess.run([str(runner),str(plan),str(out/plan.stem)],check=True)
(out/'execution.json').write_text(json.dumps({'platform':platform.platform(),
    'revision':subprocess.check_output(['git','rev-parse','HEAD'],cwd=root,text=True).strip(),
    'backend':os.environ.get('MIXTURE_GPU_BACKEND','dx12')},indent=2)+'\n')

subprocess.run([sys.executable,str(here/"analyze.py")],check=True)
