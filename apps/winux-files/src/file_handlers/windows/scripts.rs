//! Windows script file handlers (.bat, .cmd, .ps1)
//!
//! Provides syntax highlighting information and script analysis
//! for Windows batch and PowerShell scripts.

use crate::file_handlers::common::{FileHandlerError, FileHandlerResult, FileInfo};
use std::fs;
use std::path::Path;

/// Script type
#[derive(Debug, Clone, Copy)]
pub enum ScriptType {
    Batch,      // .bat, .cmd
    PowerShell, // .ps1
}

/// Parsed script information
#[derive(Debug)]
pub struct ScriptInfo {
    pub script_type: ScriptType,
    pub line_count: usize,
    pub commands: Vec<String>,
    pub variables: Vec<String>,
    pub comments: usize,
    pub has_admin_requirement: bool,
    pub calls_external: Vec<String>,
}

/// Analyze a batch script
pub fn analyze_batch(content: &str) -> ScriptInfo {
    let lines: Vec<&str> = content.lines().collect();
    let mut commands = Vec::new();
    let mut variables = Vec::new();
    let mut comments = 0;
    let mut has_admin = false;
    let mut external_calls = Vec::new();

    // Common batch commands
    let batch_commands = [
        "echo", "set", "if", "for", "goto", "call", "start", "exit",
        "cd", "dir", "copy", "move", "del", "mkdir", "rmdir", "type",
        "ren", "rename", "cls", "pause", "rem", "setlocal", "endlocal",
        "pushd", "popd", "title", "color", "prompt", "path", "assoc",
        "ftype", "reg", "net", "sc", "tasklist", "taskkill", "shutdown",
    ];

    for line in &lines {
        let trimmed = line.trim().to_lowercase();

        // Count comments (REM and ::)
        if trimmed.starts_with("rem ") || trimmed.starts_with("::") {
            comments += 1;
            continue;
        }

        // Skip empty lines
        if trimmed.is_empty() {
            continue;
        }

        // Check for admin requirement
        if trimmed.contains("net session") ||
           trimmed.contains("runas") ||
           trimmed.contains("administrator") {
            has_admin = true;
        }

        // Extract variables (%var% or !var!)
        let mut chars: Vec<char> = line.chars().collect();
        let mut i = 0;
        while i < chars.len() {
            if chars[i] == '%' || chars[i] == '!' {
                let delimiter = chars[i];
                let start = i + 1;
                i += 1;
                while i < chars.len() && chars[i] != delimiter {
                    i += 1;
                }
                if i < chars.len() && start < i {
                    let var: String = chars[start..i].iter().collect();
                    if !var.is_empty() && !var.chars().all(|c| c.is_numeric()) {
                        if !variables.contains(&var) {
                            variables.push(var);
                        }
                    }
                }
            }
            i += 1;
        }

        // Extract first command
        let first_word = trimmed
            .split_whitespace()
            .next()
            .unwrap_or("")
            .trim_start_matches('@');

        if batch_commands.contains(&first_word) {
            if !commands.contains(&first_word.to_string()) {
                commands.push(first_word.to_string());
            }
        } else if !first_word.is_empty() &&
                  !first_word.starts_with(':') &&
                  !first_word.starts_with('%') {
            // Potential external command
            if !external_calls.contains(&first_word.to_string()) {
                external_calls.push(first_word.to_string());
            }
        }
    }

    ScriptInfo {
        script_type: ScriptType::Batch,
        line_count: lines.len(),
        commands,
        variables,
        comments,
        has_admin_requirement: has_admin,
        calls_external: external_calls,
    }
}

