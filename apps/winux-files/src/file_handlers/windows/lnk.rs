//! Windows shortcut (.lnk) file handler
//!
//! Parses Windows LNK files to extract:
//! - Target path
//! - Working directory
//! - Arguments
//! - Icon location
//! - Description

use crate::file_handlers::common::{
    FileHandlerError, FileHandlerResult, FileInfo, read_file_header,
};
use std::path::Path;

/// LNK file magic bytes
const LNK_MAGIC: [u8; 4] = [0x4C, 0x00, 0x00, 0x00]; // 'L' followed by three NULLs

/// LNK CLSID (identifies shell link)
const LNK_CLSID: [u8; 16] = [
    0x01, 0x14, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00,
    0xC0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x46,
];

/// Link flags
#[derive(Debug, Clone, Copy)]
struct LinkFlags(u32);

impl LinkFlags {
    fn has_link_target_id_list(&self) -> bool { self.0 & 0x01 != 0 }
    fn has_link_info(&self) -> bool { self.0 & 0x02 != 0 }
    fn has_name(&self) -> bool { self.0 & 0x04 != 0 }
    fn has_relative_path(&self) -> bool { self.0 & 0x08 != 0 }
    fn has_working_dir(&self) -> bool { self.0 & 0x10 != 0 }
    fn has_arguments(&self) -> bool { self.0 & 0x20 != 0 }
    fn has_icon_location(&self) -> bool { self.0 & 0x40 != 0 }
    fn is_unicode(&self) -> bool { self.0 & 0x80 != 0 }
}

/// Parsed LNK file information
#[derive(Debug, Default)]
pub struct LnkInfo {
    pub target_path: Option<String>,
    pub working_directory: Option<String>,
    pub arguments: Option<String>,
    pub icon_location: Option<String>,
    pub description: Option<String>,
    pub relative_path: Option<String>,
    pub file_attributes: u32,
    pub creation_time: u64,
    pub access_time: u64,
    pub write_time: u64,
    pub file_size: u32,
}

/// Parse a LNK file
pub fn parse_lnk(path: &Path) -> FileHandlerResult<LnkInfo> {
    use std::io::{Read, Seek, SeekFrom, Cursor};
    use std::fs::File;

    let mut file = File::open(path)?;
    let mut data = Vec::new();
    file.read_to_end(&mut data)?;

    if data.len() < 76 {
        return Err(FileHandlerError::Parse("File too small to be a valid LNK".to_string()));
    }

    // Verify magic
    if data[0..4] != LNK_MAGIC {
        return Err(FileHandlerError::Parse("Invalid LNK magic number".to_string()));
    }

    // Verify CLSID
    if data[4..20] != LNK_CLSID {
        return Err(FileHandlerError::Parse("Invalid LNK CLSID".to_string()));
    }

    let mut info = LnkInfo::default();

    // Read link flags
    let flags = LinkFlags(u32::from_le_bytes([data[20], data[21], data[22], data[23]]));

    // Read file attributes
    info.file_attributes = u32::from_le_bytes([data[24], data[25], data[26], data[27]]);

    // Read timestamps
    info.creation_time = u64::from_le_bytes([
        data[28], data[29], data[30], data[31],
        data[32], data[33], data[34], data[35],
    ]);
    info.access_time = u64::from_le_bytes([
        data[36], data[37], data[38], data[39],
        data[40], data[41], data[42], data[43],
    ]);
    info.write_time = u64::from_le_bytes([
        data[44], data[45], data[46], data[47],
        data[48], data[49], data[50], data[51],
    ]);

    // Read file size
    info.file_size = u32::from_le_bytes([data[52], data[53], data[54], data[55]]);

    // Current position in file
    let mut pos: usize = 76; // Shell Link Header size

    // Skip Link Target ID List if present
    if flags.has_link_target_id_list() && pos + 2 <= data.len() {
        let list_size = u16::from_le_bytes([data[pos], data[pos + 1]]) as usize;
        pos += 2 + list_size;
    }

    // Parse Link Info if present
    if flags.has_link_info() && pos + 4 <= data.len() {
        let link_info_size = u32::from_le_bytes([
            data[pos], data[pos + 1], data[pos + 2], data[pos + 3]
        ]) as usize;

        if pos + link_info_size <= data.len() && link_info_size >= 28 {
            let link_info = &data[pos..pos + link_info_size];

            // Local base path offset
            let local_base_path_offset = u32::from_le_bytes([
                link_info[16], link_info[17], link_info[18], link_info[19]
            ]) as usize;

            if local_base_path_offset > 0 && local_base_path_offset < link_info_size {
                info.target_path = read_null_terminated_string(&link_info[local_base_path_offset..]);
            }
        }

        pos += link_info_size;
    }

    // Parse string data
    let is_unicode = flags.is_unicode();

    // Read description/name
    if flags.has_name() {
        if let Some((s, new_pos)) = read_string_data(&data, pos, is_unicode) {
            info.description = Some(s);
            pos = new_pos;
        }
    }

    // Read relative path
    if flags.has_relative_path() {
        if let Some((s, new_pos)) = read_string_data(&data, pos, is_unicode) {
            info.relative_path = Some(s);
            pos = new_pos;
        }
    }

    // Read working directory
    if flags.has_working_dir() {
        if let Some((s, new_pos)) = read_string_data(&data, pos, is_unicode) {
            info.working_directory = Some(s);
            pos = new_pos;
        }
    }

    // Read arguments
    if flags.has_arguments() {
        if let Some((s, new_pos)) = read_string_data(&data, pos, is_unicode) {
            info.arguments = Some(s);
            pos = new_pos;
        }
    }

    // Read icon location
    if flags.has_icon_location() {
        if let Some((s, _)) = read_string_data(&data, pos, is_unicode) {
            info.icon_location = Some(s);
        }
    }

    Ok(info)
}

