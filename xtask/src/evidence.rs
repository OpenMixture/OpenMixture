//! Evidence growth guard: raw run output and oversized files need an explicit decision.
//!
//! Evidence areas (`docs/evidence`, `docs/reviews` and `fixtures/**/reports`) reject raw
//! `.log`/`.stdout`/`.stderr` output, archives and per-run `cli-<pid>-<time>` folders. Every
//! tracked file is limited to 4 MiB. Sizes are indexed blob sizes, so line-ending settings
//! cannot change a verdict. [`EXCEPTIONS`] lists, per directory, how many such files are
//! retained deliberately: records that predate this guard and failure evidence promoted under
//! the evidence policy. Stale counts must be lowered, so the list shrinks as records are
//! pruned; adding or raising a line is a retention decision reviewed in the pull request.

use std::{
    collections::BTreeMap,
    io::Write,
    path::Path,
    process::{Command, Stdio},
};

use crate::TaskResult;

/// Deliberately retained files, one `<count> <directory>` line per directory.
const EXCEPTIONS: &str = "docs/evidence/retention-exceptions.txt";
const SIZE_LIMIT: u64 = 4 << 20;

pub(super) fn check(root: &Path) -> TaskResult {
    let tracked = tracked_blobs(root)?;
    let exceptions = parse_exceptions(&indexed_exceptions(root)?)?;
    let errors = verify(&tracked, &exceptions);
    if !errors.is_empty() {
        return Err(format!(
            "{}\n\nKeep ordinary run output in CI artifacts or ignored tmp/ directories. Retaining \
             such a file is a deliberate decision under docs/evidence-policy.md: list its \
             directory in {EXCEPTIONS} and justify it in the pull request.",
            errors.join("\n")
        )
        .into());
    }
    println!(
        "Evidence growth guard passed ({} tracked files; {} directories with retention exceptions).",
        tracked.len(),
        exceptions.len()
    );
    Ok(())
}

/// Read the policy from the same index as the files being checked.
fn indexed_exceptions(root: &Path) -> TaskResult<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["show", &format!(":{EXCEPTIONS}")])
        .output()?;
    if !output.status.success() {
        return Err(format!("Cannot read {EXCEPTIONS} from the Git index; stage the retention policy before checking evidence.").into());
    }
    Ok(String::from_utf8(output.stdout)?)
}

/// Every indexed blob with its stored size, in index order.
fn tracked_blobs(root: &Path) -> TaskResult<Vec<(String, u64)>> {
    let listing = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["ls-files", "--stage", "-z"])
        .output()?;
    if !listing.status.success() {
        return Err("git ls-files failed; the evidence guard requires a Git checkout".into());
    }
    let mut entries = Vec::new();
    for record in listing.stdout.split(|byte| *byte == 0) {
        if record.is_empty() {
            continue;
        }
        let record = std::str::from_utf8(record)?;
        let (meta, path) = record
            .split_once('\t')
            .ok_or("malformed git ls-files record")?;
        let mut fields = meta.split(' ');
        let (Some(mode), Some(object)) = (fields.next(), fields.next()) else {
            return Err("malformed git ls-files record".into());
        };
        // Submodule commits (160000) carry no blob content in this repository.
        if mode != "160000" {
            entries.push((path.to_owned(), object.to_owned()));
        }
    }
    let mut sizes = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["cat-file", "--batch-check=%(objectsize)"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;
    let mut input = sizes.stdin.take().ok_or("git cat-file has no stdin")?;
    let objects: String = entries
        .iter()
        .map(|(_, object)| format!("{object}\n"))
        .collect();
    // Write from a separate thread so a full stdout pipe cannot deadlock the request.
    let output = std::thread::scope(|scope| {
        let writer = scope.spawn(move || input.write_all(objects.as_bytes()));
        let output = sizes.wait_with_output();
        (writer.join(), output)
    });
    let (Ok(written), output) = output else {
        return Err("git cat-file input thread panicked".into());
    };
    written?;
    let output = output?;
    if !output.status.success() {
        return Err("git cat-file failed while measuring tracked blobs".into());
    }
    let lines: Vec<&str> = std::str::from_utf8(&output.stdout)?.lines().collect();
    if lines.len() != entries.len() {
        return Err("git cat-file returned an unexpected number of sizes".into());
    }
    entries
        .into_iter()
        .zip(lines)
        .map(|((path, _), size)| Ok((path, size.trim().parse()?)))
        .collect()
}

