//! Starts [jinja-lsp](https://github.com/uros-5/jinja-lsp) for Jinja buffers.
//!
//! The binary is resolved in this order:
//!
//! 1. `lsp.jinja-lsp.binary.path` from the user's settings
//! 2. `jinja-lsp` on the worktree's `PATH`
//! 3. the release binary for this platform, downloaded from GitHub into the
//!    extension's working directory

use std::fs;

use zed_extension_api::{
    self as zed, serde_json, settings::LspSettings, Architecture, DownloadedFileType,
    GithubReleaseOptions, LanguageServerId, LanguageServerInstallationStatus, Os, Result,
};

const SERVER_NAME: &str = "jinja-lsp";
const GITHUB_REPO: &str = "uros-5/jinja-lsp";
/// Downloads live in `jinja-lsp-<version>/` inside the extension's directory.
const DOWNLOAD_DIR_PREFIX: &str = "jinja-lsp-";

struct JinjaExtension {
    /// Path of the downloaded binary, so GitHub is asked once per session.
    cached_binary_path: Option<String>,
}

impl JinjaExtension {
    /// Name of the release asset for the current platform.
    ///
    /// Releases ship bare executables named `jinja-lsp-<target triple>`. On
    /// Linux the statically linked musl build is used because it runs
    /// regardless of the system's libc.
    fn asset_name(os: Os, arch: Architecture) -> Result<String> {
        let target = match (os, arch) {
            (Os::Mac, Architecture::Aarch64) => "aarch64-apple-darwin",
            (Os::Mac, Architecture::X8664) => "x86_64-apple-darwin",
            (Os::Linux, Architecture::Aarch64) => "aarch64-unknown-linux-musl",
            (Os::Linux, Architecture::X8664) => "x86_64-unknown-linux-musl",
            (Os::Windows, Architecture::Aarch64) => "aarch64-pc-windows-msvc.exe",
            (Os::Windows, Architecture::X8664) => "x86_64-pc-windows-msvc.exe",
            (Os::Windows, Architecture::X86) => "i686-pc-windows-msvc.exe",
            (os, arch) => {
                return Err(format!(
                    "jinja-lsp publishes no release binary for {os:?} {arch:?}; \
                     install it with `cargo install jinja-lsp` so it is on your PATH"
                ))
            }
        };
        Ok(format!("{SERVER_NAME}-{target}"))
    }

    fn binary_file_name(os: Os) -> &'static str {
        match os {
            Os::Windows => "jinja-lsp.exe",
            Os::Mac | Os::Linux => "jinja-lsp",
        }
    }

    /// A binary left behind by an earlier download, if any. Used when GitHub
    /// cannot be reached.
    fn previously_downloaded_binary(os: Os) -> Option<String> {
        let mut candidates: Vec<String> = fs::read_dir(".")
            .ok()?
            .flatten()
            .filter_map(|entry| entry.file_name().into_string().ok())
            .filter(|name| name.starts_with(DOWNLOAD_DIR_PREFIX))
            .map(|dir| format!("{dir}/{}", Self::binary_file_name(os)))
            .filter(|path| fs::metadata(path).is_ok_and(|stat| stat.is_file()))
            .collect();
        candidates.sort();
        candidates.pop()
    }

    fn remove_other_versions(keep_dir: &str) {
        let Ok(entries) = fs::read_dir(".") else {
            return;
        };
        for entry in entries.flatten() {
            let Ok(name) = entry.file_name().into_string() else {
                continue;
            };
            if name.starts_with(DOWNLOAD_DIR_PREFIX) && name != keep_dir {
                fs::remove_dir_all(entry.path()).ok();
            }
        }
    }

    fn downloaded_binary(&mut self, language_server_id: &LanguageServerId) -> Result<String> {
        if let Some(path) = &self.cached_binary_path {
            if fs::metadata(path).is_ok_and(|stat| stat.is_file()) {
                return Ok(path.clone());
            }
        }

        let (os, arch) = zed::current_platform();
        let asset_name = Self::asset_name(os, arch)?;

        zed::set_language_server_installation_status(
            language_server_id,
            &LanguageServerInstallationStatus::CheckingForUpdate,
        );
        let release = match zed::latest_github_release(
            GITHUB_REPO,
            GithubReleaseOptions {
                require_assets: true,
                pre_release: false,
            },
        ) {
            Ok(release) => release,
            Err(error) => {
                // Offline or rate limited: keep working with what is on disk.
                let path = Self::previously_downloaded_binary(os).ok_or_else(|| {
                    format!("could not look up the latest jinja-lsp release: {error}")
                })?;
                self.cached_binary_path = Some(path.clone());
                return Ok(path);
            }
        };

        let asset = release
            .assets
            .iter()
            .find(|asset| asset.name == asset_name)
            .ok_or_else(|| {
                format!(
                    "jinja-lsp release {} has no asset named {asset_name}",
                    release.version
                )
            })?;

        let version_dir = format!("{DOWNLOAD_DIR_PREFIX}{}", release.version);
        let binary_path = format!("{version_dir}/{}", Self::binary_file_name(os));

        if !fs::metadata(&binary_path).is_ok_and(|stat| stat.is_file()) {
            zed::set_language_server_installation_status(
                language_server_id,
                &LanguageServerInstallationStatus::Downloading,
            );
            fs::create_dir_all(&version_dir)
                .map_err(|error| format!("could not create {version_dir}: {error}"))?;
            zed::download_file(
                &asset.download_url,
                &binary_path,
                DownloadedFileType::Uncompressed,
            )
            .map_err(|error| format!("could not download {asset_name}: {error}"))?;
            zed::make_file_executable(&binary_path)?;
            Self::remove_other_versions(&version_dir);
        }

        self.cached_binary_path = Some(binary_path.clone());
        Ok(binary_path)
    }
}

impl zed::Extension for JinjaExtension {
    fn new() -> Self {
        Self {
            cached_binary_path: None,
        }
    }

    fn language_server_command(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<zed::Command> {
        let binary_settings = LspSettings::for_worktree(SERVER_NAME, worktree)
            .ok()
            .and_then(|settings| settings.binary);
        let args = binary_settings
            .as_ref()
            .and_then(|settings| settings.arguments.clone())
            .unwrap_or_default();
        let env = worktree.shell_env();

        if let Some(path) = binary_settings.and_then(|settings| settings.path) {
            return Ok(zed::Command {
                command: path,
                args,
                env,
            });
        }

        if let Some(path) = worktree.which(SERVER_NAME) {
            return Ok(zed::Command {
                command: path,
                args,
                env,
            });
        }

        // Relative to the extension's working directory, which is what Zed
        // resolves a relative command against.
        Ok(zed::Command {
            command: self.downloaded_binary(language_server_id)?,
            args,
            env,
        })
    }

    /// Forwards `lsp.jinja-lsp.initialization_options`, which is where
    /// jinja-lsp reads `templates`, `backend` and `lang` from.
    fn language_server_initialization_options(
        &mut self,
        _language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<Option<serde_json::Value>> {
        Ok(LspSettings::for_worktree(SERVER_NAME, worktree)
            .ok()
            .and_then(|settings| settings.initialization_options))
    }

    fn language_server_workspace_configuration(
        &mut self,
        _language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<Option<serde_json::Value>> {
        Ok(LspSettings::for_worktree(SERVER_NAME, worktree)
            .ok()
            .and_then(|settings| settings.settings))
    }
}

zed::register_extension!(JinjaExtension);
