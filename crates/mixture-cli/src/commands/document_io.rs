//! Shared bounded source loading for CPU-only document commands.
use mixture_core::{
    Diagnostic, DiagnosticCode, EvidenceValue, MaterialDocument, RecipeLimits, RecipeSet,
    SafetyLimits, Stage, ValidatedDocument, decode_with_recipes,
};
use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};

/// Load a `.mix` source. Without recipe files this is the unchanged v1-only path;
/// with explicit `--recipe` files Core decodes a v2 parent against exactly them.
/// Diagnostics raised by a recipe file name that file as their document path.
pub(super) fn load(
    path: &Path,
    limits: &SafetyLimits,
    recipes: &[PathBuf],
) -> Result<ValidatedDocument, (Vec<Diagnostic>, u8)> {
    let bytes = read(path, limits.decoded_bytes, "material")?;
    if recipes.is_empty() {
        return MaterialDocument::decode(&bytes, limits)
            .and_then(|doc| doc.into_validated(limits))
            .map_err(|error| (error.report().diagnostics().to_vec(), 2));
    }
    let recipe_limits = RecipeLimits::default();
    let mut sources = Vec::new();
    if recipes.len() as u64 > recipe_limits.recipe_count {
        // Let Core report the count ceiling without reading any recipe file.
        sources.resize(recipes.len(), Vec::new());
    } else {
        let mut total = 0_u64;
        for recipe in recipes {
            let bytes = read(recipe, limits.decoded_bytes, "recipe").map_err(|(mut d, exit)| {
                d.iter_mut()
                    .for_each(|d| d.document_path = Some(recipe.to_string_lossy().into_owned()));
                (d, exit)
            })?;
            total = total.saturating_add(bytes.len() as u64);
            sources.push(bytes);
            if total > recipe_limits.recipe_bytes {
                break; // Core reports the first exceeding total; later files stay unread.
            }
        }
    }
    let slices: Vec<&[u8]> = sources.iter().map(Vec::as_slice).collect();
    RecipeSet::decode(&slices, limits, &recipe_limits)
        .and_then(|set| decode_with_recipes(&bytes, &set, limits, &recipe_limits))
        .map_err(|error| {
            let diagnostics = error.report().diagnostics().iter().cloned().map(|mut d| {
                if let Some(EvidenceValue::Unsigned(index)) = d.evidence.get("recipeIndex")
                    && let Some(recipe) = recipes.get(*index as usize)
                {
                    d.document_path = Some(recipe.to_string_lossy().into_owned());
                }
                d
            });
            (diagnostics.collect(), 2)
        })
}

/// Read at most `limit` bytes plus one detection byte, so Core reports size.
fn read(path: &Path, limit: u64, kind: &str) -> Result<Vec<u8>, (Vec<Diagnostic>, u8)> {
    let read = || -> std::io::Result<Vec<u8>> {
        let mut bytes = Vec::new();
        File::open(path)?
            .take(limit.saturating_add(1))
            .read_to_end(&mut bytes)?;
        Ok(bytes)
    };
    read().map_err(|source| {
        (
            vec![
                Diagnostic::error(
                    DiagnosticCode::IoReadFailed,
                    Stage::Parse,
                    format!("Could not read the {kind} file."),
                )
                .with_evidence("sourceMessage", source.to_string())
                .with_source(source)
                .with_suggestion("Check the input path, file type, and read permissions."),
            ],
            1,
        )
    })
}