/// Read a null-terminated string from bytes
fn read_null_terminated_string(data: &[u8]) -> Option<String> {
    let end = data.iter().position(|&b| b == 0)?;
    String::from_utf8(data[..end].to_vec()).ok()
}

/// Read a string data structure (length-prefixed)
fn read_string_data(data: &[u8], pos: usize, is_unicode: bool) -> Option<(String, usize)> {
    if pos + 2 > data.len() {
        return None;
    }

    let char_count = u16::from_le_bytes([data[pos], data[pos + 1]]) as usize;
    let byte_count = if is_unicode { char_count * 2 } else { char_count };

    if pos + 2 + byte_count > data.len() {
        return None;
    }

    let string_data = &data[pos + 2..pos + 2 + byte_count];

    let string = if is_unicode {
        // Convert from UTF-16LE
        let u16_chars: Vec<u16> = string_data
            .chunks(2)
            .filter_map(|chunk| {
                if chunk.len() == 2 {
                    Some(u16::from_le_bytes([chunk[0], chunk[1]]))
                } else {
                    None
                }
            })
            .collect();
        String::from_utf16(&u16_chars).ok()?
    } else {
        String::from_utf8(string_data.to_vec()).ok()?
    };

    Some((string, pos + 2 + byte_count))
}

/// Get information about a LNK file
pub fn get_lnk_info(path: &Path) -> FileHandlerResult<FileInfo> {
    let mut info = FileInfo::new(path)?.with_type("Windows Shortcut");

    match parse_lnk(path) {
        Ok(lnk) => {
            if let Some(target) = &lnk.target_path {
                info.add_property("Target", target);
            }
            if let Some(relative) = &lnk.relative_path {
                info.add_property("Relative Path", relative);
            }
            if let Some(working_dir) = &lnk.working_directory {
                info.add_property("Working Directory", working_dir);
            }
            if let Some(arguments) = &lnk.arguments {
                info.add_property("Arguments", arguments);
            }
            if let Some(icon) = &lnk.icon_location {
                info.add_property("Icon Location", icon);
            }
            if let Some(desc) = &lnk.description {
                info.add_property("Description", desc);
            }
            if lnk.file_size > 0 {
                info.add_property("Target Size", &format!("{} bytes", lnk.file_size));
            }
        }
        Err(e) => {
            info.add_property("Parse Error", &e.to_string());
        }
    }

    Ok(info)
}

/// Follow the link and return the target path
pub fn follow_link(path: &Path) -> FileHandlerResult<String> {
    let lnk = parse_lnk(path)?;

    // Try target path first, then relative path
    if let Some(target) = lnk.target_path {
        return Ok(target);
    }

    if let Some(relative) = lnk.relative_path {
        // Resolve relative to the LNK file's directory
        if let Some(parent) = path.parent() {
            let resolved = parent.join(&relative);
            return Ok(resolved.to_string_lossy().to_string());
        }
        return Ok(relative);
    }

    Err(FileHandlerError::NotFound("No target path found in shortcut".to_string()))
}
