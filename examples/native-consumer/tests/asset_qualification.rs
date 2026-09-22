//! Public asset/loose equivalence, also executed against isolated Cargo archives.
mod asset_support;
use asset_support::{SIZES, SOURCE, archive, binding, pixels};
use mixture_asset::{AssetView, OwnedAsset};
use mixture_core::{CompileRequest, MaterialDocument, OutputChannel, prepare};
use mixture_wgpu::{BackendPreference, GpuContext, GpuContextOptions, Renderer};
use serde_json::json;
fn request(size: [u32; 2], weight: f64) -> CompileRequest {
    CompileRequest {
        size,
        outputs: vec![OutputChannel::Height, OutputChannel::Normal],
        overrides: [("detailWeight".into(), json!(weight))].into(),
        ..Default::default()
    }
}
#[test]
fn package_matrix_preserves_public_preparation_and_ownership() {
    let doc = MaterialDocument::decode(SOURCE, &Default::default())
        .unwrap()
        .into_validated(&Default::default())
        .unwrap();
    for size in SIZES {
        let bytes = archive(size);
        let data = pixels(size);
        for weight in [0., 0.25, 0.5, 1.] {
            let req = request(size, weight);
            let mut transport = bytes.clone();
            let owned = OwnedAsset::copy_from(&transport, &Default::default()).unwrap();
            transport.fill(0);
            let packaged = owned.prepare(&req).unwrap();
            drop(owned);
            let loose = prepare(&doc, &req, &[binding(size, &data)], &Default::default()).unwrap();
            assert_eq!(packaged.plan().hash(), loose.plan().hash());
            assert_eq!(packaged.resources()[0].data(), data);
        }
    }
}
#[test]
#[ignore = "requires explicit GPU; source and isolated package gpu-smoke run this"]
fn package_matrix_gpu() {
    let backend = match std::env::var("MIXTURE_GPU_BACKEND").as_deref() {
        Ok("dx12") => BackendPreference::Dx12,
        Ok("vulkan") => BackendPreference::Vulkan,
        Ok("metal") => BackendPreference::Metal,
        _ => panic!("explicit backend required"),
    };
    let mut renderer = Renderer::new(
        pollster::block_on(GpuContext::request(GpuContextOptions {
            backend,
            software_adapter: std::env::var("MIXTURE_GPU_SOFTWARE").as_deref() == Ok("1"),
            ..Default::default()
        }))
        .unwrap(),
    );
    let doc = MaterialDocument::decode(SOURCE, &Default::default())
        .unwrap()
        .into_validated(&Default::default())
        .unwrap();
    let mut rows = Vec::new();
    let mut retained = Vec::new();
    for size in SIZES {
        let data = pixels(size);
        let bytes = archive(size);
        let identity = AssetView::load(&bytes, &Default::default())
            .unwrap()
            .inspect()
            .package_sha256;
        for weight in [0., 0.25, 0.5, 1.] {
            let req = request(size, weight);
            // Rejection must not poison the next load or renderer call.
            assert!(OwnedAsset::copy_from(&bytes[..bytes.len() - 1], &Default::default()).is_err());
            let loose = prepare(&doc, &req, &[binding(size, &data)], &Default::default()).unwrap();
            let reference = pollster::block_on(renderer.render_prepared(&loose)).unwrap();
            let mut comparisons = Vec::new();
            for iteration in 0..2 {
                let asset = OwnedAsset::copy_from(&bytes, &Default::default()).unwrap();
                let prepared = asset.prepare(&req).unwrap();
                drop(asset);
                assert_eq!(prepared.plan().hash(), loose.plan().hash());
                let result = pollster::block_on(renderer.render_prepared(&prepared)).unwrap();
                drop(prepared);
                assert_eq!(result.report().allocations.live_bytes, 0);
                for (a, b) in result.channels().iter().zip(reference.channels()) {
                    assert_eq!(a.pixels(), b.pixels());
                }
                if iteration == 0 {
                    if let Ok(dir) = std::env::var("MIXTURE_ASSET_BROWSER_DIR") {
                        for c in result.channels() {
                            let path = std::path::Path::new(&dir).join(format!(
                                "asset-{}x{}-{weight}-{}.png",
                                size[0],
                                size[1],
                                c.channel.as_str()
                            ));
                            let mut reader = png::Decoder::new(std::io::BufReader::new(
                                std::fs::File::open(path).unwrap(),
                            ))
                            .read_info()
                            .unwrap();
                            assert_eq!([reader.info().width, reader.info().height], size);
                            assert_eq!(reader.info().color_type, png::ColorType::Rgba);
                            assert_eq!(reader.info().bit_depth, png::BitDepth::Eight);
                            let mut browser = vec![0; reader.output_buffer_size().unwrap()];
                            reader.next_frame(&mut browser).unwrap();
                            assert_eq!(browser.len(), c.pixels().len());
                            let max = browser
                                .iter()
                                .zip(c.pixels())
                                .map(|(a, b)| a.abs_diff(*b))
                                .max()
                                .unwrap();
                            comparisons.push(json!({"channel":c.channel.as_str(),"maxComponentDelta":max,"limit":1}));
                        }
                    }
                    rows.push(json!({"size":size,"weight":weight,"packageSha256":identity,"planHash":loose.plan().hash(),"exactLoosePixels":true,"repeatedLoads":2,"browserComparison":comparisons,"execution":result.report()}));
                }
                let before: Vec<_> = result
                    .channels()
                    .iter()
                    .map(|c| c.pixels().to_vec())
                    .collect();
                retained.push((result, before));
            }
        }
    }
    drop(renderer);
    for (result, before) in retained {
        for (c, b) in result.channels().iter().zip(before) {
            assert_eq!(c.pixels(), b);
        }
    }
    let ok = rows.iter().all(|r| {
        r["browserComparison"]
            .as_array()
            .unwrap()
            .iter()
            .all(|c| c["maxComponentDelta"].as_u64().unwrap() <= 1)
    });
    let path = std::env::var("MIXTURE_ASSET_EVIDENCE")
        .unwrap_or_else(|_| "asset-qualification.json".into());
    std::fs::write(
        path,
        serde_json::to_vec_pretty(&json!({"ok":ok,"cases":rows,"ownedAfterDestroy":true})).unwrap(),
    )
    .unwrap();
    assert!(
        ok,
        "Native/browser package delta exceeded unchanged limit 1; inspect receipt"
    );
}

