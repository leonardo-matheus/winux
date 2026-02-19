//! KDE Connect compatible JSON message protocol
//!
//! Implements the network packet format used by KDE Connect
//! for communication between devices.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Protocol version
pub const PROTOCOL_VERSION: u32 = 7;

/// Network packet - the basic unit of communication
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NetworkPacket {
    /// Packet ID (timestamp in milliseconds)
    pub id: i64,

    /// Packet type (e.g., "kdeconnect.ping")
    #[serde(rename = "type")]
    pub packet_type: String,

    /// Packet body (JSON object)
    pub body: HashMap<String, serde_json::Value>,

    /// Optional payload info for file transfers
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload_size: Option<i64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload_transfer_info: Option<PayloadTransferInfo>,
}

impl NetworkPacket {
    /// Create a new packet with the given type
    pub fn new(packet_type: PacketType) -> Self {
        Self {
            id: chrono::Utc::now().timestamp_millis(),
            packet_type: packet_type.as_str().to_string(),
            body: HashMap::new(),
            payload_size: None,
            payload_transfer_info: None,
        }
    }

    /// Create a ping packet
    pub fn ping() -> Self {
        Self::new(PacketType::Ping)
    }

    /// Create a battery status packet
    pub fn battery(level: u8, charging: bool) -> Self {
        let mut packet = Self::new(PacketType::Battery);
        packet.body.insert("currentCharge".to_string(), serde_json::json!(level));
        packet.body.insert("isCharging".to_string(), serde_json::json!(charging));
        packet.body.insert("thresholdEvent".to_string(), serde_json::json!(0));
        packet
    }

    /// Create a battery request packet
    pub fn battery_request() -> Self {
        let mut packet = Self::new(PacketType::BatteryRequest);
        packet.body.insert("request".to_string(), serde_json::json!(true));
        packet
    }

    /// Create a clipboard packet
    pub fn clipboard(content: &str) -> Self {
        let mut packet = Self::new(PacketType::Clipboard);
        packet.body.insert("content".to_string(), serde_json::json!(content));
        packet
    }

    /// Create a notification packet
    pub fn notification(
        id: &str,
        app_name: &str,
        title: &str,
        text: &str,
        dismissible: bool,
    ) -> Self {
        let mut packet = Self::new(PacketType::Notification);
        packet.body.insert("id".to_string(), serde_json::json!(id));
        packet.body.insert("appName".to_string(), serde_json::json!(app_name));
        packet.body.insert("title".to_string(), serde_json::json!(title));
        packet.body.insert("text".to_string(), serde_json::json!(text));
        packet.body.insert("ticker".to_string(), serde_json::json!(format!("{}: {}", title, text)));
        packet.body.insert("isClearable".to_string(), serde_json::json!(dismissible));
        packet
    }

    /// Create a notification dismiss request
    pub fn notification_dismiss(id: &str) -> Self {
        let mut packet = Self::new(PacketType::NotificationRequest);
        packet.body.insert("cancel".to_string(), serde_json::json!(id));
        packet
    }

    /// Create a share request packet (file transfer)
    pub fn share_file(filename: &str, size: i64) -> Self {
        let mut packet = Self::new(PacketType::Share);
        packet.body.insert("filename".to_string(), serde_json::json!(filename));
        packet.payload_size = Some(size);
        packet
    }

    /// Create a share URL packet
    pub fn share_url(url: &str) -> Self {
        let mut packet = Self::new(PacketType::Share);
        packet.body.insert("url".to_string(), serde_json::json!(url));
        packet
    }

    /// Create a share text packet
    pub fn share_text(text: &str) -> Self {
        let mut packet = Self::new(PacketType::Share);
        packet.body.insert("text".to_string(), serde_json::json!(text));
        packet
    }

    /// Create an MPRIS packet for media control
    pub fn mpris_action(player: &str, action: MprisAction) -> Self {
        let mut packet = Self::new(PacketType::Mpris);
        packet.body.insert("player".to_string(), serde_json::json!(player));
        packet.body.insert("action".to_string(), serde_json::json!(action.as_str()));
        packet
    }

