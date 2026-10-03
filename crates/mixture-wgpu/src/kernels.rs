//! Exhaustive plan-kernel ABI and renderer-owned pipeline cache.
use crate::operation::{GpuOperationError, checked};
use mixture_core::{
    Stage,
    plan::{
        BlendMode, KernelId, KernelInvocation, MorphologyAxis, MorphologyOperation, NoiseBasis,
        WeaveMode,
    },
};
use serde::Serialize;

pub(crate) fn shader(id: KernelId) -> (&'static str, &'static str) {
    match id {
        KernelId::WeavePattern => (
            concat!(
                include_str!("../shaders/precision.wgsl"),
                "\n",
                include_str!("../shaders/nodes/weave-pattern.wgsl")
            ),
            "weave_pattern",
        ),
        KernelId::BrickPattern => (
            concat!(
                include_str!("../shaders/precision.wgsl"),
                "\n",
                include_str!("../shaders/nodes/brick-pattern.wgsl")
            ),
            "brick_pattern",
        ),
        KernelId::ImageInput => (
            concat!(
                include_str!("../shaders/precision.wgsl"),
                "\n",
                include_str!("../shaders/nodes/image-input.wgsl")
            ),
            "image_input",
        ),
        KernelId::Constant => (
            concat!(
                include_str!("../shaders/precision.wgsl"),
                "\n",
                include_str!("../shaders/nodes/constant.wgsl")
            ),
            "constant",
        ),
        KernelId::Checker => (
            concat!(
                include_str!("../shaders/precision.wgsl"),
                "\n",
                include_str!("../shaders/nodes/checker.wgsl")
            ),
            "checker",
        ),
        KernelId::Levels => (
            concat!(
                include_str!("../shaders/precision.wgsl"),
                "\n",
                include_str!("../shaders/nodes/levels.wgsl")
            ),
            "levels",
        ),
        KernelId::ScalarBlend => (
            concat!(
                include_str!("../shaders/precision.wgsl"),
                "\n",
                include_str!("../shaders/nodes/scalar-blend.wgsl")
            ),
            "scalar_blend",
        ),
        KernelId::ScalarMaskBlend => (
            concat!(
                include_str!("../shaders/precision.wgsl"),
                "\n",
                include_str!("../shaders/nodes/scalar-mask-blend.wgsl")
            ),
            "scalar_mask_blend",
        ),
        KernelId::ScalarSubtract => (
            concat!(
                include_str!("../shaders/precision.wgsl"),
                "\n",
                include_str!("../shaders/nodes/scalar-subtract.wgsl")
            ),
            "scalar_subtract",
        ),
        KernelId::ScalarMorphology => (
            concat!(
                include_str!("../shaders/precision.wgsl"),
                "\n",
                include_str!("../shaders/nodes/scalar-morphology.wgsl")
            ),
            "scalar_morphology",
        ),
        KernelId::Blend => (
            concat!(
                include_str!("../shaders/precision.wgsl"),
                "\n",
                include_str!("../shaders/nodes/blend.wgsl")
            ),
            "blend",
        ),
        KernelId::FractalNoise => (
            concat!(
                include_str!("../shaders/precision.wgsl"),
                "\n",
                include_str!("../shaders/nodes/fractal-noise.wgsl")
            ),
            "fractal_noise",
        ),
        KernelId::GradientMap => (
            concat!(
                include_str!("../shaders/precision.wgsl"),
                "\n",
                include_str!("../shaders/nodes/gradient-map.wgsl")
            ),
            "gradient_map",
        ),
        KernelId::HeightToNormal => (
            concat!(
                include_str!("../shaders/precision.wgsl"),
                "\n",
                include_str!("../shaders/nodes/height-to-normal.wgsl")
            ),
            "height_to_normal",
        ),
        KernelId::Transform2d => (
            concat!(
                include_str!("../shaders/precision.wgsl"),
                "\n",
                include_str!("../shaders/nodes/transform-2d.wgsl")
            ),
            "transform_2d",
        ),
        KernelId::Warp => (
            concat!(
                include_str!("../shaders/precision.wgsl"),
                "\n",
                include_str!("../shaders/nodes/warp.wgsl")
            ),
            "warp",
        ),
    }
}

