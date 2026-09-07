//! Basic offline Markdown file-link checks; no network or heading-anchor checks.

use std::{
    fs,
    path::{Path, PathBuf},
};

use pulldown_cmark::{Event, Parser, Tag};

use crate::TaskResult;

pub(super) fn check(root: &Path) -> TaskResult {
    let mut documents = Vec::new();
    collect_markdown(root, &mut documents)?;
    documents.sort();
    let mut errors = Vec::new();
    for document in &documents {
        let text = fs::read_to_string(document)?;
        let directory = document.parent().ok_or("document has no parent")?;
        for (destination, offset) in destinations(&text) {
            let Some(path) = local_path(&destination) else {
                continue;
            };
            let target = directory.join(path);
            if !target.exists() {
                let line = text[..offset].bytes().filter(|byte| *byte == b'\n').count() + 1;
                errors.push(format!(
                    "{}:{line}: missing link target {destination}",
                    document.strip_prefix(root)?.display()
                ));
            }
        }
    }
    if !errors.is_empty() {
        return Err(errors.join("\n").into());
    }
    println!(
        "Local document links passed ({} Markdown files; external URLs and anchors excluded).",
        documents.len()
    );
    Ok(())
}

fn collect_markdown(directory: &Path, documents: &mut Vec<PathBuf>) -> TaskResult {
    let mut entries = fs::read_dir(directory)?.collect::<Result<Vec<_>, _>>()?;
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        let kind = entry.file_type()?;
        if kind.is_dir() {
            if !matches!(
                name.as_ref(),
                ".git" | "target" | "tmp" | "out" | "node_modules"
            ) {
                collect_markdown(&entry.path(), documents)?;
            }
        } else if kind.is_file()
            && entry
                .path()
                .extension()
                .is_some_and(|extension| extension == "md")
        {
            documents.push(entry.path());
        }
    }
    Ok(())
}

fn destinations(text: &str) -> Vec<(String, usize)> {
    Parser::new(text)
        .into_offset_iter()
        .filter_map(|(event, range)| match event {
            Event::Start(Tag::Link { dest_url, .. } | Tag::Image { dest_url, .. }) => {
                Some((dest_url.into_string(), range.start))
            }
            _ => None,
        })
        .collect()
}

fn local_path(destination: &str) -> Option<&str> {
    if destination.starts_with("//") || destination.contains(':') {
        return None;
    }
    let path = destination.split(['#', '?']).next()?;
    if path.is_empty() { None } else { Some(path) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_missing_files_with_locations_and_ignores_build_outputs() {
        struct Scratch(PathBuf);
        impl Drop for Scratch {
            fn drop(&mut self) {
                let _ = fs::remove_dir_all(&self.0);
            }
        }

        let scratch =
            Scratch(std::env::temp_dir().join(format!("mixture-link-test-{}", std::process::id())));
        fs::create_dir(&scratch.0).expect("create isolated test directory");
        fs::create_dir(scratch.0.join("docs")).expect("create docs directory");
        fs::create_dir(scratch.0.join("target")).expect("create build directory");
        fs::write(scratch.0.join("target/ignored.md"), "[broken](missing.md)")
            .expect("write ignored build artifact");
        fs::write(
            scratch.0.join("README.md"),
            "# Guide\n\n[doc](docs/guide.md)\n",
        )
        .expect("write source document");
        let error = check(&scratch.0).expect_err("missing file must fail");
        assert!(
            error
                .to_string()
                .contains("README.md:3: missing link target docs/guide.md")
        );
        fs::write(
            scratch.0.join("docs/guide.md"),
            "[back](../README.md#guide)",
        )
        .expect("write link target");
        check(&scratch.0).expect("existing local links must pass");
    }

    #[test]
    fn parses_inline_reference_and_image_links_but_ignores_code() {
        let text = "[guide](docs/guide.md)\n![image](fixtures/checker.png)\n[reference][id]\n\n[id]: <docs/a guide.md>\n\n`[ignore](missing.md)`\n\n```md\n[ignore](also-missing.md)\n```\n";
        let links: Vec<_> = destinations(text)
            .into_iter()
            .map(|(path, _)| path)
            .collect();
        assert_eq!(
            links,
            ["docs/guide.md", "fixtures/checker.png", "docs/a guide.md"]
        );
    }

    #[test]
    fn separates_local_paths_from_external_urls_and_anchors() {
        assert_eq!(local_path("../README.md#mission"), Some("../README.md"));
        assert_eq!(local_path("guide.md?view=source#title"), Some("guide.md"));
        for destination in [
            "#title",
            "https://example.com",
            "mailto:author@example.com",
            "//example.com/file",
            "",
        ] {
            assert_eq!(local_path(destination), None);
        }
    }
}
