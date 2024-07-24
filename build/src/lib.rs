use std::{
    fs::create_dir_all,
    path::{Path, PathBuf},
};

pub mod download;
pub use download::*;
mod validate;
use ultralight_errors::{UltralightError, UltralightResult};
use validate::*;
mod utils;
use utils::copy_dir_all;

pub struct UltralightBuild {
    version: Option<String>,
    platform: Option<Platform>,
    download_headers: bool,
    download_resources: bool,
    download_binaries: bool,
    download_libs: bool,
    headers_out_dir: Option<PathBuf>,
    resources_out_dir: Option<PathBuf>,
    binaries_out_dir: Option<PathBuf>,
    libs_out_dir: Option<PathBuf>,
}

impl UltralightBuild {
    pub fn new() -> Self {
        Self {
            version: None,
            platform: None,
            download_headers: false,
            download_resources: false,
            download_binaries: false,
            download_libs: false,
            headers_out_dir: None,
            resources_out_dir: None,
            binaries_out_dir: None,
            libs_out_dir: None,
        }
    }

    pub fn with_version(mut self, version: &str) -> Self {
        self.version = Some(version.to_string());
        self
    }

    pub fn with_platform(mut self, platform: Platform) -> Self {
        self.platform = Some(platform);
        self
    }

    pub fn download_headers(mut self) -> Self {
        self.download_headers = true;
        self
    }

    pub fn download_resources(mut self) -> Self {
        self.download_resources = true;
        self
    }

    pub fn download_binaries(mut self) -> Self {
        self.download_binaries = true;
        self
    }

    pub fn download_libs(mut self) -> Self {
        self.download_libs = true;
        self
    }

    pub fn with_headers_out_dir<P: AsRef<Path>>(mut self, out_dir: P) -> Self {
        self.headers_out_dir = Some(out_dir.as_ref().to_path_buf());
        self
    }

    pub fn with_resources_out_dir<P: AsRef<Path>>(mut self, out_dir: P) -> Self {
        self.resources_out_dir = Some(out_dir.as_ref().to_path_buf());
        self
    }

    pub fn with_binaries_out_dir<P: AsRef<Path>>(mut self, out_dir: P) -> Self {
        self.binaries_out_dir = Some(out_dir.as_ref().to_path_buf());
        self
    }

    pub fn with_libs_out_dir<P: AsRef<Path>>(mut self, out_dir: P) -> Self {
        self.libs_out_dir = Some(out_dir.as_ref().to_path_buf());
        self
    }

    pub fn build(&self) -> UltralightResult<()> {
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

        let version = self.version.as_deref().unwrap_or("latest");
        let out_dir: PathBuf = std::env::var("OUT_DIR")?.into();

        if self.need_download_any(platform)? {
            let dl_dir = out_dir.join("ultralight-download");
            create_dir_all(dl_dir.clone()).map_err(|e| UltralightError::IoError(e))?;
            DownloadBuilder::new()
                .with_platform(platform)
                .with_version(version)
                .with_out_dir(dl_dir.clone())
                .start()?;

            self.handle_new_resources(dl_dir, platform)?;
        }

        println!("cargo:rustc-link-search=native={}", out_dir.display());
        println!("cargo:rustc-link-lib=Ultralight");
        println!("cargo:rustc-link-lib=WebCore");
        println!("cargo:rustc-link-lib=AppCore");

        Ok(())
    }

    fn need_download_headers(&self) -> UltralightResult<bool> {
        if self.download_headers {
            let headers_out_dir = if let Some(headers_out_dir) = &self.headers_out_dir {
                headers_out_dir.clone()
            } else {
                let out: PathBuf = std::env::var("OUT_DIR")?.into();
                out.join("headers")
            };

            if !validate_directory_contents(
                &headers_out_dir,
                &[
                    "./AppCore/CAPI.h",
                    "./Ultralight/CAPI.h",
                    "./Ultralight/CAPI/CAPI_Defines.h",
                    "./Ultralight/CAPI/CAPI_Bitmap.h",
                    "./Ultralight/CAPI/CAPI_Buffer.h",
                    "./Ultralight/CAPI/CAPI_Clipboard.h",
                    "./Ultralight/CAPI/CAPI_Config.h",
                    "./Ultralight/CAPI/CAPI_FileSystem.h",
                    "./Ultralight/CAPI/CAPI_FontFile.h",
                    "./Ultralight/CAPI/CAPI_FontLoader.h",
                    "./Ultralight/CAPI/CAPI_FontLoader.h",
                    "./Ultralight/CAPI/CAPI_Geometry.h",
                    "./Ultralight/CAPI/CAPI_Geometry.h",
                    "./Ultralight/CAPI/CAPI_GPUDriver.h",
                    "./Ultralight/CAPI/CAPI_KeyEvent.h",
                    "./Ultralight/CAPI/CAPI_Logger.h",
                    "./Ultralight/CAPI/CAPI_MouseEvent.h",
                    "./Ultralight/CAPI/CAPI_Platform.h",
                    "./Ultralight/CAPI/CAPI_Renderer.h",
                    "./Ultralight/CAPI/CAPI_ScrollEvent.h",
                    "./Ultralight/CAPI/CAPI_GamepadEvent.h",
                    "./Ultralight/CAPI/CAPI_Session.h",
                    "./Ultralight/CAPI/CAPI_String.h",
                    "./Ultralight/CAPI/CAPI_Surface.h",
                    "./Ultralight/CAPI/CAPI_View.h",
                ],
            ) {
                return Ok(true);
            }
        }

        Ok(false)
    }

