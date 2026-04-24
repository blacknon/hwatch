use assert_cmd::Command;
use predicates::prelude::*;
use predicates::str::is_match;
use regex::Regex;
use std::fs;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
#[cfg(unix)]
use std::process::Command as ProcessCommand;
use std::time::Duration;
use tempfile::tempdir;

fn strip_ansi(text: &str) -> String {
    let ansi = Regex::new(r"\x1b\[[0-9;]*m").unwrap();
    ansi.replace_all(text, "").into_owned()
}

fn stdout_text(assert: &assert_cmd::assert::Assert) -> String {
    String::from_utf8_lossy(assert.get_output().stdout.as_slice()).into_owned()
}

fn stdout_text_without_ansi(assert: &assert_cmd::assert::Assert) -> String {
    strip_ansi(&stdout_text(assert))
}

#[cfg(unix)]
fn write_executable_script(script_path: &std::path::Path, body: &str) {
    fs::write(script_path, body).unwrap();
    let mut perms = fs::metadata(script_path).unwrap().permissions();
    perms.set_mode(0o755);
    fs::set_permissions(script_path, perms).unwrap();
}

#[test]
fn help_flag_prints_usage() {
    let mut cmd = Command::cargo_bin("hwatch").unwrap();
    cmd.arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Usage:"))
        .stdout(predicate::str::contains("--batch"));
}

#[test]
fn unreadable_logfile_fails_fast_in_non_interactive_mode() {
    let temp = tempdir().unwrap();
    let logfile = temp.path().join("broken.jsonl");
    fs::write(&logfile, "{not-json}\n").unwrap();

    let mut cmd = Command::cargo_bin("hwatch").unwrap();
    cmd.args([
        "--logfile",
        logfile.to_str().unwrap(),
        "-b",
        "echo",
        "hello",
    ]);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("--force-logfile-overwrite"));
}

#[cfg(unix)]
#[test]
fn force_logfile_overwrite_allows_reusing_unreadable_logfile() {
    let temp = tempdir().unwrap();
    let logfile = temp.path().join("broken.jsonl");
    let counter_path = temp.path().join("counter.txt");
    let script_path = temp.path().join("increment.sh");
    fs::write(&logfile, "{not-json}\n").unwrap();
    write_executable_script(
        &script_path,
        format!(
            "#!/bin/sh\ncount_file=\"{}\"\ncount=0\nif [ -f \"$count_file\" ]; then\n  count=$(cat \"$count_file\")\nfi\ncount=$((count + 1))\nprintf '%s' \"$count\" > \"$count_file\"\nprintf 'hello-%s\\n' \"$count\"\n",
            counter_path.display()
        )
        .as_str(),
    );

    let mut cmd = Command::cargo_bin("hwatch").unwrap();
    cmd.args([
        "--force-logfile-overwrite",
        "--logfile",
        logfile.to_str().unwrap(),
        "-b",
        "-g",
        "1",
        "-n",
        "0.1",
        "sh",
        script_path.to_str().unwrap(),
    ]);
    cmd.timeout(Duration::from_secs(5));

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("hello-1"))
        .stdout(predicate::str::contains("hello-2"));
}

#[cfg(unix)]
#[test]
fn batch_mode_with_chgexit_runs_command_until_change_limit() {
    let temp = tempdir().unwrap();
    let counter_path = temp.path().join("counter.txt");
    let script_path = temp.path().join("increment.sh");

    write_executable_script(
        &script_path,
        format!(
            "#!/bin/sh\ncount_file=\"{}\"\ncount=0\nif [ -f \"$count_file\" ]; then\n  count=$(cat \"$count_file\")\nfi\ncount=$((count + 1))\nprintf '%s' \"$count\" > \"$count_file\"\nprintf '%s\\n' \"$count\"\n",
            counter_path.display()
        )
        .as_str(),
    );

    let mut cmd = Command::cargo_bin("hwatch").unwrap();
    cmd.args([
        "-b",
        "-g",
        "1",
        "-n",
        "0.1",
        "sh",
        script_path.to_str().unwrap(),
    ]);

    cmd.timeout(Duration::from_secs(5));

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("1"))
        .stdout(predicate::str::contains("2"));

    let counter = fs::read_to_string(&counter_path).unwrap();
    assert_eq!(counter, "2");
}

