use crate::error::{AssetError, limit};
use mixture_core::{ResourceLimits, SafetyLimits};
/// Optional package policy, independent of graph and GPU budgets.
#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PackageLimits {
    /// Maximum encoded bytes; v1 ceiling 67 MiB.
    pub package_bytes: u64,
    /// Maximum manifest bytes; v1 ceiling 64 KiB.
    pub manifest_bytes: u64,
    /// Maximum simultaneously charged byte buffers; v1 ceiling 202 MiB.
    pub package_buffer_bytes: u64,
}
impl Default for PackageLimits {
    fn default() -> Self {
        Self {
            package_bytes: 67 * 1024 * 1024,
            manifest_bytes: 65536,
            package_buffer_bytes: 202 * 1024 * 1024,
        }
    }
}
/// Explicit package, Core and resource policy for one asset operation.
#[derive(Clone, Copy, Debug, Default)]
pub struct AssetLimits {
    /// Transport policy; ceilings may only be lowered.
    pub package: PackageLimits,
    /// Existing graph policy, intersected with profile size limits.
    pub safety: SafetyLimits,
    /// Existing resource policy, intersected with v1 profile ceilings.
    pub resources: ResourceLimits,
}
impl AssetLimits {
    /// Check policy and account for simultaneously retained adapter buffers.
    /// Returned policy reserves that charge before the codec allocates or captures.
    pub fn with_retained_bytes(self, bytes: u64) -> Result<Self, AssetError> {
        let mut limits = self.checked()?;
        limits.buffers(bytes)?;
        limits.package.package_buffer_bytes -= bytes;
        Ok(limits)
    }
    pub(crate) fn checked(mut self) -> Result<Self, AssetError> {
        let max = PackageLimits::default();
        limit(
            "packageBytes",
            self.package.package_bytes,
            max.package_bytes,
        )?;
        limit(
            "manifestBytes",
            self.package.manifest_bytes,
            max.manifest_bytes,
        )?;
        limit(
            "packageBufferBytes",
            self.package.package_buffer_bytes,
            max.package_buffer_bytes,
        )?;
        self.safety.decoded_bytes = self.safety.decoded_bytes.min(2 * 1024 * 1024);
        self.safety.output_dimension = self.safety.output_dimension.min(2048);
        self.resources.resource_count = self.resources.resource_count.min(8);
        self.resources.resource_pixels = self.resources.resource_pixels.min(16_777_216);
        self.resources.resource_bytes = self.resources.resource_bytes.min(64 * 1024 * 1024);
        Ok(self)
    }
    pub(crate) fn buffers(&self, bytes: u64) -> Result<(), AssetError> {
        limit(
            "packageBufferBytes",
            bytes,
            self.package.package_buffer_bytes,
        )
    }
}
pub(crate) fn intersect(a: SafetyLimits, b: SafetyLimits) -> SafetyLimits {
    SafetyLimits {
        decoded_bytes: a.decoded_bytes.min(b.decoded_bytes),
        nodes: a.nodes.min(b.nodes),
        edges: a.edges.min(b.edges),
        exposed_parameters: a.exposed_parameters.min(b.exposed_parameters),
        output_dimension: a.output_dimension.min(b.output_dimension),
        requested_outputs: a.requested_outputs.min(b.requested_outputs),
        transient_bytes: a.transient_bytes.min(b.transient_bytes),
    }
}
