//! Measurements and contact sheets of already-rendered pixels, never a renderer.

use super::model::{Change, Structure, Tolerance};
use crate::TaskResult;
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    fs::File,
    io::{BufReader, BufWriter},
    path::Path,
};

#[derive(Clone)]
pub(super) struct Image {
    pub size: u32,
    pub pixels: Vec<u8>,
}

impl Image {
    pub fn read(path: &Path, size: u32, encoding: &str) -> TaskResult<Self> {
        // Bound decompression before allocating, even for an accidentally corrupt golden.
        let mut decoder = png::Decoder::new(BufReader::new(File::open(path)?));
        decoder.set_limits(png::Limits {
            bytes: 32 * 1024 * 1024,
        });
        let mut reader = decoder.read_info()?;
        let info = reader.info();
        if info.width != size
            || info.height != size
            || info.color_type != png::ColorType::Rgba
            || info.bit_depth != png::BitDepth::Eight
            || info.animation_control.is_some()
            || (encoding == "rgba8-srgb" && info.srgb.is_none())
            || (encoding == "rgba8-linear"
                && (info.srgb.is_some()
                    || info.gama_chunk.map(|v| v.into_scaled()) != Some(100_000)))
        {
            return Err(format!("{}: wrong PNG dimensions/encoding", path.display()).into());
        }
        let mut pixels = vec![
            0;
            (size as usize)
                .checked_mul(size as usize)
                .and_then(|n| n.checked_mul(4))
                .ok_or("image size overflow")?
        ];
        let output = reader.next_frame(&mut pixels)?;
        if output.buffer_size() != pixels.len() {
            return Err("truncated material PNG".into());
        }
        Ok(Self { size, pixels })
    }

    fn pixel(&self, x: u32, y: u32) -> &[u8] {
        let start = ((y as usize * self.size as usize) + x as usize) * 4;
        &self.pixels[start..start + 4]
    }

    pub fn statistics(&self) -> Value {
        let (mut min, mut max, mut total) = ([255u8; 4], [0u8; 4], [0u64; 4]);
        for pixel in self.pixels.as_chunks::<4>().0.iter() {
            for c in 0..4 {
                min[c] = min[c].min(pixel[c]);
                max[c] = max[c].max(pixel[c]);
                total[c] += u64::from(pixel[c]);
            }
        }
        let count = self.pixels.len() / 4;
        json!({"min":min,"max":max,"mean":total.map(|n| n as f64 / count as f64),"units":"RGBA8 encoded component values (0..255)"})
    }
}

pub(super) fn compare(before: &Image, after: &Image, tolerance: Tolerance) -> Value {
    assert_eq!(before.size, after.size, "internal comparison dimensions");
    let (mut max, mut sum, mut changed, mut above) = ([0u8; 4], [0u64; 4], 0u64, 0u64);
    for (a, b) in before
        .pixels
        .as_chunks::<4>()
        .0
        .iter()
        .zip(after.pixels.as_chunks::<4>().0.iter())
    {
        let mut pixel_max = 0;
        for c in 0..4 {
            let delta = a[c].abs_diff(b[c]);
            max[c] = max[c].max(delta);
            sum[c] += u64::from(delta);
            pixel_max = pixel_max.max(delta);
        }
        changed += u64::from(pixel_max > 0);
        above += u64::from(pixel_max > tolerance.pixel_threshold);
    }
    let count = (after.pixels.len() / 4) as f64;
    let maximum = max.into_iter().max().unwrap_or(0);
    let mean = sum.iter().sum::<u64>() as f64 / (count * 4.0);
    let ratio = above as f64 / count;
    json!({"status":"compared","ok":maximum <= tolerance.max_absolute && mean <= tolerance.mean_absolute && ratio <= tolerance.max_changed_pixel_ratio,
        "maxAbsolute":maximum,"meanAbsolute":mean,"maxByComponent":max,"meanByComponent":sum.map(|n| n as f64 / count),
        "changedPixelRatio":changed as f64 / count,"aboveThresholdPixelRatio":ratio,"tolerance":tolerance})
}

