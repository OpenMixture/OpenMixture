//! Exercise real archive/capture buffers at normal and maximum resource budgets.
use mixture_asset::{AssetLimits, AssetView, OwnedAsset, write};
use mixture_core::{CompileRequest, ImageBinding, OutputChannel};

#[test]
fn actual_buffer_ledger_at_normal_and_maximum_payloads() {
    for (width, height, count) in [(1024, 1024, 1), (2048, 1024, 8)] {
        let mut nodes = vec![
            serde_json::json!({"id":"base","type":"constant-color","version":1}),
            serde_json::json!({"id":"out","type":"material-output","version":1}),
        ];
        let mut edges = vec![
            serde_json::json!({"from":{"nodeId":"base","portId":"color"},"to":{"nodeId":"out","portId":"baseColor"}}),
        ];
        let ids: Vec<_> = (0..count).map(|i| format!("Image{i}")).collect();
        for id in &ids {
            nodes.push(serde_json::json!({"id":id,"type":"image-input","version":1,"parameters":{"resourceId":id}}));
        }
        let mut output = ids[0].clone();
        for (i, id) in ids.iter().enumerate().skip(1) {
            let blend = format!("blend{i}");
            nodes.push(serde_json::json!({"id":blend,"type":"scalar-blend","version":1}));
            for (source, port) in [(&output, "a"), (id, "b")] {
                edges.push(serde_json::json!({"from":{"nodeId":source,"portId":"value"},"to":{"nodeId":blend,"portId":port}}));
            }
            output = blend;
        }
        edges.push(serde_json::json!({"from":{"nodeId":output,"portId":"value"},"to":{"nodeId":"out","portId":"height"}}));
        let source =
            serde_json::to_vec(&serde_json::json!({"version":1,"nodes":nodes,"edges":edges}))
                .unwrap();
        let pixels = vec![127; (width * height * 4) as usize];
        let bindings: Vec<_> = ids
            .iter()
            .map(|id| ImageBinding {
                id,
                width,
                height,
                format: "rgba8-linear",
                bytes_per_row: u64::from(width) * 4,
                data: &pixels,
            })
            .collect();
        let limits = AssetLimits::default();
        let bytes = write(&source, &bindings, &limits).unwrap();
        let request = CompileRequest {
            size: [width, height],
            outputs: vec![OutputChannel::Height],
            ..Default::default()
        };
        let view = AssetView::load(&bytes, &limits).unwrap();
        let borrowed = view.preparation_buffer_bytes(&request).unwrap();
        for id in &ids {
            let image = view.image(id).unwrap();
            assert!(
                (bytes.as_ptr() as usize..bytes.as_ptr() as usize + bytes.len())
                    .contains(&(image.as_ptr() as usize))
            );
        }
        let ptr = bytes.as_ptr();
        let capacity = bytes.capacity();
        drop(view);
        let charged = capacity as u64 + borrowed;
        let mut exact = limits;
        exact.package.package_buffer_bytes = charged;
        let owned = OwnedAsset::from_vec(bytes, &exact).unwrap();
        assert_eq!(owned.as_bytes().as_ptr(), ptr);
        assert_eq!(
            owned.view().preparation_buffer_bytes(&request).unwrap(),
            charged
        );
        let prepared = owned.prepare(&request).unwrap();
        let captured: usize = prepared.resources().iter().map(|r| r.data().len()).sum();
        assert_eq!(captured, count * pixels.len());
        assert_eq!(prepared.resources().len(), count);
        for r in prepared.resources() {
            assert_ne!(
                r.data().as_ptr(),
                owned.view().image(&r.image().id).unwrap().as_ptr()
            );
        }
        eprintln!(
            "{count} x {width}x{height}: P={}, retainedCapacity={}, selectedBytes={captured}, borrowedCharge={borrowed}, ownedCharge={charged}",
            owned.as_bytes().len(),
            owned.archive_capacity()
        );
        drop(prepared);
        exact.package.package_buffer_bytes -= 1;
        let denied = OwnedAsset::copy_from(owned.as_bytes(), &exact).unwrap();
        drop(owned);
        assert_eq!(
            denied.prepare(&request).unwrap_err().code(),
            "MIX_PACKAGE_LIMIT_EXCEEDED"
        );
        // A second retained full archive would exceed this explicit exact ledger.
        assert!(charged + denied.as_bytes().len() as u64 > exact.package.package_buffer_bytes);
    }
}
