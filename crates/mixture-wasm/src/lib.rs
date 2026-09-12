//! Browser transport only. Core owns parsing/compilation; wgpu owns every pixel.

use std::collections::BTreeMap;

use js_sys::{Array, Object, Reflect, Uint8Array};
use mixture_core::registry::{BUILT_INS, ParameterContract, node_contract};
use mixture_core::{
    CompileRequest, Diagnostic, MaterialDocument, RenderPlan, SafetyLimits, ValidatedDocument,
    compile,
};
use mixture_wgpu::{GpuContext, GpuContextOptions, PowerPreference, Renderer};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use wasm_bindgen::prelude::*;

fn project(value: &impl Serialize) -> Result<JsValue, JsValue> {
    value
        .serialize(
            &serde_wasm_bindgen::Serializer::new()
                .serialize_maps_as_objects(true)
                .serialize_missing_as_null(true)
                .serialize_large_number_types_as_bigints(true),
        )
        .map_err(|error| JsValue::from_str(&error.to_string()))
}

fn field(object: &JsValue, name: &str, value: &JsValue) -> Result<(), JsValue> {
    Reflect::set(object, &JsValue::from_str(name), value).map(|_| ())
}

fn engine_error(operation: &str, diagnostics: &[Diagnostic]) -> Result<JsValue, JsValue> {
    #[derive(Serialize)]
    struct Failure<'a> {
        operation: &'a str,
        diagnostics: &'a [Diagnostic],
    }
    project(&Failure {
        operation,
        diagnostics,
    })
}

fn invalid(operation: &str, message: &str) -> JsValue {
    #[derive(Serialize)]
    struct BrowserFailure<'a> {
        code: &'a str,
        operation: &'a str,
        message: &'a str,
    }
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Failure<'a> {
        operation: &'a str,
        diagnostics: Vec<Diagnostic>,
        browser_failure: BrowserFailure<'a>,
    }
    project(&Failure {
        operation,
        diagnostics: vec![],
        browser_failure: BrowserFailure {
            code: "MIX_BROWSER_INVALID_ARGUMENT",
            operation,
            message,
        },
    })
    .unwrap_or_else(|_| JsValue::from_str(message))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Request {
    size: [u32; 2],
    channels: Vec<String>,
    overrides_json: String,
    limits: SafetyLimits,
}

fn prepare(
    source: &[u8],
    options: JsValue,
    operation: &str,
) -> Result<(ValidatedDocument, CompileRequest, RenderPlan), JsValue> {
    let wire: Request = serde_wasm_bindgen::from_value(options)
        .map_err(|error| invalid(operation, &error.to_string()))?;
    let outputs = wire
        .channels
        .iter()
        .map(|name| name.parse())
        .collect::<Result<Vec<_>, mixture_core::CompileError>>()
        .map_err(|error| {
            engine_error(operation, error.report().diagnostics()).unwrap_or_else(|e| e)
        })?;
    // The facade captures and validates JS values, then serializes integer-valued
    // numbers as integer JSON tokens. The original source bytes never take this path.
    let overrides: BTreeMap<String, Value> = serde_json::from_str(&wire.overrides_json)
        .map_err(|error| invalid(operation, &error.to_string()))?;
    let request = CompileRequest {
        size: wire.size,
        outputs,
        overrides,
        limits: wire.limits,
    };
    let document = MaterialDocument::decode(source, &request.limits)
        .and_then(|document| document.into_validated(&request.limits))
        .map_err(|error| {
            engine_error(operation, error.report().diagnostics()).unwrap_or_else(|e| e)
        })?;
    let plan = compile(&document, &request).map_err(|error| {
        engine_error(operation, error.report().diagnostics()).unwrap_or_else(|e| e)
    })?;
    Ok((document, request, plan))
}

// Validated parameter values are u32/finite float/color/enum values. Project their
// JSON representation as JS numbers; do not mistake serde_json's u64 storage for
// a typed engine u64 report field, which must remain bigint.
struct ParameterValue<'a>(&'a Value);
impl Serialize for ParameterValue<'_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self.0 {
            Value::Number(value) => match value.as_f64() {
                Some(value) => serializer.serialize_f64(value),
                None => Err(serde::ser::Error::custom(
                    "Invalid validated numeric parameter",
                )),
            },
            Value::Array(values) => values
                .iter()
                .map(ParameterValue)
                .collect::<Vec<_>>()
                .serialize(serializer),
            Value::String(value) => serializer.serialize_str(value),
            _ => Err(serde::ser::Error::custom(
                "Unexpected validated parameter representation",
            )),
        }
    }
}

