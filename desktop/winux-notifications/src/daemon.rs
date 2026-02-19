//! D-Bus notification daemon implementation
//!
//! Implements the org.freedesktop.Notifications specification.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

use async_channel::{Receiver, Sender};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use zbus::{interface, Connection, SignalContext};
use zvariant::OwnedValue;

use crate::config::NotificationConfig;
use crate::history::NotificationHistory;
use crate::notification::{CloseReason, Notification};

/// Notification ID counter
static NOTIFICATION_ID: AtomicU32 = AtomicU32::new(1);

/// Get the next notification ID
fn next_notification_id() -> u32 {
    NOTIFICATION_ID.fetch_add(1, Ordering::SeqCst)
}

/// Events sent from the D-Bus daemon to the UI
#[derive(Debug, Clone)]
pub enum DaemonEvent {
    /// New notification received
    NewNotification(Notification),
    /// Notification should be closed
    CloseNotification(u32, CloseReason),
    /// Update DND status
    DndChanged(bool),
}

/// Events sent from the UI to the D-Bus daemon
#[derive(Debug, Clone)]
pub enum UiEvent {
    /// User invoked an action
    ActionInvoked(u32, String),
    /// User closed a notification
    NotificationClosed(u32, CloseReason),
}

/// Shared state between D-Bus daemon and UI
pub struct DaemonState {
    /// Configuration
    pub config: RwLock<NotificationConfig>,
    /// Notification history
    pub history: RwLock<NotificationHistory>,
    /// Do Not Disturb mode
    pub dnd_enabled: RwLock<bool>,
    /// Active (visible) notifications
    pub active_notifications: RwLock<HashMap<u32, Notification>>,
}

impl DaemonState {
    pub fn new(config: NotificationConfig) -> Self {
        let history = NotificationHistory::load().unwrap_or_default();
        let dnd_enabled = config.dnd.enabled;

        Self {
            config: RwLock::new(config),
            history: RwLock::new(history),
            dnd_enabled: RwLock::new(dnd_enabled),
            active_notifications: RwLock::new(HashMap::new()),
        }
    }
}

/// D-Bus interface for org.freedesktop.Notifications
pub struct NotificationDaemon {
    /// Shared state
    state: Arc<DaemonState>,
    /// Channel to send events to UI
    event_sender: Sender<DaemonEvent>,
    /// Channel to receive events from UI
    ui_receiver: Receiver<UiEvent>,
}

impl NotificationDaemon {
    pub fn new(
        state: Arc<DaemonState>,
        event_sender: Sender<DaemonEvent>,
        ui_receiver: Receiver<UiEvent>,
    ) -> Self {
        Self {
            state,
            event_sender,
            ui_receiver,
        }
    }

    /// Start the D-Bus service
    pub async fn start(self) -> zbus::Result<Connection> {
        let connection = Connection::session().await?;

        connection
            .object_server()
            .at("/org/freedesktop/Notifications", self)
            .await?;

        connection
            .request_name("org.freedesktop.Notifications")
            .await?;

        info!("D-Bus notification service started");
        Ok(connection)
    }
}

#[interface(name = "org.freedesktop.Notifications")]
impl NotificationDaemon {
    /// Get the capabilities of this notification server
    async fn get_capabilities(&self) -> Vec<String> {
        vec![
            "actions".to_string(),
            "action-icons".to_string(),
            "body".to_string(),
            "body-hyperlinks".to_string(),
            "body-images".to_string(),
            "body-markup".to_string(),
            "icon-multi".to_string(),
            "icon-static".to_string(),
            "persistence".to_string(),
            "sound".to_string(),
        ]
    }