/// Analyze a PowerShell script
pub fn analyze_powershell(content: &str) -> ScriptInfo {
    let lines: Vec<&str> = content.lines().collect();
    let mut commands = Vec::new();
    let mut variables = Vec::new();
    let mut comments = 0;
    let mut has_admin = false;
    let mut external_calls = Vec::new();

    // Common PowerShell cmdlets
    let ps_cmdlets = [
        "get-", "set-", "new-", "remove-", "add-", "import-", "export-",
        "invoke-", "start-", "stop-", "write-", "read-", "out-", "format-",
        "select-", "where-", "foreach-", "sort-", "group-", "measure-",
        "test-", "copy-", "move-", "rename-", "clear-", "split-", "join-",
    ];

    for line in &lines {
        let trimmed = line.trim().to_lowercase();

        // Count comments
        if trimmed.starts_with('#') {
            comments += 1;
            continue;
        }

        // Count block comments
        if trimmed.starts_with("<#") {
            comments += 1;
            continue;
        }

        // Skip empty lines
        if trimmed.is_empty() {
            continue;
        }

        // Check for admin requirement
        if trimmed.contains("#requires -runas") ||
           trimmed.contains("runas /user:administrator") ||
           trimmed.contains("start-process") && trimmed.contains("-verb runas") {
            has_admin = true;
        }

        // Extract variables ($var)
        for word in line.split(|c: char| !c.is_alphanumeric() && c != '$' && c != '_') {
            if word.starts_with('$') && word.len() > 1 {
                let var = word[1..].to_string();
                if !var.is_empty() && !variables.contains(&var) {
                    variables.push(var);
                }
            }
        }

        // Extract cmdlets
        for word in trimmed.split_whitespace() {
            let word = word.trim_start_matches('(').trim_end_matches(')');
            for prefix in &ps_cmdlets {
                if word.starts_with(prefix) {
                    if !commands.contains(&word.to_string()) {
                        commands.push(word.to_string());
                    }
                    break;
                }
            }

            // Check for external calls
            if word.ends_with(".exe") || word.ends_with(".bat") || word.ends_with(".cmd") {
                if !external_calls.contains(&word.to_string()) {
                    external_calls.push(word.to_string());
                }
            }
        }
    }

    ScriptInfo {
        script_type: ScriptType::PowerShell,
        line_count: lines.len(),
        commands,
        variables,
        comments,
        has_admin_requirement: has_admin,
        calls_external: external_calls,
    }
}

/// Get information about a script file
pub fn get_script_info(path: &Path) -> FileHandlerResult<FileInfo> {
    let extension = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .unwrap_or_default();

    let content = fs::read_to_string(path)?;

    let (file_type, script_info) = match extension.as_str() {
        "bat" | "cmd" => ("Windows Batch Script", analyze_batch(&content)),
        "ps1" => ("PowerShell Script", analyze_powershell(&content)),
        _ => return Err(FileHandlerError::NotSupported("Unknown script type".to_string())),
    };

    let mut info = FileInfo::new(path)?.with_type(file_type);

    info.add_property("Lines", &script_info.line_count.to_string());
    info.add_property("Comments", &script_info.comments.to_string());

    if !script_info.commands.is_empty() {
        let commands_preview: Vec<&str> = script_info.commands.iter()
            .take(10)
            .map(|s| s.as_str())
            .collect();
        info.add_property("Commands Used", &commands_preview.join(", "));
    }

    if !script_info.variables.is_empty() {
        let vars_preview: Vec<&str> = script_info.variables.iter()
            .take(10)
            .map(|s| s.as_str())
            .collect();
        info.add_property("Variables", &vars_preview.join(", "));
    }

    if script_info.has_admin_requirement {
        info.add_property("Requires Admin", "Yes");
    }

    if !script_info.calls_external.is_empty() {
        let external: Vec<&str> = script_info.calls_external.iter()
            .take(5)
            .map(|s| s.as_str())
            .collect();
        info.add_property("External Calls", &external.join(", "));
    }

    Ok(info)
}

