//! BlueZ Pairing Agent
//!
//! The pairing agent handles PIN entry, passkey confirmation,
//! and authorization requests during the pairing process.

use std::sync::Arc;
use tokio::sync::{mpsc, oneshot, RwLock};
use anyhow::Result;
use tracing::{info, warn, debug};

/// Pairing method used during device pairing
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PairingMethod {
    /// No PIN required (Just Works)
    JustWorks,
    /// PIN entry required
    PinCode,
    /// Passkey display (show to user)
    PasskeyDisplay,
    /// Passkey entry (user enters passkey)
    PasskeyEntry,
    /// Passkey confirmation (yes/no)
    PasskeyConfirmation,
}

impl PairingMethod {
    pub fn display_name(&self) -> &'static str {
        match self {
            PairingMethod::JustWorks => "Conexao Direta",
            PairingMethod::PinCode => "Codigo PIN",
            PairingMethod::PasskeyDisplay => "Exibir Codigo",
            PairingMethod::PasskeyEntry => "Digitar Codigo",
            PairingMethod::PasskeyConfirmation => "Confirmar Codigo",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            PairingMethod::JustWorks => "O dispositivo sera pareado automaticamente",
            PairingMethod::PinCode => "Digite o PIN mostrado no dispositivo",
            PairingMethod::PasskeyDisplay => "Digite este codigo no outro dispositivo",
            PairingMethod::PasskeyEntry => "Digite o codigo mostrado no outro dispositivo",
            PairingMethod::PasskeyConfirmation => "Confirme se os codigos correspondem",
        }
    }
}

/// Request from BlueZ agent to UI
#[derive(Debug)]
pub enum AgentRequest {
    /// Request PIN code entry
    RequestPinCode {
        device_path: String,
        response: oneshot::Sender<Option<String>>,
    },
    /// Display PIN code to user
    DisplayPinCode {
        device_path: String,
        pin_code: String,
    },
    /// Request passkey entry (6-digit number)
    RequestPasskey {
        device_path: String,
        response: oneshot::Sender<Option<u32>>,
    },
    /// Display passkey to user
    DisplayPasskey {
        device_path: String,
        passkey: u32,
        entered: u16,
    },
    /// Request passkey confirmation (yes/no)
    RequestConfirmation {
        device_path: String,
        passkey: u32,
        response: oneshot::Sender<bool>,
    },
    /// Request authorization (for incoming connections)
    RequestAuthorization {
        device_path: String,
        response: oneshot::Sender<bool>,
    },
    /// Authorize a service
    AuthorizeService {
        device_path: String,
        uuid: String,
        response: oneshot::Sender<bool>,
    },
    /// Cancel current request
    Cancel,
}

/// Response from UI to agent
#[derive(Debug)]
pub enum AgentResponse {
    /// PIN code entered
    PinCode(String),
    /// Passkey entered
    Passkey(u32),
    /// Confirmation response
    Confirmed(bool),
    /// Authorization response
    Authorized(bool),
    /// Request cancelled
    Cancelled,
}

/// BlueZ Pairing Agent
///
/// This agent is registered with BlueZ to handle pairing requests.
/// It communicates with the UI through channels.
pub struct PairingAgent {
    /// Channel to send requests to UI
    request_tx: mpsc::Sender<AgentRequest>,
    /// Whether agent is currently handling a request
    busy: Arc<RwLock<bool>>,
    /// Agent capability (determines pairing method)
    capability: AgentCapability,
}

/// Agent capability determines what pairing methods are supported
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentCapability {
    /// Display only (can show passkey)
    DisplayOnly,
    /// Display with yes/no buttons
    DisplayYesNo,
    /// Keyboard only (can enter passkey)
    KeyboardOnly,
    /// No input/output
    NoInputNoOutput,
    /// Full keyboard and display
    KeyboardDisplay,
}

