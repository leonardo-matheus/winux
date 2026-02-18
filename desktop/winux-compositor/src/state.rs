//! Compositor state management
//!
//! This module defines the core state structure for the Winux compositor,
//! managing outputs, surfaces, windows, and client connections.

use crate::config::CompositorConfig;
use crate::input::InputHandler;
use smithay::{
    delegate_compositor, delegate_data_device, delegate_output, delegate_seat, delegate_shm,
    delegate_xdg_shell,
    desktop::{Space, Window},
    input::{pointer::PointerHandle, Seat, SeatState},
    reexports::{
        calloop::{generic::Generic, Interest, LoopHandle, Mode, PostAction},
        wayland_server::{
            backend::{ClientData, ClientId, DisconnectReason},
            protocol::wl_surface::WlSurface,
            Display, DisplayHandle,
        },
    },
    wayland::{
        buffer::BufferHandler,
        compositor::{CompositorClientState, CompositorHandler, CompositorState},
        data_device::{
            ClientDndGrabHandler, DataDeviceHandler, DataDeviceState, ServerDndGrabHandler,
        },
        output::OutputManagerState,
        shell::xdg::{XdgShellHandler, XdgShellState},
        shm::{ShmHandler, ShmState},
        socket::ListeningSocketSource,
    },
};
use std::{
    collections::HashMap,
    sync::{atomic::AtomicBool, Arc},
};

/// Main state structure for the Winux compositor
pub struct WinuxState {
    /// Configuration
    pub config: CompositorConfig,

    /// Display handle
    pub display_handle: DisplayHandle,

    /// Event loop handle
    pub loop_handle: LoopHandle<'static, Self>,

    /// Running flag
    pub running: Arc<AtomicBool>,

    /// Desktop space managing window layout
    pub space: Space<Window>,

    /// Input seat state
    pub seat_state: SeatState<Self>,

    /// Primary seat
    pub seat: Seat<Self>,

    /// Compositor protocol state
    pub compositor_state: CompositorState,

    /// XDG shell state
    pub xdg_shell_state: XdgShellState,

    /// Shared memory state
    pub shm_state: ShmState,

    /// Output manager state
    pub output_manager_state: OutputManagerState,

    /// Data device (clipboard/drag-and-drop) state
    pub data_device_state: DataDeviceState,

    /// Input handler
    pub input_handler: InputHandler,

    /// Map of surface to window
    surface_to_window: HashMap<WlSurface, Window>,

    /// Focused surface
    focused_surface: Option<WlSurface>,

    /// Socket name for client connections
    pub socket_name: String,

    /// Start time for uptime tracking
    pub start_time: std::time::Instant,
}

impl WinuxState {
    /// Create a new compositor state
    pub fn new(
        display: Display<Self>,
        loop_handle: LoopHandle<'static, Self>,
        config: CompositorConfig,
    ) -> Self {
        let display_handle = display.handle();
        let running = Arc::new(AtomicBool::new(true));

        // Initialize Wayland protocols
        let compositor_state = CompositorState::new::<Self>(&display_handle);
        let xdg_shell_state = XdgShellState::new::<Self>(&display_handle);
        let shm_state = ShmState::new::<Self>(&display_handle, vec![]);
        let output_manager_state = OutputManagerState::new_with_xdg_output::<Self>(&display_handle);
        let data_device_state = DataDeviceState::new::<Self>(&display_handle);

        // Initialize seat
        let mut seat_state = SeatState::new();
        let seat = seat_state.new_wl_seat(&display_handle, "winux-seat");

        // Initialize input handler
        let input_handler = InputHandler::new(&config.input);

        // Create desktop space
        let space = Space::default();

        Self {
            config,
            display_handle,
            loop_handle,
            running,
            space,
            seat_state,
            seat,
            compositor_state,
            xdg_shell_state,
            shm_state,
            output_manager_state,
            data_device_state,
            input_handler,
            surface_to_window: HashMap::new(),
            focused_surface: None,
            socket_name: String::new(),
            start_time: std::time::Instant::now(),
        }
    }

    /// Initialize the Wayland socket
    pub fn init_wayland_socket(&mut self, display: &mut Display<Self>) -> anyhow::Result<String> {
        // Create the Wayland socket
        let socket = ListeningSocketSource::new_auto()?;
        let socket_name = socket.socket_name().to_string_lossy().to_string();

        // Add the socket to the event loop
        self.loop_handle
            .insert_source(socket, |client_stream, _, state| {
                if let Err(err) = state
                    .display_handle
                    .insert_client(client_stream, Arc::new(ClientState::default()))
                {
                    tracing::error!("Failed to insert client: {:?}", err);
                }
            })?;

        // Add the display to the event loop
        self.loop_handle.insert_source(
            Generic::new(display.backend().poll_fd(), Interest::READ, Mode::Level),
            |_, _, state| {
                // Process pending Wayland events
                // This is handled in the main loop
                Ok(PostAction::Continue)
            },
        )?;

        self.socket_name = socket_name.clone();
        tracing::info!("Wayland socket: {}", socket_name);

        Ok(socket_name)
    }

