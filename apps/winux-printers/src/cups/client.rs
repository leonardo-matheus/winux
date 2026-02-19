//! CUPS client for printer management
//!
//! Handles communication with CUPS server via command-line tools
//! (lpstat, lpadmin, lp, cancel, etc.) or IPP protocol.

use std::collections::HashMap;
use std::process::Command;

use anyhow::{Result, Context};
use tracing::{info, warn, error};

use super::{Printer, PrinterStatus, PrintJob, JobStatus, DiscoveredPrinter, PrintOptions};

/// CUPS server manager
pub struct CupsManager {
    /// CUPS server address
    server: String,
    /// Cached printers list
    printers: Vec<Printer>,
    /// Cached jobs list
    jobs: Vec<PrintJob>,
    /// Default printer name
    default_printer: Option<String>,
}

impl CupsManager {
    /// Create a new CUPS manager connecting to localhost
    pub fn new() -> Self {
        Self::with_server("localhost:631".to_string())
    }

    /// Create a new CUPS manager with custom server
    pub fn with_server(server: String) -> Self {
        let mut manager = Self {
            server,
            printers: Vec::new(),
            jobs: Vec::new(),
            default_printer: None,
        };
        // Initial refresh
        let _ = manager.refresh_printers();
        let _ = manager.refresh_jobs();
        manager
    }

    /// Get the CUPS server address
    pub fn server(&self) -> &str {
        &self.server
    }

    /// Refresh the list of printers from CUPS
    pub fn refresh_printers(&mut self) -> Result<()> {
        info!("Refreshing printers list from CUPS");

        // Use lpstat -p to get printers
        let output = Command::new("lpstat")
            .args(["-p", "-d"])
            .output()
            .context("Failed to execute lpstat")?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            self.parse_lpstat_output(&stdout);
        } else {
            warn!("lpstat command failed, using sample data");
            // In case CUPS is not available, we keep existing data
        }

