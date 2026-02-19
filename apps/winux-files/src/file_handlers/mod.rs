//! File handlers for various file formats across Windows, macOS, and Linux
//!
//! This module provides native support for handling platform-specific file formats
//! including executables, packages, archives, and disk images.

pub mod windows;
pub mod macos;
pub mod linux;
pub mod archives;
pub mod common;

use std::path::Path;

pub use common::{FileInfo, FileHandlerResult, FileHandlerError, FileAction};

/// Represents the type of file being handled
#[derive(Debug, Clone, PartialEq)]
pub enum FileType {
    // Windows formats
    WindowsExecutable,      // .exe
    WindowsInstaller,       // .msi
    WindowsLibrary,         // .dll
    WindowsShortcut,        // .lnk
    WindowsRegistry,        // .reg
    WindowsBatch,           // .bat, .cmd
    WindowsPowerShell,      // .ps1

    // macOS formats
    MacOSDiskImage,         // .dmg
    MacOSApplication,       // .app
    MacOSPackage,           // .pkg
    MacOSPropertyList,      // .plist
    MacOSIcons,             // .icns
    MacOSLibrary,           // .dylib

    // Linux formats
    DebianPackage,          // .deb
    RpmPackage,             // .rpm
    AppImage,               // .AppImage
    Flatpak,                // .flatpak, .flatpakref
    Snap,                   // .snap
    LinuxLibrary,           // .so

    // Archive formats
    Zip,
    Rar,
    SevenZip,
    TarGz,
    TarXz,
    TarBz2,
    Tar,
    Iso,
    Img,

    // Other
    Unknown,
}

impl FileType {
    /// Detect file type from path extension
    pub fn from_path(path: &Path) -> Self {
        let extension = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase())
            .unwrap_or_default();

        // Check for compound extensions like .tar.gz
        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default()
            .to_lowercase();

        if file_name.ends_with(".tar.gz") || file_name.ends_with(".tgz") {
            return FileType::TarGz;
        }
        if file_name.ends_with(".tar.xz") || file_name.ends_with(".txz") {
            return FileType::TarXz;
        }
        if file_name.ends_with(".tar.bz2") || file_name.ends_with(".tbz2") {
            return FileType::TarBz2;
        }