fn evidence_area(path: &str) -> bool {
    path.starts_with("docs/evidence/")
        || path.starts_with("docs/reviews/")
        || (path.starts_with("fixtures/") && path.contains("/reports/"))
}

/// `cli-<pid>-<timestamp>` folders written once per consumer run.
fn per_run_directory(segment: &str) -> bool {
    let mut parts = segment.split('-');
    let digits = |part: Option<&str>| {
        part.is_some_and(|part| !part.is_empty() && part.bytes().all(|b| b.is_ascii_digit()))
    };
    parts.next() == Some("cli")
        && digits(parts.next())
        && digits(parts.next())
        && parts.next().is_none()
}

fn violation(path: &str, bytes: u64) -> Option<&'static str> {
    if bytes > SIZE_LIMIT {
        return Some("tracked file above 4 MiB");
    }
    if !evidence_area(path) {
        return None;
    }
    let (directory, name) = path.rsplit_once('/').unwrap_or(("", path));
    if [".log", ".stdout", ".stderr"]
        .iter()
        .any(|suffix| name.ends_with(suffix))
    {
        Some("raw run output")
    } else if [".tar.gz", ".tgz", ".zip"]
        .iter()
        .any(|suffix| name.ends_with(suffix))
    {
        Some("archive in an evidence area")
    } else if directory.split('/').any(per_run_directory) {
        Some("per-run CLI output folder")
    } else {
        None
    }
}

fn parse_exceptions(text: &str) -> TaskResult<BTreeMap<String, usize>> {
    let mut exceptions = BTreeMap::new();
    let mut previous: Option<&str> = None;
    for (index, line) in text.lines().enumerate() {
        let line_number = index + 1;
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let malformed = || format!("{EXCEPTIONS}:{line_number}: expected `<count> <directory>`");
        let (count, directory) = line.split_once(' ').ok_or_else(malformed)?;
        let count: usize = count.parse().map_err(|_| malformed())?;
        if count == 0 || directory.is_empty() || directory.ends_with('/') {
            return Err(malformed().into());
        }
        if previous.is_some_and(|previous| previous >= directory) {
            return Err(format!(
                "{EXCEPTIONS}:{line_number}: directories must be sorted and unique"
            )
            .into());
        }
        previous = Some(directory);
        exceptions.insert(directory.to_owned(), count);
    }
    Ok(exceptions)
}

