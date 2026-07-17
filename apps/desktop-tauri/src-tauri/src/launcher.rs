use std::{
    env,
    ffi::OsString,
    path::{Path, PathBuf},
};

#[derive(Clone, Debug, PartialEq)]
pub struct LaunchSpec {
    pub program: PathBuf,
    pub args: Vec<OsString>,
    pub display: String,
}

impl LaunchSpec {
    fn direct(path: PathBuf) -> Self {
        Self {
            display: path.display().to_string(),
            program: path,
            args: vec!["app-server".into(), "--stdio".into()],
        }
    }

    #[cfg(windows)]
    fn windows_script(path: PathBuf) -> Self {
        let quoted = format!("\"{}\" app-server --stdio", path.display());
        Self {
            display: path.display().to_string(),
            program: PathBuf::from("cmd.exe"),
            args: vec!["/D".into(), "/S".into(), "/C".into(), quoted.into()],
        }
    }
}

pub fn locate_codex() -> Option<LaunchSpec> {
    if let Some(override_path) = env::var_os("CODEX_BINARY").filter(|value| !value.is_empty()) {
        let path = PathBuf::from(override_path);
        if is_candidate(&path) {
            return Some(spec_for(path));
        }
    }

    #[cfg(target_os = "macos")]
    for path in [
        "/Applications/ChatGPT.app/Contents/Resources/codex",
        "/Applications/Codex.app/Contents/Resources/codex",
    ] {
        let path = PathBuf::from(path);
        if is_candidate(&path) {
            return Some(LaunchSpec::direct(path));
        }
    }

    if let Some(path) = locate_on_path() {
        return Some(spec_for(path));
    }

    #[cfg(windows)]
    if env::var("CODEX_USE_WSL").as_deref() == Ok("1") {
        return Some(LaunchSpec {
            program: PathBuf::from("wsl.exe"),
            args: vec![
                "--".into(),
                "codex".into(),
                "app-server".into(),
                "--stdio".into(),
            ],
            display: "WSL: codex".to_owned(),
        });
    }

    None
}

fn locate_on_path() -> Option<PathBuf> {
    let path = env::var_os("PATH")?;
    for directory in env::split_paths(&path) {
        #[cfg(windows)]
        let names = ["codex.exe", "codex.cmd", "codex.bat", "codex"];
        #[cfg(not(windows))]
        let names = ["codex"];

        for name in names {
            let candidate = directory.join(name);
            if is_candidate(&candidate) {
                return Some(candidate);
            }
        }
    }
    None
}

fn is_candidate(path: &Path) -> bool {
    if !path.is_file() {
        return false;
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        path.metadata()
            .map(|metadata| metadata.permissions().mode() & 0o111 != 0)
            .unwrap_or(false)
    }

    #[cfg(windows)]
    {
        true
    }
}

fn spec_for(path: PathBuf) -> LaunchSpec {
    #[cfg(windows)]
    if matches!(
        path.extension().and_then(|extension| extension.to_str()),
        Some("cmd" | "bat")
    ) {
        return LaunchSpec::windows_script(path);
    }

    LaunchSpec::direct(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn direct_spec_uses_fixed_app_server_arguments() {
        let spec = LaunchSpec::direct(PathBuf::from("codex"));
        assert_eq!(spec.program, PathBuf::from("codex"));
        assert_eq!(spec.args, vec!["app-server", "--stdio"]);
    }
}
