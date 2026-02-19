package com.winux.connect.data

import com.google.gson.Gson
import com.google.gson.annotations.SerializedName
import java.util.UUID

/**
 * Protocol message types for Winux Connect
 */
enum class MessageType {
    // Connection
    @SerializedName("hello") HELLO,
    @SerializedName("pair_request") PAIR_REQUEST,
    @SerializedName("pair_response") PAIR_RESPONSE,
    @SerializedName("pair_confirm") PAIR_CONFIRM,
    @SerializedName("disconnect") DISCONNECT,
    @SerializedName("ping") PING,
    @SerializedName("pong") PONG,

    // Notifications
    @SerializedName("notification") NOTIFICATION,
    @SerializedName("notification_action") NOTIFICATION_ACTION,
    @SerializedName("notification_dismiss") NOTIFICATION_DISMISS,

    // Clipboard
    @SerializedName("clipboard_content") CLIPBOARD_CONTENT,
    @SerializedName("clipboard_request") CLIPBOARD_REQUEST,

    // Files
    @SerializedName("file_transfer_request") FILE_TRANSFER_REQUEST,
    @SerializedName("file_transfer_accept") FILE_TRANSFER_ACCEPT,
    @SerializedName("file_transfer_reject") FILE_TRANSFER_REJECT,
    @SerializedName("file_transfer_progress") FILE_TRANSFER_PROGRESS,
    @SerializedName("file_transfer_complete") FILE_TRANSFER_COMPLETE,
    @SerializedName("file_transfer_error") FILE_TRANSFER_ERROR,

    // Media Control
    @SerializedName("media_control") MEDIA_CONTROL,
    @SerializedName("media_state") MEDIA_STATE,

    // Commands
    @SerializedName("command_ring") COMMAND_RING,
    @SerializedName("command_find_phone") COMMAND_FIND_PHONE,
    @SerializedName("command_lock") COMMAND_LOCK,
    @SerializedName("command_screenshot") COMMAND_SCREENSHOT,

    // Battery
    @SerializedName("battery_status") BATTERY_STATUS,
    @SerializedName("battery_request") BATTERY_REQUEST,

    // Capabilities
    @SerializedName("capabilities") CAPABILITIES
}

/**
 * Base message structure for Winux Protocol
 */
data class WinuxMessage(
    val id: String = UUID.randomUUID().toString(),
    val type: MessageType,
    val timestamp: Long = System.currentTimeMillis(),
    val payload: Map<String, Any?> = emptyMap()
) {
    companion object {
        private val gson = Gson()

        fun fromJson(json: String): WinuxMessage? {
            return try {
                gson.fromJson(json, WinuxMessage::class.java)
            } catch (e: Exception) {
                null
            }
        }
    }

    fun toJson(): String = gson.toJson(this)
}

/**
 * Notification payload
 */
data class NotificationPayload(
    val id: String,
    val appName: String,
    val appPackage: String,
    val title: String,
    val text: String,
    val icon: String? = null, // Base64 encoded
    val timestamp: Long = System.currentTimeMillis(),
    val actions: List<NotificationAction> = emptyList(),
    val isOngoing: Boolean = false,
    val priority: Int = 0
)

data class NotificationAction(
    val id: String,
    val label: String
)

/**
 * File transfer payload
 */
data class FileTransferPayload(
    val transferId: String = UUID.randomUUID().toString(),
    val fileName: String,
    val fileSize: Long,
    val mimeType: String,
    val checksum: String? = null
)

/**
 * Media control payload
 */
data class MediaControlPayload(
    val action: MediaAction,
    val value: Int? = null // For volume/seek
)

enum class MediaAction {
    @SerializedName("play") PLAY,
    @SerializedName("pause") PAUSE,
    @SerializedName("play_pause") PLAY_PAUSE,
    @SerializedName("next") NEXT,
    @SerializedName("previous") PREVIOUS,
    @SerializedName("stop") STOP,
    @SerializedName("volume_up") VOLUME_UP,
    @SerializedName("volume_down") VOLUME_DOWN,
    @SerializedName("volume_set") VOLUME_SET,
    @SerializedName("mute") MUTE,
    @SerializedName("seek") SEEK
}

/**
 * Battery status payload
 */
data class BatteryPayload(
    val level: Int,
    val isCharging: Boolean,
    val chargingType: String? = null // USB, AC, Wireless
)

/**
 * Clipboard payload
 */
data class ClipboardPayload(
    val content: String,
    val mimeType: String = "text/plain"
)

/**
 * Capabilities payload
 */
data class CapabilitiesPayload(
    val deviceName: String,
    val deviceType: String,
    val osVersion: String,
    val appVersion: String,
    val capabilities: List<String>
)

/**
 * Pairing payload
 */
data class PairingPayload(
    val publicKey: String,
    val pin: String? = null,
    val deviceName: String,
    val deviceType: String
)