fn verify(tracked: &[(String, u64)], exceptions: &BTreeMap<String, usize>) -> Vec<String> {
    let mut observed: BTreeMap<&str, Vec<(&str, &str)>> = BTreeMap::new();
    for (path, bytes) in tracked {
        if let Some(reason) = violation(path, *bytes) {
            let directory = path.rsplit_once('/').map_or("", |(directory, _)| directory);
            observed.entry(directory).or_default().push((path, reason));
        }
    }
    let mut errors = Vec::new();
    for (directory, files) in &observed {
        match exceptions.get(*directory) {
            Some(allowed) if files.len() <= *allowed => {}
            Some(allowed) => {
                errors.push(format!(
                    "{directory}: {} files need a retention exception, but {EXCEPTIONS} lists {allowed}:",
                    files.len()
                ));
                errors.extend(
                    files
                        .iter()
                        .map(|(path, reason)| format!("  {path}: {reason}")),
                );
            }
            None => errors.extend(
                files
                    .iter()
                    .map(|(path, reason)| format!("{path}: {reason}")),
            ),
        }
    }
    for (directory, allowed) in exceptions {
        let remaining = observed.get(directory.as_str()).map_or(0, Vec::len);
        if remaining == 0 {
            errors.push(format!(
                "{EXCEPTIONS}: remove `{allowed} {directory}`; no such files remain"
            ));
        } else if remaining < *allowed {
            errors.push(format!(
                "{EXCEPTIONS}: lower `{allowed} {directory}` to `{remaining} {directory}`"
            ));
        }
    }
    errors
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unstaged_exceptions_cannot_authorize_indexed_files() {
        let root = std::env::temp_dir().join(format!(
            "mixture-evidence-index-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(root.join("docs/evidence/new")).unwrap();
        let git = |args: &[&str]| {
            let output = Command::new("git")
                .arg("-C")
                .arg(&root)
                .args(args)
                .output()
                .unwrap();
            assert!(output.status.success(), "{output:?}");
        };
        git(&["init", "--quiet"]);
        assert!(indexed_exceptions(&root).is_err());
        std::fs::write(root.join(EXCEPTIONS), "# no exceptions\n").unwrap();
        std::fs::write(root.join("docs/evidence/new/run.log"), "output\n").unwrap();
        git(&["add", "--", "docs/evidence"]);
        std::fs::write(root.join(EXCEPTIONS), "1 docs/evidence/new\n").unwrap();
        assert!(
            check(&root).is_err(),
            "unstaged exception must not authorize a log"
        );
        git(&["add", "--", EXCEPTIONS]);
        assert!(
            check(&root).is_ok(),
            "staged exception must authorize the log"
        );
        std::fs::remove_file(root.join(EXCEPTIONS)).unwrap();
        assert!(
            check(&root).is_ok(),
            "working-tree deletion must not change the index policy"
        );
        std::fs::remove_dir_all(root).unwrap();
    }

    fn tracked(entries: &[(&str, u64)]) -> Vec<(String, u64)> {
        entries
            .iter()
            .map(|(path, bytes)| ((*path).to_owned(), *bytes))
            .collect()
    }

    #[test]
    fn classifies_raw_output_archives_run_folders_and_sizes() {
        let cases = [
            ("docs/evidence/x/run.log", 1, Some("raw run output")),
            ("docs/reviews/m3/doctor.stderr", 1, Some("raw run output")),
            (
                "fixtures/materials/wood/reports/render.stdout",
                1,
                Some("raw run output"),
            ),
            (
                "docs/evidence/x/bundle.tar.gz",
                1,
                Some("archive in an evidence area"),
            ),
            (
                "docs/evidence/x/bundle.tar.gz",
                SIZE_LIMIT + 1,
                Some("tracked file above 4 MiB"),
            ),
            (
                "docs/evidence/x/cli-12-345/report.json",
                1,
                Some("per-run CLI output folder"),
            ),
            ("docs/evidence/x/cli-12-345-6/report.json", 1, None),
            ("docs/evidence/x/cli-12/report.json", 1, None),
            ("docs/evidence/x/sheet.png", SIZE_LIMIT, None),
            (
                "docs/evidence/x/sheet.png",
                SIZE_LIMIT + 1,
                Some("tracked file above 4 MiB"),
            ),
            (
                "fixtures/materials/leather/expected/normal.png",
                SIZE_LIMIT,
                None,
            ),
            (
                "crates/mixture-core/tests/large.bin",
                SIZE_LIMIT + 1,
                Some("tracked file above 4 MiB"),
            ),
            ("fixtures/materials/leather/expected/render.log", 1, None),
            ("crates/mixture-cli/tests/output.log", 1, None),
            ("docs/guide.md", 1, None),
        ];
        for (path, bytes, expected) in cases {
            assert_eq!(violation(path, bytes), expected, "{path} ({bytes} bytes)");
        }
    }

    #[test]
    fn exceptions_must_cover_violations_and_track_deletions() {
        let exceptions =
            parse_exceptions("# comment\n1 docs/evidence/gone\n2 docs/evidence/old\n").unwrap();
        let old = [
            ("docs/evidence/old/a.log", 1),
            ("docs/evidence/old/b.log", 1),
        ];
        let mut files = tracked(&old);
        files.push(("docs/evidence/gone/c.log".into(), 1));
        assert!(verify(&files, &exceptions).is_empty());

        files.push(("docs/evidence/old/new.stdout".into(), 1));
        files.push(("docs/evidence/new/huge.png".into(), SIZE_LIMIT + 1));
        let errors = verify(&files, &exceptions).join("\n");
        assert!(
            errors.contains("docs/evidence/old: 3 files need a retention exception"),
            "{errors}"
        );
        assert!(
            errors.contains("docs/evidence/new/huge.png: tracked file above 4 MiB"),
            "{errors}"
        );

        let errors = verify(&tracked(&old[..1]), &exceptions).join("\n");
        assert!(
            errors.contains("lower `2 docs/evidence/old` to `1 docs/evidence/old`"),
            "{errors}"
        );
        assert!(errors.contains("remove `1 docs/evidence/gone`"), "{errors}");
    }

    #[test]
    fn rejects_malformed_or_unsorted_exceptions() {
        for text in [
            "docs/evidence/a\n",
            "0 docs/evidence/a\n",
            "x docs/evidence/a\n",
            "1 docs/evidence/a/\n",
            "1 b\n1 a\n",
            "1 a\n1 a\n",
        ] {
            assert!(parse_exceptions(text).is_err(), "{text:?}");
        }
    }
}
