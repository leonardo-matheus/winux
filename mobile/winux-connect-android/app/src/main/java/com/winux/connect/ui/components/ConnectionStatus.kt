package com.winux.connect.ui.components

import androidx.compose.animation.animateColorAsState
import androidx.compose.animation.core.*
import androidx.compose.foundation.background
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
import androidx.compose.ui.draw.scale
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import com.winux.connect.data.ConnectionState
import com.winux.connect.data.Device
import com.winux.connect.ui.theme.*

/**
 * Connection status indicator dot with animation
 */
@Composable
fun ConnectionStatusIndicator(
    state: ConnectionState,
    modifier: Modifier = Modifier,
    size: Dp = 12.dp
) {
    val color by animateColorAsState(
        targetValue = when (state) {
            ConnectionState.CONNECTED -> ConnectedColor
            ConnectionState.CONNECTING, ConnectionState.PAIRING -> ConnectingColor
            ConnectionState.ERROR -> ErrorColor
            ConnectionState.DISCONNECTED -> DisconnectedColor
        },
        label = "status_color"
    )

    // Pulsing animation for connecting state
    val infiniteTransition = rememberInfiniteTransition(label = "pulse")
    val scale by infiniteTransition.animateFloat(
        initialValue = 1f,
        targetValue = if (state == ConnectionState.CONNECTING || state == ConnectionState.PAIRING) 1.3f else 1f,
        animationSpec = infiniteRepeatable(
            animation = tween(800, easing = FastOutSlowInEasing),
            repeatMode = RepeatMode.Reverse
        ),
        label = "scale"
    )

    Box(
        modifier = modifier
            .size(size)
            .scale(scale)
            .clip(CircleShape)
            .background(color)
    )
}

/**
 * Connection status banner shown at the top of screens
 */
@Composable
fun ConnectionStatusBanner(
    state: ConnectionState,
    deviceName: String?,
    onDisconnect: () -> Unit,
    modifier: Modifier = Modifier
) {
    val backgroundColor by animateColorAsState(
        targetValue = when (state) {
            ConnectionState.CONNECTED -> WinuxCyan.copy(alpha = 0.15f)
            ConnectionState.CONNECTING, ConnectionState.PAIRING -> WinuxMagenta.copy(alpha = 0.15f)
            ConnectionState.ERROR -> ErrorColor.copy(alpha = 0.15f)
            ConnectionState.DISCONNECTED -> Color.Transparent
        },
        label = "banner_bg"
    )

    if (state == ConnectionState.DISCONNECTED) return

    Card(
        modifier = modifier.fillMaxWidth(),
        shape = RoundedCornerShape(12.dp),
        colors = CardDefaults.cardColors(containerColor = backgroundColor)
    ) {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .padding(horizontal = 16.dp, vertical = 12.dp),
            verticalAlignment = Alignment.CenterVertically
        ) {
            ConnectionStatusIndicator(state = state, size = 10.dp)

            Spacer(modifier = Modifier.width(12.dp))

            Column(modifier = Modifier.weight(1f)) {
                Text(
                    text = when (state) {
                        ConnectionState.CONNECTED -> "Connected"
                        ConnectionState.CONNECTING -> "Connecting..."
                        ConnectionState.PAIRING -> "Pairing..."
                        ConnectionState.ERROR -> "Connection Error"
                        ConnectionState.DISCONNECTED -> ""
                    },
                    style = MaterialTheme.typography.labelLarge,
                    fontWeight = FontWeight.Medium
                )

                deviceName?.let { name ->
                    Text(
                        text = name,
                        style = MaterialTheme.typography.bodySmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant
                    )
                }
            }

            if (state == ConnectionState.CONNECTED) {
                IconButton(onClick = onDisconnect) {
                    Icon(
                        imageVector = Icons.Filled.Close,
                        contentDescription = "Disconnect",
                        modifier = Modifier.size(20.dp)
                    )
                }
            }

            if (state == ConnectionState.CONNECTING || state == ConnectionState.PAIRING) {
                CircularProgressIndicator(
                    modifier = Modifier.size(20.dp),
                    strokeWidth = 2.dp
                )
            }
        }
    }
}

/**
 * Large connection status display for home screen
 */