pub(super) fn structure(image: &Image, rule: &Structure) -> Value {
    match rule {
        Structure::Uniform { rgba, tolerance } => {
            let maximum = image
                .pixels
                .as_chunks::<4>()
                .0
                .iter()
                .flat_map(|pixel| pixel.iter().zip(rgba).map(|(a, b)| a.abs_diff(*b)))
                .max()
                .unwrap_or(0);
            json!({"kind":"uniform","ok":maximum <= *tolerance,"expected":rgba,"maxAbsolute":maximum,"tolerance":tolerance,
                "seam":seam(image, [1,1])})
        }
        Structure::Alternating {
            cells,
            min_contrast,
            max_balance_error,
        } => {
            let mut colors = BTreeMap::<[u8; 4], u64>::new();
            for pixel in image.pixels.as_chunks::<4>().0 {
                *colors.entry(*pixel).or_default() += 1;
            }
            let count = (image.pixels.len() / 4) as f64;
            let balance = colors
                .values()
                .map(|n| (*n as f64 / count - 0.5).abs())
                .fold(0.0, f64::max);
            let contrast = colors
                .keys()
                .next()
                .zip(colors.keys().next_back())
                .map(|(a, b)| (0..3).map(|c| a[c].abs_diff(b[c])).max().unwrap_or(0))
                .unwrap_or(0);
            let mut counts = [[u32::MAX, 0]; 2];
            let mut period_error = 0u8;
            for axis in 0..2 {
                let period = image.size / cells[axis] * 2;
                for row in 0..image.size {
                    let mut transitions = 0;
                    for column in 0..image.size {
                        let xy = |c| if axis == 0 { [c, row] } else { [row, c] };
                        let [x, y] = xy(column);
                        let pixel = image.pixel(x, y);
                        let [x, y] = xy((column + 1) % image.size);
                        transitions += u32::from(pixel != image.pixel(x, y));
                        let [x, y] = xy((column + period) % image.size);
                        for (a, b) in pixel.iter().zip(image.pixel(x, y)) {
                            period_error = period_error.max(a.abs_diff(*b));
                        }
                    }
                    counts[axis][0] = counts[axis][0].min(transitions);
                    counts[axis][1] = counts[axis][1].max(transitions);
                }
            }
            let seams = seam(image, [image.size / cells[0], image.size / cells[1]]);
            let ok = colors.len() == 2
                && balance <= *max_balance_error
                && contrast >= *min_contrast
                && counts == [[cells[0]; 2], [cells[1]; 2]]
                && period_error == 0
                && seams["maxExcess"] == 0;
            json!({"kind":"alternating","ok":ok,"distinctColors":colors.len(),"balanceError":balance,
                "contrast":contrast,"transitionsIncludingWrap":counts,"expectedCells":cells,
                "periodMaxError":period_error,"seam":seams})
        }
    }
}

// A checker has an intentional jump at each tile boundary. Compare wrap jumps with
// interior tile-boundary jumps; requiring equal first/last pixels would reject it.
fn seam(image: &Image, interior: [u32; 2]) -> Value {
    let (mut wrap, mut inside, mut excess) = ([0u64; 2], [0u64; 2], 0u8);
    for axis in 0..2 {
        for row in 0..image.size {
            let pixel = |c| {
                if axis == 0 {
                    image.pixel(c, row)
                } else {
                    image.pixel(row, c)
                }
            };
            for c in 0..3 {
                let outer = pixel(0)[c].abs_diff(pixel(image.size - 1)[c]);
                let inner = pixel(interior[axis])[c].abs_diff(pixel(interior[axis] - 1)[c]);
                wrap[axis] += u64::from(outer);
                inside[axis] += u64::from(inner);
                excess = excess.max(outer.abs_diff(inner));
            }
        }
    }
    let samples = f64::from(image.size) * 3.0;
    json!({"wrapMeanAbsolute":wrap.map(|n| n as f64/samples),"interiorMeanAbsolute":inside.map(|n| n as f64/samples),"maxExcess":excess})
}

