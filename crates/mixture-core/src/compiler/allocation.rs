//! Stable per-call physical slots; logical pass order and shader semantics are unchanged.
use crate::{
    CompileError,
    compiler::invariant,
    plan::{AllocationPlan, ComputePass, PlanOutput, TextureSlotId},
};

pub(super) fn plan(
    passes: &[ComputePass],
    outputs: &[PlanOutput],
) -> Result<AllocationPlan, CompileError> {
    let end = u32::try_from(passes.len())
        .map_err(|_| invariant("Allocation pass count exceeds the plan index range."))?;
    let mut last_uses: Vec<_> = (0..end).collect();
    for (index, pass) in passes.iter().enumerate() {
        if pass.id.index() as usize != index || pass.output.index() as usize != index {
            return Err(invariant(
                "Allocation requires consecutive producer identities.",
            ));
        }
        for input in pass.kernel.inputs() {
            if input.index() >= pass.id.index() {
                return Err(invariant("Allocation input must precede its consumer."));
            }
            let last = last_uses
                .get_mut(input.index() as usize)
                .ok_or_else(|| invariant("Allocation input producer is missing."))?;
            *last = (*last).max(pass.id.index());
        }
    }
    for output in outputs {
        let last = last_uses
            .get_mut(output.resource.index() as usize)
            .ok_or_else(|| invariant("Allocation output producer is missing."))?;
        *last = end;
    }
    let mut slots = Vec::new();
    let mut occupants = Vec::new();
    let mut resource_slots = Vec::with_capacity(passes.len());
    for (pass, last) in passes.iter().zip(&last_uses) {
        let free = slots
            .iter()
            .zip(&occupants)
            .position(|(desc, occupied_until)| {
                *desc == pass.output_desc && *occupied_until < pass.id.index()
            });
        let index = match free {
            Some(index) => {
                let occupant = occupants
                    .get_mut(index)
                    .ok_or_else(|| invariant("Physical slot lifetime is missing."))?;
                *occupant = *last;
                index
            }
            None => {
                let index = slots.len();
                slots.push(pass.output_desc);
                occupants.push(*last);
                index
            }
        };
        let index = u32::try_from(index)
            .map_err(|_| invariant("Physical slot count exceeds the plan index range."))?;
        resource_slots.push(TextureSlotId(index));
    }
    Ok(AllocationPlan {
        slots,
        resource_slots,
        last_uses,
    })
}
