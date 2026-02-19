//! macOS Dynamic Library (.dylib) file handler
//!
//! Parses Mach-O format dylib files to extract:
//! - Architecture information
//! - Version information
//! - Dependencies
//! - Exported symbols

use crate::file_handlers::common::{
    FileHandlerError, FileHandlerResult, FileInfo, run_command, command_exists,
    read_file_header,
};
use std::path::Path;

/// Mach-O magic numbers
const MH_MAGIC: u32 = 0xFEEDFACE;      // 32-bit
const MH_MAGIC_64: u32 = 0xFEEDFACF;   // 64-bit
const MH_CIGAM: u32 = 0xCEFAEDFE;      // 32-bit (byte-swapped)
const MH_CIGAM_64: u32 = 0xCFFAEDFE;   // 64-bit (byte-swapped)
const FAT_MAGIC: u32 = 0xCAFEBABE;     // Universal binary
const FAT_CIGAM: u32 = 0xBEBAFECA;     // Universal binary (byte-swapped)

/// Mach-O file types
const MH_DYLIB: u32 = 0x6;

/// CPU types
const CPU_TYPE_I386: u32 = 7;
const CPU_TYPE_X86_64: u32 = 7 | 0x01000000;
const CPU_TYPE_ARM: u32 = 12;
const CPU_TYPE_ARM64: u32 = 12 | 0x01000000;

/// Mach-O architecture info
#[derive(Debug, Clone)]
pub struct MachOArch {
    pub cpu_type: String,
    pub is_64bit: bool,
    pub file_type: String,
}

/// Parsed dylib info
#[derive(Debug)]
pub struct DylibInfo {
    pub architectures: Vec<MachOArch>,
    pub is_universal: bool,
    pub version: Option<String>,
    pub compatibility_version: Option<String>,
    pub dependencies: Vec<String>,
}

/// Check Mach-O magic and determine architecture
fn check_macho_magic(magic: u32) -> Option<(bool, bool)> {
    // Returns (is_macho, is_64bit)
    match magic {
        MH_MAGIC => Some((true, false)),
        MH_MAGIC_64 => Some((true, true)),
        MH_CIGAM => Some((true, false)),
        MH_CIGAM_64 => Some((true, true)),
        _ => None,
    }
}

/// Get CPU type name
fn cpu_type_name(cpu_type: u32) -> String {
    match cpu_type {
        CPU_TYPE_I386 => "x86 (32-bit)".to_string(),
        CPU_TYPE_X86_64 => "x86_64 (64-bit)".to_string(),
        CPU_TYPE_ARM => "ARM".to_string(),
        CPU_TYPE_ARM64 => "ARM64".to_string(),
        other => format!("Unknown (0x{:x})", other),
    }
}

/// Parse basic Mach-O header
fn parse_macho_header(data: &[u8]) -> FileHandlerResult<MachOArch> {
    if data.len() < 28 {
        return Err(FileHandlerError::Parse("Header too small".to_string()));
    }

    let magic = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);

    let (is_macho, is_64bit) = check_macho_magic(magic)
        .or_else(|| check_macho_magic(u32::from_be_bytes([data[0], data[1], data[2], data[3]])))
        .ok_or_else(|| FileHandlerError::Parse("Not a Mach-O file".to_string()))?;

    if !is_macho {
        return Err(FileHandlerError::Parse("Not a Mach-O file".to_string()));
    }

    // Determine byte order
    let is_big_endian = magic == MH_CIGAM || magic == MH_CIGAM_64 ||
                        u32::from_be_bytes([data[0], data[1], data[2], data[3]]) == MH_MAGIC ||
                        u32::from_be_bytes([data[0], data[1], data[2], data[3]]) == MH_MAGIC_64;

    let read_u32 = |offset: usize| -> u32 {
        if is_big_endian {
            u32::from_be_bytes([data[offset], data[offset+1], data[offset+2], data[offset+3]])
        } else {
            u32::from_le_bytes([data[offset], data[offset+1], data[offset+2], data[offset+3]])
        }
    };

    let cpu_type = read_u32(4);
    let file_type = read_u32(12);

    let file_type_str = match file_type {
        MH_DYLIB => "Dynamic Library",
        0x1 => "Object",
        0x2 => "Executable",
        0x8 => "Bundle",
        _ => "Unknown",
    };

    Ok(MachOArch {
        cpu_type: cpu_type_name(cpu_type),
        is_64bit,
        file_type: file_type_str.to_string(),
    })
}