        Ok(())
    }

    /// Parse lpstat output to populate printers
    fn parse_lpstat_output(&mut self, output: &str) {
        self.printers.clear();
        self.default_printer = None;

        for line in output.lines() {
            // Parse printer lines: "printer HP-LaserJet is idle."
            if line.starts_with("printer ") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 3 {
                    let name = parts[1].to_string();
                    let status = if line.contains("idle") {
                        PrinterStatus::Ready
                    } else if line.contains("printing") {
                        PrinterStatus::Printing
                    } else if line.contains("disabled") {
                        PrinterStatus::Paused
                    } else {
                        PrinterStatus::Offline
                    };

                    let printer = Printer::new(
                        &name,
                        &name,
                        &format!("ipp://localhost:631/printers/{}", name),
                        status,
                        !line.contains("disabled"),
                        false,
                    );
                    self.printers.push(printer);
                }
            }
            // Parse default printer: "system default destination: HP-LaserJet"
            else if line.starts_with("system default destination:") {
                if let Some(name) = line.split(':').nth(1) {
                    let default_name = name.trim().to_string();
                    self.default_printer = Some(default_name.clone());

                    // Mark the default printer
                    for printer in &mut self.printers {
                        if printer.name == default_name {
                            printer.is_default = true;
                        }
                    }
                }
            }
        }
    }

    /// Get list of configured printers
    pub fn printers(&self) -> &[Printer] {
        &self.printers
    }

    /// Get the default printer
    pub fn default_printer(&self) -> Option<&Printer> {
        self.printers.iter().find(|p| p.is_default)
    }

    /// Set the default printer
    pub fn set_default_printer(&mut self, name: &str) -> Result<()> {
        info!("Setting default printer to: {}", name);

        let output = Command::new("lpadmin")
            .args(["-d", name])
            .output()
            .context("Failed to execute lpadmin")?;

        if output.status.success() {
            // Update local state
            for printer in &mut self.printers {
                printer.is_default = printer.name == name;
            }
            self.default_printer = Some(name.to_string());
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Failed to set default printer: {}", stderr)
        }
    }

    /// Enable or disable a printer
    pub fn set_printer_enabled(&mut self, name: &str, enabled: bool) -> Result<()> {
        info!("Setting printer {} enabled: {}", name, enabled);

        let cmd = if enabled { "cupsenable" } else { "cupsdisable" };
        let output = Command::new(cmd)
            .arg(name)
            .output()
            .context(format!("Failed to execute {}", cmd))?;

        if output.status.success() {
            // Update local state
            if let Some(printer) = self.printers.iter_mut().find(|p| p.name == name) {
                printer.enabled = enabled;
                if !enabled {
                    printer.status = PrinterStatus::Paused;
                } else {
                    printer.status = PrinterStatus::Ready;
                }
            }
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Failed to change printer state: {}", stderr)
        }
    }

    /// Add a new printer
    pub fn add_printer(
        &mut self,
        name: &str,
        uri: &str,
        description: &str,
        ppd: Option<&str>,
    ) -> Result<()> {
        info!("Adding printer: {} at {}", name, uri);

        let mut args = vec![
            "-p".to_string(),
            name.to_string(),
            "-v".to_string(),
            uri.to_string(),
            "-D".to_string(),
            description.to_string(),
            "-E".to_string(), // Enable the printer
        ];

        if let Some(ppd_file) = ppd {
            args.push("-P".to_string());
            args.push(ppd_file.to_string());
        } else {
            // Use driverless/everywhere if no PPD specified
            args.push("-m".to_string());
            args.push("everywhere".to_string());
        }

        let output = Command::new("lpadmin")
            .args(&args)
            .output()
            .context("Failed to execute lpadmin")?;

        if output.status.success() {
            // Refresh printer list
            self.refresh_printers()?;
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Failed to add printer: {}", stderr)
        }
    }

    /// Remove a printer
    pub fn remove_printer(&mut self, name: &str) -> Result<()> {
        info!("Removing printer: {}", name);

        let output = Command::new("lpadmin")
            .args(["-x", name])
            .output()
            .context("Failed to execute lpadmin")?;

        if output.status.success() {
            self.printers.retain(|p| p.name != name);
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Failed to remove printer: {}", stderr)
        }
    }

    /// Discover available printers on the network
    pub fn discover_printers(&self) -> Result<Vec<DiscoveredPrinter>> {
        info!("Discovering printers on network");

        // Use lpinfo to discover network printers
        let output = Command::new("lpinfo")
            .args(["--include-schemes", "dnssd,ipp,ipps,socket", "-v"])
            .output()
            .context("Failed to execute lpinfo")?;

        let mut discovered = Vec::new();

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                // Parse lines like: "network dnssd://Printer%20Name._ipp._tcp.local"
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 && parts[0] == "network" {
                    let uri = parts[1];
                    if let Some(printer) = DiscoveredPrinter::from_uri(uri) {
                        discovered.push(printer);
                    }
                }
            }
        }

        Ok(discovered)
    }

    // ==================== Job Management ====================

    /// Refresh the list of print jobs
    pub fn refresh_jobs(&mut self) -> Result<()> {
        info!("Refreshing jobs list from CUPS");

        let output = Command::new("lpstat")
            .args(["-o", "-W", "not-completed"])
            .output()
            .context("Failed to execute lpstat")?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            self.parse_jobs_output(&stdout);
        }

        Ok(())
    }

    /// Parse lpstat -o output to populate jobs
    fn parse_jobs_output(&mut self, output: &str) {
        self.jobs.clear();

        for line in output.lines() {
            // Parse job lines: "HP-LaserJet-123 user 1024 Mon Jan 1 12:00:00 2026"
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 4 {
                // Extract job ID from "PRINTER-123" format
                if let Some(job_id_str) = parts[0].split('-').last() {
                    if let Ok(job_id) = job_id_str.parse::<u32>() {
                        let printer = parts[0]
                            .rsplit_once('-')
                            .map(|(p, _)| p)
                            .unwrap_or(parts[0])
                            .to_string();
                        let user = parts[1].to_string();
                        let size: u64 = parts[2].parse().unwrap_or(0);

                        let job = PrintJob::new(
                            job_id,
                            "Document",
                            &printer,
                            &user,
                            JobStatus::Pending,
                            1,
                            size,
                            chrono::Utc::now(),
                        );
                        self.jobs.push(job);
                    }
                }
            }
        }
    }

    /// Get list of print jobs
    pub fn jobs(&self) -> &[PrintJob] {
        &self.jobs
    }

    /// Print a file
    pub fn print_file(
        &self,
        file_path: &str,
        printer: Option<&str>,
        options: &PrintOptions,
    ) -> Result<u32> {
        info!("Printing file: {} to {:?}", file_path, printer);

        let mut args = vec![file_path.to_string()];

        if let Some(p) = printer {
            args.push("-d".to_string());
            args.push(p.to_string());
        }

        // Add print options
        for (key, value) in options.to_lp_options() {
            args.push("-o".to_string());
            args.push(format!("{}={}", key, value));
        }

        if options.copies > 1 {
            args.push("-n".to_string());
            args.push(options.copies.to_string());
        }

        let output = Command::new("lp")
            .args(&args)
            .output()
            .context("Failed to execute lp")?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            // Parse job ID from output like "request id is HP-LaserJet-123"
            if let Some(id) = stdout.split('-').last() {
                if let Some(id_clean) = id.split_whitespace().next() {
                    if let Ok(job_id) = id_clean.parse::<u32>() {
                        return Ok(job_id);
                    }
                }
            }
            Ok(0) // Unknown job ID
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Failed to print: {}", stderr)
        }
    }

    /// Print a test page
    pub fn print_test_page(&self, printer: Option<&str>) -> Result<u32> {
        let printer_name = printer
            .or(self.default_printer.as_deref())
            .ok_or_else(|| anyhow::anyhow!("No printer specified and no default printer set"))?;

        info!("Printing test page to: {}", printer_name);

        // CUPS provides a test page at this path
        let test_page = "/usr/share/cups/data/testprint";

        let output = Command::new("lp")
            .args(["-d", printer_name, test_page])
            .output()
            .context("Failed to execute lp")?;

        if output.status.success() {
            Ok(0)
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Failed to print test page: {}", stderr)
        }
    }

    /// Cancel a print job
    pub fn cancel_job(&mut self, job_id: u32) -> Result<()> {
        info!("Cancelling job: {}", job_id);

        let output = Command::new("cancel")
            .arg(job_id.to_string())
            .output()
            .context("Failed to execute cancel")?;

        if output.status.success() {
            if let Some(job) = self.jobs.iter_mut().find(|j| j.id == job_id) {
                job.status = JobStatus::Cancelled;
            }
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Failed to cancel job: {}", stderr)
        }
    }

    /// Cancel all jobs for a printer
    pub fn cancel_all_jobs(&mut self, printer: Option<&str>) -> Result<()> {
        info!("Cancelling all jobs for: {:?}", printer);

        let mut args = vec!["-a".to_string()];
        if let Some(p) = printer {
            args.push(p.to_string());
        }

        let output = Command::new("cancel")
            .args(&args)
            .output()
            .context("Failed to execute cancel")?;

        if output.status.success() {
            if let Some(p) = printer {
                self.jobs.retain(|j| j.printer != p);
            } else {
                self.jobs.clear();
            }
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Failed to cancel jobs: {}", stderr)
        }
    }

    /// Hold a print job
    pub fn hold_job(&mut self, job_id: u32) -> Result<()> {
        info!("Holding job: {}", job_id);

        let output = Command::new("lp")
            .args(["-i", &job_id.to_string(), "-H", "hold"])
            .output()
            .context("Failed to execute lp")?;

        if output.status.success() {
            if let Some(job) = self.jobs.iter_mut().find(|j| j.id == job_id) {
                job.status = JobStatus::Held;
            }
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Failed to hold job: {}", stderr)
        }
    }

    /// Release a held print job
    pub fn release_job(&mut self, job_id: u32) -> Result<()> {
        info!("Releasing job: {}", job_id);

        let output = Command::new("lp")
            .args(["-i", &job_id.to_string(), "-H", "resume"])
            .output()
            .context("Failed to execute lp")?;

        if output.status.success() {
            if let Some(job) = self.jobs.iter_mut().find(|j| j.id == job_id) {
                job.status = JobStatus::Pending;
            }
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Failed to release job: {}", stderr)
        }
    }

    /// Move a job to a different printer
    pub fn move_job(&mut self, job_id: u32, new_printer: &str) -> Result<()> {
        info!("Moving job {} to printer: {}", job_id, new_printer);

        let output = Command::new("lpmove")
            .args([&job_id.to_string(), new_printer])
            .output()
            .context("Failed to execute lpmove")?;

        if output.status.success() {
            if let Some(job) = self.jobs.iter_mut().find(|j| j.id == job_id) {
                job.printer = new_printer.to_string();
            }
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Failed to move job: {}", stderr)
        }
    }

    /// Get printer options/capabilities
    pub fn get_printer_options(&self, printer: &str) -> Result<HashMap<String, Vec<String>>> {
        info!("Getting options for printer: {}", printer);

        let output = Command::new("lpoptions")
            .args(["-p", printer, "-l"])
            .output()
            .context("Failed to execute lpoptions")?;

        let mut options = HashMap::new();

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                // Parse lines like: "PageSize/Media Size: Letter *A4 Legal"
                if let Some((key_part, values_part)) = line.split_once(':') {
                    let key = key_part.split('/').next().unwrap_or(key_part).trim();
                    let values: Vec<String> = values_part
                        .split_whitespace()
                        .map(|s| s.trim_start_matches('*').to_string())
                        .collect();
                    options.insert(key.to_string(), values);
                }
            }
        }

        Ok(options)
    }

    /// Set printer options
    pub fn set_printer_option(&self, printer: &str, option: &str, value: &str) -> Result<()> {
        info!("Setting option for {}: {}={}", printer, option, value);

        let output = Command::new("lpadmin")
            .args(["-p", printer, "-o", &format!("{}={}", option, value)])
            .output()
            .context("Failed to execute lpadmin")?;

        if output.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Failed to set option: {}", stderr)
        }
    }

    /// Check if CUPS service is running
    pub fn is_cups_running(&self) -> bool {
        Command::new("lpstat")
            .args(["-r"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    /// Get CUPS version
    pub fn cups_version(&self) -> Option<String> {
        Command::new("lpstat")
            .args(["-V"])
            .output()
            .ok()
            .and_then(|o| {
                if o.status.success() {
                    String::from_utf8(o.stdout).ok()
                } else {
                    None
                }
            })
            .map(|s| s.trim().to_string())
    }

    /// Open CUPS web interface in default browser
    pub fn open_cups_web(&self) -> Result<()> {
        let url = format!("http://{}", self.server);
        open::that(&url).context("Failed to open CUPS web interface")
    }
}

impl Default for CupsManager {
    fn default() -> Self {
        Self::new()
    }
}
