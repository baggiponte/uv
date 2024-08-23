use anstream::println;
use anyhow::Result;

use uv_cache::Cache;
use uv_fs::Simplified;
use uv_python::{EnvironmentPreference, PythonInstallation, PythonPreference};

use crate::commands::{project::find_python_request, ExitStatus};

/// Find a Python interpreter.
pub(crate) async fn find(
    request: Option<String>,
    no_project: bool,
    no_config: bool,
    python_preference: PythonPreference,
    cache: &Cache,
) -> Result<ExitStatus> {
    let request = find_python_request(request, no_project, no_config).await?;

    let python = PythonInstallation::find(
        &request.unwrap_or_default(),
        EnvironmentPreference::OnlySystem,
        python_preference,
        cache,
    )?;

    println!("{}", python.interpreter().sys_executable().user_display());

    Ok(ExitStatus::Success)
}
