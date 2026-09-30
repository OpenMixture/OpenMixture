//! Require evidence that filtered libtest runs actually passed tests.
use crate::TaskResult;
use std::{io::Write, process::Command};

fn passed_tests(stdout: &str) -> bool {
    stdout.lines().any(|line| {
        line.strip_prefix("test result: ok. ")
            .and_then(|rest| rest.split_once(" passed;"))
            .and_then(|(count, _)| count.parse::<u64>().ok())
            .is_some_and(|count| count > 0)
    })
}

pub(super) fn run(command: &mut Command) -> TaskResult {
    let output = command.output()?;
    std::io::stdout().write_all(&output.stdout)?;
    std::io::stderr().write_all(&output.stderr)?;
    if !output.status.success() {
        return Err(format!("filtered test run failed ({})", output.status).into());
    }
    if !passed_tests(&String::from_utf8_lossy(&output.stdout)) {
        return Err("filtered test run passed zero tests (or had no libtest summary)".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn successful_process_with_zero_filtered_tests_is_rejected() {
        let mut command = Command::new(std::env::current_exe().unwrap());
        command.args(["__no_matching_adversarial_test__", "--exact"]);
        let error = run(&mut command).unwrap_err();
        assert!(error.to_string().contains("zero tests"));
    }
    #[test]
    fn rejects_empty_filtered_ignored_and_incomplete_runs() {
        for output in [
            "",
            "running 0 tests\n",
            "running 1 test\n",
            "test result: ok. 0 passed; 0 failed; 1 ignored; 0 measured; 9 filtered out;\n",
            "test result: FAILED. 1 passed; 1 failed;",
            "test result: ok. invalid passed;",
        ] {
            assert!(!passed_tests(output), "{output}");
        }
    }
    #[test]
    fn accepts_completed_tests_among_empty_harnesses_and_windows_lines() {
        for output in [
            "test result: ok. 1 passed; 0 failed;\r\n",
            "test result: ok. 0 passed; 0 failed;\ntest result: ok. 13 passed; 0 failed;\n",
        ] {
            assert!(passed_tests(output), "{output}");
        }
    }
}