fn inspection(
    document: &ValidatedDocument,
    request: &CompileRequest,
    plan: &RenderPlan,
) -> Result<JsValue, JsValue> {
    let object: JsValue = Object::new().into();
    field(&object, "ok", &JsValue::TRUE)?;
    field(&object, "diagnostics", &Array::new())?;
    field(
        &object,
        "documentVersion",
        &JsValue::from(document.document().version),
    )?;
    field(&object, "plan", &project(plan)?)?;
    field(
        &object,
        "materialChannels",
        &project(&document.material_channels())?,
    )?;
    let parameters = Array::new();
    let mut bindings = document
        .document()
        .exposed_parameters
        .iter()
        .collect::<Vec<_>>();
    bindings.sort_by(|a, b| a.id.cmp(&b.id));
    for binding in bindings {
        let node = document
            .document()
            .nodes
            .iter()
            .find(|node| node.id == binding.node_id)
            .ok_or_else(|| invalid("inspect", "Validated exposed node is missing"))?;
        let contract = node_contract(&node.type_id)
            .and_then(|contract| contract.parameter(&binding.parameter_id))
            .ok_or_else(|| invalid("inspect", "Validated parameter contract is missing"))?;
        let source_value = document
            .parameter(&binding.node_id, &binding.parameter_id)
            .ok_or_else(|| invalid("inspect", "Validated source parameter is missing"))?;
        let effective = request.overrides.get(&binding.id).unwrap_or(&source_value);
        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct Binding<'a> {
            id: &'a str,
            node_id: &'a str,
            node_type: &'a str,
            node_version: u32,
            parameter_id: &'a str,
            contract: &'a ParameterContract,
            source_value: ParameterValue<'a>,
            effective_value: ParameterValue<'a>,
        }
        parameters.push(&project(&Binding {
            id: &binding.id,
            node_id: &node.id,
            node_type: &node.type_id,
            node_version: node.version,
            parameter_id: &binding.parameter_id,
            contract,
            source_value: ParameterValue(&source_value),
            effective_value: ParameterValue(effective),
        })?);
    }
    field(&object, "exposedParameters", &parameters)?;
    Ok(object)
}

/// Validate raw UTF-8 bytes and the entire compile request without a GPU.
#[wasm_bindgen]
pub fn validate_source(source: &[u8], request: JsValue) -> Result<JsValue, JsValue> {
    match prepare(source, request, "validate") {
        Ok((document, request, plan)) => inspection(&document, &request, &plan),
        Err(failure) => {
            if Reflect::has(&failure, &JsValue::from_str("browserFailure"))? {
                return Err(failure);
            }
            let object: JsValue = Object::new().into();
            field(&object, "ok", &JsValue::FALSE)?;
            field(
                &object,
                "diagnostics",
                &Reflect::get(&failure, &JsValue::from_str("diagnostics"))?,
            )?;
            Ok(object)
        }
    }
}

/// Inspect a valid document/request or return the unchanged core diagnostics.
#[wasm_bindgen]
pub fn inspect_source(source: &[u8], request: JsValue) -> Result<JsValue, JsValue> {
    let (document, request, plan) = prepare(source, request, "inspect")?;
    inspection(&document, &request, &plan)
}

/// Fresh Rust-registry catalog projection with no shared mutable aliases.
#[wasm_bindgen]
pub fn node_catalog() -> Result<JsValue, JsValue> {
    project(&BUILT_INS)
}

/// Current authoritative core policy, projected losslessly for argument capture.
#[wasm_bindgen]
pub fn default_limits() -> Result<JsValue, JsValue> {
    project(&SafetyLimits::default())
}

