//! PE (Portable Executable) file handler for .exe and .dll files
//!
//! Parses Windows executable files to extract metadata including:
//! - File version information
//! - Architecture (32/64 bit)
//! - Subsystem (GUI/Console)
//! - Import/Export tables
//! - Resources

use crate::file_handlers::common::{
    FileHandlerError, FileHandlerResult, FileInfo, read_file_header, run_command,
};
use std::path::Path;

/// DOS header magic number
const DOS_MAGIC: [u8; 2] = [0x4D, 0x5A]; // "MZ"

/// PE signature
const PE_SIGNATURE: [u8; 4] = [0x50, 0x45, 0x00, 0x00]; // "PE\0\0"

/// Machine types
#[derive(Debug, Clone, Copy)]
pub enum MachineType {
    Unknown,
    I386,      // 0x014c
    AMD64,     // 0x8664
    ARM,       // 0x01c0
    ARM64,     // 0xaa64
    IA64,      // 0x0200
}

impl MachineType {
    fn from_u16(value: u16) -> Self {
        match value {
            0x014c => MachineType::I386,
            0x8664 => MachineType::AMD64,
            0x01c0 => MachineType::ARM,
            0xaa64 => MachineType::ARM64,
            0x0200 => MachineType::IA64,
            _ => MachineType::Unknown,
        }
    }

    fn to_string(&self) -> &'static str {
        match self {
            MachineType::Unknown => "Unknown",
            MachineType::I386 => "x86 (32-bit)",
            MachineType::AMD64 => "x64 (64-bit)",
            MachineType::ARM => "ARM",
            MachineType::ARM64 => "ARM64",
            MachineType::IA64 => "IA-64 (Itanium)",
        }
    }
}

/// PE subsystem types
#[derive(Debug, Clone, Copy)]
pub enum Subsystem {
    Unknown,
    Native,
    WindowsGUI,
    WindowsCUI,
    OS2CUI,
    PosixCUI,
    WindowsCEGUI,
    EfiApplication,
    EfiBootServiceDriver,
    EfiRuntimeDriver,
    EfiRom,
    Xbox,
    WindowsBootApplication,
}

impl Subsystem {
    fn from_u16(value: u16) -> Self {
        match value {
            0 => Subsystem::Unknown,
            1 => Subsystem::Native,
            2 => Subsystem::WindowsGUI,
            3 => Subsystem::WindowsCUI,
            5 => Subsystem::OS2CUI,
            7 => Subsystem::PosixCUI,
            9 => Subsystem::WindowsCEGUI,
            10 => Subsystem::EfiApplication,
            11 => Subsystem::EfiBootServiceDriver,
            12 => Subsystem::EfiRuntimeDriver,
            13 => Subsystem::EfiRom,
            14 => Subsystem::Xbox,
            16 => Subsystem::WindowsBootApplication,
            _ => Subsystem::Unknown,
        }
    }

    fn to_string(&self) -> &'static str {
        match self {
            Subsystem::Unknown => "Unknown",
            Subsystem::Native => "Native (Driver)",
            Subsystem::WindowsGUI => "Windows GUI",
            Subsystem::WindowsCUI => "Windows Console",
            Subsystem::OS2CUI => "OS/2 Console",
            Subsystem::PosixCUI => "POSIX Console",
            Subsystem::WindowsCEGUI => "Windows CE GUI",
            Subsystem::EfiApplication => "EFI Application",
            Subsystem::EfiBootServiceDriver => "EFI Boot Service Driver",
            Subsystem::EfiRuntimeDriver => "EFI Runtime Driver",
            Subsystem::EfiRom => "EFI ROM",
            Subsystem::Xbox => "Xbox",
            Subsystem::WindowsBootApplication => "Windows Boot Application",
        }
    }
}

