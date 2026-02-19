//! Linux Shared Object (.so) file handler
//!
//! Parses ELF format shared libraries to extract:
//! - Architecture information
//! - Dependencies
//! - Exported symbols
//! - Version information

use crate::file_handlers::common::{
    FileHandlerError, FileHandlerResult, FileInfo, run_command, command_exists,
    read_file_header,
};
use std::path::Path;

/// ELF magic number
const ELF_MAGIC: [u8; 4] = [0x7f, b'E', b'L', b'F'];

/// ELF class (32 or 64 bit)
#[derive(Debug, Clone, Copy)]
pub enum ElfClass {
    Elf32,
    Elf64,
    Unknown,
}

/// ELF file type
#[derive(Debug, Clone)]
pub enum ElfType {
    None,
    Relocatable,
    Executable,
    SharedObject,
    Core,
    Unknown(u16),
}

impl ElfType {
    fn from_u16(value: u16) -> Self {
        match value {
            0 => ElfType::None,
            1 => ElfType::Relocatable,
            2 => ElfType::Executable,
            3 => ElfType::SharedObject,
            4 => ElfType::Core,
            v => ElfType::Unknown(v),
        }
    }

    fn to_string(&self) -> String {
        match self {
            ElfType::None => "None".to_string(),
            ElfType::Relocatable => "Relocatable".to_string(),
            ElfType::Executable => "Executable".to_string(),
            ElfType::SharedObject => "Shared Object".to_string(),
            ElfType::Core => "Core Dump".to_string(),
            ElfType::Unknown(v) => format!("Unknown ({})", v),
        }
    }
}

/// ELF machine type (architecture)
#[derive(Debug, Clone)]
pub enum ElfMachine {
    None,
    I386,
    X86_64,
    ARM,
    AARCH64,
    RISCV,
    Unknown(u16),
}

impl ElfMachine {
    fn from_u16(value: u16) -> Self {
        match value {
            0 => ElfMachine::None,
            3 => ElfMachine::I386,
            62 => ElfMachine::X86_64,
            40 => ElfMachine::ARM,
            183 => ElfMachine::AARCH64,
            243 => ElfMachine::RISCV,
            v => ElfMachine::Unknown(v),
        }
    }

    fn to_string(&self) -> String {
        match self {
            ElfMachine::None => "None".to_string(),
            ElfMachine::I386 => "x86 (32-bit)".to_string(),
            ElfMachine::X86_64 => "x86-64 (64-bit)".to_string(),
            ElfMachine::ARM => "ARM".to_string(),
            ElfMachine::AARCH64 => "ARM64 (AArch64)".to_string(),
            ElfMachine::RISCV => "RISC-V".to_string(),
            ElfMachine::Unknown(v) => format!("Unknown ({})", v),
        }
    }
}

/// Basic ELF header info
#[derive(Debug)]
pub struct ElfInfo {
    pub class: ElfClass,
    pub is_little_endian: bool,
    pub elf_type: ElfType,
    pub machine: ElfMachine,
    pub entry_point: u64,
}

/// Parse ELF header
pub fn parse_elf_header(path: &Path) -> FileHandlerResult<ElfInfo> {
    let header = read_file_header(path, 64)?; // ELF header is up to 64 bytes

    // Check magic
    if header[0..4] != ELF_MAGIC {
        return Err(FileHandlerError::Parse("Not an ELF file".to_string()));
    }

    // Get class (32/64 bit)
    let class = match header[4] {
        1 => ElfClass::Elf32,
        2 => ElfClass::Elf64,
        _ => ElfClass::Unknown,
    };

    // Get endianness
    let is_little_endian = header[5] == 1;

    // Helper to read u16
    let read_u16 = |offset: usize| -> u16 {
        if is_little_endian {
            u16::from_le_bytes([header[offset], header[offset + 1]])
        } else {
            u16::from_be_bytes([header[offset], header[offset + 1]])
        }
    };

    // Get file type (offset 16)
    let elf_type = ElfType::from_u16(read_u16(16));

    // Get machine type (offset 18)
    let machine = ElfMachine::from_u16(read_u16(18));

    // Get entry point (offset 24 for 64-bit, different for 32-bit)
    let entry_point = match class {
        ElfClass::Elf64 => {
            if is_little_endian {
                u64::from_le_bytes([
                    header[24], header[25], header[26], header[27],
                    header[28], header[29], header[30], header[31],
                ])
            } else {
                u64::from_be_bytes([
                    header[24], header[25], header[26], header[27],
                    header[28], header[29], header[30], header[31],
                ])
            }
        }
        ElfClass::Elf32 => {
            if is_little_endian {
                u32::from_le_bytes([header[24], header[25], header[26], header[27]]) as u64
            } else {
                u32::from_be_bytes([header[24], header[25], header[26], header[27]]) as u64
            }
        }
        ElfClass::Unknown => 0,
    };

    Ok(ElfInfo {
        class,
        is_little_endian,
        elf_type,
        machine,
        entry_point,
    })
}