/// Build identity injected by the producer's package build command.
#[wasm_bindgen]
pub fn build_info() -> Result<JsValue, JsValue> {
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct BuildInfo {
        runtime_version: &'static str,
        api_schema_version: u32,
        engine_version: &'static str,
        engine_revision: Option<&'static str>,
        engine_dirty: Option<bool>,
        build_id: &'static str,
    }
    project(&BuildInfo {
        runtime_version: option_env!("MIXTURE_RUNTIME_VERSION").unwrap_or("unpackaged"),
        api_schema_version: 1,
        engine_version: env!("CARGO_PKG_VERSION"),
        engine_revision: option_env!("MIXTURE_ENGINE_REVISION"),
        engine_dirty: option_env!("MIXTURE_ENGINE_DIRTY").map(|value| value == "1"),
        build_id: option_env!("MIXTURE_BUILD_ID").unwrap_or("unpackaged"),
    })
}

/// One explicit renderer. The public facade bounds render/destroy concurrency.
#[wasm_bindgen]
pub struct BrowserGpu {
    renderer: Option<Renderer>,
}

/// Request browser WebGPU explicitly; CPU exports never call this function.
#[wasm_bindgen]
pub async fn create_gpu(power_preference: &str) -> Result<BrowserGpu, JsValue> {
    let power_preference = match power_preference {
        "low-power" => PowerPreference::LowPower,
        "high-performance" => PowerPreference::HighPerformance,
        _ => return Err(invalid("createGpu", "Unknown power preference")),
    };
    let context = GpuContext::request(GpuContextOptions {
        power_preference,
        ..Default::default()
    })
    .await
    .map_err(|error| {
        let failure = engine_error("createGpu", error.report().diagnostics().diagnostics())
            .unwrap_or_else(|e| e);
        let _ = project(error.report()).and_then(|report| field(&failure, "evidence", &report));
        failure
    })?;
    Ok(BrowserGpu {
        renderer: Some(Renderer::new(context)),
    })
}

#[wasm_bindgen]
impl BrowserGpu {
    /// Acquisition evidence; acquiring alone does not verify rendering.
    pub fn context_report(&self) -> Result<JsValue, JsValue> {
        self.renderer
            .as_ref()
            .ok_or_else(|| invalid("createGpu", "Renderer is destroyed"))
            .and_then(|renderer| project(renderer.context().report()))
    }

    /// Compile source and execute the existing immutable RenderPlan, asynchronously.
    pub async fn render(&mut self, source: &[u8], request: JsValue) -> Result<JsValue, JsValue> {
        let (document, _, plan) = prepare(source, request, "render")?;
        let renderer = self
            .renderer
            .as_mut()
            .ok_or_else(|| invalid("render", "Renderer is destroyed"))?;
        let output = renderer.render(&plan).await.map_err(|error| {
            let failure = engine_error("render", std::slice::from_ref(error.diagnostic()))
                .unwrap_or_else(|e| e);
            #[derive(Serialize)]
            #[serde(rename_all = "camelCase")]
            struct Evidence<'a> {
                adapter: Option<&'a mixture_wgpu::AdapterDiagnostics>,
                device_loss: Option<&'a mixture_wgpu::DeviceLoss>,
                allocations: Option<&'a mixture_wgpu::AllocationReport>,
            }
            let _ = project(&Evidence {
                adapter: error.adapter(),
                device_loss: error.device_loss(),
                allocations: error.allocations(),
            })
            .and_then(|evidence| field(&failure, "evidence", &evidence));
            failure
        })?;
        let result: JsValue = Object::new().into();
        field(
            &result,
            "documentVersion",
            &JsValue::from(document.document().version),
        )?;
        field(&result, "plan", &project(&plan)?)?;
        field(&result, "report", &project(output.report())?)?;
        let channels = Array::new();
        for channel in output.channels() {
            #[derive(Serialize)]
            struct Channel<'a> {
                channel: mixture_core::OutputChannel,
                kind: mixture_core::registry::PortKind,
                source: &'a mixture_core::InputSource,
                size: [u32; 2],
                encoding: mixture_wgpu::OutputEncoding,
            }
            let object = project(&Channel {
                channel: channel.channel,
                kind: channel.kind,
                source: &channel.source,
                size: channel.size,
                encoding: channel.encoding,
            })?;
            // Uint8Array::from copies. Never expose a view over WASM memory.
            field(&object, "pixels", &Uint8Array::from(channel.pixels()))?;
            channels.push(&object);
        }
        field(&result, "channels", &channels)?;
        Ok(result)
    }

    /// Release explicit device ownership after the facade has awaited active work.
    pub fn destroy(&mut self) {
        if let Some(renderer) = self.renderer.take() {
            renderer.context().device().destroy();
        }
    }
}
