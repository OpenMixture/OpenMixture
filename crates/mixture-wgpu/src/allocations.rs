//! Per-render accounting for successful wgpu resource descriptors, not driver memory.

use crate::GpuOperationError;
use mixture_core::Stage;
use serde::Serialize;

/// Successful per-call texture, uniform, and staging descriptor allocations.
/// Bytes exclude driver allocation granularity, pipeline/bind-group overhead,
/// and CPU pixel/PNG buffers. Release means the resource's `destroy` was called,
/// not that the driver or operating system immediately returned physical memory.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AllocationReport {
    /// Unique image IDs successfully submitted for upload in this call.
    pub resource_count: u64,
    /// Packed RGBA8 bytes successfully submitted, excluding row padding.
    pub resource_upload_bytes: u64,
    /// Uploaded image texture descriptors; included in texture_bytes.
    pub resource_texture_bytes: u64,
    /// Upload staging descriptors; included in staging_bytes.
    pub resource_staging_bytes: u64,
    /// Number of created pass and uploaded image textures.
    pub texture_count: u64,
    /// Sum of pass (eight bytes/pixel) and image (four bytes/pixel) descriptors.
    pub texture_bytes: u64,
    /// Number of created typed parameter buffers.
    pub uniform_count: u64,
    /// Sum of created uniform buffer descriptor sizes.
    pub uniform_bytes: u64,
    /// Number of created staging buffers, including separate aliased readbacks.
    pub staging_count: u64,
    /// Sum of created staging buffer descriptor sizes across uploads and readbacks.
    pub staging_bytes: u64,
    /// Maximum simultaneously live staging descriptor bytes.
    pub peak_staging_bytes: u64,
    /// Sum of all successful texture, uniform, and staging descriptor allocations.
    pub cumulative_bytes: u64,
    /// Maximum simultaneously live descriptor bytes before explicit destruction.
    pub peak_bytes: u64,
    /// Descriptor bytes not yet destroyed; zero on a completed render report.
    pub live_bytes: u64,
    /// Descriptor bytes explicitly destroyed before this report was returned.
    pub released_bytes: u64,
    /// Logical pass-result bytes served by already allocated physical slots.
    pub reused_bytes: u64,
}

#[derive(Clone, Default)]
pub(crate) struct Allocations {
    report: AllocationReport,
    live_staging_bytes: u64,
}

impl Allocations {
    pub fn report(&self) -> AllocationReport {
        self.report.clone()
    }

    pub fn texture(&mut self, texture: u64) -> Result<(), GpuOperationError> {
        let stage = Stage::GpuExecution;
        let texture_count = add(self.report.texture_count, 1, stage)?;
        let texture_bytes = add(self.report.texture_bytes, texture, stage)?;
        self.allocate(texture, stage)?;
        self.report.texture_count = texture_count;
        self.report.texture_bytes = texture_bytes;
        Ok(())
    }

    pub fn uniform(&mut self, uniform: u64) -> Result<(), GpuOperationError> {
        let stage = Stage::GpuExecution;
        let uniform_count = add(self.report.uniform_count, 1, stage)?;
        let uniform_bytes = add(self.report.uniform_bytes, uniform, stage)?;
        self.allocate(uniform, stage)?;
        self.report.uniform_count = uniform_count;
        self.report.uniform_bytes = uniform_bytes;
        Ok(())
    }

    pub fn reuse(&mut self, bytes: u64) -> Result<(), GpuOperationError> {
        self.report.reused_bytes = add(self.report.reused_bytes, bytes, Stage::GpuExecution)?;
        Ok(())
    }