/// Get information about a shared object file
pub fn get_so_info(path: &Path) -> FileHandlerResult<FileInfo> {
    let mut info = FileInfo::new(path)?.with_type("Linux Shared Object");

    let path_str = path.to_str()
        .ok_or_else(|| FileHandlerError::Parse("Invalid path".to_string()))?;

    // Parse ELF header
    match parse_elf_header(path) {
        Ok(elf) => {
            info.add_property("Architecture", &elf.machine.to_string());
            info.add_property("64-bit", match elf.class {
                ElfClass::Elf64 => "Yes",
                ElfClass::Elf32 => "No",
                ElfClass::Unknown => "Unknown",
            });
            info.add_property("Endianness", if elf.is_little_endian { "Little Endian" } else { "Big Endian" });
            info.add_property("ELF Type", &elf.elf_type.to_string());

            if elf.entry_point != 0 {
                info.add_property("Entry Point", &format!("0x{:x}", elf.entry_point));
            }
        }
        Err(e) => {
            info.add_property("Parse Error", &e.to_string());
        }
    }

    // Get dependencies using ldd
    if command_exists("ldd") {
        if let Ok(output) = run_command("ldd", &[path_str]) {
            let deps: Vec<&str> = output.lines()
                .take(10)
                .map(|l| l.trim())
                .collect();
            if !deps.is_empty() {
                info.add_property("Dependencies", &deps.join("\n"));
            }
        }
    }

    // Get SONAME using objdump or readelf
    if command_exists("objdump") {
        if let Ok(output) = run_command("objdump", &["-p", path_str]) {
            for line in output.lines() {
                if line.contains("SONAME") {
                    if let Some(soname) = line.split_whitespace().last() {
                        info.add_property("SONAME", soname);
                    }
                }
                if line.contains("NEEDED") {
                    // Already covered by ldd
                }
            }
        }
    } else if command_exists("readelf") {
        if let Ok(output) = run_command("readelf", &["-d", path_str]) {
            for line in output.lines() {
                if line.contains("SONAME") {
                    if let Some(start) = line.find('[') {
                        if let Some(end) = line.find(']') {
                            info.add_property("SONAME", &line[start + 1..end]);
                        }
                    }
                }
            }
        }
    }

    // Count exported symbols using nm
    if command_exists("nm") {
        if let Ok(output) = run_command("nm", &["-D", "--defined-only", path_str]) {
            let symbol_count = output.lines().count();
            info.add_property("Exported Symbols", &symbol_count.to_string());

            // Show sample of symbols
            let sample: Vec<&str> = output.lines()
                .take(5)
                .filter_map(|l| l.split_whitespace().last())
                .collect();
            if !sample.is_empty() {
                info.add_property("Symbol Sample", &sample.join(", "));
            }
        }
    }

    // Get debug info status
    if command_exists("file") {
        if let Ok(output) = run_command("file", &[path_str]) {
            let has_debug = output.contains("with debug_info") || output.contains("not stripped");
            info.add_property("Debug Symbols", if has_debug { "Yes" } else { "No (stripped)" });
        }
    }

    Ok(info)
}

/// List dependencies of a shared object
pub fn list_dependencies(path: &Path) -> FileHandlerResult<Vec<String>> {
    let path_str = path.to_str()
        .ok_or_else(|| FileHandlerError::Parse("Invalid path".to_string()))?;

    // Try ldd
    if command_exists("ldd") {
        let output = run_command("ldd", &[path_str])?;
        let deps: Vec<String> = output.lines()
            .filter_map(|line| {
                // Format: "libname.so => /path/to/lib (address)" or "libname.so (address)"
                let parts: Vec<&str> = line.trim().split_whitespace().collect();
                if !parts.is_empty() {
                    Some(parts[0].to_string())
                } else {
                    None
                }
            })
            .collect();
        return Ok(deps);
    }

    // Try readelf
    if command_exists("readelf") {
        let output = run_command("readelf", &["-d", path_str])?;
        let deps: Vec<String> = output.lines()
            .filter(|l| l.contains("NEEDED"))
            .filter_map(|l| {
                if let Some(start) = l.find('[') {
                    if let Some(end) = l.find(']') {
                        return Some(l[start + 1..end].to_string());
                    }
                }
                None
            })
            .collect();
        return Ok(deps);
    }

    Err(FileHandlerError::NotSupported(
        "Listing dependencies requires ldd or readelf".to_string()
    ))
}

/// List exported symbols
pub fn list_symbols(path: &Path) -> FileHandlerResult<Vec<String>> {
    let path_str = path.to_str()
        .ok_or_else(|| FileHandlerError::Parse("Invalid path".to_string()))?;

    if command_exists("nm") {
        let output = run_command("nm", &["-D", "--defined-only", path_str])?;
        let symbols: Vec<String> = output.lines()
            .filter_map(|l| l.split_whitespace().last())
            .map(|s| s.to_string())
            .collect();
        return Ok(symbols);
    }

    if command_exists("readelf") {
        let output = run_command("readelf", &["--dyn-syms", path_str])?;
        let symbols: Vec<String> = output.lines()
            .filter(|l| l.contains("FUNC") || l.contains("OBJECT"))
            .filter_map(|l| l.split_whitespace().last())
            .map(|s| s.to_string())
            .collect();
        return Ok(symbols);
    }

    Err(FileHandlerError::NotSupported(
        "Listing symbols requires nm or readelf".to_string()
    ))
}

/// Get SONAME of a library
pub fn get_soname(path: &Path) -> FileHandlerResult<String> {
    let path_str = path.to_str()
        .ok_or_else(|| FileHandlerError::Parse("Invalid path".to_string()))?;

    if command_exists("objdump") {
        let output = run_command("objdump", &["-p", path_str])?;
        for line in output.lines() {
            if line.contains("SONAME") {
                if let Some(soname) = line.split_whitespace().last() {
                    return Ok(soname.to_string());
                }
            }
        }
    }

    if command_exists("readelf") {
        let output = run_command("readelf", &["-d", path_str])?;
        for line in output.lines() {
            if line.contains("SONAME") {
                if let Some(start) = line.find('[') {
                    if let Some(end) = line.find(']') {
                        return Ok(line[start + 1..end].to_string());
                    }
                }
            }
        }
    }

    Err(FileHandlerError::NotFound("SONAME not found".to_string()))
}