impl AgentCapability {
    pub fn to_bluez_string(&self) -> &'static str {
        match self {
            AgentCapability::DisplayOnly => "DisplayOnly",
            AgentCapability::DisplayYesNo => "DisplayYesNo",
            AgentCapability::KeyboardOnly => "KeyboardOnly",
            AgentCapability::NoInputNoOutput => "NoInputNoOutput",
            AgentCapability::KeyboardDisplay => "KeyboardDisplay",
        }
    }
}

impl PairingAgent {
    /// Create a new pairing agent
    pub fn new(request_tx: mpsc::Sender<AgentRequest>) -> Self {
        Self {
            request_tx,
            busy: Arc::new(RwLock::new(false)),
            capability: AgentCapability::KeyboardDisplay,
        }
    }

    /// Get agent capability
    pub fn capability(&self) -> AgentCapability {
        self.capability
    }

    /// Handle PIN code request from BlueZ
    pub async fn request_pin_code(&self, device_path: &str) -> Result<String> {
        info!("PIN code requested for device: {}", device_path);

        let (tx, rx) = oneshot::channel();

        self.request_tx.send(AgentRequest::RequestPinCode {
            device_path: device_path.to_string(),
            response: tx,
        }).await?;

        match rx.await? {
            Some(pin) => Ok(pin),
            None => anyhow::bail!("PIN entry cancelled"),
        }
    }

    /// Display PIN code to user
    pub async fn display_pin_code(&self, device_path: &str, pin_code: &str) -> Result<()> {
        info!("Displaying PIN code {} for device: {}", pin_code, device_path);

        self.request_tx.send(AgentRequest::DisplayPinCode {
            device_path: device_path.to_string(),
            pin_code: pin_code.to_string(),
        }).await?;

        Ok(())
    }

    /// Handle passkey request from BlueZ
    pub async fn request_passkey(&self, device_path: &str) -> Result<u32> {
        info!("Passkey requested for device: {}", device_path);

        let (tx, rx) = oneshot::channel();

        self.request_tx.send(AgentRequest::RequestPasskey {
            device_path: device_path.to_string(),
            response: tx,
        }).await?;

        match rx.await? {
            Some(passkey) => Ok(passkey),
            None => anyhow::bail!("Passkey entry cancelled"),
        }
    }

    /// Display passkey with entry progress
    pub async fn display_passkey(&self, device_path: &str, passkey: u32, entered: u16) -> Result<()> {
        debug!("Displaying passkey {} (entered: {}) for device: {}",
               passkey, entered, device_path);

        self.request_tx.send(AgentRequest::DisplayPasskey {
            device_path: device_path.to_string(),
            passkey,
            entered,
        }).await?;

        Ok(())
    }

    /// Request passkey confirmation
    pub async fn request_confirmation(&self, device_path: &str, passkey: u32) -> Result<()> {
        info!("Confirmation requested for passkey {} on device: {}", passkey, device_path);

        let (tx, rx) = oneshot::channel();

        self.request_tx.send(AgentRequest::RequestConfirmation {
            device_path: device_path.to_string(),
            passkey,
            response: tx,
        }).await?;

        if rx.await? {
            Ok(())
        } else {
            anyhow::bail!("Pairing rejected by user")
        }
    }

    /// Request authorization for incoming connection
    pub async fn request_authorization(&self, device_path: &str) -> Result<()> {
        info!("Authorization requested for device: {}", device_path);

        let (tx, rx) = oneshot::channel();

        self.request_tx.send(AgentRequest::RequestAuthorization {
            device_path: device_path.to_string(),
            response: tx,
        }).await?;

        if rx.await? {
            Ok(())
        } else {
            anyhow::bail!("Authorization rejected by user")
        }
    }

    /// Authorize a service
    pub async fn authorize_service(&self, device_path: &str, uuid: &str) -> Result<()> {
        info!("Service authorization requested for {} on device: {}", uuid, device_path);

        let (tx, rx) = oneshot::channel();

        self.request_tx.send(AgentRequest::AuthorizeService {
            device_path: device_path.to_string(),
            uuid: uuid.to_string(),
            response: tx,
        }).await?;

        if rx.await? {
            Ok(())
        } else {
            anyhow::bail!("Service authorization rejected by user")
        }
    }