/// Basic PE file information
#[derive(Debug)]
pub struct PeInfo {
    pub machine_type: MachineType,
    pub subsystem: Subsystem,
    pub is_dll: bool,
    pub is_64bit: bool,
    pub timestamp: u32,
    pub entry_point: u32,
    pub image_base: u64,
    pub section_count: u16,
}

/// Parse basic PE header information
pub fn parse_pe_header(path: &Path) -> FileHandlerResult<PeInfo> {
    use std::io::{Read, Seek, SeekFrom};
    use std::fs::File;

    let mut file = File::open(path)?;
    let mut buffer = [0u8; 64];

    // Read DOS header
    file.read_exact(&mut buffer)?;

    if buffer[0..2] != DOS_MAGIC {
        return Err(FileHandlerError::Parse("Not a valid DOS/PE file".to_string()));
    }

    // Get PE header offset from DOS header (at offset 0x3C)
    let pe_offset = u32::from_le_bytes([buffer[0x3C], buffer[0x3D], buffer[0x3E], buffer[0x3F]]);

    // Seek to PE header
    file.seek(SeekFrom::Start(pe_offset as u64))?;

    // Read PE signature and COFF header
    let mut pe_header = [0u8; 24];
    file.read_exact(&mut pe_header)?;

    if pe_header[0..4] != PE_SIGNATURE {
        return Err(FileHandlerError::Parse("Invalid PE signature".to_string()));
    }

    let machine = u16::from_le_bytes([pe_header[4], pe_header[5]]);
    let machine_type = MachineType::from_u16(machine);

    let section_count = u16::from_le_bytes([pe_header[6], pe_header[7]]);
    let timestamp = u32::from_le_bytes([pe_header[8], pe_header[9], pe_header[10], pe_header[11]]);
    let characteristics = u16::from_le_bytes([pe_header[22], pe_header[23]]);

    let is_dll = (characteristics & 0x2000) != 0;
    let is_64bit = machine == 0x8664 || machine == 0xaa64;

    // Read optional header magic to confirm 32/64 bit
    let mut opt_header = [0u8; 96];
    file.read_exact(&mut opt_header)?;

    let magic = u16::from_le_bytes([opt_header[0], opt_header[1]]);
    let is_pe32plus = magic == 0x20b;

    let (entry_point, image_base, subsystem_offset) = if is_pe32plus {
        // PE32+
        let entry = u32::from_le_bytes([opt_header[16], opt_header[17], opt_header[18], opt_header[19]]);
        let base = u64::from_le_bytes([
            opt_header[24], opt_header[25], opt_header[26], opt_header[27],
            opt_header[28], opt_header[29], opt_header[30], opt_header[31],
        ]);
        (entry, base, 68)
    } else {
        // PE32
        let entry = u32::from_le_bytes([opt_header[16], opt_header[17], opt_header[18], opt_header[19]]);
        let base = u32::from_le_bytes([opt_header[28], opt_header[29], opt_header[30], opt_header[31]]) as u64;
        (entry, base, 68)
    };

    let subsystem_value = u16::from_le_bytes([opt_header[subsystem_offset], opt_header[subsystem_offset + 1]]);
    let subsystem = Subsystem::from_u16(subsystem_value);

    Ok(PeInfo {
        machine_type,
        subsystem,
        is_dll,
        is_64bit: is_pe32plus,
        timestamp,
        entry_point,
        image_base,
        section_count,
    })
}

