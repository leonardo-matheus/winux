package com.winux.connect.data

import androidx.room.Entity
import androidx.room.PrimaryKey
import java.util.UUID

/**
 * Represents a paired Winux desktop device
 */
@Entity(tableName = "devices")
data class Device(
    @PrimaryKey
    val id: String = UUID.randomUUID().toString(),
    val name: String,
    val hostname: String,
    val ipAddress: String,
    val port: Int = 51820,
    val publicKey: String = "",
    val isPaired: Boolean = false,
    val lastConnected: Long = 0,
    val lastSeen: Long = System.currentTimeMillis(),
    val osVersion: String = "",
    val deviceType: DeviceType = DeviceType.DESKTOP,
    val capabilities: List<String> = emptyList()
)

enum class DeviceType {
    DESKTOP,
    LAPTOP,
    SERVER
}

/**
 * Connection state for a device
 */
enum class ConnectionState {
    DISCONNECTED,
    CONNECTING,
    CONNECTED,
    PAIRING,
    ERROR
}

/**
 * Device with runtime connection state
 */
data class DeviceWithState(
    val device: Device,
    val connectionState: ConnectionState = ConnectionState.DISCONNECTED,
    val batteryLevel: Int? = null,
    val isCharging: Boolean? = null,
    val errorMessage: String? = null
)