/// Get syntax highlighting tokens for batch script
pub fn get_batch_tokens(content: &str) -> Vec<(usize, usize, &'static str)> {
    let mut tokens = Vec::new();
    let mut pos = 0;

    let keywords = [
        "echo", "set", "if", "else", "for", "in", "do", "goto", "call",
        "exit", "rem", "setlocal", "endlocal", "not", "exist", "defined",
        "equ", "neq", "lss", "leq", "gtr", "geq", "on", "off",
    ];

    for line in content.lines() {
        let line_lower = line.to_lowercase();

        // Check for comments
        if line.trim().starts_with("rem ") || line.trim().starts_with("REM ") {
            tokens.push((pos, pos + line.len(), "comment"));
        } else if line.trim().starts_with("::") {
            tokens.push((pos, pos + line.len(), "comment"));
        } else {
            // Check for keywords
            for keyword in &keywords {
                if let Some(kw_pos) = line_lower.find(keyword) {
                    let end = kw_pos + keyword.len();
                    // Verify it's a whole word
                    let before_ok = kw_pos == 0 ||
                        !line.chars().nth(kw_pos - 1).unwrap_or(' ').is_alphanumeric();
                    let after_ok = end >= line.len() ||
                        !line.chars().nth(end).unwrap_or(' ').is_alphanumeric();
                    if before_ok && after_ok {
                        tokens.push((pos + kw_pos, pos + end, "keyword"));
                    }
                }
            }

            // Check for variables
            let chars: Vec<char> = line.chars().collect();
            let mut i = 0;
            while i < chars.len() {
                if chars[i] == '%' {
                    let start = i;
                    i += 1;
                    while i < chars.len() && chars[i] != '%' && chars[i] != ' ' {
                        i += 1;
                    }
                    if i < chars.len() && chars[i] == '%' {
                        tokens.push((pos + start, pos + i + 1, "variable"));
                    }
                }
                i += 1;
            }

            // Check for labels
            if line.trim().starts_with(':') && !line.trim().starts_with("::") {
                tokens.push((pos, pos + line.len(), "label"));
            }

            // Check for strings
            let mut in_string = false;
            let mut string_start = 0;
            for (i, c) in line.chars().enumerate() {
                if c == '"' {
                    if in_string {
                        tokens.push((pos + string_start, pos + i + 1, "string"));
                        in_string = false;
                    } else {
                        string_start = i;
                        in_string = true;
                    }
                }
            }
        }

        pos += line.len() + 1; // +1 for newline
    }

    tokens
}

/// Get syntax highlighting tokens for PowerShell script
pub fn get_powershell_tokens(content: &str) -> Vec<(usize, usize, &'static str)> {
    let mut tokens = Vec::new();
    let mut pos = 0;

    let keywords = [
        "if", "else", "elseif", "switch", "while", "for", "foreach", "do",
        "until", "break", "continue", "return", "exit", "throw", "try",
        "catch", "finally", "trap", "param", "function", "filter", "begin",
        "process", "end", "class", "enum", "using", "in", "hidden",
    ];

    for line in content.lines() {
        let line_lower = line.to_lowercase();

        // Check for comments
        if line.trim().starts_with('#') && !line.trim().starts_with("#requires") {
            tokens.push((pos, pos + line.len(), "comment"));
        } else {
            // Check for keywords
            for keyword in &keywords {
                if let Some(kw_pos) = line_lower.find(keyword) {
                    let end = kw_pos + keyword.len();
                    let before_ok = kw_pos == 0 ||
                        !line.chars().nth(kw_pos - 1).unwrap_or(' ').is_alphanumeric();
                    let after_ok = end >= line.len() ||
                        !line.chars().nth(end).unwrap_or(' ').is_alphanumeric();
                    if before_ok && after_ok {
                        tokens.push((pos + kw_pos, pos + end, "keyword"));
                    }
                }
            }

            // Check for variables ($var)
            let chars: Vec<char> = line.chars().collect();
            let mut i = 0;
            while i < chars.len() {
                if chars[i] == '$' {
                    let start = i;
                    i += 1;
                    while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                        i += 1;
                    }
                    if i > start + 1 {
                        tokens.push((pos + start, pos + i, "variable"));
                    }
                } else {
                    i += 1;
                }
            }

            // Check for cmdlets (Verb-Noun pattern)
            for word in line.split_whitespace() {
                if word.contains('-') && word.chars().next().map_or(false, |c| c.is_alphabetic()) {
                    if let Some(word_pos) = line.find(word) {
                        tokens.push((pos + word_pos, pos + word_pos + word.len(), "cmdlet"));
                    }
                }
            }

            // Check for strings
            let mut in_string = false;
            let mut string_char = '"';
            let mut string_start = 0;
            for (i, c) in line.chars().enumerate() {
                if !in_string && (c == '"' || c == '\'') {
                    string_start = i;
                    string_char = c;
                    in_string = true;
                } else if in_string && c == string_char {
                    tokens.push((pos + string_start, pos + i + 1, "string"));
                    in_string = false;
                }
            }
        }

        pos += line.len() + 1;
    }

    tokens
}