        match extension.as_str() {
            // Windows
            "exe" => FileType::WindowsExecutable,
            "msi" => FileType::WindowsInstaller,
            "dll" => FileType::WindowsLibrary,
            "lnk" => FileType::WindowsShortcut,
            "reg" => FileType::WindowsRegistry,
            "bat" | "cmd" => FileType::WindowsBatch,
            "ps1" => FileType::WindowsPowerShell,

            // macOS
            "dmg" => FileType::MacOSDiskImage,
            "app" => FileType::MacOSApplication,
            "pkg" => FileType::MacOSPackage,
            "plist" => FileType::MacOSPropertyList,
            "icns" => FileType::MacOSIcons,
            "dylib" => FileType::MacOSLibrary,

            // Linux
            "deb" => FileType::DebianPackage,
            "rpm" => FileType::RpmPackage,
            "appimage" => FileType::AppImage,
            "flatpak" | "flatpakref" => FileType::Flatpak,
            "snap" => FileType::Snap,
            "so" => FileType::LinuxLibrary,

            // Archives
            "zip" => FileType::Zip,
            "rar" => FileType::Rar,
            "7z" => FileType::SevenZip,
            "tar" => FileType::Tar,
            "iso" => FileType::Iso,
            "img" => FileType::Img,

            _ => FileType::Unknown,
        }
    }

    /// Get available actions for this file type
    pub fn get_actions(&self) -> Vec<FileAction> {
        match self {
            // Windows
            FileType::WindowsExecutable => vec![
                FileAction::ViewInfo,
                FileAction::RunWithWine,
                FileAction::Extract,
            ],
            FileType::WindowsInstaller => vec![
                FileAction::ViewInfo,
                FileAction::Install,
                FileAction::Extract,
            ],
            FileType::WindowsLibrary => vec![FileAction::ViewInfo],
            FileType::WindowsShortcut => vec![FileAction::ViewInfo, FileAction::FollowLink],
            FileType::WindowsRegistry => vec![FileAction::ViewContent, FileAction::Import],
            FileType::WindowsBatch | FileType::WindowsPowerShell => {
                vec![FileAction::ViewContent, FileAction::Edit]
            }

            // macOS
            FileType::MacOSDiskImage => vec![FileAction::Mount, FileAction::Extract],
            FileType::MacOSApplication => vec![FileAction::ViewInfo, FileAction::Browse],
            FileType::MacOSPackage => vec![FileAction::ViewInfo, FileAction::Extract],
            FileType::MacOSPropertyList => vec![FileAction::ViewContent, FileAction::Edit],
            FileType::MacOSIcons => vec![FileAction::ViewContent],
            FileType::MacOSLibrary => vec![FileAction::ViewInfo],

            // Linux
            FileType::DebianPackage => vec![
                FileAction::ViewInfo,
                FileAction::Install,
                FileAction::Extract,
            ],
            FileType::RpmPackage => vec![
                FileAction::ViewInfo,
                FileAction::Install,
                FileAction::Extract,
            ],
            FileType::AppImage => vec![
                FileAction::Run,
                FileAction::Extract,
                FileAction::MakeExecutable,
            ],
            FileType::Flatpak => vec![FileAction::ViewInfo, FileAction::Install],
            FileType::Snap => vec![FileAction::ViewInfo, FileAction::Install],
            FileType::LinuxLibrary => vec![FileAction::ViewInfo],

            // Archives
            FileType::Zip
            | FileType::Rar
            | FileType::SevenZip
            | FileType::TarGz
            | FileType::TarXz
            | FileType::TarBz2
            | FileType::Tar => vec![FileAction::Extract, FileAction::Browse, FileAction::ViewInfo],
            FileType::Iso | FileType::Img => vec![FileAction::Mount, FileAction::Extract],

            FileType::Unknown => vec![],
        }
    }

    /// Get icon name for this file type
    pub fn get_icon(&self) -> &'static str {
        match self {
            FileType::WindowsExecutable | FileType::WindowsInstaller => {
                "application-x-executable-symbolic"
            }
            FileType::WindowsLibrary | FileType::MacOSLibrary | FileType::LinuxLibrary => {
                "application-x-sharedlib-symbolic"
            }
            FileType::WindowsShortcut => "emblem-symbolic-link-symbolic",
            FileType::WindowsRegistry => "text-x-generic-symbolic",
            FileType::WindowsBatch | FileType::WindowsPowerShell => "text-x-script-symbolic",
            FileType::MacOSDiskImage | FileType::Iso | FileType::Img => {
                "drive-optical-symbolic"
            }
            FileType::MacOSApplication => "application-x-executable-symbolic",
            FileType::MacOSPackage => "package-x-generic-symbolic",
            FileType::MacOSPropertyList => "text-x-generic-symbolic",
            FileType::MacOSIcons => "image-x-generic-symbolic",
            FileType::DebianPackage | FileType::RpmPackage => "package-x-generic-symbolic",
            FileType::AppImage | FileType::Flatpak | FileType::Snap => {
                "application-x-executable-symbolic"
            }
            FileType::Zip
            | FileType::Rar
            | FileType::SevenZip
            | FileType::TarGz
            | FileType::TarXz
            | FileType::TarBz2
            | FileType::Tar => "package-x-generic-symbolic",
            FileType::Unknown => "text-x-generic-symbolic",
        }
    }
}

/// Main file handler that delegates to specific handlers
pub struct FileHandler;

impl FileHandler {
    /// Get information about a file
    pub fn get_info(path: &Path) -> FileHandlerResult<FileInfo> {
        let file_type = FileType::from_path(path);

        match file_type {
            // Windows handlers
            FileType::WindowsExecutable => windows::pe::get_exe_info(path),
            FileType::WindowsInstaller => windows::msi::get_msi_info(path),
            FileType::WindowsLibrary => windows::pe::get_dll_info(path),
            FileType::WindowsShortcut => windows::lnk::get_lnk_info(path),
            FileType::WindowsRegistry => windows::reg::get_reg_info(path),
            FileType::WindowsBatch | FileType::WindowsPowerShell => {
                windows::scripts::get_script_info(path)
            }

            // macOS handlers
            FileType::MacOSDiskImage => macos::dmg::get_dmg_info(path),
            FileType::MacOSApplication => macos::app::get_app_info(path),
            FileType::MacOSPackage => macos::pkg::get_pkg_info(path),
            FileType::MacOSPropertyList => macos::plist::get_plist_info(path),
            FileType::MacOSIcons => macos::icns::get_icns_info(path),
            FileType::MacOSLibrary => macos::dylib::get_dylib_info(path),

            // Linux handlers
            FileType::DebianPackage => linux::deb::get_deb_info(path),
            FileType::RpmPackage => linux::rpm::get_rpm_info(path),
            FileType::AppImage => linux::appimage::get_appimage_info(path),
            FileType::Flatpak => linux::flatpak::get_flatpak_info(path),
            FileType::Snap => linux::snap::get_snap_info(path),
            FileType::LinuxLibrary => linux::so::get_so_info(path),

            // Archive handlers
            FileType::Zip
            | FileType::Rar
            | FileType::SevenZip
            | FileType::TarGz
            | FileType::TarXz
            | FileType::TarBz2
            | FileType::Tar => archives::get_archive_info(path),
            FileType::Iso => archives::iso::get_iso_info(path),
            FileType::Img => archives::img::get_img_info(path),

            FileType::Unknown => common::get_generic_info(path),
        }
    }

