use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::time::Duration;
use tempfile::tempdir;

#[test]
fn help_flag_prints_usage() {
    let mut cmd = Command::cargo_bin("hwatch").unwrap();
    cmd.arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Usage:"))
        .stdout(predicate::str::contains("--batch"));
}

#[cfg(unix)]
#[test]
fn batch_mode_with_chgexit_runs_command_until_change_limit() {
    let temp = tempdir().unwrap();
    let counter_path = temp.path().join("counter.txt");
    let script_path = temp.path().join("increment.sh");

    fs::write(
        &script_path,
        format!(
            "#!/bin/sh\ncount_file=\"{}\"\ncount=0\nif [ -f \"$count_file\" ]; then\n  count=$(cat \"$count_file\")\nfi\ncount=$((count + 1))\nprintf '%s' \"$count\" > \"$count_file\"\nprintf '%s\\n' \"$count\"\n",
            counter_path.display()
        ),
    )
    .unwrap();

    let mut perms = fs::metadata(&script_path).unwrap().permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&script_path, perms).unwrap();

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

    fs::write(
        &script_path,
        format!(
            "#!/bin/sh\ncount_file=\"{}\"\ncount=0\nif [ -f \"$count_file\" ]; then\n  count=$(cat \"$count_file\")\nfi\ncount=$((count + 1))\nprintf '%s' \"$count\" > \"$count_file\"\nprintf 'tick-%s\\n' \"$count\"\n",
            counter_path.display()
        ),
    )
    .unwrap();

    let mut perms = fs::metadata(&script_path).unwrap().permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&script_path, perms).unwrap();

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

    fs::write(
        &script_path,
        format!(
            "#!/bin/sh\ncount_file=\"{}\"\ncount=0\nif [ -f \"$count_file\" ]; then\n  count=$(cat \"$count_file\")\nfi\ncount=$((count + 1))\nprintf '%s' \"$count\" > \"$count_file\"\nprintf 'stdout-static\\n'\nprintf 'stderr-dynamic-%s\\n' \"$count\" >&2\n",
            counter_path.display()
        ),
    )
    .unwrap();

    let mut perms = fs::metadata(&script_path).unwrap().permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&script_path, perms).unwrap();

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
