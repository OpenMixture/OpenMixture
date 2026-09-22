use mixture_core::ImageBinding;
pub const SOURCE: &[u8] = include_bytes!("../image-input.mix");
pub const SIZES: [[u32; 2]; 2] = [[1024, 1024], [65, 3]];
pub fn pixels([w, h]: [u32; 2]) -> Vec<u8> {
    (0..w * h)
        .flat_map(|i| {
            let x = i % w;
            let y = i / w;
            let r = if w == 1024 {
                let u = x % 64;
                let v = y % 64;
                ((u.min(63 - u) * 5 + v.min(63 - v) * 3) % 256) as u8
            } else {
                ((x * 17 + y * 71) % 256) as u8
            };
            [r, 19, 201, 0]
        })
        .collect()
}
pub fn binding(size: [u32; 2], data: &[u8]) -> ImageBinding<'_> {
    ImageBinding {
        id: "heightSource",
        width: size[0],
        height: size[1],
        format: "rgba8-linear",
        bytes_per_row: u64::from(size[0]) * 4,
        data,
    }
}
pub fn archive(size: [u32; 2]) -> Vec<u8> {
    mixture_asset::write(SOURCE, &[binding(size, &pixels(size))], &Default::default()).unwrap()
}
