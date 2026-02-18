//! Winux Compositor - Wayland compositor placeholder
//! 
//! This is a placeholder compositor for the Winux desktop environment.
//! In practice, Winux uses mutter/gnome-shell or another established
//! Wayland compositor as the backend.
//!
//! This stub demonstrates the intended architecture and could be
//! extended to wrap or configure the actual compositor.

use std::env;
use std::process::{Command, exit};

const VERSION: &str = "0.1.0";

fn main() {
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║              Winux Compositor v{}                        ║", VERSION);
    println!("╠════════════════════════════════════════════════════════════╣");
    println!("║  This is a placeholder compositor component.               ║");
    println!("║                                                            ║");
    println!("║  Winux desktop environment uses established Wayland        ║");
    println!("║  compositors as backends:                                  ║");
    println!("║                                                            ║");
    println!("║    • mutter (GNOME Shell) - Recommended                    ║");
    println!("║    • wlroots-based (sway, etc.)                            ║");
    println!("║    • smithay (Rust-native, experimental)                   ║");
    println!("║                                                            ║");
    println!("╚════════════════════════════════════════════════════════════╝");
    println!();

    // Check for available compositors
    check_compositor_availability();

    // Parse arguments
    let args: Vec<String> = env::args().collect();
    
    if args.len() > 1 {
        match args[1].as_str() {
            "--help" | "-h" => print_help(),
            "--version" | "-v" => println!("winux-compositor {}", VERSION),
            "--check" => check_compositor_availability(),
            "--launch-mutter" => launch_mutter(),
            "--launch-gnome-shell" => launch_gnome_shell(),
            _ => {
                eprintln!("Unknown option: {}", args[1]);
                eprintln!("Use --help for usage information");
                exit(1);
            }
        }
    } else {
        println!("[winux-compositor] No compositor backend specified.");
        println!("[winux-compositor] Use --launch-mutter or --launch-gnome-shell to start a session.");
        println!();
        suggest_next_steps();
    }
}

fn print_help() {
    println!("Usage: winux-compositor [OPTIONS]");
    println!();
    println!("Options:");
    println!("  --help, -h           Show this help message");
    println!("  --version, -v        Show version information");
    println!("  --check              Check for available compositor backends");
    println!("  --launch-mutter      Launch mutter as the compositor");
    println!("  --launch-gnome-shell Launch GNOME Shell as the compositor");
    println!();
    println!("Environment Variables:");
    println!("  WINUX_COMPOSITOR     Override default compositor (mutter|gnome-shell)");
    println!("  WAYLAND_DISPLAY      Wayland display socket name");
}

fn check_compositor_availability() {
    println!("[winux-compositor] Checking for available compositor backends...");
    println!();

    let compositors = vec![
        ("mutter", "mutter --version"),
        ("gnome-shell", "gnome-shell --version"),
        ("sway", "sway --version"),
        ("weston", "weston --version"),
    ];

    for (name, cmd) in compositors {
        let parts: Vec<&str> = cmd.split_whitespace().collect();
        let result = Command::new(parts[0])
            .args(&parts[1..])
            .output();

        match result {
            Ok(output) if output.status.success() => {
                let version = String::from_utf8_lossy(&output.stdout);
                let version_line = version.lines().next().unwrap_or("unknown version");
                println!("  ✓ {} - Available ({})", name, version_line.trim());
            }
            _ => {
                println!("  ✗ {} - Not found", name);
            }
        }
    }
    println!();
}

fn launch_mutter() {
    println!("[winux-compositor] Launching mutter...");
    
    let result = Command::new("mutter")
        .arg("--wayland")
        .spawn();

    match result {
        Ok(mut child) => {
            println!("[winux-compositor] mutter started with PID: {}", child.id());
            let _ = child.wait();
        }
        Err(e) => {
            eprintln!("[winux-compositor] Failed to launch mutter: {}", e);
            eprintln!("[winux-compositor] Make sure mutter is installed.");
            exit(1);
        }
    }
}

fn launch_gnome_shell() {
    println!("[winux-compositor] Launching GNOME Shell...");
    
    let result = Command::new("gnome-shell")
        .arg("--wayland")
        .spawn();

    match result {
        Ok(mut child) => {
            println!("[winux-compositor] GNOME Shell started with PID: {}", child.id());
            let _ = child.wait();
        }
        Err(e) => {
            eprintln!("[winux-compositor] Failed to launch GNOME Shell: {}", e);
            eprintln!("[winux-compositor] Make sure gnome-shell is installed.");
            exit(1);
        }
    }
}

fn suggest_next_steps() {
    println!("Next steps for Winux desktop development:");
    println!();
    println!("  1. For a full desktop session, use GNOME Shell or mutter");
    println!("  2. Winux shell and panel components layer on top of the compositor");
    println!("  3. Run: winux-compositor --launch-gnome-shell");
    println!();
    println!("For a custom Rust-native compositor, consider using:");
    println!("  • smithay - Pure Rust Wayland compositor library");
    println!("  • https://github.com/Smithay/smithay");
}