    /// Send a notification
    async fn notify(
        &self,
        app_name: String,
        replaces_id: u32,
        app_icon: String,
        summary: String,
        body: String,
        actions: Vec<String>,
        hints: HashMap<String, OwnedValue>,
        expire_timeout: i32,
    ) -> u32 {
        debug!(
            "Received notification: app={}, summary={}, replaces={}",
            app_name, summary, replaces_id
        );

        let config = self.state.config.read().await;

        // Check if app is allowed to notify
        if !config.should_notify_app(&app_name) {
            debug!("Notifications disabled for app: {}", app_name);
            return 0;
        }

        // Determine notification ID
        let id = if replaces_id > 0 {
            replaces_id
        } else {
            next_notification_id()
        };

        // Parse actions from alternating key/label pairs
        let mut parsed_actions = Vec::new();
        let mut iter = actions.into_iter();
        while let (Some(key), Some(label)) = (iter.next(), iter.next()) {
            parsed_actions.push((key, label));
        }

        // Create notification
        let notification = Notification::new(
            id,
            app_name.clone(),
            replaces_id,
            app_icon,
            summary,
            body,
            parsed_actions,
            hints,
            expire_timeout,
        );

        // Check DND mode
        let dnd_active = config.is_dnd_active();
        let should_show = !dnd_active
            || notification.should_show_during_dnd()
            || config.is_priority_app(&app_name);

        drop(config);

        // Add to history
        {
            let mut history = self.state.history.write().await;
            history.add(notification.clone());
            if let Err(e) = history.save() {
                warn!("Failed to save history: {}", e);
            }
        }

        // Add to active notifications
        {
            let mut active = self.state.active_notifications.write().await;
            active.insert(id, notification.clone());
        }

        // Send to UI if should show
        if should_show {
            if let Err(e) = self.event_sender.send(DaemonEvent::NewNotification(notification)).await {
                error!("Failed to send notification to UI: {}", e);
            }
        }

        id
    }

    /// Close a notification
    async fn close_notification(&self, id: u32) {
        debug!("CloseNotification called for id={}", id);

        // Remove from active notifications
        {
            let mut active = self.state.active_notifications.write().await;
            active.remove(&id);
        }

        // Send close event to UI
        if let Err(e) = self
            .event_sender
            .send(DaemonEvent::CloseNotification(id, CloseReason::Closed))
            .await
        {
            error!("Failed to send close event to UI: {}", e);
        }
    }

    /// Get server information
    async fn get_server_information(&self) -> (String, String, String, String) {
        (
            "Winux Notifications".to_string(),
            "Winux".to_string(),
            env!("CARGO_PKG_VERSION").to_string(),
            "1.2".to_string(), // Spec version
        )
    }

    /// Signal: NotificationClosed
    #[zbus(signal)]
    async fn notification_closed(
        ctx: &SignalContext<'_>,
        id: u32,
        reason: u32,
    ) -> zbus::Result<()>;

    /// Signal: ActionInvoked
    #[zbus(signal)]
    async fn action_invoked(
        ctx: &SignalContext<'_>,
        id: u32,
        action_key: &str,
    ) -> zbus::Result<()>;
}

/// Helper to emit signals from outside the interface
pub struct SignalEmitter {
    connection: Connection,
}

impl SignalEmitter {
    pub fn new(connection: Connection) -> Self {
        Self { connection }
    }

    pub async fn emit_closed(&self, id: u32, reason: CloseReason) -> zbus::Result<()> {
        let iface_ref = self
            .connection
            .object_server()
            .interface::<_, NotificationDaemon>("/org/freedesktop/Notifications")
            .await?;

        NotificationDaemon::notification_closed(iface_ref.signal_context(), id, reason as u32)
            .await
    }

    pub async fn emit_action_invoked(&self, id: u32, action_key: &str) -> zbus::Result<()> {
        let iface_ref = self
            .connection
            .object_server()
            .interface::<_, NotificationDaemon>("/org/freedesktop/Notifications")
            .await?;

        NotificationDaemon::action_invoked(iface_ref.signal_context(), id, action_key).await
    }
}

/// Create channels for daemon-UI communication
pub fn create_channels() -> (
    (Sender<DaemonEvent>, Receiver<DaemonEvent>),
    (Sender<UiEvent>, Receiver<UiEvent>),
) {
    let daemon_channel = async_channel::unbounded();
    let ui_channel = async_channel::unbounded();
    (daemon_channel, ui_channel)
}
