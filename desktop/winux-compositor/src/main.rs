//! Winux Compositor - Wayland compositor for Winux OS
//!
//! This is the main entry point for the Winux compositor.
//! It initializes the backend, sets up the event loop, and manages
//! the compositor lifecycle.

use anyhow::{Context, Result};
use smithay::{
    backend::{
        renderer::gles::GlesRenderer,
        winit::{self, WinitEvent, WinitGraphicsBackend},
    },
    output::{Mode, Output, PhysicalProperties, Subpixel},
    reexports::{
        calloop::{
            timer::{TimeoutAction, Timer},
            EventLoop,
        },
        wayland_server::Display,
    },
    utils::{Rectangle, Transform},
};
use std::{sync::atomic::Ordering, time::Duration};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};
use winux_compositor::{
    config::CompositorConfig,
    input::process_input_event,
    rendering::{GlesCompositorRenderer, Renderer},
    state::WinuxState,
    NAME, VERSION,
};

/// Main entry point
fn main() -> Result<()> {
    // Initialize logging
    init_logging()?;

    tracing::info!("{} v{} starting...", NAME, VERSION);

    // Load configuration
    let config = CompositorConfig::load().unwrap_or_else(|e| {
        tracing::warn!("Failed to load config, using defaults: {}", e);
        CompositorConfig::default()
    });

    tracing::debug!("Configuration loaded: {:?}", config.general);

    // Run the compositor
    match config.general.backend.as_str() {
        "winit" => run_winit_backend(config),
        "drm" | "auto" => {
            // Try DRM first, fall back to winit
            tracing::info!("Attempting native DRM backend...");
            if let Err(e) = run_drm_backend(&config) {
                tracing::warn!("DRM backend failed: {}, falling back to winit", e);
                run_winit_backend(config)
            } else {
                Ok(())
            }
        }
        unknown => {
            tracing::error!("Unknown backend: {}", unknown);
            anyhow::bail!("Unknown backend: {}", unknown)
        }
    }
}

/// Initialize logging with tracing
fn init_logging() -> Result<()> {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,winux_compositor=debug,smithay=warn"));

    tracing_subscriber::registry()
        .with(fmt::layer().with_target(true).with_thread_ids(true))
        .with(filter)
        .init();

    Ok(())
}

/// Run the compositor with the Winit backend (for development/testing)
fn run_winit_backend(config: CompositorConfig) -> Result<()> {
    tracing::info!("Starting Winit backend");

    // Create event loop
    let mut event_loop: EventLoop<WinuxState> =
        EventLoop::try_new().context("Failed to create event loop")?;
    let loop_handle = event_loop.handle();

    // Create Wayland display
    let display: Display<WinuxState> = Display::new().context("Failed to create display")?;

    // Initialize Winit backend
    let (mut backend, mut winit_event_loop) = winit::init::<GlesRenderer>()
        .map_err(|e| anyhow::anyhow!("Failed to initialize winit backend: {:?}", e))?;

    // Create output
    let output = create_output(&backend);

    // Create compositor state
    let mut state = WinuxState::new(display.clone(), loop_handle.clone(), config);

    // Initialize Wayland socket
    let mut display_ref = display;
    let socket_name = state.init_wayland_socket(&mut display_ref)?;

    // Set WAYLAND_DISPLAY environment variable
    std::env::set_var("WAYLAND_DISPLAY", &socket_name);
    tracing::info!("WAYLAND_DISPLAY={}", socket_name);

    // Add output to state
    state.space.map_output(&output, (0, 0));

    // Initialize renderer
    let mut renderer = GlesCompositorRenderer::new(backend.renderer().clone());
    if let Some(color) =
        GlesCompositorRenderer::parse_color(&state.config.appearance.background_color)
    {
        renderer.set_clear_color(color);
    }
    renderer
        .init()
        .context("Failed to initialize renderer")?;

    // Set up render timer (60 FPS)
    let timer = Timer::immediate();
    loop_handle
        .insert_source(timer, move |_, _, state| {
            // Render frame
            TimeoutAction::ToDuration(Duration::from_millis(16))
        })
        .map_err(|e| anyhow::anyhow!("Failed to insert timer: {:?}", e))?;

    tracing::info!("Compositor initialized, entering event loop");

    // Main event loop
    while state.is_running() {
        // Process Winit events
        winit_event_loop
            .dispatch_new_events(|event| match event {
                WinitEvent::Resized { size, .. } => {
                    tracing::debug!("Window resized to {:?}", size);
                    let mode = Mode {
                        size,
                        refresh: 60_000,
                    };
                    output.change_current_state(Some(mode), None, None, None);
                }
                WinitEvent::Input(event) => {
                    process_input_event(&mut state, event);
                }
                WinitEvent::Redraw => {
                    // Render frame
                    if let Err(e) = renderer.render_frame(&mut state, &output) {
                        tracing::error!("Render error: {}", e);
                    }

                    // Submit frame
                    backend.submit(None).ok();

                    // Send frame callbacks
                    state.space.elements().for_each(|window| {
                        window.send_frame(
                            &output,
                            state.start_time.elapsed(),
                            Some(Duration::ZERO),
                            |_, _| Some(output.clone()),
                        );
                    });
                }
                WinitEvent::CloseRequested => {
                    tracing::info!("Close requested");
                    state.stop();
                }
                WinitEvent::Focus(_) => {}
                WinitEvent::Refresh => {}
            })
            .map_err(|e| anyhow::anyhow!("Winit dispatch error: {:?}", e))?;

        // Dispatch Wayland events
        display_ref
            .dispatch_clients(&mut state)
            .context("Failed to dispatch clients")?;
        display_ref.flush_clients().ok();

        // Dispatch event loop
        event_loop
            .dispatch(Some(Duration::from_millis(1)), &mut state)
            .context("Event loop dispatch failed")?;
    }

    tracing::info!("Compositor shutting down");
    Ok(())
}

/// Run the compositor with the DRM backend (for native hardware)
fn run_drm_backend(config: &CompositorConfig) -> Result<()> {
    tracing::info!("Starting DRM backend");

    // DRM backend requires proper session management and hardware access
    // This is a placeholder for the full implementation
    anyhow::bail!("DRM backend not yet fully implemented")
}

/// Create an output for the backend
fn create_output<R>(backend: &WinitGraphicsBackend<R>) -> Output {
    let mode = Mode {
        size: backend.window_size(),
        refresh: 60_000,
    };

    let physical_properties = PhysicalProperties {
        size: (0, 0).into(),
        subpixel: Subpixel::Unknown,
        make: "Winux".into(),
        model: "Virtual Output".into(),
    };

    let output = Output::new("winux-0".to_string(), physical_properties);
    output.change_current_state(Some(mode), Some(Transform::Flipped180), None, Some((0, 0).into()));
    output.set_preferred(mode);

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
        assert!(!NAME.is_empty());
    }
}
