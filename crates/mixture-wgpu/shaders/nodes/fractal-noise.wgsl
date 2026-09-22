// v1/v2: periodic value/cellular noise, center-sampled UV with v down, octave doubling.
struct Parameters {
    seed: u32, scale: u32, octaves: u32, basis: u32,
    persistence: f32, _pad0: f32, _pad1: f32, _pad2: f32,
}
@group(0) @binding(0) var<uniform> parameters: Parameters;
@group(0) @binding(1) var output: texture_storage_2d<rgba16float, write>;

// Unsigned arithmetic intentionally wraps modulo 2^32. No transcendental hash.
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
// Q0.24 values include both endpoints [0, 2^24]. Every intermediate is u32.
const Q: u32 = 16777216u;
fn qmul(a: u32, b: u32) -> u32 {
    // 12-bit limbs retain the exact 48-bit product without optional u64 support.
    let low = (a & 4095u) * (b & 4095u);
    let middle = (a >> 12u) * (b & 4095u) + (b >> 12u) * (a & 4095u) + (low >> 12u);
    let high = (a >> 12u) * (b >> 12u) + (middle >> 12u);
    let remainder = ((middle & 4095u) << 12u) | (low & 4095u);
    return high + select(0u, 1u, remainder > 8388608u || (remainder == 8388608u && (high & 1u) == 1u));
}
fn qmix(a: u32, b: u32, t: u32) -> u32 {
    if b >= a { return a + qmul(b - a, t); }
    return a - qmul(a - b, t);
}
fn qfade(t: u32) -> u32 {
    // de Casteljau evaluation of quintic Bezier controls [0,0,0,1,1,1].
    // Unlike signed Horner evaluation, every intermediate remains in [0,Q].
    let b1 = qmul(t, t);
    let b2 = qmix(t, Q, t);
    let c0 = qmul(b1, t);
    let c1 = qmix(b1, b2, t);
    let c2 = qmix(b2, Q, t);
    return qmix(qmix(c0, c1, t), qmix(c1, c2, t), t);
}
fn qratio(numerator: u32, denominator: u32) -> u32 {
    if numerator == denominator { return Q; }
    var remainder = numerator;
    var result = 0u;
    for (var bit = 0u; bit < 24u; bit++) {
        remainder *= 2u;
        result *= 2u;
        if remainder >= denominator { remainder -= denominator; result += 1u; }
    }
    return result;
}
fn qcoordinate(pixel: u32, size: u32, period: u32) -> vec2<u32> {
    let numerator = (2u * pixel + 1u) * period;
    let denominator = 2u * size;
    return vec2<u32>(numerator / denominator, qratio(numerator % denominator, denominator));
}
fn fixed_value_noise(pixel: vec2<u32>, size: vec2<u32>, period: u32, seed: u32) -> u32 {
    let x = qcoordinate(pixel.x, size.x, period);
    let y = qcoordinate(pixel.y, size.y, period);
    let cell = vec2<u32>(x.x, y.x);
    let tx = qfade(x.y);
    let ty = qfade(y.y);
    let a = u32(lattice(cell, period, seed) * f32(Q));
    let b = u32(lattice(cell + vec2<u32>(1u, 0u), period, seed) * f32(Q));
    let c = u32(lattice(cell + vec2<u32>(0u, 1u), period, seed) * f32(Q));
    let d = u32(lattice(cell + vec2<u32>(1u, 1u), period, seed) * f32(Q));
    return qmix(qmix(a,b,tx), qmix(c,d,tx), ty);
}
fn fixed_fractal(pixel: vec2<u32>, size: vec2<u32>) -> f32 {
    let persistence = u32(parameters.persistence * f32(Q));
    var weight = Q;
    var sum = 0u;
    var weights = 0u;
    var period = parameters.scale;
    for (var octave = 0u; octave < parameters.octaves; octave++) {
        let seed = avalanche(parameters.seed + octave * 0x9e3779b9u);
        sum += qmul(fixed_value_noise(pixel, size, period, seed), weight);
        weights += weight;
        weight = qmul(weight, persistence);
        period *= 2u;
    }
    return f32(qratio(sum, weights)) * (1.0 / 16777216.0);
}

@compute @workgroup_size(8, 8, 1)
fn fractal_noise(@builtin(global_invocation_id) id: vec3<u32>) {
    let size = textureDimensions(output);
    if any(id.xy >= size) { return; }
    if parameters.basis == 2u {
        textureStore(output, vec2<i32>(id.xy), mixture_half4(vec4<f32>(fixed_fractal(id.xy, size), 0.0, 0.0, 1.0)));
        return;
    }
    let uv = (vec2<f32>(id.xy) + vec2<f32>(0.5)) / vec2<f32>(size);
    var sum = 0.0;
    var weights = 0.0;
    var weight = 1.0;
    var period = parameters.scale;
    for (var octave = 0u; octave < parameters.octaves; octave++) {
        let seed = avalanche(parameters.seed + octave * 0x9e3779b9u);
        var value: f32;
        if parameters.basis == 0u {
            value = value_noise(uv, period, seed);
        } else {
            value = cellular_noise(uv, period, seed);
        }
        sum += value * weight;
        weights += weight;
        weight *= parameters.persistence;
        period *= 2u;
    }
    textureStore(output, vec2<i32>(id.xy), mixture_half4(vec4<f32>(clamp(sum / weights, 0.0, 1.0), 0.0, 0.0, 1.0)));
}