pub(crate) fn parameters(invocation: &KernelInvocation) -> Vec<u8> {
    let floats = |values: &[f32]| {
        values
            .iter()
            .flat_map(|v| v.to_le_bytes())
            .collect::<Vec<_>>()
    };
    match invocation {
        KernelInvocation::WeavePattern {
            counts,
            widths,
            bevel,
            crown,
            under_ratio,
            mode,
        } => {
            let mode: u32 = match mode {
                WeaveMode::Height => 0,
                WeaveMode::Coverage => 1,
                WeaveMode::WarpShare => 2,
            };
            let mut bytes: Vec<_> = [counts[0], counts[1], mode, 0]
                .into_iter()
                .flat_map(u32::to_le_bytes)
                .collect();
            bytes.extend(floats(&[widths[0], widths[1], *bevel, *crown]));
            bytes.extend(floats(&[*under_ratio, 0., 0., 0.]));
            bytes
        }
        KernelInvocation::BrickPattern {
            cells,
            seed,
            row_offset,
            mortar,
            bevel,
            variation,
        } => {
            let mut bytes: Vec<_> = [cells[0], cells[1], *seed, 0]
                .into_iter()
                .flat_map(u32::to_le_bytes)
                .collect();
            bytes.extend(floats(&[*row_offset, mortar[0], mortar[1], *bevel]));
            bytes.extend(floats(&[*variation, 0., 0., 0.]));
            bytes
        }
        KernelInvocation::ImageInput { .. } | KernelInvocation::ScalarSubtract { .. } => {
            vec![0; 16]
        }
        KernelInvocation::Constant { value } => floats(value),
        KernelInvocation::Checker {
            cells,
            color_a,
            color_b,
        } => {
            let mut bytes: Vec<_> = [cells[0], cells[1], 0, 0]
                .into_iter()
                .flat_map(u32::to_le_bytes)
                .collect();
            bytes.extend(floats(color_a));
            bytes.extend(floats(color_b));
            bytes
        }
        KernelInvocation::Levels {
            input_min,
            input_max,
            gamma,
            output_min,
            output_max,
            ..
        } => floats(&[
            *input_min,
            *input_max,
            *gamma,
            *output_min,
            *output_max,
            0.,
            0.,
            0.,
        ]),
        KernelInvocation::ScalarBlend { weight, .. } => floats(&[*weight, 0., 0., 0.]),
        KernelInvocation::ScalarMaskBlend { opacity, .. } => floats(&[*opacity, 0., 0., 0.]),
        KernelInvocation::ScalarMorphology {
            operation,
            axis,
            radius,
            ..
        } => {
            let operation: u32 = match operation {
                MorphologyOperation::Erode => 0,
                MorphologyOperation::Dilate => 1,
            };
            let axis: u32 = match axis {
                MorphologyAxis::X => 0,
                MorphologyAxis::Y => 1,
            };
            [operation, axis, *radius, 0]
                .into_iter()
                .flat_map(u32::to_le_bytes)
                .collect()
        }
        KernelInvocation::Blend { mode, opacity, .. } => {
            let mode: u32 = match mode {
                BlendMode::Normal => 0,
                BlendMode::Multiply => 1,
                BlendMode::Screen => 2,
            };
            [mode.to_le_bytes(), opacity.to_le_bytes(), [0; 4], [0; 4]].concat()
        }
        KernelInvocation::FractalNoise {
            seed,
            scale,
            octaves,
            persistence,
            basis,
        } => {
            let basis = match basis {
                NoiseBasis::Value => 0,
                NoiseBasis::Cellular => 1,
                NoiseBasis::StableValue => 2,
            };
            let mut bytes: Vec<_> = [*seed, *scale, *octaves, basis]
                .into_iter()
                .flat_map(u32::to_le_bytes)
                .collect();
            bytes.extend(floats(&[*persistence, 0., 0., 0.]));
            bytes
        }
        KernelInvocation::GradientMap {
            color_a, color_b, ..
        } => {
            let mut bytes = floats(color_a);
            bytes.extend(floats(color_b));
            bytes
        }
        KernelInvocation::HeightToNormal { strength, .. } => floats(&[*strength, 0., 0., 0.]),
        KernelInvocation::Transform2d {
            scale,
            quarter_turns,
            offset,
            ..
        } => {
            let mut bytes: Vec<_> = [scale[0], scale[1], *quarter_turns, 0]
                .into_iter()
                .flat_map(u32::to_le_bytes)
                .collect();
            bytes.extend(floats(&[offset[0], offset[1], 0., 0.]));
            bytes
        }
        KernelInvocation::Warp { strength, .. } => floats(&[strength[0], strength[1], 0., 0.]),
    }
}

