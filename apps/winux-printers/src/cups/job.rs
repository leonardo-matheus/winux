//! Print job data structures

use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Print job status
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum JobStatus {
    /// Job is waiting in queue
    Pending,
    /// Job is held (paused by user)
    Held,
    /// Job is being processed (RIP, etc.)
    Processing,
    /// Job is currently printing
    Printing(u8), // Progress percentage
    /// Job completed successfully
    Completed,
    /// Job was cancelled by user
    Cancelled,
    /// Job was aborted due to error
    Aborted(String),
}

impl JobStatus {
    /// Check if job is active (not completed/cancelled/aborted)
    pub fn is_active(&self) -> bool {
        matches!(
            self,
            JobStatus::Pending | JobStatus::Held | JobStatus::Processing | JobStatus::Printing(_)
        )
    }

    /// Check if job is finished
    pub fn is_finished(&self) -> bool {
        matches!(
            self,
            JobStatus::Completed | JobStatus::Cancelled | JobStatus::Aborted(_)
        )
    }

    /// Get display string for the status
    pub fn display_string(&self) -> String {
        match self {
            JobStatus::Pending => "Aguardando".to_string(),
            JobStatus::Held => "Retido".to_string(),
            JobStatus::Processing => "Processando".to_string(),
            JobStatus::Printing(p) => format!("Imprimindo {}%", p),
            JobStatus::Completed => "Concluido".to_string(),
            JobStatus::Cancelled => "Cancelado".to_string(),
            JobStatus::Aborted(reason) => format!("Abortado: {}", reason),
        }
    }

    /// Get icon name for this status
    pub fn icon_name(&self) -> &str {
        match self {
            JobStatus::Pending => "content-loading-symbolic",
            JobStatus::Held => "media-playback-pause-symbolic",
            JobStatus::Processing => "emblem-synchronizing-symbolic",
            JobStatus::Printing(_) => "printer-printing-symbolic",
            JobStatus::Completed => "emblem-ok-symbolic",
            JobStatus::Cancelled => "window-close-symbolic",
            JobStatus::Aborted(_) => "dialog-error-symbolic",
        }
    }
}

/// A print job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrintJob {
    /// Job ID
    pub id: u32,
    /// Document name
    pub document_name: String,
    /// Printer name
    pub printer: String,
    /// User who submitted the job
    pub user: String,
    /// Current status
    pub status: JobStatus,
    /// Number of pages
    pub pages: u32,
    /// Job size in bytes
    pub size: u64,
    /// Submission time
    pub submitted_at: DateTime<Utc>,
    /// Completion time (if completed)
    pub completed_at: Option<DateTime<Utc>>,
    /// Number of copies
    pub copies: u32,
    /// Print options used
    pub options: PrintOptions,
    /// Priority (1-100, higher is more urgent)
    pub priority: u8,
}

impl PrintJob {
    /// Create a new print job
    pub fn new(
        id: u32,
        document_name: &str,
        printer: &str,
        user: &str,
        status: JobStatus,
        pages: u32,
        size: u64,
        submitted_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            document_name: document_name.to_string(),
            printer: printer.to_string(),
            user: user.to_string(),
            status,
            pages,
            size,
            submitted_at,
            completed_at: None,
            copies: 1,
            options: PrintOptions::default(),
            priority: 50,
        }
    }

    /// Get formatted size string
    pub fn formatted_size(&self) -> String {
        bytesize::ByteSize::b(self.size).to_string()
    }

    /// Check if the job can be cancelled
    pub fn can_cancel(&self) -> bool {
        self.status.is_active()
    }

    /// Check if the job can be held/released
    pub fn can_hold(&self) -> bool {
        matches!(self.status, JobStatus::Pending)
    }

    /// Check if the job can be released (unheld)
    pub fn can_release(&self) -> bool {
        matches!(self.status, JobStatus::Held)
    }

    /// Check if the job can be moved to another printer
    pub fn can_move(&self) -> bool {
        matches!(self.status, JobStatus::Pending | JobStatus::Held)
    }

    /// Check if the job can be reprinted
    pub fn can_reprint(&self) -> bool {
        matches!(self.status, JobStatus::Completed)
    }
}

/// Print options for a job
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PrintOptions {
    /// Number of copies
    pub copies: u32,
    /// Paper size (e.g., "A4", "Letter")
    pub paper_size: Option<String>,
    /// Print quality (e.g., "Draft", "Normal", "High")
    pub quality: Option<String>,
    /// Color mode (e.g., "Color", "Grayscale", "Monochrome")
    pub color_mode: Option<String>,
    /// Duplex mode
    pub duplex: DuplexMode,
    /// Orientation
    pub orientation: Orientation,
    /// Pages per sheet
    pub pages_per_sheet: u8,
    /// Page ranges (e.g., "1-5,7,9-12")
    pub page_ranges: Option<String>,
    /// Input tray
    pub input_tray: Option<String>,
    /// Output bin
    pub output_bin: Option<String>,
    /// Collate copies
    pub collate: bool,
    /// Reverse order
    pub reverse: bool,
    /// Scale percentage (100 = actual size)
    pub scale: u8,
    /// Additional raw options
    pub extra_options: HashMap<String, String>,
}

