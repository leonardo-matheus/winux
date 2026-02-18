//! Winux Compositor - Placeholder/Launcher
//!
//! This is a lightweight placeholder compositor for the Winux desktop environment.
//! Winux uses established Wayland compositors (mutter/gnome-shell) as the backend
//! rather than implementing a custom compositor from scratch.
//!
//! This binary serves as a launcher/wrapper that:
//! - Provides information about available compositor backends
//! - Can launch mutter or gnome-shell on request
//! - Uses only the Rust standard library (no external dependencies)

use std::env;
use std::process::{Command, exit};

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() > 1 {
        match args[1].as_str() {
            "--help" | "-h" => {
                print_help();
                return;
            }
            "--version" | "-v" => {
                println!("winux-compositor {}", VERSION);
                return;
            }
            "--check" => {
                check_compositor_availability();
                return;
            }
            "--launch-mutter" => {
                launch_compositor("mutter", &["--wayland"]);
                return;
            }
            "--launch-gnome-shell" => {
                launch_compositor("gnome-shell", &["--wayland"]);
                return;
            }
            "--launch" => {
                // Auto-detect and launch best available compositor
                auto_launch_compositor();
                return;
            }
            arg => {
                eprintln!("Unknown option: {}", arg);
                eprintln!("Use --help for usage information");
                exit(1);
            }
        }
    }

    // Default: show info
    print_banner();
    println!();
    check_compositor_availability();
    suggest_next_steps();
}

fn print_banner() {
    println!("================================================================");
    println!("              Winux Compositor v{}                      ", VERSION);
    println!("================================================================");
    println!();
    println!("  This is a placeholder/launcher component.");
    println!();
    println!("  Winux desktop environment uses established Wayland");
    println!("  compositors as backends:");
    println!();
    println!("    - mutter (GNOME Shell) - Recommended");
    println!("    - wlroots-based (sway, etc.)");
    println!();
    println!("  A custom smithay-based compositor may be added in the future");
    println!("  when the ecosystem is more mature.");
    println!();
    println!("================================================================");
}

fn print_help() {
    println!("winux-compositor - Wayland compositor launcher for Winux OS");
    println!();
    println!("Usage: winux-compositor [OPTIONS]");
    println!();
    println!("Options:");
    println!("  --help, -h           Show this help message");
    println!("  --version, -v        Show version information");
    println!("  --check              Check for available compositor backends");
    println!("  --launch             Auto-detect and launch best compositor");
    println!("  --launch-mutter      Launch mutter as the compositor");
    println!("  --launch-gnome-shell Launch GNOME Shell as the compositor");
    println!();
    println!("Environment Variables:");
    println!("  WINUX_COMPOSITOR     Override compositor choice (mutter|gnome-shell)");
    println!("  WAYLAND_DISPLAY      Wayland display socket name");
    println!();
    println!("Note: This is a launcher/wrapper. The actual compositor work is");
    println!("done by mutter, gnome-shell, or another Wayland compositor.");
}

fn check_compositor_availability() {
    println!("[winux-compositor] Checking available compositor backends...");
    println!();

    let compositors = [
        ("mutter", "mutter"),
        ("gnome-shell", "gnome-shell"),
        ("sway", "sway"),
        ("weston", "weston"),
    ];

    for (name, cmd) in compositors {
        let available = Command::new(cmd)
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if available {
            println!("  [OK] {} - Available", name);
        } else {
            println!("  [--] {} - Not found", name);
        }
    }
    println!();
}

fn launch_compositor(name: &str, args: &[&str]) {
    println!("[winux-compositor] Launching {}...", name);
    
    match Command::new(name).args(args).spawn() {
        Ok(mut child) => {
            println!("[winux-compositor] {} started with PID: {}", name, child.id());
            match child.wait() {
                Ok(status) => {
                    if !status.success() {
                        eprintln!("[winux-compositor] {} exited with status: {}", name, status);
                    }
                }
                Err(e) => {
                    eprintln!("[winux-compositor] Error waiting for {}: {}", name, e);
                }
            }
        }
        Err(e) => {
            eprintln!("[winux-compositor] Failed to launch {}: {}", name, e);
            eprintln!("[winux-compositor] Make sure {} is installed.", name);
            exit(1);
        }
    }
}

fn auto_launch_compositor() {
    // Check environment variable override
    if let Ok(compositor) = env::var("WINUX_COMPOSITOR") {
        match compositor.as_str() {
            "mutter" => {
                launch_compositor("mutter", &["--wayland"]);
                return;
            }
            "gnome-shell" => {
                launch_compositor("gnome-shell", &["--wayland"]);
                return;
            }
            other => {
                eprintln!("[winux-compositor] Unknown WINUX_COMPOSITOR value: {}", other);
                eprintln!("[winux-compositor] Supported values: mutter, gnome-shell");
            }
        }
    }

    // Auto-detect: prefer gnome-shell, then mutter
    let candidates = [
        ("gnome-shell", &["--wayland"][..]),
        ("mutter", &["--wayland"][..]),
    ];

    for (name, args) in candidates {
        let available = Command::new(name)
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if available {
            println!("[winux-compositor] Auto-detected: {}", name);
            launch_compositor(name, args);
            return;
        }
    }

    eprintln!("[winux-compositor] No suitable compositor found!");
    eprintln!("[winux-compositor] Please install gnome-shell or mutter.");
    exit(1);
}

fn suggest_next_steps() {
    println!("To start the Winux desktop:");
    println!();
    println!("  winux-compositor --launch           # Auto-detect and launch");
    println!("  winux-compositor --launch-mutter    # Launch mutter directly");
    println!("  winux-compositor --launch-gnome-shell");
    println!();
    println!("Or set WINUX_COMPOSITOR=mutter and run --launch");
    println!();
}
