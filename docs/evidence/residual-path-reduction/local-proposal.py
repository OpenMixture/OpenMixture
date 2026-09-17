from pathlib import Path
r=Path('tmp/path-reduction');s=(r/'warp-only-mix.wgsl').read_text();s=s.replace('let t=fract(p);','let t=fract((2.0*field-1.0)*strength*1024.0);');(r/'warp-local-texel.wgsl').write_text(s)
