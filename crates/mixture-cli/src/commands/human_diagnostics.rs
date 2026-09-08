//! Shared presentation of the existing public diagnostics; no error semantics.
use mixture_core::DiagnosticReport;
use std::io::{self, Write};

pub(super) fn write(out: &mut impl Write, report: &DiagnosticReport) -> io::Result<()> {
    for diagnostic in report.diagnostics() {
        writeln!(out, "{diagnostic}")?;
        writeln!(
            out,
            "  stage: {:?}, severity: {:?}",
            diagnostic.stage, diagnostic.severity
        )?;
        if let Some(path) = &diagnostic.document_path {
            writeln!(out, "  document: {path}")?;
        }
        if let Some(node) = &diagnostic.node_id {
            writeln!(out, "  node: {node}")?;
        }
        if let Some(port) = &diagnostic.port_id {
            writeln!(out, "  port: {port}")?;
        }
        if let Some(parameter) = &diagnostic.parameter_id {
            writeln!(out, "  parameter: {parameter}")?;
        }
        for (key, value) in &diagnostic.evidence {
            writeln!(out, "  {key}: {value:?}")?;
        }
        if let Some(suggestion) = &diagnostic.suggestion {
            writeln!(out, "  Suggestion: {suggestion}")?;
        }
    }
    Ok(())
}
