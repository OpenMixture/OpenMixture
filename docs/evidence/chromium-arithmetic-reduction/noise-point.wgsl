// Round f32 to binary16, ties to even, while retaining an exactly representable
// f32 result. Texture storage then cannot choose a different rounding direction.
// Integer rounding avoids backend-dependent quantizeToF16 implementations.
fn mixture_half(value: f32) -> f32 {
    let bits = bitcast<u32>(value);
    let sign = bits & 0x80000000u;
    let magnitude = bits & 0x7fffffffu;
    if magnitude >= 0x7f800000u { return value; } // Preserve invalid data for readback diagnostics.
    if magnitude >= 0x38800000u {
        let rounded = (magnitude + 0xfffu + ((magnitude >> 13u) & 1u)) & 0xffffe000u;
        return bitcast<f32>(sign | rounded);
    }
    if magnitude < 0x33000000u { return bitcast<f32>(sign); }
    // Half subnormals are integer multiples of 2^-24. Here shift is 14..24.
    let shift = 126u - (magnitude >> 23u);
    let mantissa = (magnitude & 0x7fffffu) | 0x800000u;
    let rounded = (mantissa + (1u << (shift - 1u)) - 1u + ((mantissa >> shift) & 1u)) >> shift;
    return bitcast<f32>(sign | bitcast<u32>(f32(rounded) * (1.0 / 16777216.0)));
}
fn mixture_half4(value: vec4<f32>) -> vec4<f32> {
    return vec4<f32>(mixture_half(value.x), mixture_half(value.y), mixture_half(value.z), mixture_half(value.w));
}

@group(0) @binding(0) var<storage, read> inputs: array<f32>;
@group(0) @binding(1) var<storage, read_write> outputs: array<f32>;
fn avalanche(value: u32) -> u32 {
    var h = value;
    h = (h ^ (h >> 16u)) * 0x7feb352du;
    h = (h ^ (h >> 15u)) * 0x846ca68bu;
    return h ^ (h >> 16u);
}
fn lattice(cell: vec2<u32>, period: u32, seed: u32) -> f32 {
    let wrapped = cell % vec2<u32>(period);
    let h = avalanche(seed ^ avalanche(wrapped.x + 0x9e3779b9u) ^ avalanche(wrapped.y + 0x85ebca6bu));
    // The high 24 bits convert exactly to f32, then scale by an exact power of two.
    return f32(h >> 8u) * (1.0 / 16777216.0);
}
fn value_noise(uv: vec2<f32>, period: u32, seed: u32) -> f32 {
    let p = uv * f32(period);
    let cell = vec2<u32>(floor(p));
    let t = fract(p);
    // Quintic fade has zero first/second derivatives at lattice boundaries.
    let fade = t * t * t * (t * (t * 6.0 - 15.0) + 10.0);
    let a = lattice(cell, period, seed);
    let b = lattice(cell + vec2<u32>(1u, 0u), period, seed);
    let c = lattice(cell + vec2<u32>(0u, 1u), period, seed);
    let d = lattice(cell + vec2<u32>(1u, 1u), period, seed);
    return mix(mix(a, b, fade.x), mix(c, d, fade.x), fade.y);
}
fn cellular_noise(uv: vec2<f32>, period: u32, seed: u32) -> f32 {
    let p = uv * f32(period);
    let cell = vec2<i32>(floor(p));
    var nearest = 100.0;
    var second = 100.0;
    // A bounded local cellular field: one site in the central 60% of each cell.
    // Coordinates remain unwrapped for distance; only site identity wraps.
    for (var y = -1; y <= 1; y++) {
        for (var x = -1; x <= 1; x++) {
            let neighbor = cell + vec2<i32>(x, y);
            let wrapped = vec2<u32>((neighbor + vec2<i32>(i32(period))) % vec2<i32>(i32(period)));
            let jitter = vec2<f32>(lattice(wrapped, period, seed), lattice(wrapped, period, seed ^ 0x68bc21ebu));
            let delta = vec2<f32>(neighbor) + vec2<f32>(0.2) + 0.6 * jitter - p;
            let distance = dot(delta, delta);
            if distance < nearest {
                second = nearest;
                nearest = distance;
            } else {
                second = min(second, distance);
            }
        }
    }
    return clamp(sqrt(second) - sqrt(nearest), 0.0, 1.0);
}

@compute @workgroup_size(64) fn probe(@builtin(global_invocation_id) id: vec3<u32>) {
 let i=id.x; if i>=arrayLength(&inputs) {return;}
 let pixel=270u;
 let uv=(vec2<f32>(f32(pixel%1024u),f32(pixel/1024u))+vec2<f32>(0.5))/1024.0;
 var sum=inputs[i]; var weights=0.0; var weight=1.0; var period=64u;
 for(var octave=0u;octave<3u;octave++) {
  let seed=avalanche(271828u+octave*0x9e3779b9u);
  let value=cellular_noise(uv,period,seed);
  sum+=value*weight;weights+=weight;weight*=0.35;period*=2u;
 }
 let value=clamp(sum/weights,0.0,1.0);
 outputs[i]=select(value,mixture_half(value),i%2u==1u);
}