#[cfg(unix)]
#[test]
fn batch_mode_with_chgexit_two_waits_for_two_changes() {
    let temp = tempdir().unwrap();
    let counter_path = temp.path().join("counter.txt");
    let script_path = temp.path().join("increment.sh");

    write_executable_script(
        &script_path,
        format!(
            "#!/bin/sh\ncount_file=\"{}\"\ncount=0\nif [ -f \"$count_file\" ]; then\n  count=$(cat \"$count_file\")\nfi\ncount=$((count + 1))\nprintf '%s' \"$count\" > \"$count_file\"\nprintf 'tick-%s\\n' \"$count\"\n",
            counter_path.display()
        )
        .as_str(),
    );

    let mut cmd = Command::cargo_bin("hwatch").unwrap();
    cmd.args([
        "-b",
        "-g",
        "2",
        "-n",
        "0.1",
        "sh",
        script_path.to_str().unwrap(),
    ]);

    cmd.timeout(Duration::from_secs(5));

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("tick-1"))
        .stdout(predicate::str::contains("tick-2"))
        .stdout(predicate::str::contains("tick-3"));

    let counter = fs::read_to_string(&counter_path).unwrap();
    assert_eq!(counter, "3");
}

#[cfg(unix)]
#[test]
fn batch_mode_stdout_output_ignores_stderr_only_changes_in_printed_diff() {
    let temp = tempdir().unwrap();
    let counter_path = temp.path().join("counter.txt");
    let script_path = temp.path().join("stdout_stderr.sh");

    write_executable_script(
        &script_path,
        format!(
            "#!/bin/sh\ncount_file=\"{}\"\ncount=0\nif [ -f \"$count_file\" ]; then\n  count=$(cat \"$count_file\")\nfi\ncount=$((count + 1))\nprintf '%s' \"$count\" > \"$count_file\"\nprintf 'stdout-static\\n'\nprintf 'stderr-dynamic-%s\\n' \"$count\" >&2\n",
            counter_path.display()
        )
        .as_str(),
    );

    let mut cmd = Command::cargo_bin("hwatch").unwrap();
    cmd.args([
        "-b",
        "-g",
        "1",
        "-n",
        "0.1",
        "-o",
        "stdout",
        "sh",
        script_path.to_str().unwrap(),
    ]);

    cmd.timeout(Duration::from_secs(5));

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("stdout-static"))
        .stdout(predicate::str::contains("stderr-dynamic-1").not())
        .stdout(predicate::str::contains("stderr-dynamic-2").not());

    let counter = fs::read_to_string(&counter_path).unwrap();
    assert_eq!(counter, "2");
}

#[cfg(unix)]
#[test]
fn batch_mode_stderr_output_ignores_stdout_only_changes_in_printed_diff() {
    let temp = tempdir().unwrap();
    let counter_path = temp.path().join("counter.txt");
    let script_path = temp.path().join("stdout_stderr.sh");

    write_executable_script(
        &script_path,
        format!(
            "#!/bin/sh\ncount_file=\"{}\"\ncount=0\nif [ -f \"$count_file\" ]; then\n  count=$(cat \"$count_file\")\nfi\ncount=$((count + 1))\nprintf '%s' \"$count\" > \"$count_file\"\nprintf 'stdout-dynamic-%s\\n' \"$count\"\nprintf 'stderr-static\\n' >&2\n",
            counter_path.display()
        )
        .as_str(),
    );

    let mut cmd = Command::cargo_bin("hwatch").unwrap();
    cmd.args([
        "-b",
        "-g",
        "1",
        "-n",
        "0.1",
        "-o",
        "stderr",
        "sh",
        script_path.to_str().unwrap(),
    ]);

    cmd.timeout(Duration::from_secs(5));

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("stderr-static"))
        .stdout(predicate::str::contains("stdout-dynamic-1").not())
        .stdout(predicate::str::contains("stdout-dynamic-2").not());

    let counter = fs::read_to_string(&counter_path).unwrap();
    assert_eq!(counter, "2");
}