/// Get information about an EXE file
pub fn get_exe_info(path: &Path) -> FileHandlerResult<FileInfo> {
    let mut info = FileInfo::new(path)?.with_type("Windows Executable");

    match parse_pe_header(path) {
        Ok(pe_info) => {
            info.add_property("Architecture", pe_info.machine_type.to_string());
            info.add_property("Subsystem", pe_info.subsystem.to_string());
            info.add_property("64-bit", if pe_info.is_64bit { "Yes" } else { "No" });
            info.add_property("Sections", &pe_info.section_count.to_string());
            info.add_property("Entry Point", &format!("0x{:08X}", pe_info.entry_point));
            info.add_property("Image Base", &format!("0x{:016X}", pe_info.image_base));

            // Format timestamp
            let timestamp = pe_info.timestamp;
            info.add_property("Build Timestamp", &format!("{} (Unix timestamp)", timestamp));
        }
        Err(e) => {
            info.add_property("Parse Error", &e.to_string());
        }
    }

    // Try to get additional info using objdump if available
    if let Ok(output) = run_command("objdump", &["-f", path.to_str().unwrap_or("")]) {
        for line in output.lines() {
            if line.contains("file format") {
                info.add_property("Format", line.trim());
            }
        }
    }

    Ok(info)
}

/// Get information about a DLL file
pub fn get_dll_info(path: &Path) -> FileHandlerResult<FileInfo> {
    let mut info = FileInfo::new(path)?.with_type("Windows Dynamic Library");

    match parse_pe_header(path) {
        Ok(pe_info) => {
            info.add_property("Architecture", pe_info.machine_type.to_string());
            info.add_property("64-bit", if pe_info.is_64bit { "Yes" } else { "No" });
            info.add_property("Sections", &pe_info.section_count.to_string());
            info.add_property("Image Base", &format!("0x{:016X}", pe_info.image_base));

            if !pe_info.is_dll {
                info.add_property("Warning", "File does not have DLL flag set in PE header");
            }
        }
        Err(e) => {
            info.add_property("Parse Error", &e.to_string());
        }
    }

    // Try to list exports using objdump
    if let Ok(output) = run_command("objdump", &["-p", path.to_str().unwrap_or("")]) {
        let exports: Vec<&str> = output
            .lines()
            .filter(|l| l.contains("[") && l.contains("]"))
            .take(10)
            .collect();
        if !exports.is_empty() {
            info.add_property("Exports (sample)", &exports.join("\n"));
        }
    }

    Ok(info)
}

/// Run an EXE file with Wine
pub fn run_with_wine(path: &Path) -> FileHandlerResult<String> {
    use std::process::Command;

    let path_str = path.to_str()
        .ok_or_else(|| FileHandlerError::Parse("Invalid path".to_string()))?;

    // Check if Wine is available
    let wine_cmd = if cfg!(target_os = "macos") {
        "wine64"
    } else {
        "wine"
    };

    let status = Command::new(wine_cmd)
        .arg(path_str)
        .spawn()
        .map_err(|e| FileHandlerError::CommandFailed(format!("Failed to run Wine: {}", e)))?;

    Ok(format!("Started {} with Wine (PID: {:?})", path_str, status.id()))
}

/// Extract resources from an EXE file
pub fn extract_resources(path: &Path) -> FileHandlerResult<String> {
    let path_str = path.to_str()
        .ok_or_else(|| FileHandlerError::Parse("Invalid path".to_string()))?;

    // Create output directory
    let parent = path.parent().unwrap_or(Path::new("."));
    let stem = path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("extracted");
    let output_dir = parent.join(format!("{}_resources", stem));

    std::fs::create_dir_all(&output_dir)?;

    // Try wrestool (from icoutils) for icon extraction
    if let Ok(output) = run_command("wrestool", &[
        "-x", "-t", "14",
        "-o", output_dir.to_str().unwrap_or("."),
        path_str
    ]) {
        return Ok(format!("Extracted resources to {}\n{}", output_dir.display(), output));
    }

    // Try 7z as fallback
    if let Ok(output) = run_command("7z", &[
        "x", "-y",
        &format!("-o{}", output_dir.display()),
        path_str
    ]) {
        return Ok(format!("Extracted with 7z to {}\n{}", output_dir.display(), output));
    }

    Err(FileHandlerError::NotSupported(
        "No suitable tool found for resource extraction (install icoutils or 7z)".to_string()
    ))
}