    fn need_download_resources(&self) -> UltralightResult<bool> {
        if self.download_resources {
            let resources_out_dir = if let Some(resources_out_dir) = &self.resources_out_dir {
                resources_out_dir.clone()
            } else {
                let out: PathBuf = std::env::var("OUT_DIR")?.into();
                out.join("resources")
            };

            if !validate_directory_contents(&resources_out_dir, &["cacert.pem", "icudt67l.dat"]) {
                return Ok(true);
            }
        }

        Ok(false)
    }

    fn need_download_binaries(&self, platform: Platform) -> UltralightResult<bool> {
        if self.download_binaries {
            let binaries_out_dir = if let Some(binaries_out_dir) = &self.binaries_out_dir {
                binaries_out_dir.clone()
            } else {
                let out: PathBuf = std::env::var("OUT_DIR")?.into();
                out.join("binaries")
            };

            match platform {
                Platform::Windows => {
                    if !validate_directory_contents(
                        &binaries_out_dir,
                        &[
                            "Ultralight.dll",
                            "UltralightCore.dll",
                            "AppCore.dll",
                            "WebCore.dll",
                        ],
                    ) {
                        return Ok(true);
                    }
                }
                Platform::Linux => {
                    if !validate_directory_contents(
                        &binaries_out_dir,
                        &[
                            "libUltralight.so",
                            "libUltralightCore.so",
                            "libAppCore.so",
                            "libWebCore.so",
                        ],
                    ) {
                        return Ok(true);
                    }
                }
                Platform::MacOS => {
                    if !validate_directory_contents(
                        &binaries_out_dir,
                        &[
                            "libUltralight.dylib",
                            "libUltralightCore.dylib",
                            "libAppCore.dylib",
                            "libWebCore.dylib",
                        ],
                    ) {
                        return Ok(true);
                    }
                }
            }
        }

        Ok(false)
    }

    fn need_download_libs(&self, platform: Platform) -> UltralightResult<bool> {
        if self.download_libs {
            let libs_out_dir = if let Some(libs_out_dir) = &self.libs_out_dir {
                libs_out_dir.clone()
            } else {
                let out: PathBuf = std::env::var("OUT_DIR")?.into();
                out.join("libs")
            };

            if platform == Platform::Windows {
                if !validate_directory_contents(
                    &libs_out_dir,
                    &[
                        "Ultralight.lib",
                        "UltralightCore.lib",
                        "AppCore.lib",
                        "WebCore.lib",
                    ],
                ) {
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }

    fn need_download_any(&self, platform: Platform) -> UltralightResult<bool> {
        Ok(self.need_download_headers()?
            || self.need_download_resources()?
            || self.need_download_binaries(platform)?
            || self.need_download_libs(platform)?)
    }

    fn handle_new_resources(&self, dl_dir: PathBuf, platform: Platform) -> UltralightResult<()> {
        if self.download_headers {
            self.fetch_headers(dl_dir.clone())?;
        }

        if self.download_resources {
            self.fetch_resources(dl_dir.clone())?;
        }

        if self.download_binaries {
            self.fetch_binaries(dl_dir.clone())?;
        }

        if self.download_libs {
            self.fetch_libs(dl_dir.clone(), platform)?;
        }

        Ok(())
    }

    fn fetch_headers(&self, dl_dir: PathBuf) -> UltralightResult<()> {
        let headers_out_dir = if let Some(headers_out_dir) = &self.headers_out_dir {
            headers_out_dir.clone()
        } else {
            let out: PathBuf = std::env::var("OUT_DIR")?.into();
            out.join("headers")
        };

        copy_dir_all(dl_dir.join("include"), headers_out_dir)?;

        Ok(())
    }

    fn fetch_resources(&self, dl_dir: PathBuf) -> UltralightResult<()> {
        let resources_out_dir = if let Some(resources_out_dir) = &self.resources_out_dir {
            resources_out_dir.clone()
        } else {
            let out: PathBuf = std::env::var("OUT_DIR")?.into();
            out.join("resources")
        };

        copy_dir_all(dl_dir.join("resources"), resources_out_dir)?;

        Ok(())
    }

    fn fetch_binaries(&self, dl_dir: PathBuf) -> UltralightResult<()> {
        let binaries_out_dir = if let Some(binaries_out_dir) = &self.binaries_out_dir {
            binaries_out_dir.clone()
        } else {
            let out: PathBuf = std::env::var("OUT_DIR")?.into();
            out.join("binaries")
        };

        copy_dir_all(dl_dir.join("bin"), binaries_out_dir)?;

        Ok(())
    }

    fn fetch_libs(&self, dl_dir: PathBuf, platform: Platform) -> UltralightResult<()> {
        if platform != Platform::Windows {
            return Ok(());
        }

        let libs_out_dir = if let Some(libs_out_dir) = &self.libs_out_dir {
            libs_out_dir.clone()
        } else {
            let out: PathBuf = std::env::var("OUT_DIR")?.into();
            out.join("libs")
        };

        copy_dir_all(dl_dir.join("lib"), libs_out_dir)?;

        Ok(())
    }
}