pub(super) fn causality(default: &Image, variant: &Image, rule: &Change) -> Value {
    let comparison = compare(default, variant, Tolerance::EXACT);
    let changed = comparison["changedPixelRatio"].as_f64().unwrap_or(0.0);
    // Red is the scalar payload. This is an output measurement, not a material formula.
    let red_mean = |image: &Image| {
        image
            .pixels
            .as_chunks::<4>()
            .0
            .iter()
            .map(|p| u64::from(p[0]))
            .sum::<u64>() as f64
            / ((image.pixels.len() / 4) as f64 * 255.0)
    };
    let delta = red_mean(variant) - red_mean(default);
    let ok = match rule {
        Change::Unchanged => changed == 0.0,
        Change::Changed { min_pixel_ratio } => changed >= *min_pixel_ratio,
        Change::MeanIncreases { min_delta } => delta >= *min_delta,
    };
    json!({"ok":ok,"rule":rule,"changedPixelRatio":changed,"redMeanDeltaNormalized":delta})
}

fn write_png(path: &Path, width: u32, height: u32, pixels: &[u8]) -> TaskResult {
    let mut encoder = png::Encoder::new(BufWriter::new(File::create(path)?), width, height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.set_source_srgb(png::SrgbRenderingIntent::Perceptual);
    let mut writer = encoder.write_header()?;
    writer.write_image_data(pixels)?;
    writer.finish()?;
    Ok(())
}

/// Rows retain full 1K artifacts separately; the review sheet uses nearest sampling.
type SheetRow = (String, Vec<(String, Option<Image>)>);

pub(super) fn sheet(path: &Path, rows: &[SheetRow]) -> TaskResult {
    const TILE: u32 = 192;
    const LABEL: u32 = 24;
    let columns = rows
        .iter()
        .map(|(_, images)| images.len())
        .max()
        .ok_or("empty contact sheet")? as u32;
    let width = columns * TILE;
    let height = rows.len() as u32 * (TILE + LABEL * 2);
    let mut pixels = vec![24; (width * height * 4) as usize];
    for p in pixels.as_chunks_mut::<4>().0.iter_mut() {
        p[3] = 255;
    }
    for (row, (label, images)) in rows.iter().enumerate() {
        let top = row as u32 * (TILE + LABEL * 2);
        draw_text(&mut pixels, width, 6, top + 7, label);
        for (column, (title, image)) in images.iter().enumerate() {
            let left = column as u32 * TILE;
            draw_text(&mut pixels, width, left + 6, top + LABEL + 7, title);
            for y in 0..TILE {
                for x in 0..TILE {
                    let source = image
                        .as_ref()
                        .map(|i| i.pixel(x * i.size / TILE, y * i.size / TILE));
                    let missing = if (x + y) / 12 % 2 == 0 {
                        [65, 65, 65, 255]
                    } else {
                        [45, 45, 45, 255]
                    };
                    let start = (((top + LABEL * 2 + y) * width + left + x) * 4) as usize;
                    pixels[start..start + 4].copy_from_slice(source.unwrap_or(&missing));
                }
            }
        }
    }
    write_png(path, width, height, &pixels)
}

pub(super) fn difference(before: &Image, after: &Image) -> Image {
    Image {
        size: after.size,
        pixels: before
            .pixels
            .as_chunks::<4>()
            .0
            .iter()
            .zip(after.pixels.as_chunks::<4>().0.iter())
            .flat_map(|(a, b)| {
                let alpha = a[3].abs_diff(b[3]);
                [
                    a[0].abs_diff(b[0]).max(alpha).saturating_mul(4),
                    a[1].abs_diff(b[1]).max(alpha).saturating_mul(4),
                    a[2].abs_diff(b[2]).max(alpha).saturating_mul(4),
                    255,
                ]
            })
            .collect(),
    }
}

pub(super) fn tiled(image: &Image) -> Image {
    let size = image.size;
    let mut pixels = Vec::with_capacity(image.pixels.len());
    for y in 0..size {
        for x in 0..size {
            pixels.extend_from_slice(image.pixel((x * 2) % size, (y * 2) % size));
        }
    }
    Image { size, pixels }
}

// Small repository-owned label glyphs keep exported evidence readable without a font dependency.
fn draw_text(pixels: &mut [u8], width: u32, left: u32, top: u32, text: &str) {
    for (index, character) in text.to_ascii_uppercase().chars().enumerate() {
        let glyph = match character {
            'A' => [14, 17, 17, 31, 17, 17, 17],
            'B' => [30, 17, 17, 30, 17, 17, 30],
            'C' => [14, 17, 16, 16, 16, 17, 14],
            'D' => [30, 17, 17, 17, 17, 17, 30],
            'E' => [31, 16, 16, 30, 16, 16, 31],
            'F' => [31, 16, 16, 30, 16, 16, 16],
            'G' => [14, 17, 16, 23, 17, 17, 15],
            'H' => [17, 17, 17, 31, 17, 17, 17],
            'I' => [14, 4, 4, 4, 4, 4, 14],
            'J' => [7, 2, 2, 2, 18, 18, 12],
            'K' => [17, 18, 20, 24, 20, 18, 17],
            'L' => [16, 16, 16, 16, 16, 16, 31],
            'M' => [17, 27, 21, 21, 17, 17, 17],
            'N' => [17, 25, 21, 19, 17, 17, 17],
            'O' => [14, 17, 17, 17, 17, 17, 14],
            'P' => [30, 17, 17, 30, 16, 16, 16],
            'Q' => [14, 17, 17, 17, 21, 18, 13],
            'R' => [30, 17, 17, 30, 20, 18, 17],
            'S' => [15, 16, 16, 14, 1, 1, 30],
            'T' => [31, 4, 4, 4, 4, 4, 4],
            'U' => [17, 17, 17, 17, 17, 17, 14],
            'V' => [17, 17, 17, 17, 17, 10, 4],
            'W' => [17, 17, 17, 21, 21, 21, 10],
            'X' => [17, 17, 10, 4, 10, 17, 17],
            'Y' => [17, 17, 10, 4, 4, 4, 4],
            'Z' => [31, 1, 2, 4, 8, 16, 31],
            '0' => [14, 17, 19, 21, 25, 17, 14],
            '1' => [4, 12, 4, 4, 4, 4, 14],
            '2' => [14, 17, 1, 2, 4, 8, 31],
            '3' => [30, 1, 1, 14, 1, 1, 30],
            '4' => [2, 6, 10, 18, 31, 2, 2],
            '5' => [31, 16, 16, 30, 1, 1, 30],
            '6' => [14, 16, 16, 30, 17, 17, 14],
            '7' => [31, 1, 2, 4, 8, 8, 8],
            '8' => [14, 17, 17, 14, 17, 17, 14],
            '9' => [14, 17, 17, 15, 1, 1, 14],
            '-' => [0, 0, 0, 31, 0, 0, 0],
            _ => [0; 7],
        };
        for (y, bits) in glyph.into_iter().enumerate() {
            for x in 0..5 {
                let column = left + index as u32 * 6 + x;
                let start = (((top + y as u32) * width + column) * 4) as usize;
                if bits & (1 << (4 - x)) != 0 && column < width && start + 4 <= pixels.len() {
                    pixels[start..start + 4].copy_from_slice(&[225, 232, 240, 255]);
                }
            }
        }
    }
}
