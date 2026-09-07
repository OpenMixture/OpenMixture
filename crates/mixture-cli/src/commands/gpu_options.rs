//! Shared CLI spelling of the library's explicit GPU selection policy.

use mixture_wgpu::{BackendPreference, GpuContextOptions, PowerPreference};
use std::{ffi::OsString, slice::Iter};

pub(super) fn parse_option(
    flag: &str,
    args: &mut Iter<'_, OsString>,
    options: &mut GpuContextOptions,
) -> Result<bool, String> {
    match flag {
        "--software" => options.software_adapter = true,
        "--backend" | "--power-preference" => {
            let value = args
                .next()
                .and_then(|value| value.to_str())
                .ok_or_else(|| format!("Missing value for {flag}"))?;
            if flag == "--backend" {
                options.backend = match value {
                    "auto" => BackendPreference::Auto,
                    "metal" => BackendPreference::Metal,
                    "vulkan" => BackendPreference::Vulkan,
                    "dx12" => BackendPreference::Dx12,
                    "none" => BackendPreference::None,
                    _ => return Err(format!("Invalid backend: {value}")),
                };
            } else {
                options.power_preference = match value {
                    "high-performance" => PowerPreference::HighPerformance,
                    "low-power" => PowerPreference::LowPower,
                    _ => return Err(format!("Invalid power preference: {value}")),
                };
            }
        }
        _ => return Ok(false),
    }
    Ok(true)
}
