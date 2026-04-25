use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::tempdir;

#[test]
fn diff_plugin_load_failure_reports_clear_error() {
    let temp = tempdir().unwrap();
    let plugin_path = temp.path().join("missing-plugin.dylib");

    let mut cmd = Command::cargo_bin("hwatch").unwrap();
    cmd.args([
        "--diff-plugin",
        plugin_path.to_str().unwrap(),
        "-b",
        "echo",
        "hello",
    ]);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("load failed"))
        .stderr(predicate::str::contains(plugin_path.to_str().unwrap()));
}
