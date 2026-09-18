from pathlib import Path
r=Path('tmp/path-reduction');s=(r/'warp-trace.wgsl').read_text(); pre=s[:s.index('outputs[o]=uv.x;')];
for name,expr in [('uv','sample_uv.x'),('p','p.x'),('t','t.x'),('mix','mix(inputs[o+4u],inputs[o+5u],t.x)'),('half','mixture_half(mix(inputs[o+4u],inputs[o+5u],t.x))')]:
 (r/f'warp-only-{name}.wgsl').write_text(pre+'outputs[o]='+expr+';}')