    /// Create a find my phone request
    pub fn find_phone() -> Self {
        Self::new(PacketType::FindMyPhone)
    }

    /// Create an SMS request packet
    pub fn sms_request() -> Self {
        let mut packet = Self::new(PacketType::SmsRequest);
        packet.body.insert("request".to_string(), serde_json::json!(true));
        packet
    }

    /// Create an SMS send packet
    pub fn sms_send(phone_number: &str, message: &str) -> Self {
        let mut packet = Self::new(PacketType::SmsSend);

        let message_obj = serde_json::json!({
            "address": phone_number,
            "body": message
        });

        packet.body.insert("messages".to_string(), serde_json::json!([message_obj]));
        packet
    }

    /// Create a pairing request packet
    pub fn pair_request(pair: bool) -> Self {
        let mut packet = Self::new(PacketType::Pair);
        packet.body.insert("pair".to_string(), serde_json::json!(pair));
        packet
    }

    /// Serialize to JSON string
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Deserialize from JSON string
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Get a body field as string
    pub fn get_string(&self, key: &str) -> Option<String> {
        self.body.get(key)?.as_str().map(|s| s.to_string())
    }

    /// Get a body field as i64
    pub fn get_i64(&self, key: &str) -> Option<i64> {
        self.body.get(key)?.as_i64()
    }

    /// Get a body field as bool
    pub fn get_bool(&self, key: &str) -> Option<bool> {
        self.body.get(key)?.as_bool()
    }
}

/// Payload transfer information for file transfers
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PayloadTransferInfo {
    pub port: u16,
}

/// Packet types supported by the protocol
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PacketType {
    // Identity and pairing
    Identity,
    Pair,

    // Basic features
    Ping,
    Battery,
    BatteryRequest,
    Clipboard,
    ClipboardConnect,

    // Notifications
    Notification,
    NotificationRequest,
    NotificationReply,

    // File sharing
    Share,
    ShareRequest,

    // Media control
    Mpris,
    MprisRequest,

    // Find my phone
    FindMyPhone,
    FindMyPhoneRequest,

    // SMS
    Sms,
    SmsRequest,
    SmsSend,

    // Telephony
    Telephony,
    TelephonyRequest,

    // Connectivity report
    ConnectivityReport,
    ConnectivityReportRequest,

    // Contacts
    Contacts,
    ContactsRequest,

    // Unknown/custom
    Unknown(String),
}

impl PacketType {
    /// Get the string representation of the packet type
    pub fn as_str(&self) -> &str {
        match self {
            Self::Identity => "kdeconnect.identity",
            Self::Pair => "kdeconnect.pair",
            Self::Ping => "kdeconnect.ping",
            Self::Battery => "kdeconnect.battery",
            Self::BatteryRequest => "kdeconnect.battery.request",
            Self::Clipboard => "kdeconnect.clipboard",
            Self::ClipboardConnect => "kdeconnect.clipboard.connect",
            Self::Notification => "kdeconnect.notification",
            Self::NotificationRequest => "kdeconnect.notification.request",
            Self::NotificationReply => "kdeconnect.notification.reply",
            Self::Share => "kdeconnect.share.request",
            Self::ShareRequest => "kdeconnect.share.request.update",
            Self::Mpris => "kdeconnect.mpris",
            Self::MprisRequest => "kdeconnect.mpris.request",
            Self::FindMyPhone => "kdeconnect.findmyphone.request",
            Self::FindMyPhoneRequest => "kdeconnect.findmyphone",
            Self::Sms => "kdeconnect.sms.messages",
            Self::SmsRequest => "kdeconnect.sms.request",
            Self::SmsSend => "kdeconnect.sms.request",
            Self::Telephony => "kdeconnect.telephony",
            Self::TelephonyRequest => "kdeconnect.telephony.request",
            Self::ConnectivityReport => "kdeconnect.connectivity_report",
            Self::ConnectivityReportRequest => "kdeconnect.connectivity_report.request",
            Self::Contacts => "kdeconnect.contacts.response_vcards",
            Self::ContactsRequest => "kdeconnect.contacts.request_all_uids_timestamps",
            Self::Unknown(s) => s.as_str(),
        }
    }

