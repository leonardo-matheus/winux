// Terminal Executor - Execute commands and capture output

use anyhow::{anyhow, Result};
use std::process::{Command, Output, Stdio};
use std::time::Duration;
use tokio::process::Command as AsyncCommand;

pub struct TerminalExecutor;

impl TerminalExecutor {
    /// Execute a command synchronously
    pub fn execute(command: &str) -> Result<CommandResult> {
        let output = if cfg!(target_os = "windows") {
            Command::new("cmd")
                .args(["/C", command])
                .output()
        } else {
            Command::new("sh")
                .args(["-c", command])
                .output()
        };

        match output {
            Ok(output) => Ok(Self::parse_output(output)),
            Err(e) => Err(anyhow!("Failed to execute command: {}", e)),
        }
    }

    /// Execute a command asynchronously
    pub async fn execute_async(command: &str) -> Result<CommandResult> {
        let output = if cfg!(target_os = "windows") {
            AsyncCommand::new("cmd")
                .args(["/C", command])
                .output()
                .await
        } else {
            AsyncCommand::new("sh")
                .args(["-c", command])
                .output()
                .await
        };

        match output {
            Ok(output) => Ok(Self::parse_output(output)),
            Err(e) => Err(anyhow!("Failed to execute command: {}", e)),
        }
    }

    /// Execute a command with timeout
    pub async fn execute_with_timeout(command: &str, timeout_secs: u64) -> Result<CommandResult> {
        let result = tokio::time::timeout(
            Duration::from_secs(timeout_secs),
            Self::execute_async(command),
        )
        .await;

        match result {
            Ok(inner_result) => inner_result,
            Err(_) => Err(anyhow!("Command timed out after {} seconds", timeout_secs)),
        }
    }

    /// Execute a command and stream output
    pub async fn execute_streaming(
        command: &str,
        on_stdout: impl Fn(&str) + Send + 'static,
        on_stderr: impl Fn(&str) + Send + 'static,
    ) -> Result<i32> {
        use tokio::io::{AsyncBufReadExt, BufReader};

        let mut child = if cfg!(target_os = "windows") {
            AsyncCommand::new("cmd")
                .args(["/C", command])
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()?
        } else {
            AsyncCommand::new("sh")
                .args(["-c", command])
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()?
        };

        let stdout = child.stdout.take().expect("Failed to capture stdout");
        let stderr = child.stderr.take().expect("Failed to capture stderr");

        let stdout_handle = tokio::spawn(async move {
            let reader = BufReader::new(stdout);
            let mut lines = reader.lines();
            while let Ok(Some(line)) = lines.next_line().await {
                on_stdout(&line);
            }
        });

        let stderr_handle = tokio::spawn(async move {
            let reader = BufReader::new(stderr);
            let mut lines = reader.lines();
            while let Ok(Some(line)) = lines.next_line().await {
                on_stderr(&line);
            }
        });

        let status = child.wait().await?;
        let _ = stdout_handle.await;
        let _ = stderr_handle.await;

        Ok(status.code().unwrap_or(-1))
    }

    /// Check if a command exists
    pub fn command_exists(command: &str) -> bool {
        let result = if cfg!(target_os = "windows") {
            Command::new("where")
                .arg(command)
                .output()
        } else {
            Command::new("which")
                .arg(command)
                .output()
        };

        result.map(|o| o.status.success()).unwrap_or(false)
    }

    /// Get current shell
    pub fn get_shell() -> String {
        std::env::var("SHELL").unwrap_or_else(|_| {
            if cfg!(target_os = "windows") {
                "cmd.exe".to_string()
            } else {
                "/bin/sh".to_string()
            }
        })
    }

    fn parse_output(output: Output) -> CommandResult {
        CommandResult {
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            exit_code: output.status.code().unwrap_or(-1),
            success: output.status.success(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CommandResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub success: bool,
}

impl CommandResult {
    /// Get combined output (stdout + stderr)
    pub fn combined_output(&self) -> String {
        let mut output = self.stdout.clone();
        if !self.stderr.is_empty() {
            if !output.is_empty() {
                output.push_str("\n");
            }
            output.push_str(&self.stderr);
        }
        output
    }

    /// Format as display string
    pub fn to_display_string(&self) -> String {
        let mut result = String::new();

        if !self.stdout.is_empty() {
            result.push_str(&self.stdout);
        }

        if !self.stderr.is_empty() {
            if !result.is_empty() {
                result.push_str("\n\n");
            }
            result.push_str("STDERR:\n");
            result.push_str(&self.stderr);
        }

        if !self.success {
            result.push_str(&format!("\n\nExit code: {}", self.exit_code));
        }

        result
    }
}

/// Validate command safety (basic checks)
pub fn is_safe_command(command: &str) -> (bool, Option<String>) {
    let dangerous_patterns = [
        ("rm -rf /", "This would delete the entire filesystem"),
        ("rm -rf /*", "This would delete all files"),
        (":(){:|:&};:", "Fork bomb - would crash the system"),
        ("dd if=/dev/zero of=/dev/sda", "This would destroy the disk"),
        ("mkfs.", "This would format a filesystem"),
        ("> /dev/sda", "This would destroy the disk"),
        ("chmod -R 777 /", "This would make all files world-writable"),
        ("wget -O- | sh", "Piping remote scripts to shell is dangerous"),
        ("curl | sh", "Piping remote scripts to shell is dangerous"),
    ];

    let command_lower = command.to_lowercase();

    for (pattern, warning) in dangerous_patterns {
        if command_lower.contains(pattern) {
            return (false, Some(warning.to_string()));
        }
    }

    // Check for sudo with dangerous commands
    if command_lower.contains("sudo") {
        return (true, Some("Command uses sudo - will require elevated privileges".to_string()));
    }

    (true, None)
}