impl PrintOptions {
    /// Create options with sensible defaults
    pub fn new() -> Self {
        Self {
            copies: 1,
            paper_size: Some("A4".to_string()),
            quality: Some("Normal".to_string()),
            color_mode: None,
            duplex: DuplexMode::None,
            orientation: Orientation::Portrait,
            pages_per_sheet: 1,
            page_ranges: None,
            input_tray: None,
            output_bin: None,
            collate: true,
            reverse: false,
            scale: 100,
            extra_options: HashMap::new(),
        }
    }

    /// Convert options to CUPS lp command options
    pub fn to_lp_options(&self) -> Vec<(String, String)> {
        let mut opts = Vec::new();

        if let Some(ref paper) = self.paper_size {
            opts.push(("media".to_string(), paper.clone()));
        }

        if let Some(ref quality) = self.quality {
            let cups_quality = match quality.to_lowercase().as_str() {
                "draft" | "rascunho" => "3",
                "normal" => "4",
                "high" | "alta" => "5",
                _ => "4",
            };
            opts.push(("print-quality".to_string(), cups_quality.to_string()));
        }

        if let Some(ref color) = self.color_mode {
            let cups_color = match color.to_lowercase().as_str() {
                "color" | "colorido" => "Color",
                "grayscale" | "escala de cinza" => "Gray",
                "monochrome" | "preto e branco" => "Mono",
                _ => "Color",
            };
            opts.push(("print-color-mode".to_string(), cups_color.to_string()));
        }

        match self.duplex {
            DuplexMode::None => {
                opts.push(("sides".to_string(), "one-sided".to_string()));
            }
            DuplexMode::LongEdge => {
                opts.push(("sides".to_string(), "two-sided-long-edge".to_string()));
            }
            DuplexMode::ShortEdge => {
                opts.push(("sides".to_string(), "two-sided-short-edge".to_string()));
            }
        }

        match self.orientation {
            Orientation::Portrait => {
                opts.push(("orientation-requested".to_string(), "3".to_string()));
            }
            Orientation::Landscape => {
                opts.push(("orientation-requested".to_string(), "4".to_string()));
            }
            Orientation::ReversePortrait => {
                opts.push(("orientation-requested".to_string(), "5".to_string()));
            }
            Orientation::ReverseLandscape => {
                opts.push(("orientation-requested".to_string(), "6".to_string()));
            }
        }

        if self.pages_per_sheet > 1 {
            opts.push((
                "number-up".to_string(),
                self.pages_per_sheet.to_string(),
            ));
        }

        if let Some(ref ranges) = self.page_ranges {
            opts.push(("page-ranges".to_string(), ranges.clone()));
        }

        if let Some(ref tray) = self.input_tray {
            opts.push(("InputSlot".to_string(), tray.clone()));
        }

        if let Some(ref bin) = self.output_bin {
            opts.push(("OutputBin".to_string(), bin.clone()));
        }

        if self.collate {
            opts.push(("Collate".to_string(), "True".to_string()));
        }

        if self.reverse {
            opts.push(("outputorder".to_string(), "reverse".to_string()));
        }

        if self.scale != 100 {
            opts.push((
                "fit-to-page".to_string(),
                "true".to_string(),
            ));
        }

        // Add any extra raw options
        for (key, value) in &self.extra_options {
            opts.push((key.clone(), value.clone()));
        }

        opts
    }
}

/// Duplex (double-sided) printing mode
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum DuplexMode {
    /// Single-sided (no duplex)
    #[default]
    None,
    /// Two-sided, flip on long edge (book-style)
    LongEdge,
    /// Two-sided, flip on short edge (notepad-style)
    ShortEdge,
}

impl DuplexMode {
    /// Get display name
    pub fn display_name(&self) -> &str {
        match self {
            DuplexMode::None => "Desligado (Apenas um lado)",
            DuplexMode::LongEdge => "Borda Longa (Livro)",
            DuplexMode::ShortEdge => "Borda Curta (Bloco)",
        }
    }
}

/// Page orientation
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Orientation {
    /// Portrait (vertical)
    #[default]
    Portrait,
    /// Landscape (horizontal)
    Landscape,
    /// Reverse portrait (upside down)
    ReversePortrait,
    /// Reverse landscape
    ReverseLandscape,
}

impl Orientation {
    /// Get display name
    pub fn display_name(&self) -> &str {
        match self {
            Orientation::Portrait => "Retrato",
            Orientation::Landscape => "Paisagem",
            Orientation::ReversePortrait => "Retrato Invertido",
            Orientation::ReverseLandscape => "Paisagem Invertida",
        }
    }
}