    /// Get the pointer handle
    pub fn pointer(&self) -> Option<PointerHandle<Self>> {
        self.seat.get_pointer()
    }

    /// Get the focused surface
    pub fn focused_surface(&self) -> Option<&WlSurface> {
        self.focused_surface.as_ref()
    }

    /// Set the focused surface
    pub fn set_focus(&mut self, surface: Option<WlSurface>) {
        self.focused_surface = surface;
    }

    /// Check if the compositor is running
    pub fn is_running(&self) -> bool {
        self.running.load(std::sync::atomic::Ordering::SeqCst)
    }

    /// Stop the compositor
    pub fn stop(&self) {
        self.running
            .store(false, std::sync::atomic::Ordering::SeqCst);
    }

    /// Get uptime in seconds
    pub fn uptime(&self) -> u64 {
        self.start_time.elapsed().as_secs()
    }

    /// Reload configuration
    pub fn reload_config(&mut self) -> anyhow::Result<()> {
        self.config.reload()?;
        self.input_handler.update_config(&self.config.input);
        tracing::info!("Configuration reloaded");
        Ok(())
    }
}

/// Client state for tracking per-client data
#[derive(Default)]
pub struct ClientState {
    pub compositor_state: CompositorClientState,
}

impl ClientData for ClientState {
    fn initialized(&self, _client_id: ClientId) {}
    fn disconnected(&self, _client_id: ClientId, _reason: DisconnectReason) {}
}

// Implement Smithay handlers

impl CompositorHandler for WinuxState {
    fn compositor_state(&mut self) -> &mut CompositorState {
        &mut self.compositor_state
    }

    fn client_compositor_state<'a>(&self, client: &'a smithay::reexports::wayland_server::Client) -> &'a CompositorClientState {
        &client.get_data::<ClientState>().unwrap().compositor_state
    }

    fn commit(&mut self, surface: &WlSurface) {
        // Handle surface commits
        if let Some(window) = self.surface_to_window.get(surface).cloned() {
            // Refresh the window in the space
            self.space.refresh();
        }
    }
}

impl BufferHandler for WinuxState {
    fn buffer_destroyed(&mut self, _buffer: &smithay::reexports::wayland_server::protocol::wl_buffer::WlBuffer) {
        // Handle buffer destruction
    }
}

impl ShmHandler for WinuxState {
    fn shm_state(&self) -> &ShmState {
        &self.shm_state
    }
}

impl XdgShellHandler for WinuxState {
    fn xdg_shell_state(&mut self) -> &mut XdgShellState {
        &mut self.xdg_shell_state
    }

    fn new_toplevel(&mut self, surface: smithay::wayland::shell::xdg::ToplevelSurface) {
        let window = Window::new_wayland_window(surface);
        self.space.map_element(window.clone(), (0, 0), false);

        if let Some(wl_surface) = window.wl_surface() {
            self.surface_to_window.insert(wl_surface.clone(), window);
        }
    }

    fn new_popup(
        &mut self,
        _surface: smithay::wayland::shell::xdg::PopupSurface,
        _positioner: smithay::wayland::shell::xdg::PositionerState,
    ) {
        // Handle popup creation
    }

    fn grab(
        &mut self,
        _surface: smithay::wayland::shell::xdg::PopupSurface,
        _seat: smithay::reexports::wayland_server::protocol::wl_seat::WlSeat,
        _serial: smithay::utils::Serial,
    ) {
        // Handle popup grab
    }

    fn reposition_request(
        &mut self,
        _surface: smithay::wayland::shell::xdg::PopupSurface,
        _positioner: smithay::wayland::shell::xdg::PositionerState,
        _token: u32,
    ) {
        // Handle popup reposition
    }
}

impl DataDeviceHandler for WinuxState {
    fn data_device_state(&self) -> &DataDeviceState {
        &self.data_device_state
    }
}

impl ClientDndGrabHandler for WinuxState {}
impl ServerDndGrabHandler for WinuxState {}

impl smithay::input::SeatHandler for WinuxState {
    type KeyboardFocus = WlSurface;
    type PointerFocus = WlSurface;
    type TouchFocus = WlSurface;

    fn seat_state(&mut self) -> &mut SeatState<Self> {
        &mut self.seat_state
    }

    fn cursor_image(
        &mut self,
        _seat: &Seat<Self>,
        _image: smithay::input::pointer::CursorImageStatus,
    ) {
        // Handle cursor image changes
    }

    fn focus_changed(&mut self, _seat: &Seat<Self>, _focused: Option<&Self::KeyboardFocus>) {
        // Handle focus changes
    }
}

// Delegate macros for Smithay protocols
delegate_compositor!(WinuxState);
delegate_shm!(WinuxState);
delegate_xdg_shell!(WinuxState);
delegate_data_device!(WinuxState);
delegate_output!(WinuxState);
delegate_seat!(WinuxState);