@Composable
fun ConnectionStatusLarge(
    state: ConnectionState,
    device: Device?,
    modifier: Modifier = Modifier
) {
    Column(
        modifier = modifier,
        horizontalAlignment = Alignment.CenterHorizontally
    ) {
        // Large status circle
        Box(
            modifier = Modifier
                .size(120.dp)
                .clip(CircleShape)
                .background(
                    when (state) {
                        ConnectionState.CONNECTED -> WinuxCyan.copy(alpha = 0.2f)
                        ConnectionState.CONNECTING, ConnectionState.PAIRING -> WinuxMagenta.copy(alpha = 0.2f)
                        else -> MaterialTheme.colorScheme.surfaceVariant
                    }
                ),
            contentAlignment = Alignment.Center
        ) {
            Box(
                modifier = Modifier
                    .size(80.dp)
                    .clip(CircleShape)
                    .background(
                        when (state) {
                            ConnectionState.CONNECTED -> WinuxCyan.copy(alpha = 0.4f)
                            ConnectionState.CONNECTING, ConnectionState.PAIRING -> WinuxMagenta.copy(alpha = 0.4f)
                            else -> MaterialTheme.colorScheme.surfaceContainerHigh
                        }
                    ),
                contentAlignment = Alignment.Center
            ) {
                when (state) {
                    ConnectionState.CONNECTED -> {
                        Icon(
                            imageVector = Icons.Filled.Check,
                            contentDescription = "Connected",
                            tint = WinuxCyan,
                            modifier = Modifier.size(40.dp)
                        )
                    }
                    ConnectionState.CONNECTING, ConnectionState.PAIRING -> {
                        CircularProgressIndicator(
                            modifier = Modifier.size(40.dp),
                            color = WinuxMagenta,
                            strokeWidth = 3.dp
                        )
                    }
                    ConnectionState.ERROR -> {
                        Icon(
                            imageVector = Icons.Filled.ErrorOutline,
                            contentDescription = "Error",
                            tint = ErrorColor,
                            modifier = Modifier.size(40.dp)
                        )
                    }
                    ConnectionState.DISCONNECTED -> {
                        Icon(
                            imageVector = Icons.Outlined.Computer,
                            contentDescription = "Disconnected",
                            tint = MaterialTheme.colorScheme.onSurfaceVariant,
                            modifier = Modifier.size(40.dp)
                        )
                    }
                }
            }
        }

        Spacer(modifier = Modifier.height(24.dp))

        // Status text
        Text(
            text = when (state) {
                ConnectionState.CONNECTED -> "Connected"
                ConnectionState.CONNECTING -> "Connecting..."
                ConnectionState.PAIRING -> "Pairing..."
                ConnectionState.ERROR -> "Connection Failed"
                ConnectionState.DISCONNECTED -> "Not Connected"
            },
            style = MaterialTheme.typography.headlineSmall,
            fontWeight = FontWeight.SemiBold,
            color = when (state) {
                ConnectionState.CONNECTED -> WinuxCyan
                ConnectionState.CONNECTING, ConnectionState.PAIRING -> WinuxMagenta
                ConnectionState.ERROR -> ErrorColor
                ConnectionState.DISCONNECTED -> MaterialTheme.colorScheme.onSurfaceVariant
            }
        )

        device?.let {
            Spacer(modifier = Modifier.height(8.dp))
            Text(
                text = it.name,
                style = MaterialTheme.typography.bodyLarge,
                color = MaterialTheme.colorScheme.onSurfaceVariant
            )
            Text(
                text = it.ipAddress,
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.7f)
            )
        }
    }
}

/**
 * Quick action button for connection features
 */
@Composable
fun QuickActionButton(
    icon: androidx.compose.ui.graphics.vector.ImageVector,
    label: String,
    onClick: () -> Unit,
    enabled: Boolean = true,
    modifier: Modifier = Modifier
) {
    Column(
        modifier = modifier,
        horizontalAlignment = Alignment.CenterHorizontally
    ) {
        FilledIconButton(
            onClick = onClick,
            enabled = enabled,
            modifier = Modifier.size(56.dp),
            colors = IconButtonDefaults.filledIconButtonColors(
                containerColor = MaterialTheme.colorScheme.primaryContainer,
                contentColor = MaterialTheme.colorScheme.primary,
                disabledContainerColor = MaterialTheme.colorScheme.surfaceVariant,
                disabledContentColor = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.5f)
            )
        ) {
            Icon(
                imageVector = icon,
                contentDescription = label,
                modifier = Modifier.size(24.dp)
            )
        }

        Spacer(modifier = Modifier.height(8.dp))

        Text(
            text = label,
            style = MaterialTheme.typography.labelSmall,
            color = if (enabled) {
                MaterialTheme.colorScheme.onSurface
            } else {
                MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.5f)
            }
        )
    }
}