    pub fn image(&mut self, texture: u64, staging: u64) -> Result<(), GpuOperationError> {
        let mut next = self.clone();
        let stage = Stage::GpuExecution;
        next.staging(staging)?;
        next.allocate(texture, stage)?;
        next.report.texture_count = add(next.report.texture_count, 1, stage)?;
        next.report.texture_bytes = add(next.report.texture_bytes, texture, stage)?;
        next.report.resource_texture_bytes =
            add(next.report.resource_texture_bytes, texture, stage)?;
        next.report.resource_staging_bytes =
            add(next.report.resource_staging_bytes, staging, stage)?;
        *self = next;
        Ok(())
    }
    pub fn uploaded(&mut self, packed: u64) -> Result<(), GpuOperationError> {
        let count = add(self.report.resource_count, 1, Stage::GpuExecution)?;
        let bytes = add(
            self.report.resource_upload_bytes,
            packed,
            Stage::GpuExecution,
        )?;
        self.report.resource_count = count;
        self.report.resource_upload_bytes = bytes;
        Ok(())
    }
    pub fn staging(&mut self, bytes: u64) -> Result<(), GpuOperationError> {
        let stage = Stage::Readback;
        let count = add(self.report.staging_count, 1, stage)?;
        let total = add(self.report.staging_bytes, bytes, stage)?;
        let live = add(self.live_staging_bytes, bytes, stage)?;
        self.allocate(bytes, stage)?;
        self.report.staging_count = count;
        self.report.staging_bytes = total;
        self.live_staging_bytes = live;
        self.report.peak_staging_bytes = self.report.peak_staging_bytes.max(live);
        Ok(())
    }

    fn allocate(&mut self, bytes: u64, stage: Stage) -> Result<(), GpuOperationError> {
        let cumulative = add(self.report.cumulative_bytes, bytes, stage)?;
        let live = add(self.report.live_bytes, bytes, stage)?;
        self.report.cumulative_bytes = cumulative;
        self.report.live_bytes = live;
        self.report.peak_bytes = self.report.peak_bytes.max(live);
        Ok(())
    }

    pub fn release_staging(&mut self, bytes: u64) {
        // Each staging guard releases its recorded size exactly once on drop.
        self.live_staging_bytes -= bytes;
        self.release(bytes);
    }

    pub fn release_passes(&mut self) {
        // Resources calls this once after draining its recorded textures/uniforms.
        self.release(self.report.texture_bytes + self.report.uniform_bytes);
    }

    fn release(&mut self, bytes: u64) {
        // Guards own recorded allocations, so releases cannot exceed allocation
        // totals, whose addition was checked before the allocation was retained.
        self.report.live_bytes -= bytes;
        self.report.released_bytes += bytes;
    }
}

fn add(a: u64, b: u64, stage: Stage) -> Result<u64, GpuOperationError> {
    a.checked_add(b).ok_or_else(|| {
        GpuOperationError::at(stage, "GPU descriptor allocation accounting overflowed.")
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allocation_accounting_rejects_overflow_without_partial_counters() {
        let mut allocations = Allocations::default();
        allocations.texture(64).unwrap();
        allocations.uniform(16).unwrap();
        let before = allocations.report();
        let error = allocations.texture(u64::MAX).unwrap_err();
        assert_eq!(error.diagnostic().stage, Stage::GpuExecution);
        assert_eq!(allocations.report(), before);
        assert!(allocations.uniform(u64::MAX).is_err());
        assert_eq!(allocations.report(), before);
        let error = allocations.staging(u64::MAX).unwrap_err();
        assert_eq!(error.diagnostic().stage, Stage::Readback);
        assert_eq!(allocations.report(), before);
        assert!(allocations.image(u64::MAX, 1536).is_err());
        assert_eq!(allocations.report(), before);
        allocations.release_passes();
        assert_eq!(allocations.report().live_bytes, 0);
        assert_eq!(allocations.report().released_bytes, 80);
    }

    #[test]
    fn image_descriptors_and_upload_counts_are_distinct_and_release_once() {
        let mut allocations = Allocations::default();
        allocations.image(780, 1536).unwrap();
        assert_eq!(allocations.report().resource_count, 0);
        allocations.uploaded(780).unwrap();
        let report = allocations.report();
        assert_eq!(report.resource_count, 1);
        assert_eq!(report.resource_upload_bytes, 780);
        assert_eq!(report.texture_bytes, report.resource_texture_bytes);
        assert_eq!(report.staging_bytes, report.resource_staging_bytes);
        assert_eq!(report.cumulative_bytes, 2316);
        allocations.release_staging(1536);
        allocations.release_passes();
        assert_eq!(allocations.report().released_bytes, 2316);
        assert_eq!(allocations.report().live_bytes, 0);
    }
}