/// Pipeline lookups for one render call. The cache has at most sixteen kernel identities, including image upload.
#[derive(Clone, Debug, Default, Serialize)]
pub struct PipelineCacheReport {
    /// Passes whose kernel was already cached, including earlier passes this call.
    pub hits: u32,
    /// Newly created kernel pipelines during this call.
    pub misses: u32,
    /// Resident pipelines after this call; fixed shader ABI/format/device per renderer.
    pub entries: usize,
}
#[derive(Default)]
pub(crate) struct PipelineCache(Vec<(KernelId, wgpu::ComputePipeline)>);
impl PipelineCache {
    pub fn clear(&mut self) {
        self.0.clear();
    }
    pub fn len(&self) -> usize {
        self.0.len()
    }
    pub async fn get(
        &mut self,
        device: &wgpu::Device,
        id: KernelId,
        report: &mut PipelineCacheReport,
    ) -> Result<wgpu::ComputePipeline, GpuOperationError> {
        if let Some((_, pipeline)) = self.0.iter().find(|(key, _)| *key == id) {
            report.hits += 1;
            return Ok(pipeline.clone());
        }
        let (source, entry) = shader(id);
        let pipeline = create_pipeline(device, source, entry).await?;
        self.0.push((id, pipeline.clone()));
        report.misses += 1;
        report.entries = self.len();
        Ok(pipeline)
    }
}
pub(crate) async fn create_pipeline(
    device: &wgpu::Device,
    source: &str,
    entry: &str,
) -> Result<wgpu::ComputePipeline, GpuOperationError> {
    let module = checked(
        device,
        Stage::GpuShader,
        "Compute shader validation failed.",
        || {
            device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some(entry),
                source: wgpu::ShaderSource::Wgsl(source.into()),
            })
        },
    )
    .await?;
    checked(
        device,
        Stage::GpuPipeline,
        "Compute pipeline creation failed.",
        || {
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some(entry),
                layout: None,
                module: &module,
                entry_point: Some(entry),
                compilation_options: Default::default(),
                cache: None,
            })
        },
    )
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn graph_shader_validates_without_a_gpu_and_matches_uniform_abi() {
        for (id, size) in [
            (KernelId::ImageInput, 16),
            (KernelId::Constant, 16),
            (KernelId::Checker, 48),
            (KernelId::Levels, 32),
            (KernelId::Blend, 16),
            (KernelId::ScalarBlend, 16),
            (KernelId::ScalarMaskBlend, 16),
            (KernelId::ScalarMorphology, 16),
            (KernelId::ScalarSubtract, 16),
            (KernelId::BrickPattern, 48),
            (KernelId::WeavePattern, 48),
            (KernelId::FractalNoise, 32),
            (KernelId::GradientMap, 32),
            (KernelId::HeightToNormal, 16),
            (KernelId::Transform2d, 32),
            (KernelId::Warp, 16),
        ] {
            let (source, entry) = shader(id);
            let module = naga::front::wgsl::parse_str(source).unwrap();
            naga::valid::Validator::new(
                naga::valid::ValidationFlags::all(),
                naga::valid::Capabilities::empty(),
            )
            .validate(&module)
            .unwrap();
            assert_eq!(module.entry_points.len(), 1);
            assert_eq!(module.entry_points[0].name, entry);
            assert_eq!(module.entry_points[0].workgroup_size, [8, 8, 1]);
            let uniform = module
                .global_variables
                .iter()
                .find(|(_, v)| v.space == naga::AddressSpace::Uniform)
                .unwrap()
                .1;
            assert!(
                matches!(module.types[uniform.ty].inner,naga::TypeInner::Struct {span,..} if span==size)
            );
        }
    }
    #[test]
    fn weave_uniform_has_frozen_modes_and_zero_padding() {
        for (mode, code) in [
            (WeaveMode::Height, 0u32),
            (WeaveMode::Coverage, 1),
            (WeaveMode::WarpShare, 2),
        ] {
            let bytes = parameters(&KernelInvocation::WeavePattern {
                counts: [12, 8],
                widths: [0.55, 0.9],
                bevel: 0.12,
                crown: 1.,
                under_ratio: 0.75,
                mode,
            });
            let expected: Vec<_> = [
                12u32,
                8,
                code,
                0,
                0.55f32.to_bits(),
                0.9f32.to_bits(),
                0.12f32.to_bits(),
                1f32.to_bits(),
                0.75f32.to_bits(),
                0,
                0,
                0,
            ]
            .into_iter()
            .flat_map(u32::to_le_bytes)
            .collect();
            assert_eq!(bytes, expected);
            assert_eq!(bytes.len(), 48);
        }
    }
    #[test]
    fn morphology_parameter_upload_has_explicit_integer_discriminants() {
        use mixture_core::{
            CompileRequest, MaterialDocument, OutputChannel, SafetyLimits, compile,
        };
        let document = MaterialDocument::decode(
            include_bytes!("testdata/scalar-morphology.mix"),
            &SafetyLimits::default(),
        )
        .unwrap()
        .into_validated(&SafetyLimits::default())
        .unwrap();
        for (operation, op) in [("erode", 0u32), ("dilate", 1)] {
            for (axis, ax) in [("x", 0u32), ("y", 1)] {
                for radius in [0u32, 2, 16] {
                    let plan=compile(&document,&CompileRequest {outputs:vec![OutputChannel::Height],overrides:serde_json::from_value(serde_json::json!({"operation":operation,"axis":axis,"radius":radius})).unwrap(),..Default::default()}).unwrap();
                    let kernel = &plan.passes().last().unwrap().kernel;
                    assert_eq!(
                        parameters(kernel),
                        [op, ax, radius, 0]
                            .into_iter()
                            .flat_map(u32::to_le_bytes)
                            .collect::<Vec<_>>()
                    );
                    assert_eq!(kernel.uniform_bytes(), 16);
                }
            }
        }
    }
    #[test]
    fn subtraction_has_a_reserved_zero_uniform_and_two_inputs() {
        use mixture_core::{
            CompileRequest, MaterialDocument, OutputChannel, SafetyLimits, compile,
        };
        let doc = MaterialDocument::decode(
            include_bytes!("testdata/scalar-subtract.mix"),
            &SafetyLimits::default(),
        )
        .unwrap()
        .into_validated(&SafetyLimits::default())
        .unwrap();
        let plan = compile(
            &doc,
            &CompileRequest {
                outputs: vec![OutputChannel::Height],
                ..Default::default()
            },
        )
        .unwrap();
        let kernel = &plan.passes().last().unwrap().kernel;
        assert_eq!(kernel.id(), KernelId::ScalarSubtract);
        assert_eq!(kernel.inputs().count(), 2);
        assert_eq!(kernel.uniform_bytes(), 16);
        assert_eq!(parameters(kernel), vec![0; 16]);
    }
    #[test]
    fn typed_parameter_uploads_match_shader_offsets_and_plan_estimates() {
        let kernel = KernelInvocation::Checker {
            cells: [3, 17],
            color_a: [0., 0.25, 0.5, 1.],
            color_b: [1., 0.5, 0.25, 0.],
        };
        let bytes = parameters(&kernel);
        assert_eq!(bytes.len() as u64, kernel.uniform_bytes());
        assert_eq!(
            &bytes[..8],
            &[3u32.to_le_bytes(), 17u32.to_le_bytes()].concat()
        );
        assert_eq!(&bytes[8..16], &[0; 8]);
        assert_eq!(&bytes[20..24], &0.25f32.to_le_bytes());
        assert_eq!(&bytes[32..36], &1f32.to_le_bytes());
    }

    #[test]
    fn brick_and_mask_uniforms_preserve_each_field_and_padding() {
        let brick = KernelInvocation::BrickPattern {
            cells: [3, 7],
            seed: 19,
            row_offset: 0.375,
            mortar: [0.0625, 0.125],
            bevel: 0.1875,
            variation: 0.75,
        };
        let bytes = parameters(&brick);
        assert_eq!(bytes.len(), 48);
        assert_eq!(bytes.len() as u64, brick.uniform_bytes());
        for (offset, value) in [(0, 3u32), (4, 7), (8, 19), (12, 0)] {
            assert_eq!(&bytes[offset..offset + 4], &value.to_le_bytes());
        }
        for (offset, value) in [
            (16, 0.375f32),
            (20, 0.0625),
            (24, 0.125),
            (28, 0.1875),
            (32, 0.75),
        ] {
            assert_eq!(&bytes[offset..offset + 4], &value.to_le_bytes());
        }
        assert_eq!(&bytes[36..48], &[0; 12]);
        let request = mixture_core::CompileRequest::default();
        let document = mixture_core::MaterialDocument::decode(
            include_bytes!("testdata/constant-scalar.mix"),
            &request.limits,
        )
        .unwrap()
        .into_validated(&request.limits)
        .unwrap();
        let plan = mixture_core::compile(&document, &request).unwrap();
        let resource = plan.passes()[0].output;
        let mask = KernelInvocation::ScalarMaskBlend {
            a: resource,
            b: resource,
            mask: resource,
            opacity: 0.625,
        };
        let bytes = parameters(&mask);
        assert_eq!(bytes.len(), 16);
        assert_eq!(bytes.len() as u64, mask.uniform_bytes());
        assert_eq!(&bytes[..4], &0.625f32.to_le_bytes());
        assert_eq!(&bytes[4..], &[0; 12]);
    }

    #[test]
    fn resampling_parameter_uploads_match_typed_plan_and_shader_offsets() {
        use mixture_core::{
            CompileRequest, MaterialDocument, OutputChannel, SafetyLimits, compile,
        };
        for (source, overrides, integers, floats) in [
            (
                include_bytes!("testdata/transform-2d.mix").as_slice(),
                serde_json::json!({"scaleX":64,"scaleY":3,"quarterTurns":3,"offsetX":-1,"offsetY":0.25}),
                vec![64u32, 3, 3, 0],
                vec![-1f32, 0.25, 0., 0.],
            ),
            (
                include_bytes!("testdata/warp.mix").as_slice(),
                serde_json::json!({"strengthX":-1,"strengthY":0.5}),
                vec![],
                vec![-1f32, 0.5, 0., 0.],
            ),
        ] {
            let limits = SafetyLimits::default();
            let document = MaterialDocument::decode(source, &limits)
                .unwrap()
                .into_validated(&limits)
                .unwrap();
            let plan = compile(
                &document,
                &CompileRequest {
                    outputs: vec![OutputChannel::Height],
                    overrides: serde_json::from_value(overrides).unwrap(),
                    ..Default::default()
                },
            )
            .unwrap();
            let kernel = &plan.passes().last().unwrap().kernel;
            let mut expected: Vec<_> = integers.into_iter().flat_map(u32::to_le_bytes).collect();
            expected.extend(floats.into_iter().flat_map(f32::to_le_bytes));
            let actual = parameters(kernel);
            assert_eq!(actual, expected);
            assert_eq!(actual.len() as u64, kernel.uniform_bytes());
        }
    }
}
