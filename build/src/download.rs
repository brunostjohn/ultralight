use std::io::Cursor;
use std::path::{Path, PathBuf};
use ultralight_errors::{UltralightError, UltralightResult};

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Platform {
    Windows,
    Linux,
    MacOS,
}

pub(crate) struct DownloadBuilder {
    platform: Option<Platform>,
    version: Option<String>,
    out_dir: Option<PathBuf>,
}

impl DownloadBuilder {
    pub(crate) fn new() -> Self {
        Self {
            platform: None,
            version: None,
            out_dir: None,
        }
    }

    pub(crate) fn with_platform(mut self, platform: Platform) -> Self {
        self.platform = Some(platform);
        self
    }

    pub(crate) fn with_version(mut self, version: &str) -> Self {
        self.version = Some(version.to_string());
        self
    }

    pub(crate) fn with_out_dir<P: AsRef<Path>>(mut self, out_dir: P) -> Self {
        self.out_dir = Some(out_dir.as_ref().to_path_buf());
        self
    }

    fn create_download_url(platform: Platform, version: Option<&str>) -> String {
        let platform = match platform {
            Platform::Windows => "win",
            Platform::Linux => "linux",
            Platform::MacOS => "mac",
        };

        format!(
            "https://ultralight-sdk.sfo2.cdn.digitaloceanspaces.com/ultralight-sdk-{}-{}-x64.7z",
            version.unwrap_or("latest"),
            platform
        )
    }

    pub(crate) fn build(self) -> Download {
        let platform = self.platform.unwrap_or_else(|| {
            if cfg!(target_os = "windows") {
                Platform::Windows
            } else if cfg!(target_os = "linux") {
                Platform::Linux
            } else if cfg!(target_os = "macos") {
                Platform::MacOS
            } else {
                panic!("Unsupported platform!");
            }
        });

        let url = Self::create_download_url(platform, self.version.as_deref());

        Download {
            url,
            out_dir: self.out_dir,
        }
    }

    pub fn start(self) -> UltralightResult<()> {
        let download = self.build();

        download.start()
    }
}

pub(crate) struct Download {
    url: String,
    out_dir: Option<PathBuf>,
}

impl Download {
    pub(crate) fn start(&self) -> UltralightResult<()> {
        let out_dir = if let Some(out_dir) = self.out_dir.clone() {
            out_dir
        } else {
            std::env::var("OUT_DIR")
                .map_err(|e| UltralightError::EnvVarError(e))?
                .into()
        };

        let bytes = reqwest::blocking::get(&self.url)
            .map_err(|e| UltralightError::RequestError(e))?
            .bytes()
            .map_err(|e| UltralightError::RequestError(e))?
            .to_vec();

        sevenz_rust::decompress(Cursor::new(bytes), out_dir)
            .map_err(|e| UltralightError::DecompressionError(e))?;

        Ok(())
    }
}