/// Get information about a dylib file
pub fn get_dylib_info(path: &Path) -> FileHandlerResult<FileInfo> {
    let mut info = FileInfo::new(path)?.with_type("macOS Dynamic Library");

    let path_str = path.to_str()
        .ok_or_else(|| FileHandlerError::Parse("Invalid path".to_string()))?;

    // Read file header
    let header = read_file_header(path, 32)?;

    // Check for fat/universal binary
    let first_four = u32::from_be_bytes([header[0], header[1], header[2], header[3]]);
    let is_fat = first_four == FAT_MAGIC || first_four == FAT_CIGAM;

    if is_fat {
        info.add_property("Format", "Universal Binary (Fat)");

        // Try to get architectures using lipo or file
        if command_exists("lipo") {
            if let Ok(output) = run_command("lipo", &["-info", path_str]) {
                info.add_property("Architectures", output.trim());
            }
        } else if command_exists("file") {
            if let Ok(output) = run_command("file", &[path_str]) {
                if let Some(desc) = output.split(':').nth(1) {
                    info.add_property("Type", desc.trim());
                }
            }
        }
    } else {
        // Single architecture
        match parse_macho_header(&header) {
            Ok(arch) => {
                info.add_property("Format", "Mach-O");
                info.add_property("Architecture", &arch.cpu_type);
                info.add_property("64-bit", if arch.is_64bit { "Yes" } else { "No" });
                info.add_property("File Type", &arch.file_type);
            }
            Err(e) => {
                info.add_property("Parse Error", &e.to_string());
            }
        }
    }

    // Get dependencies using otool
    if command_exists("otool") {
        if let Ok(output) = run_command("otool", &["-L", path_str]) {
            let deps: Vec<&str> = output.lines()
                .skip(1) // Skip the first line (file name)
                .take(10)
                .map(|l| l.trim())
                .collect();
            if !deps.is_empty() {
                info.add_property("Dependencies", &deps.join("\n"));
            }
        }

        // Get version info
        if let Ok(output) = run_command("otool", &["-D", path_str]) {
            if let Some(id) = output.lines().nth(1) {
                info.add_property("Install Name", id.trim());
            }
        }
    }

    // Alternative: use objdump
    if !command_exists("otool") && command_exists("objdump") {
        if let Ok(output) = run_command("objdump", &["-p", path_str]) {
            for line in output.lines() {
                if line.contains("NEEDED") {
                    info.add_property("Dependency", line.trim());
                }
            }
        }
    }

    // Use nm for symbols count
    if command_exists("nm") {
        if let Ok(output) = run_command("nm", &["-U", path_str]) {
            let symbol_count = output.lines().count();
            info.add_property("Exported Symbols", &symbol_count.to_string());
        }
    }

    Ok(info)
}

/// List dependencies of a dylib
pub fn list_dependencies(path: &Path) -> FileHandlerResult<Vec<String>> {
    let path_str = path.to_str()
        .ok_or_else(|| FileHandlerError::Parse("Invalid path".to_string()))?;

    // Try otool first
    if command_exists("otool") {
        let output = run_command("otool", &["-L", path_str])?;
        let deps: Vec<String> = output.lines()
            .skip(1)
            .map(|l| {
                // Format is: "path (compatibility version X, current version Y)"
                l.trim()
                    .split(" (")
                    .next()
                    .unwrap_or(l.trim())
                    .to_string()
            })
            .collect();
        return Ok(deps);
    }

    // Try objdump
    if command_exists("objdump") {
        let output = run_command("objdump", &["-p", path_str])?;
        let deps: Vec<String> = output.lines()
            .filter(|l| l.contains("NEEDED"))
            .filter_map(|l| l.split_whitespace().last())
            .map(|s| s.to_string())
            .collect();
        return Ok(deps);
    }

    Err(FileHandlerError::NotSupported(
        "Dependency listing requires otool or objdump".to_string()
    ))
}

/// List exported symbols
pub fn list_symbols(path: &Path) -> FileHandlerResult<Vec<String>> {
    let path_str = path.to_str()
        .ok_or_else(|| FileHandlerError::Parse("Invalid path".to_string()))?;

    if command_exists("nm") {
        let output = run_command("nm", &["-U", "-g", path_str])?;
        let symbols: Vec<String> = output.lines()
            .filter_map(|l| {
                // Format is: "address type name"
                let parts: Vec<&str> = l.split_whitespace().collect();
                if parts.len() >= 3 {
                    Some(parts[2].to_string())
                } else if parts.len() == 2 {
                    Some(parts[1].to_string())
                } else {
                    None
                }
            })
            .collect();
        return Ok(symbols);
    }

    Err(FileHandlerError::NotSupported(
        "Symbol listing requires nm".to_string()
    ))
}
