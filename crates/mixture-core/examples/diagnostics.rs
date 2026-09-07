//! Demonstrate only the public PR-002 API; no graph or GPU is initialized.

use mixture_core::{DiagnosticReport, LimitKind, SafetyLimits, Stage};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let limits = SafetyLimits::default();
    let diagnostics = [(LimitKind::Nodes, 129), (LimitKind::OutputDimension, 2049)]
        .into_iter()
        .filter_map(|(kind, observed)| limits.check(kind, observed).err())
        .map(|violation| violation.diagnostic(Stage::Validation));
    let report = DiagnosticReport::new(diagnostics);
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}
