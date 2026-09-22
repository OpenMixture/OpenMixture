//! Bounded JS-to-Rust package transfer; the shared codec owns all decoding.
use super::{BrowserPrepared, Request, engine_error, invalid, project};
use js_sys::Uint8Array;
use mixture_asset::{AssetError, AssetLimits, OwnedAsset, PackageLimits};
use mixture_core::CompileRequest;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Options {
    request: Request,
    package_limits: PackageLimits,
}

fn failure(operation: &str, error: AssetError) -> JsValue {
    match error {
        AssetError::Document(e) => {
            engine_error(operation, e.report().diagnostics()).unwrap_or_else(|e| e)
        }
        AssetError::Compile(e) => {
            engine_error(operation, e.report().diagnostics()).unwrap_or_else(|e| e)
        }
        AssetError::Package(d) => {
            #[derive(Serialize)]
            struct Diagnostic<'a> {
                #[serde(flatten)]
                diagnostic: &'a mixture_asset::PackageDiagnostic,
                severity: &'static str,
            }
            #[derive(Serialize)]
            struct Failure<'a> {
                operation: &'a str,
                diagnostics: [Diagnostic<'a>; 1],
            }
            project(&Failure {
                operation,
                diagnostics: [Diagnostic {
                    diagnostic: &d,
                    severity: "error",
                }],
            })
            .unwrap_or_else(|e| e)
        }
    }
}

fn load(
    bytes: Uint8Array,
    options: JsValue,
    operation: &str,
) -> Result<(OwnedAsset, Request, u64), JsValue> {
    let options: Options =
        serde_wasm_bindgen::from_value(options).map_err(|e| invalid(operation, &e.to_string()))?;
    if !options.request.resources.is_empty() {
        return Err(invalid(
            operation,
            "Package requests cannot supply loose resources",
        ));
    }
    let limits = AssetLimits {
        package: options.package_limits,
        safety: options.request.limits,
        resources: options.request.resource_limits,
    };
    let length = bytes.length() as usize;
    // JS owns one accepted snapshot. Charge both packages before reserving Rust bytes.
    limits
        .with_retained_bytes((length as u64) * 2)
        .map_err(|e| failure(operation, e))?;
    if length as u64 > limits.package.package_bytes {
        return Err(package_failure(
            operation,
            "MIX_PACKAGE_LIMIT_EXCEEDED",
            "Package byte limit exceeded",
            length as u64,
            limits.package.package_bytes,
        ));
    }
    let mut copy = Vec::new();
    copy.try_reserve_exact(length).map_err(|_| {
        package_failure(
            operation,
            "MIX_PACKAGE_ALLOCATION_FAILED",
            "Cannot reserve package bytes",
            length as u64,
            limits.package.package_bytes,
        )
    })?;
    limits
        .with_retained_bytes(length as u64 + copy.capacity() as u64)
        .map_err(|e| failure(operation, e))?;
    copy.resize(length, 0);
    bytes.copy_to(&mut copy);
    let remaining = limits
        .with_retained_bytes(length as u64)
        .map_err(|e| failure(operation, e))?;
    let asset = OwnedAsset::from_vec(copy, &remaining).map_err(|e| failure(operation, e))?;
    Ok((asset, options.request, length as u64))
}
fn package_failure(
    operation: &str,
    code: &str,
    message: &str,
    observed: u64,
    configured: u64,
) -> JsValue {
    #[derive(Serialize)]
    struct D<'a> {
        code: &'a str,
        stage: &'static str,
        severity: &'static str,
        message: &'a str,
        suggestion: &'static str,
        evidence: E,
    }
    #[derive(Serialize)]
    struct E {
        observed: u64,
        configured: u64,
    }
    #[derive(Serialize)]
    struct F<'a> {
        operation: &'a str,
        diagnostics: [D<'a>; 1],
    }
    project(&F {
        operation,
        diagnostics: [D {
            code,
            stage: "package",
            severity: "error",
            message,
            suggestion: "Use a smaller package or a sufficient policy within the v1 ceilings.",
            evidence: E {
                observed,
                configured,
            },
        }],
    })
    .unwrap_or_else(|e| e)
}
/// Authoritative package ceilings; import does not load WASM or request an adapter.
#[wasm_bindgen]
pub fn default_package_limits() -> Result<JsValue, JsValue> {
    project(&PackageLimits::default())
}

/// Validate and inspect all package bytes without a GPU or Core pixel capture.
#[wasm_bindgen]
pub fn inspect_package(bytes: Uint8Array, options: JsValue) -> Result<JsValue, JsValue> {
    let (asset, _, js_bytes) = load(bytes, options, "inspectPackage")?;
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Buffers {
        js_package_bytes: u64,
        rust_package_bytes: u64,
        charged_bytes: u64,
    }
    #[derive(Serialize)]
    struct Report<'a> {
        #[serde(flatten)]
        asset: mixture_asset::Inspection<'a>,
        buffers: Buffers,
    }
    let view = asset.view();
    project(&Report {
        asset: view.inspect(),
        buffers: Buffers {
            js_package_bytes: js_bytes,
            rust_package_bytes: asset.archive_capacity() as u64,
            charged_bytes: js_bytes + view.loading_buffer_bytes(),
        },
    })
}
/// Synchronously capture selected Core snapshots, dropping transfer storage before render.
#[wasm_bindgen]
pub fn prepare_package(bytes: Uint8Array, options: JsValue) -> Result<BrowserPrepared, JsValue> {
    let (asset, wire, _) = load(bytes, options, "renderPackage")?;
    let outputs = wire
        .channels
        .iter()
        .map(|c| c.parse())
        .collect::<Result<Vec<_>, mixture_core::CompileError>>()
        .map_err(|e| {
            engine_error("renderPackage", e.report().diagnostics()).unwrap_or_else(|e| e)
        })?;
    let overrides = serde_json::from_str(&wire.overrides_json)
        .map_err(|e| invalid("renderPackage", &e.to_string()))?;
    let request = CompileRequest {
        size: wire.size,
        outputs,
        overrides,
        limits: wire.limits,
    };
    let prepared = asset
        .prepare(&request)
        .map_err(|e| failure("renderPackage", e))?;
    Ok(BrowserPrepared { prepared })
}