#[cfg(unix)]
#[test]
fn batch_mode_diff_output_only_hides_unchanged_lines_after_initial_diff() {
    let temp = tempdir().unwrap();
    let counter_path = temp.path().join("counter.txt");
    let script_path = temp.path().join("line_diff.sh");

    write_executable_script(
        &script_path,
        format!(
            "#!/bin/sh\ncount_file=\"{}\"\ncount=0\nif [ -f \"$count_file\" ]; then\n  count=$(cat \"$count_file\")\nfi\ncount=$((count + 1))\nprintf '%s' \"$count\" > \"$count_file\"\nprintf 'static-line\\n'\nprintf 'dynamic-%s\\n' \"$count\"\n",
            counter_path.display()
        )
        .as_str(),
    );

    let mut cmd = Command::cargo_bin("hwatch").unwrap();
    cmd.args([
        "-b",
        "-g",
        "1",
        "-n",
        "0.1",
        "-d",
        "line",
        "-O",
        "sh",
        script_path.to_str().unwrap(),
    ]);

    cmd.timeout(Duration::from_secs(5));

    let assert = cmd
        .assert()
        .success()
        .stdout(is_match("dynamic-1").unwrap())
        .stdout(is_match(r"dynamic-(?:\x1b\[[0-9;]*m)*2").unwrap());

    let normalized_stdout = stdout_text_without_ansi(&assert);
    assert!(normalized_stdout.contains("dynamic-2"));
    assert_eq!(normalized_stdout.matches("static-line").count(), 1);

    let counter = fs::read_to_string(&counter_path).unwrap();
    assert_eq!(counter, "2");
}

#[cfg(unix)]
#[test]
fn batch_mode_with_file_driven_state_change_is_detected() {
    let temp = tempdir().unwrap();
    let state_path = temp.path().join("state.txt");
    let poll_count_path = temp.path().join("poll_count.txt");
    let script_path = temp.path().join("print_state.sh");

    fs::write(&state_path, "tick-0\n").unwrap();
    fs::write(&poll_count_path, "0").unwrap();
    write_executable_script(
        &script_path,
        format!(
            "#!/bin/sh\ncount_file=\"{}\"\ncount=0\nif [ -f \"$count_file\" ]; then\n  count=$(cat \"$count_file\")\nfi\ncount=$((count + 1))\nprintf '%s' \"$count\" > \"$count_file\"\ncat \"{}\"\n",
            poll_count_path.display(),
            state_path.display()
        )
        .as_str(),
    );

    let mut mutator = ProcessCommand::new("sh");
    mutator.arg("-c").arg(format!(
        "while [ \"$(cat \"{}\")\" -lt 2 ]; do sleep 0.02; done; printf 'tick-1\\n' > \"{}\"",
        poll_count_path.display(),
        state_path.display(),
    ));
    let mutator = mutator.spawn().unwrap();

    let mut cmd = Command::cargo_bin("hwatch").unwrap();
    cmd.args([
        "-b",
        "-g",
        "1",
        "-n",
        "0.05",
        "sh",
        script_path.to_str().unwrap(),
    ]);
    cmd.timeout(Duration::from_secs(5));

    let assert = cmd.assert().success();
    let normalized_stdout = stdout_text_without_ansi(&assert);
    assert!(normalized_stdout.contains("tick-0"));
    assert!(normalized_stdout.contains("tick-1"));
    assert!(!normalized_stdout.contains("tick-2"));

    let status = mutator.wait_with_output().unwrap().status;
    assert!(status.success());
}
