#![cfg(all(feature = "python", feature = "pypi"))]

use assert_fs::prelude::PathChild;
use assert_fs::{fixture::FileWriteStr, prelude::PathCreateDir};
use indoc::indoc;

use common::{uv_snapshot, TestContext};
use uv_python::platform::{Arch, Os};

mod common;

#[test]
fn python_find() {
    let mut context: TestContext = TestContext::new_with_versions(&["3.11", "3.12"]);

    // No interpreters on the path
    if cfg!(windows) {
        uv_snapshot!(context.filters(), context.python_find().env("UV_TEST_PYTHON_PATH", ""), @r###"
        success: false
        exit_code: 2
        ----- stdout -----

        ----- stderr -----
        error: No interpreter found in system path or `py` launcher
        "###);
    } else {
        uv_snapshot!(context.filters(), context.python_find().env("UV_TEST_PYTHON_PATH", ""), @r###"
        success: false
        exit_code: 2
        ----- stdout -----

        ----- stderr -----
        error: No interpreter found in system path
        "###);
    }

    // We find the first interpreter on the path
    uv_snapshot!(context.filters(), context.python_find(), @r###"
    success: true
    exit_code: 0
    ----- stdout -----
    [PYTHON-3.11]

    ----- stderr -----
    "###);

    // Request Python 3.12
    uv_snapshot!(context.filters(), context.python_find().arg("3.12"), @r###"
    success: true
    exit_code: 0
    ----- stdout -----
    [PYTHON-3.12]

    ----- stderr -----
    "###);

    // Request Python 3.11
    uv_snapshot!(context.filters(), context.python_find().arg("3.11"), @r###"
    success: true
    exit_code: 0
    ----- stdout -----
    [PYTHON-3.11]

    ----- stderr -----
    "###);

    // Request CPython
    uv_snapshot!(context.filters(), context.python_find().arg("cpython"), @r###"
    success: true
    exit_code: 0
    ----- stdout -----
    [PYTHON-3.11]

    ----- stderr -----
    "###);

    // Request CPython 3.12
    uv_snapshot!(context.filters(), context.python_find().arg("cpython@3.12"), @r###"
    success: true
    exit_code: 0
    ----- stdout -----
    [PYTHON-3.12]

    ----- stderr -----
    "###);

    // Request CPython 3.12 via partial key syntax
    uv_snapshot!(context.filters(), context.python_find().arg("cpython-3.12"), @r###"
    success: true
    exit_code: 0
    ----- stdout -----
    [PYTHON-3.12]

    ----- stderr -----
    "###);

    // Request CPython 3.12 for the current platform
    let os = Os::from_env();
    let arch = Arch::from_env();

    uv_snapshot!(context.filters(), context.python_find()
    .arg(format!("cpython-3.12-{os}-{arch}"))
    , @r###"
    success: true
    exit_code: 0
    ----- stdout -----
    [PYTHON-3.12]

    ----- stderr -----
    "###);

    // Request PyPy (which should be missing)
    if cfg!(windows) {
        uv_snapshot!(context.filters(), context.python_find().arg("pypy"), @r###"
        success: false
        exit_code: 2
        ----- stdout -----

        ----- stderr -----
        error: No interpreter found for PyPy in system path or `py` launcher
        "###);
    } else {
        uv_snapshot!(context.filters(), context.python_find().arg("pypy"), @r###"
        success: false
        exit_code: 2
        ----- stdout -----

        ----- stderr -----
        error: No interpreter found for PyPy in system path
        "###);
    }

    // Swap the order of the Python versions
    context.python_versions.reverse();

    uv_snapshot!(context.filters(), context.python_find(), @r###"
    success: true
    exit_code: 0
    ----- stdout -----
    [PYTHON-3.12]

    ----- stderr -----
    "###);

    // Request Python 3.11
    uv_snapshot!(context.filters(), context.python_find().arg("3.11"), @r###"
    success: true
    exit_code: 0
    ----- stdout -----
    [PYTHON-3.11]

    ----- stderr -----
    "###);
}

#[test]
fn python_find_pin() {
    let context: TestContext = TestContext::new_with_versions(&["3.11", "3.12"]);

    // Pin to a version
    uv_snapshot!(context.filters(), context.python_pin().arg("3.12"), @r###"
    success: true
    exit_code: 0
    ----- stdout -----
    Pinned `.python-version` to `3.12`

    ----- stderr -----
    "###);

    // We should find the pinned version, not the first on the path
    uv_snapshot!(context.filters(), context.python_find(), @r###"
    success: true
    exit_code: 0
    ----- stdout -----
    [PYTHON-3.12]

    ----- stderr -----
    "###);

    // Unless explicitly requested
    uv_snapshot!(context.filters(), context.python_find().arg("3.11"), @r###"
    success: true
    exit_code: 0
    ----- stdout -----
    [PYTHON-3.11]

    ----- stderr -----
    "###);

    // Or `--no-config` is used
    uv_snapshot!(context.filters(), context.python_find().arg("--no-config"), @r###"
    success: true
    exit_code: 0
    ----- stdout -----
    [PYTHON-3.11]

    ----- stderr -----
    "###);

    let child_dir = context.temp_dir.child("child");
    child_dir.create_dir_all().unwrap();

    // We should also find pinned versions in the parent directory
    uv_snapshot!(context.filters(), context.python_find().current_dir(&child_dir), @r###"
    success: true
    exit_code: 0
    ----- stdout -----
    [PYTHON-3.12]

    ----- stderr -----
    "###);

    uv_snapshot!(context.filters(), context.python_pin().arg("3.11").current_dir(&child_dir), @r###"
    success: true
    exit_code: 0
    ----- stdout -----
    Updated `.python-version` from `3.12` -> `3.11`

    ----- stderr -----
    "###);

    // Unless the child directory also has a pin
    uv_snapshot!(context.filters(), context.python_find().current_dir(&child_dir), @r###"
    success: true
    exit_code: 0
    ----- stdout -----
    [PYTHON-3.11]

    ----- stderr -----
    "###);
}

#[test]
fn python_find_project() {
    let context: TestContext = TestContext::new_with_versions(&["3.10", "3.11", "3.12"]);

    let pyproject_toml = context.temp_dir.child("pyproject.toml");
    pyproject_toml
        .write_str(indoc! {r#"
        [project]
        name = "project"
        version = "0.1.0"
        requires-python = ">=3.11"
        dependencies = ["anyio==3.7.0"]
    "#})
        .unwrap();

    // We should respect the project's required version, not the first on the path
    uv_snapshot!(context.filters(), context.python_find(), @r###"
    success: true
    exit_code: 0
    ----- stdout -----
    [PYTHON-3.11]

    ----- stderr -----
    "###);

    // Unless explicitly requested
    uv_snapshot!(context.filters(), context.python_find().arg("3.10"), @r###"
    success: true
    exit_code: 0
    ----- stdout -----
    [PYTHON-3.10]

    ----- stderr -----
    "###);

    // Or `--no-project` is used
    uv_snapshot!(context.filters(), context.python_find().arg("--no-project"), @r###"
    success: true
    exit_code: 0
    ----- stdout -----
    [PYTHON-3.10]

    ----- stderr -----
    "###);

    // But a pin should take precedence
    uv_snapshot!(context.filters(), context.python_pin().arg("3.12"), @r###"
    success: true
    exit_code: 0
    ----- stdout -----
    Pinned `.python-version` to `3.12`

    ----- stderr -----
    "###);
    uv_snapshot!(context.filters(), context.python_find(), @r###"
    success: true
    exit_code: 0
    ----- stdout -----
    [PYTHON-3.12]

    ----- stderr -----
    "###);

    // Create a pin that's incompatible with the project
    uv_snapshot!(context.filters(), context.python_pin().arg("3.10").arg("--no-workspace"), @r###"
    success: true
    exit_code: 0
    ----- stdout -----
    Updated `.python-version` from `3.12` -> `3.10`

    ----- stderr -----
    "###);

    // We should warn on subsequent uses, but respect the version?
    uv_snapshot!(context.filters(), context.python_find(), @r###"
    success: true
    exit_code: 0
    ----- stdout -----
    [PYTHON-3.10]

    ----- stderr -----
    warning: The pinned Python version (3.10) in `.python-version` does not meet the project's Python requirement of `>=3.11`.
    "###);

    // Unless the pin file is outside the project, in which case we just ignore it
    let child_dir = context.temp_dir.child("child");
    child_dir.create_dir_all().unwrap();

    let pyproject_toml = child_dir.child("pyproject.toml");
    pyproject_toml
        .write_str(indoc! {r#"
        [project]
        name = "project"
        version = "0.1.0"
        requires-python = ">=3.11"
        dependencies = ["anyio==3.7.0"]
    "#})
        .unwrap();

    uv_snapshot!(context.filters(), context.python_find().current_dir(&child_dir), @r###"
    success: true
    exit_code: 0
    ----- stdout -----
    [PYTHON-3.11]

    ----- stderr -----
    "###);
}
