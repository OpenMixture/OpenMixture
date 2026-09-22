use super::*;
const SOURCE:&[u8]=br#"{"version":1,"nodes":[{"id":"base","type":"constant-color","version":1},{"id":"out","type":"material-output","version":1}],"edges":[{"from":{"nodeId":"base","portId":"color"},"to":{"nodeId":"out","portId":"baseColor"}}]}"#;
fn archive(json: &[u8], source: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    archive::append(&mut out, "manifest.json", json).unwrap();
    archive::append(&mut out, "material.mix", source).unwrap();
    out.resize(out.len() + 1024, 0);
    out
}
fn manifest() -> String {
    format!(
        r#"{{"format":"openmixture-asset","version":1,"document":{{"path":"material.mix","byteLength":{},"sha256":"{}"}},"resources":[]}}"#,
        SOURCE.len(),
        sha256(SOURCE)
    )
}
#[test]
fn strict_json_tokens_fields_and_digest_are_not_coerced() {
    let base = manifest();
    let mut cases = Vec::new();
    for token in [
        "1.0",
        "1e0",
        "-0",
        "-1",
        "true",
        "\"1\"",
        "4294967296",
        "null",
    ] {
        cases.push(base.replace("\"version\":1", &format!("\"version\":{token}")));
    }
    for token in [
        "0.0",
        "1e0",
        "-0",
        "-1",
        "true",
        "\"1\"",
        "18446744073709551616",
        "null",
    ] {
        cases.push(base.replace(
            &format!("\"byteLength\":{}", SOURCE.len()),
            &format!("\"byteLength\":{token}"),
        ));
    }
    cases.extend([
        format!("\u{feff}{base}"),
        format!("{base}{{}}"),
        base.replace("\"version\":1", "\"version\":1,\"version\":1"),
        base.replace("\"version\":1", "\"version\":1,\"other\":0"),
        base.replace("\"path\":", "\"path\":\"material.mix\",\"path\":"),
        base.replace("\"path\":", "\"other\":0,\"path\":"),
        base.replace(&sha256(SOURCE), &sha256(SOURCE).to_uppercase()),
    ]);
    for json in cases {
        assert_eq!(
            AssetView::load(&archive(json.as_bytes(), SOURCE), &AssetLimits::default())
                .unwrap_err()
                .code(),
            "MIX_PACKAGE_INVALID",
            "{json}"
        );
    }
    let mut corrupt = SOURCE.to_vec();
    corrupt[0] = b' ';
    assert_eq!(
        AssetView::load(&archive(base.as_bytes(), &corrupt), &AssetLimits::default())
            .unwrap_err()
            .code(),
        "MIX_PACKAGE_CONTENT_MISMATCH"
    );
}
#[test]
fn every_zero_exact_and_over_policy_is_enforced() {
    let base = manifest();
    let bytes = archive(base.as_bytes(), SOURCE);
    let mut limits = AssetLimits::default();
    limits.package.package_bytes = bytes.len() as u64;
    limits.package.manifest_bytes = base.len() as u64;
    limits.safety.decoded_bytes = SOURCE.len() as u64;
    limits.package.package_buffer_bytes = (base.len() + SOURCE.len()) as u64;
    AssetView::load(&bytes, &limits).unwrap();
    for field in 0..4 {
        for zero in [false, true] {
            let mut less = limits;
            let target = match field {
                0 => &mut less.package.package_bytes,
                1 => &mut less.package.manifest_bytes,
                2 => &mut less.safety.decoded_bytes,
                _ => &mut less.package.package_buffer_bytes,
            };
            *target = if zero { 0 } else { *target - 1 };
            assert_eq!(
                AssetView::load(&bytes, &less).unwrap_err().code(),
                "MIX_PACKAGE_LIMIT_EXCEEDED"
            );
        }
    }
    let mut source = serde_json::from_slice::<serde_json::Value>(SOURCE).unwrap();
    source["nodes"].as_array_mut().unwrap().push(serde_json::json!({"id":"image","type":"image-input","version":1,"parameters":{"resourceId":"Image"}}));
    let source = serde_json::to_vec(&source).unwrap();
    let b = ImageBinding {
        id: "Image",
        width: 2,
        height: 2,
        format: "rgba8-linear",
        bytes_per_row: 8,
        data: &[0; 16],
    };
    let bytes = write(&source, &[b], &AssetLimits::default()).unwrap();
    for field in 0..4 {
        for zero in [false, true] {
            let mut l = AssetLimits::default();
            let target = match field {
                0 => &mut l.resources.resource_count,
                1 => &mut l.resources.resource_pixels,
                2 => &mut l.resources.resource_bytes,
                _ => &mut l.safety.output_dimension,
            };
            *target = match field {
                0 => 1,
                1 => 4,
                2 => 16,
                _ => 2,
            };
            AssetView::load(&bytes, &l).unwrap();
            write(&source, &[b], &l).unwrap();
            *match field {
                0 => &mut l.resources.resource_count,
                1 => &mut l.resources.resource_pixels,
                2 => &mut l.resources.resource_bytes,
                _ => &mut l.safety.output_dimension,
            } = if zero {
                0
            } else {
                match field {
                    0 => 0,
                    1 => 3,
                    2 => 15,
                    _ => 1,
                }
            };
            assert_eq!(
                AssetView::load(&bytes, &l).unwrap_err().code(),
                "MIX_PACKAGE_LIMIT_EXCEEDED"
            );
            assert_eq!(
                write(&source, &[b], &l).unwrap_err().code(),
                "MIX_PACKAGE_LIMIT_EXCEEDED"
            );
        }
    }
    let mut l = AssetLimits::default();
    l.package.manifest_bytes += 1;
    assert_eq!(
        write(SOURCE, &[], &l).unwrap_err().code(),
        "MIX_PACKAGE_LIMIT_EXCEEDED"
    );
    l = AssetLimits::default();
    l.package.package_buffer_bytes += 1;
    assert_eq!(
        write(SOURCE, &[], &l).unwrap_err().code(),
        "MIX_PACKAGE_LIMIT_EXCEEDED"
    );
}
#[test]
fn writer_limits_and_original_core_diagnostics() {
    let limits = AssetLimits::default();
    let out = write(SOURCE, &[], &limits).unwrap();
    let view = AssetView::load(&out, &limits).unwrap();
    let budget = out.len() as u64 + SOURCE.len() as u64 + view.parsed.manifest_bytes;
    let mut limited = limits;
    limited.package.package_buffer_bytes = budget;
    assert_eq!(write(SOURCE, &[], &limited).unwrap(), out);
    limited.package.package_buffer_bytes -= 1;
    assert_eq!(
        write(SOURCE, &[], &limited).unwrap_err().code(),
        "MIX_PACKAGE_LIMIT_EXCEEDED"
    );
    assert!(matches!(
        write(b"{}", &[], &limits).unwrap_err(),
        AssetError::Document(_)
    ));
    let broken = b"{}";
    let mut manifest: serde_json::Value = serde_json::from_str(&manifest()).unwrap();
    manifest["document"]["byteLength"] = serde_json::json!(broken.len());
    manifest["document"]["sha256"] = serde_json::json!(sha256(broken));
    let out = archive(&serde_json::to_vec(&manifest).unwrap(), broken);
    assert!(matches!(
        AssetView::load(&out, &limits).unwrap_err(),
        AssetError::Document(_)
    ));
    // A declaration mismatch is malformed framing, not a caller policy violation.
    manifest["document"]["byteLength"] = serde_json::json!(broken.len() - 1);
    let out = archive(&serde_json::to_vec(&manifest).unwrap(), broken);
    assert_eq!(
        AssetView::load(&out, &limits).unwrap_err().code(),
        "MIX_PACKAGE_INVALID"
    );
}

