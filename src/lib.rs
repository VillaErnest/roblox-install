use std::{
    fmt, io,
    path::{Path, PathBuf},
};

#[cfg(target_os = "macos")]
use dirs::document_dir;

#[cfg(target_os = "windows")]
use winreg::RegKey;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    DocumentsDirectoryNotFound,
    MalformedRegistry,
    PlatformNotSupported,
    RegistryError(io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::DocumentsDirectoryNotFound => write!(formatter, "Couldn't find Documents directory"),
            Error::MalformedRegistry => write!(formatter, "The values of the registry keys used to find Roblox are malformed, maybe your Roblox installation is corrupt?"),
            Error::PlatformNotSupported => write!(formatter, "Your platform is not currently supported"),
            Error::RegistryError(error) => write!(formatter, "Couldn't find registry keys, Roblox might not be installed. ({})", error),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        if let Error::RegistryError(error) = self {
            Some(error)
        } else {
            None
        }
    }
}

#[derive(Debug)]
#[must_use]
pub struct RobloxStudio {
    application: PathBuf,
    built_in_plugins: PathBuf,
    plugins: PathBuf,
    root: PathBuf,
}

impl RobloxStudio {
    #[cfg(target_os = "windows")]
    pub fn locate() -> Result<RobloxStudio> {
        let hkcu = RegKey::predef(winreg::enums::HKEY_CURRENT_USER);

        let roblox_studio_reg = hkcu
            .open_subkey(r"Software\Roblox\RobloxStudio")
            .map_err(Error::RegistryError)?;

        let content_folder_value: String = roblox_studio_reg
            .get_value("ContentFolder")
            .map_err(Error::RegistryError)?;

        let content_folder_path = PathBuf::from(content_folder_value);

        let root = content_folder_path
            .parent()
            .ok_or(Error::MalformedRegistry)?;

        let plugins = root
            .parent()
            .ok_or(Error::MalformedRegistry)?
            .parent()
            .ok_or(Error::MalformedRegistry)?
            .join("Plugins");

        Ok(RobloxStudio {
            application: root.join("RobloxStudioBeta.exe"),
            built_in_plugins: root.join("BuiltInPlugins"),
            plugins: plugins.to_owned(),
            root: root.to_path_buf(),
        })
    }

    #[cfg(target_os = "macos")]
    pub fn locate() -> Result<RobloxStudio> {
        let root = PathBuf::from("/Applications").join("RobloxStudio.app");
        let contents = root.join("Contents");
        let exe = contents.join("MacOS").join("RobloxStudio");
        let built_in_plugins = contents.join("Resources").join("BuiltInPlugins");
        let documents = document_dir().ok_or(Error::DocumentsDirectoryNotFound)?;
        let plugins = documents.join("Roblox").join("Plugins");

        Ok(RobloxStudio {
            application,
            built_in_plugins,
            plugins,
            root,
        })
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    #[inline]
    pub fn locate() -> Result<RobloxStudio> {
        Err(Error::PlatformNotSupported)
    }

    #[deprecated(
        since = "0.2.0",
        note = "The contents of the studio directory are inconsistent across platforms. \
        Please use a dedicated method (like application_path) or file a feature request if one does not exist."
    )]
    #[must_use]
    #[inline]
    pub fn root_path(&self) -> &Path {
        &self.root
    }

    #[must_use]
    #[inline]
    pub fn application_path(&self) -> &Path {
        &self.application
    }

    #[deprecated(since = "0.2.0", note = "Please use application_path instead.")]
    #[must_use]
    #[inline]
    pub fn exe_path(&self) -> PathBuf {
        self.application_path().to_owned()
    }

    #[must_use]
    #[inline]
    pub fn built_in_plugins_path(&self) -> &Path {
        &self.built_in_plugins
    }

    #[must_use]
    #[inline]
    pub fn plugins_path(&self) -> &Path {
        &self.plugins
    }
}
