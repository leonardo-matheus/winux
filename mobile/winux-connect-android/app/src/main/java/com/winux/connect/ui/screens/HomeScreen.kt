package com.winux.connect.ui.screens

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.*
import androidx.compose.material.icons.outlined.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import com.winux.connect.data.ConnectionState
import com.winux.connect.data.MediaAction
import com.winux.connect.ui.components.ConnectionStatusLarge
import com.winux.connect.ui.components.QuickActionButton
import com.winux.connect.ui.theme.WinuxCyan
import com.winux.connect.ui.viewmodel.HomeViewModel

/**
 * Home screen showing connection status and quick actions
 */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun HomeScreen(
    onNavigateToDevices: () -> Unit,
    onNavigateToPairing: () -> Unit,
    viewModel: HomeViewModel = hiltViewModel()
) {
    val connectionState by viewModel.connectionState.collectAsState()
    val connectedDevice by viewModel.connectedDevice.collectAsState()
    val batteryStatus by viewModel.batteryStatus.collectAsState()

    val isConnected = connectionState == ConnectionState.CONNECTED

    Scaffold(
        topBar = {
            TopAppBar(
                title = {
                    Text(
                        text = "Winux Connect",
                        fontWeight = FontWeight.Bold
                    )
                },
                actions = {
                    IconButton(onClick = onNavigateToDevices) {
                        Icon(
                            imageVector = Icons.Outlined.Devices,
                            contentDescription = "Devices"
                        )
                    }
                }
            )
        }
    ) { padding ->
        Column(
            modifier = Modifier
                .fillMaxSize()
                .padding(padding)
                .verticalScroll(rememberScrollState()),
            horizontalAlignment = Alignment.CenterHorizontally
        ) {
            Spacer(modifier = Modifier.height(32.dp))

            // Connection Status
            ConnectionStatusLarge(
                state = connectionState,
                device = connectedDevice
            )

            Spacer(modifier = Modifier.height(32.dp))

            // Connect/Disconnect Button
            if (connectionState == ConnectionState.DISCONNECTED) {
                Button(
                    onClick = onNavigateToPairing,
                    modifier = Modifier
                        .padding(horizontal = 32.dp)
                        .fillMaxWidth()
                        .height(56.dp)
                ) {
                    Icon(
                        imageVector = Icons.Outlined.AddLink,
                        contentDescription = null,
                        modifier = Modifier.size(20.dp)
                    )
                    Spacer(modifier = Modifier.width(8.dp))
                    Text("Connect to Desktop")
                }
            } else if (connectionState == ConnectionState.CONNECTED) {
                OutlinedButton(
                    onClick = { viewModel.disconnect() },
                    modifier = Modifier
                        .padding(horizontal = 32.dp)
                        .fillMaxWidth()
                        .height(56.dp)
                ) {
                    Icon(
                        imageVector = Icons.Outlined.LinkOff,
                        contentDescription = null,
                        modifier = Modifier.size(20.dp)
                    )
                    Spacer(modifier = Modifier.width(8.dp))
                    Text("Disconnect")
                }
            }

            Spacer(modifier = Modifier.height(32.dp))

            // Quick Actions
            Text(
                text = "Quick Actions",
                style = MaterialTheme.typography.titleMedium,
                fontWeight = FontWeight.SemiBold,
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(horizontal = 24.dp)
            )

            Spacer(modifier = Modifier.height(16.dp))

            Row(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(horizontal = 24.dp),
                horizontalArrangement = Arrangement.SpaceEvenly
            ) {
                QuickActionButton(
                    icon = Icons.Outlined.ContentPaste,
                    label = "Clipboard",
                    enabled = isConnected,
                    onClick = { viewModel.syncClipboard() }
                )

                QuickActionButton(
                    icon = Icons.Outlined.PhoneAndroid,
                    label = "Ring PC",
                    enabled = isConnected,
                    onClick = { viewModel.ringPC() }
                )

                QuickActionButton(
                    icon = Icons.Outlined.Screenshot,
                    label = "Screenshot",
                    enabled = isConnected,
                    onClick = { viewModel.requestScreenshot() }
                )

                QuickActionButton(
                    icon = Icons.Outlined.Lock,
                    label = "Lock PC",
                    enabled = isConnected,
                    onClick = { viewModel.lockPC() }
                )
            }

            Spacer(modifier = Modifier.height(32.dp))

            // Media Controls
            if (isConnected) {
                MediaControlsCard(
                    onPlayPause = { viewModel.sendMediaControl(MediaAction.PLAY_PAUSE) },
                    onPrevious = { viewModel.sendMediaControl(MediaAction.PREVIOUS) },
                    onNext = { viewModel.sendMediaControl(MediaAction.NEXT) },
                    onVolumeUp = { viewModel.sendMediaControl(MediaAction.VOLUME_UP) },
                    onVolumeDown = { viewModel.sendMediaControl(MediaAction.VOLUME_DOWN) }
                )
            }

            Spacer(modifier = Modifier.height(24.dp))
        }
    }
}

/**
 * Media controls card
 */
@Composable
fun MediaControlsCard(
    onPlayPause: () -> Unit,
    onPrevious: () -> Unit,
    onNext: () -> Unit,
    onVolumeUp: () -> Unit,
    onVolumeDown: () -> Unit
) {
    Card(
        modifier = Modifier
            .fillMaxWidth()
            .padding(horizontal = 24.dp)
    ) {
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .padding(16.dp)
        ) {
            Text(
                text = "Media Controls",
                style = MaterialTheme.typography.titleSmall,
                fontWeight = FontWeight.SemiBold
            )

            Spacer(modifier = Modifier.height(16.dp))

            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceEvenly,
                verticalAlignment = Alignment.CenterVertically
            ) {
                IconButton(onClick = onVolumeDown) {
                    Icon(
                        imageVector = Icons.Filled.VolumeDown,
                        contentDescription = "Volume Down"
                    )
                }

                IconButton(onClick = onPrevious) {
                    Icon(
                        imageVector = Icons.Filled.SkipPrevious,
                        contentDescription = "Previous",
                        modifier = Modifier.size(32.dp)
                    )
                }

                FilledIconButton(
                    onClick = onPlayPause,
                    modifier = Modifier.size(56.dp),
                    colors = IconButtonDefaults.filledIconButtonColors(
                        containerColor = WinuxCyan
                    )
                ) {
                    Icon(
                        imageVector = Icons.Filled.PlayArrow,
                        contentDescription = "Play/Pause",
                        modifier = Modifier.size(32.dp)
                    )
                }

                IconButton(onClick = onNext) {
                    Icon(
                        imageVector = Icons.Filled.SkipNext,
                        contentDescription = "Next",
                        modifier = Modifier.size(32.dp)
                    )
                }

                IconButton(onClick = onVolumeUp) {
                    Icon(
                        imageVector = Icons.Filled.VolumeUp,
                        contentDescription = "Volume Up"
                    )
                }
            }
        }
    }
}