    /// Parse packet type from string
    pub fn from_str(s: &str) -> Self {
        match s {
            "kdeconnect.identity" => Self::Identity,
            "kdeconnect.pair" => Self::Pair,
            "kdeconnect.ping" => Self::Ping,
            "kdeconnect.battery" => Self::Battery,
            "kdeconnect.battery.request" => Self::BatteryRequest,
            "kdeconnect.clipboard" => Self::Clipboard,
            "kdeconnect.clipboard.connect" => Self::ClipboardConnect,
            "kdeconnect.notification" => Self::Notification,
            "kdeconnect.notification.request" => Self::NotificationRequest,
            "kdeconnect.notification.reply" => Self::NotificationReply,
            "kdeconnect.share.request" => Self::Share,
            "kdeconnect.share.request.update" => Self::ShareRequest,
            "kdeconnect.mpris" => Self::Mpris,
            "kdeconnect.mpris.request" => Self::MprisRequest,
            "kdeconnect.findmyphone.request" => Self::FindMyPhone,
            "kdeconnect.findmyphone" => Self::FindMyPhoneRequest,
            "kdeconnect.sms.messages" => Self::Sms,
            "kdeconnect.sms.request" => Self::SmsRequest,
            "kdeconnect.telephony" => Self::Telephony,
            "kdeconnect.telephony.request" => Self::TelephonyRequest,
            "kdeconnect.connectivity_report" => Self::ConnectivityReport,
            "kdeconnect.connectivity_report.request" => Self::ConnectivityReportRequest,
            "kdeconnect.contacts.response_vcards" => Self::Contacts,
            "kdeconnect.contacts.request_all_uids_timestamps" => Self::ContactsRequest,
            _ => Self::Unknown(s.to_string()),
        }
    }
}

/// MPRIS media control actions
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MprisAction {
    Play,
    Pause,
    PlayPause,
    Stop,
    Next,
    Previous,
    Seek,
    SetVolume,
}

impl MprisAction {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Play => "Play",
            Self::Pause => "Pause",
            Self::PlayPause => "PlayPause",
            Self::Stop => "Stop",
            Self::Next => "Next",
            Self::Previous => "Previous",
            Self::Seek => "Seek",
            Self::SetVolume => "SetVolume",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ping_packet() {
        let packet = NetworkPacket::ping();
        assert_eq!(packet.packet_type, "kdeconnect.ping");
    }

    #[test]
    fn test_battery_packet() {
        let packet = NetworkPacket::battery(85, true);
        assert_eq!(packet.packet_type, "kdeconnect.battery");
        assert_eq!(packet.get_i64("currentCharge"), Some(85));
        assert_eq!(packet.get_bool("isCharging"), Some(true));
    }

    #[test]
    fn test_clipboard_packet() {
        let packet = NetworkPacket::clipboard("Hello, World!");
        assert_eq!(packet.packet_type, "kdeconnect.clipboard");
        assert_eq!(packet.get_string("content"), Some("Hello, World!".to_string()));
    }

    #[test]
    fn test_packet_serialization() {
        let packet = NetworkPacket::ping();
        let json = packet.to_json().unwrap();
        assert!(json.contains("kdeconnect.ping"));

        let parsed = NetworkPacket::from_json(&json).unwrap();
        assert_eq!(parsed.packet_type, packet.packet_type);
    }

    #[test]
    fn test_notification_packet() {
        let packet = NetworkPacket::notification(
            "1",
            "WhatsApp",
            "Maria",
            "Oi!",
            true,
        );
        assert_eq!(packet.get_string("appName"), Some("WhatsApp".to_string()));
        assert_eq!(packet.get_string("title"), Some("Maria".to_string()));
    }

    #[test]
    fn test_sms_send_packet() {
        let packet = NetworkPacket::sms_send("+5511999991234", "Hello!");
        let json = packet.to_json().unwrap();
        assert!(json.contains("+5511999991234"));
        assert!(json.contains("Hello!"));
    }
}
