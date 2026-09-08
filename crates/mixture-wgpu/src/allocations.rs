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
    /// Number of created rgba16float pass textures.
    pub texture_count: u64,
    /// Sum of created pass texture descriptor bytes, at eight bytes per pixel.
    pub texture_bytes: u64,
    /// Number of created typed parameter buffers.
    pub uniform_count: u64,
    /// Sum of created uniform buffer descriptor sizes.
    pub uniform_bytes: u64,
    /// Number of created staging buffers, including separate aliased readbacks.
    pub staging_count: u64,
    /// Sum of created staging buffer descriptor sizes across all readbacks.
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
    /// Bytes served by resource reuse; zero for the current retain-all schedule.
    pub reused_bytes: u64,
}

#[derive(Default)]
pub(crate) struct Allocations {
    report: AllocationReport,
    live_staging_bytes: u64,
}

impl Allocations {
    pub fn report(&self) -> AllocationReport {
        self.report.clone()
    }

    pub fn pass(&mut self, texture: u64, uniform: u64) -> Result<(), GpuOperationError> {
        let stage = Stage::GpuExecution;
        let bytes = add(texture, uniform, stage)?;
        let texture_count = add(self.report.texture_count, 1, stage)?;
        let uniform_count = add(self.report.uniform_count, 1, stage)?;
        let texture_bytes = add(self.report.texture_bytes, texture, stage)?;
        let uniform_bytes = add(self.report.uniform_bytes, uniform, stage)?;
        self.allocate(bytes, stage)?;
        self.report.texture_count = texture_count;
        self.report.uniform_count = uniform_count;
        self.report.texture_bytes = texture_bytes;
        self.report.uniform_bytes = uniform_bytes;
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
        allocations.pass(64, 16).unwrap();
        let before = allocations.report();
        let error = allocations.pass(u64::MAX, 16).unwrap_err();
        assert_eq!(error.diagnostic().stage, Stage::GpuExecution);
        assert_eq!(allocations.report(), before);
        let error = allocations.staging(u64::MAX).unwrap_err();
        assert_eq!(error.diagnostic().stage, Stage::Readback);
        assert_eq!(allocations.report(), before);
        allocations.release_passes();
        assert_eq!(allocations.report().live_bytes, 0);
        assert_eq!(allocations.report().released_bytes, 80);
    }
}
