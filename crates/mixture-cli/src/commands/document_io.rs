//! Shared bounded source loading for CPU-only document commands.
use mixture_core::{
    Diagnostic, DiagnosticCode, MaterialDocument, SafetyLimits, Stage, ValidatedDocument,
};
use std::{fs::File, io::Read, path::Path};

pub(super) fn load(
    path: &Path,
    limits: &SafetyLimits,
) -> Result<ValidatedDocument, (Vec<Diagnostic>, u8)> {
    let read = || -> std::io::Result<Vec<u8>> {
        let mut bytes = Vec::new();
        File::open(path)?
            .take(limits.decoded_bytes.saturating_add(1))
            .read_to_end(&mut bytes)?;
        Ok(bytes)
    };
    let bytes = read().map_err(|source| {
        (
            vec![
                Diagnostic::error(
                    DiagnosticCode::IoReadFailed,
                    Stage::Parse,
                    "Could not read the material file.",
                )
                .with_evidence("sourceMessage", source.to_string())
                .with_source(source)
                .with_suggestion("Check the input path, file type, and read permissions."),
            ],
            1,
        )
    })?;
    MaterialDocument::decode(&bytes, limits)
        .and_then(|doc| doc.into_validated(limits))
        .map_err(|error| (error.report().diagnostics().to_vec(), 2))
}
