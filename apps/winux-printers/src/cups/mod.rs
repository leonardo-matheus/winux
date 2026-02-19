//! CUPS integration module for printer management
//!
//! This module provides abstractions for communicating with CUPS
//! (Common Unix Printing System) for printer discovery, configuration,
//! and print job management.

mod client;
mod printer;
mod job;

pub use client::CupsManager;
pub use printer::{Printer, PrinterStatus, DiscoveredPrinter, ConnectionType, PrinterCapabilities};
pub use job::{PrintJob, JobStatus, PrintOptions};
