package com.winux.connect.ui.components

import androidx.compose.animation.animateColorAsState
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.*
import androidx.compose.material.icons.outlined.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import com.winux.connect.data.ConnectionState
import com.winux.connect.data.Device
import com.winux.connect.data.DeviceType
import com.winux.connect.data.DeviceWithState
import com.winux.connect.ui.theme.*

/**
 * Card component displaying a device with its connection status
 */
@Composable
fun DeviceCard(
    deviceWithState: DeviceWithState,
    onClick: () -> Unit,
    onConnect: () -> Unit,
    onDisconnect: () -> Unit,
    modifier: Modifier = Modifier
) {
    val device = deviceWithState.device
    val connectionState = deviceWithState.connectionState

    val backgroundColor by animateColorAsState(
        targetValue = when (connectionState) {
            ConnectionState.CONNECTED -> MaterialTheme.colorScheme.primaryContainer
            ConnectionState.CONNECTING -> MaterialTheme.colorScheme.secondaryContainer
            else -> MaterialTheme.colorScheme.surfaceContainerHigh
        },
        label = "background"
    )

    Card(
        modifier = modifier
            .fillMaxWidth()
            .clickable(onClick = onClick),
        shape = RoundedCornerShape(16.dp),
        colors = CardDefaults.cardColors(containerColor = backgroundColor)
    ) {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .padding(16.dp),
            verticalAlignment = Alignment.CenterVertically
        ) {
            // Device Icon
            DeviceIcon(
                deviceType = device.deviceType,
                connectionState = connectionState,
                modifier = Modifier.size(48.dp)
            )

            Spacer(modifier = Modifier.width(16.dp))

            // Device Info
            Column(
                modifier = Modifier.weight(1f)
            ) {
                Text(
                    text = device.name,
                    style = MaterialTheme.typography.titleMedium,
                    fontWeight = FontWeight.SemiBold,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis
                )

                Spacer(modifier = Modifier.height(4.dp))

                Row(
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    ConnectionStatusIndicator(
                        state = connectionState,
                        size = 8.dp
                    )
                    Spacer(modifier = Modifier.width(8.dp))
                    Text(
                        text = getConnectionStateText(connectionState),
                        style = MaterialTheme.typography.bodySmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant
                    )
                }

                if (device.ipAddress.isNotEmpty()) {
                    Spacer(modifier = Modifier.height(2.dp))
                    Text(
                        text = device.ipAddress,
                        style = MaterialTheme.typography.bodySmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.7f)
                    )
                }

                // Battery info if available
                deviceWithState.batteryLevel?.let { battery ->
                    Spacer(modifier = Modifier.height(4.dp))
                    BatteryIndicator(
                        level = battery,
                        isCharging = deviceWithState.isCharging == true
                    )
                }
            }

            Spacer(modifier = Modifier.width(8.dp))

            // Connect/Disconnect Button
            when (connectionState) {
                ConnectionState.CONNECTED -> {
                    IconButton(onClick = onDisconnect) {
                        Icon(
                            imageVector = Icons.Filled.LinkOff,
                            contentDescription = "Disconnect",
                            tint = MaterialTheme.colorScheme.primary
                        )
                    }
                }
                ConnectionState.CONNECTING -> {
                    CircularProgressIndicator(
                        modifier = Modifier.size(24.dp),
                        strokeWidth = 2.dp,
                        color = MaterialTheme.colorScheme.secondary
                    )
                }
                else -> {
                    IconButton(onClick = onConnect) {
                        Icon(
                            imageVector = Icons.Filled.Link,
                            contentDescription = "Connect",
                            tint = MaterialTheme.colorScheme.primary
                        )
                    }
                }
            }
        }

        // Paired indicator
        if (device.isPaired) {
            Row(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(horizontal = 16.dp)
                    .padding(bottom = 12.dp),
                verticalAlignment = Alignment.CenterVertically
            ) {
                Icon(
                    imageVector = Icons.Outlined.Verified,
                    contentDescription = "Paired",
                    modifier = Modifier.size(14.dp),
                    tint = WinuxGreen
                )
                Spacer(modifier = Modifier.width(4.dp))
                Text(
                    text = "Paired",
                    style = MaterialTheme.typography.labelSmall,
                    color = WinuxGreen
                )
            }
        }
    }
}

/**
 * Device icon based on device type
 */
@Composable
fun DeviceIcon(
    deviceType: DeviceType,
    connectionState: ConnectionState,
    modifier: Modifier = Modifier
) {
    val icon = when (deviceType) {
        DeviceType.DESKTOP -> Icons.Outlined.Computer
        DeviceType.LAPTOP -> Icons.Outlined.Laptop
        DeviceType.SERVER -> Icons.Outlined.Storage
    }

    val iconColor = when (connectionState) {
        ConnectionState.CONNECTED -> WinuxCyan
        ConnectionState.CONNECTING -> WinuxMagenta
        else -> MaterialTheme.colorScheme.onSurfaceVariant
    }

    Box(
        modifier = modifier
            .clip(CircleShape)
            .background(MaterialTheme.colorScheme.surfaceVariant),
        contentAlignment = Alignment.Center
    ) {
        Icon(
            imageVector = icon,
            contentDescription = deviceType.name,
            tint = iconColor,
            modifier = Modifier.size(28.dp)
        )
    }
}

/**
 * Battery level indicator
 */
@Composable
fun BatteryIndicator(
    level: Int,
    isCharging: Boolean,
    modifier: Modifier = Modifier
) {
    Row(
        modifier = modifier,
        verticalAlignment = Alignment.CenterVertically
    ) {
        val batteryIcon = when {
            isCharging -> Icons.Outlined.BatteryChargingFull
            level >= 90 -> Icons.Outlined.BatteryFull
            level >= 50 -> Icons.Outlined.Battery5Bar
            level >= 20 -> Icons.Outlined.Battery3Bar
            else -> Icons.Outlined.Battery1Bar
        }

        val batteryColor = when {
            level <= 15 -> ErrorColor
            level <= 30 -> WarningColor
            else -> WinuxGreen
        }

        Icon(
            imageVector = batteryIcon,
            contentDescription = "Battery",
            modifier = Modifier.size(16.dp),
            tint = batteryColor
        )
        Spacer(modifier = Modifier.width(4.dp))
        Text(
            text = "$level%",
            style = MaterialTheme.typography.bodySmall,
            color = batteryColor
        )
    }
}

/**
 * Get human-readable connection state text
 */
private fun getConnectionStateText(state: ConnectionState): String {
    return when (state) {
        ConnectionState.CONNECTED -> "Connected"
        ConnectionState.CONNECTING -> "Connecting..."
        ConnectionState.PAIRING -> "Pairing..."
        ConnectionState.DISCONNECTED -> "Disconnected"
        ConnectionState.ERROR -> "Connection Error"
    }
}
