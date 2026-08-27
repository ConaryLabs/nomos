mod common;

use std::ffi::OsString;
use std::fs;

use nomos_observed_scene::{ExitCode, HELP, codes, execute};

fn args(items: &[&str]) -> Vec<OsString> {
    items.iter().map(OsString::from).collect()
}

fn code(stdout: &[u8]) -> &str {
    let text = std::str::from_utf8(stdout).expect("diagnostic UTF-8");
    &text[text.find("OS").expect("code")..][..6]
}

#[test]
fn help_and_argument_grammar_are_exact() {
    let help = execute(args(&["help"]));
    assert_eq!(help.exit(), ExitCode::Completed);
    assert_eq!(help.stdout(), HELP.as_bytes());
    for invalid in [
        vec![],
        args(&["--help"]),
        args(&["compile"]),
        args(&["compile", "--out", "a", "--input", "b"]),
        args(&["compile", "--input", "a", "--out", "b", "extra"]),
    ] {
        let result = execute(invalid);
        assert_eq!(result.exit(), ExitCode::InvalidUsage);
        assert_eq!(code(result.stdout()), codes::USAGE.as_str());
        assert_eq!(result.stdout().last(), Some(&b'\n'));
    }
}

#[cfg(unix)]
#[test]
fn non_utf8_argument_is_invalid_usage() {
    use std::os::unix::ffi::OsStringExt;
    let result = execute(vec![OsString::from_vec(vec![0xff])]);
    assert_eq!(result.exit(), ExitCode::InvalidUsage);
    assert_eq!(code(result.stdout()), codes::USAGE.as_str());
}

#[test]
fn compile_is_silent_immutable_atomic_and_refuses_existing_output() {
    let root = common::fresh_dir("command");
    let input = root.join("scene.json");
    let output = root.join("plan.json");
    let source = common::scene_one();
    fs::write(&input, &source).expect("write input");
    let result = execute(vec![
        OsString::from("compile"),
        OsString::from("--input"),
        input.as_os_str().to_owned(),
        OsString::from("--out"),
        output.as_os_str().to_owned(),
    ]);
    assert_eq!(result.exit(), ExitCode::Completed);
    assert!(result.stdout().is_empty());
    assert_eq!(fs::read(&input).expect("input"), source);
    assert_eq!(fs::read(&output).expect("output"), common::plan_one());
    assert_eq!(fs::read_dir(&root).expect("dir").count(), 2);

    let again = execute(vec![
        OsString::from("compile"),
        OsString::from("--input"),
        input.as_os_str().to_owned(),
        OsString::from("--out"),
        output.as_os_str().to_owned(),
    ]);
    assert_eq!(again.exit(), ExitCode::Rejected);
    assert_eq!(code(again.stdout()), codes::OUTPUT_UNAVAILABLE.as_str());
    assert_eq!(
        fs::read(&output).expect("unchanged output"),
        common::plan_one()
    );
}

#[test]
fn filesystem_failures_keep_their_declared_exit_classes() {
    let root = common::fresh_dir("io-codes");
    let missing = root.join("missing.json");
    let output = root.join("out.json");
    let unreadable = execute(vec![
        OsString::from("compile"),
        OsString::from("--input"),
        missing.as_os_str().to_owned(),
        OsString::from("--out"),
        output.as_os_str().to_owned(),
    ]);
    assert_eq!(unreadable.exit(), ExitCode::Environment);
    assert_eq!(code(unreadable.stdout()), codes::INPUT_UNREADABLE.as_str());

    let input = root.join("scene.json");
    fs::write(&input, common::scene_one()).expect("input");
    let absent_parent = root.join("absent/out.json");
    let environment = execute(vec![
        OsString::from("compile"),
        OsString::from("--input"),
        input.as_os_str().to_owned(),
        OsString::from("--out"),
        absent_parent.as_os_str().to_owned(),
    ]);
    assert_eq!(environment.exit(), ExitCode::Environment);
    assert_eq!(code(environment.stdout()), codes::OUTPUT_IO.as_str());
    assert!(!absent_parent.exists());
}

#[cfg(unix)]
#[test]
fn symlinked_inputs_and_output_roots_fail_closed() {
    use std::os::unix::fs::symlink;

    let root = common::fresh_dir("symlinks");
    let real_input = root.join("real.json");
    let linked_input = root.join("linked.json");
    fs::write(&real_input, common::scene_one()).expect("input");
    symlink(&real_input, &linked_input).expect("input link");
    let result = execute(vec![
        OsString::from("compile"),
        OsString::from("--input"),
        linked_input.as_os_str().to_owned(),
        OsString::from("--out"),
        root.join("out.json").as_os_str().to_owned(),
    ]);
    assert_eq!(result.exit(), ExitCode::Environment);
    assert_eq!(code(result.stdout()), codes::INPUT_UNREADABLE.as_str());

    let real_parent = root.join("real-parent");
    let linked_parent = root.join("linked-parent");
    fs::create_dir(&real_parent).expect("real parent");
    symlink(&real_parent, &linked_parent).expect("parent link");
    let result = execute(vec![
        OsString::from("compile"),
        OsString::from("--input"),
        real_input.as_os_str().to_owned(),
        OsString::from("--out"),
        linked_parent.join("out.json").as_os_str().to_owned(),
    ]);
    assert_eq!(result.exit(), ExitCode::Rejected);
    assert_eq!(code(result.stdout()), codes::OUTPUT_UNAVAILABLE.as_str());
}

#[test]
fn absolute_machine_paths_never_enter_content_diagnostics() {
    let root = common::fresh_dir("path-redaction");
    let input = root.join("invalid.json");
    let output = root.join("out.json");
    fs::write(&input, b"{").expect("invalid input");
    let result = execute(vec![
        OsString::from("compile"),
        OsString::from("--input"),
        input.as_os_str().to_owned(),
        OsString::from("--out"),
        output.as_os_str().to_owned(),
    ]);
    let text = std::str::from_utf8(result.stdout()).expect("UTF-8 diagnostic");
    assert_eq!(result.exit(), ExitCode::Rejected);
    assert_eq!(code(result.stdout()), codes::INPUT_MALFORMED.as_str());
    assert!(
        !text.contains(root.to_str().expect("UTF-8 test path")),
        "{text}"
    );
    assert!(text.contains("\"path\":\"invalid.json\""), "{text}");
    assert!(!output.exists());
}