    /// Cancel current pairing operation
    pub async fn cancel(&self) -> Result<()> {
        warn!("Pairing cancelled");
        self.request_tx.send(AgentRequest::Cancel).await?;
        Ok(())
    }

    /// Release agent (called when unregistering)
    pub fn release(&self) {
        info!("Pairing agent released");
    }
}

// D-Bus interface implementation for the agent
// This would be exposed to BlueZ to receive pairing requests

#[cfg(feature = "dbus")]
mod dbus_impl {
    use super::*;
    use zbus::interface;

    /// Agent1 D-Bus interface implementation
    pub struct Agent1 {
        agent: Arc<PairingAgent>,
    }

    impl Agent1 {
        pub fn new(agent: Arc<PairingAgent>) -> Self {
            Self { agent }
        }
    }

    #[interface(name = "org.bluez.Agent1")]
    impl Agent1 {
        /// Release the agent
        fn release(&self) {
            self.agent.release();
        }

        /// Request PIN code
        async fn request_pin_code(
            &self,
            device: zbus::zvariant::ObjectPath<'_>,
        ) -> zbus::fdo::Result<String> {
            self.agent
                .request_pin_code(device.as_str())
                .await
                .map_err(|e| zbus::fdo::Error::Failed(e.to_string()))
        }

        /// Display PIN code
        async fn display_pin_code(
            &self,
            device: zbus::zvariant::ObjectPath<'_>,
            pincode: &str,
        ) -> zbus::fdo::Result<()> {
            self.agent
                .display_pin_code(device.as_str(), pincode)
                .await
                .map_err(|e| zbus::fdo::Error::Failed(e.to_string()))
        }

        /// Request passkey
        async fn request_passkey(
            &self,
            device: zbus::zvariant::ObjectPath<'_>,
        ) -> zbus::fdo::Result<u32> {
            self.agent
                .request_passkey(device.as_str())
                .await
                .map_err(|e| zbus::fdo::Error::Failed(e.to_string()))
        }

        /// Display passkey
        async fn display_passkey(
            &self,
            device: zbus::zvariant::ObjectPath<'_>,
            passkey: u32,
            entered: u16,
        ) -> zbus::fdo::Result<()> {
            self.agent
                .display_passkey(device.as_str(), passkey, entered)
                .await
                .map_err(|e| zbus::fdo::Error::Failed(e.to_string()))
        }

        /// Request confirmation
        async fn request_confirmation(
            &self,
            device: zbus::zvariant::ObjectPath<'_>,
            passkey: u32,
        ) -> zbus::fdo::Result<()> {
            self.agent
                .request_confirmation(device.as_str(), passkey)
                .await
                .map_err(|e| zbus::fdo::Error::Failed(e.to_string()))
        }

        /// Request authorization
        async fn request_authorization(
            &self,
            device: zbus::zvariant::ObjectPath<'_>,
        ) -> zbus::fdo::Result<()> {
            self.agent
                .request_authorization(device.as_str())
                .await
                .map_err(|e| zbus::fdo::Error::Failed(e.to_string()))
        }

        /// Authorize service
        async fn authorize_service(
            &self,
            device: zbus::zvariant::ObjectPath<'_>,
            uuid: &str,
        ) -> zbus::fdo::Result<()> {
            self.agent
                .authorize_service(device.as_str(), uuid)
                .await
                .map_err(|e| zbus::fdo::Error::Failed(e.to_string()))
        }

        /// Cancel
        async fn cancel(&self) -> zbus::fdo::Result<()> {
            self.agent
                .cancel()
                .await
                .map_err(|e| zbus::fdo::Error::Failed(e.to_string()))
        }
    }
}