    /// Execute an action on a file
    pub fn execute_action(path: &Path, action: FileAction) -> FileHandlerResult<String> {
        let file_type = FileType::from_path(path);

        match (file_type, action) {
            // Windows actions
            (FileType::WindowsExecutable, FileAction::RunWithWine) => {
                windows::pe::run_with_wine(path)
            }
            (FileType::WindowsExecutable, FileAction::Extract) => {
                windows::pe::extract_resources(path)
            }
            (FileType::WindowsInstaller, FileAction::Install) => {
                windows::msi::install_msi(path)
            }
            (FileType::WindowsInstaller, FileAction::Extract) => {
                windows::msi::extract_msi(path)
            }
            (FileType::WindowsShortcut, FileAction::FollowLink) => {
                windows::lnk::follow_link(path)
            }
            (FileType::WindowsRegistry, FileAction::Import) => {
                windows::reg::import_reg(path)
            }

            // macOS actions
            (FileType::MacOSDiskImage, FileAction::Mount) => {
                macos::dmg::mount_dmg(path)
            }
            (FileType::MacOSDiskImage, FileAction::Extract) => {
                macos::dmg::extract_dmg(path)
            }
            (FileType::MacOSApplication, FileAction::Browse) => {
                macos::app::browse_app_bundle(path)
            }
            (FileType::MacOSPackage, FileAction::Extract) => {
                macos::pkg::extract_pkg(path)
            }

            // Linux actions
            (FileType::DebianPackage, FileAction::Install) => {
                linux::deb::install_deb(path)
            }
            (FileType::DebianPackage, FileAction::Extract) => {
                linux::deb::extract_deb(path)
            }
            (FileType::RpmPackage, FileAction::Install) => {
                linux::rpm::install_rpm(path)
            }
            (FileType::RpmPackage, FileAction::Extract) => {
                linux::rpm::extract_rpm(path)
            }
            (FileType::AppImage, FileAction::Run) => {
                linux::appimage::run_appimage(path)
            }
            (FileType::AppImage, FileAction::Extract) => {
                linux::appimage::extract_appimage(path)
            }
            (FileType::AppImage, FileAction::MakeExecutable) => {
                linux::appimage::make_executable(path)
            }
            (FileType::Flatpak, FileAction::Install) => {
                linux::flatpak::install_flatpak(path)
            }
            (FileType::Snap, FileAction::Install) => {
                linux::snap::install_snap(path)
            }

            // Archive actions
            (
                FileType::Zip
                | FileType::Rar
                | FileType::SevenZip
                | FileType::TarGz
                | FileType::TarXz
                | FileType::TarBz2
                | FileType::Tar,
                FileAction::Extract,
            ) => archives::extract_archive(path),
            (
                FileType::Zip
                | FileType::Rar
                | FileType::SevenZip
                | FileType::TarGz
                | FileType::TarXz
                | FileType::TarBz2
                | FileType::Tar,
                FileAction::Browse,
            ) => archives::list_contents(path),
            (FileType::Iso, FileAction::Mount) => archives::iso::mount_iso(path),
            (FileType::Iso, FileAction::Extract) => archives::iso::extract_iso(path),
            (FileType::Img, FileAction::Mount) => archives::img::mount_img(path),

            // View content for scripts and text-based formats
            (_, FileAction::ViewContent) => common::view_content(path),
            (_, FileAction::Edit) => common::open_in_editor(path),
            (_, FileAction::ViewInfo) => {
                FileHandler::get_info(path).map(|info| info.to_string())
            }

            _ => Err(FileHandlerError::UnsupportedAction(format!(
                "Action not supported for this file type"
            ))),
        }
    }
}