#[test]
fn package_noise_migration_is_explicit_and_versioned() {
    let size = [65, 3];
    let data = pixels(size);
    let request = request(size, 0.5);
    let v1 = archive(size);
    let legacy = AssetView::load(&v1, &Default::default()).unwrap();
    let before = legacy.prepare(&request).unwrap();
    let mut migrated: serde_json::Value = serde_json::from_slice(SOURCE).unwrap();
    for node in migrated["nodes"].as_array_mut().unwrap() {
        if node["type"] == "fractal-noise" {
            node["version"] = json!(2);
        }
    }
    let source = serde_json::to_vec(&migrated).unwrap();
    let v2 = mixture_asset::write(&source, &[binding(size, &data)], &Default::default()).unwrap();
    let asset = AssetView::load(&v2, &Default::default()).unwrap();
    let after = asset.prepare(&request).unwrap();
    let doc = MaterialDocument::decode(&source, &Default::default())
        .unwrap()
        .into_validated(&Default::default())
        .unwrap();
    let loose = prepare(&doc, &request, &[binding(size, &data)], &Default::default()).unwrap();
    assert_eq!(after.plan().hash(), loose.plan().hash());
    assert_ne!(before.plan().hash(), after.plan().hash());
    assert_eq!(legacy.source(), SOURCE);
    assert_eq!(asset.source(), source);
}