#[test]
fn sorted_bindings_and_full_closure_do_not_depend_on_requested_outputs() {
    let limits = AssetLimits::default();
    let mut source: serde_json::Value = serde_json::from_slice(SOURCE).unwrap();
    for id in ["a", "B"] {
        source["nodes"]
            .as_array_mut()
            .unwrap()
            .push(serde_json::json!({
                "id":id,"type":"image-input","version":1,"parameters":{"resourceId":id}
            }));
    }
    let source = serde_json::to_vec(&source).unwrap();
    let binding = |id| ImageBinding {
        id,
        width: 1,
        height: 1,
        format: "rgba8-linear",
        bytes_per_row: 4,
        data: &[1, 2, 3, 4],
    };
    let bytes = write(&source, &[binding("a"), binding("B")], &limits).unwrap();
    assert_eq!(
        bytes,
        write(&source, &[binding("B"), binding("a")], &limits).unwrap()
    );
    let view = AssetView::load(&bytes, &limits).unwrap();
    assert_eq!(
        view.resources()
            .iter()
            .map(|r| r.id.as_str())
            .collect::<Vec<_>>(),
        ["B", "a"]
    );
    let offset = view.image("a").unwrap().as_ptr() as usize - bytes.as_ptr() as usize;
    let mut corrupt = bytes.clone();
    corrupt[offset] ^= 1;
    assert_eq!(
        AssetView::load(&corrupt, &limits).unwrap_err().code(),
        "MIX_PACKAGE_CONTENT_MISMATCH"
    );
    // Valid framing and hashes, but omit the two disconnected source references.
    let mut manifest: serde_json::Value = serde_json::from_str(&manifest()).unwrap();
    manifest["document"]["byteLength"] = serde_json::json!(source.len());
    manifest["document"]["sha256"] = serde_json::json!(sha256(&source));
    assert_eq!(
        AssetView::load(
            &archive(&serde_json::to_vec(&manifest).unwrap(), &source),
            &limits
        )
        .unwrap_err()
        .code(),
        "MIX_PACKAGE_INVALID"
    );
    // Valid payloads with no corresponding source references must also fail closure.
    let mut manifest = view.parsed.manifest.clone();
    manifest.document.byte_length = SOURCE.len() as u64;
    manifest.document.sha256 = sha256(SOURCE);
    let mut extra = Vec::new();
    archive::append(
        &mut extra,
        "manifest.json",
        &serde_json::to_vec(&manifest).unwrap(),
    )
    .unwrap();
    archive::append(&mut extra, "material.mix", SOURCE).unwrap();
    for r in view.resources() {
        archive::append(&mut extra, &r.path, view.image(&r.id).unwrap()).unwrap();
    }
    extra.resize(extra.len() + 1024, 0);
    assert_eq!(
        AssetView::load(&extra, &limits).unwrap_err().code(),
        "MIX_PACKAGE_INVALID"
    );
}
