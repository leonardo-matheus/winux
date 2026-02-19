package com.winux.connect.ui.screens

import androidx.compose.animation.*
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.*
import androidx.compose.material.icons.outlined.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import com.winux.connect.data.ConnectionState
import com.winux.connect.data.Device
import com.winux.connect.data.DeviceWithState
import com.winux.connect.ui.components.DeviceCard
import com.winux.connect.ui.theme.WinuxCyan
import com.winux.connect.ui.viewmodel.DevicesViewModel

/**
 * Screen showing paired and discovered devices
 */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun DevicesScreen(
    onNavigateToPairing: () -> Unit,
    viewModel: DevicesViewModel = hiltViewModel()
) {
    val pairedDevices by viewModel.pairedDevices.collectAsState()
    val discoveredDevices by viewModel.discoveredDevices.collectAsState()
    val isDiscovering by viewModel.isDiscovering.collectAsState()
    val connectionState by viewModel.connectionState.collectAsState()
    val connectedDeviceId by viewModel.connectedDeviceId.collectAsState()

    LaunchedEffect(Unit) {
        viewModel.startDiscovery()
    }

    DisposableEffect(Unit) {
        onDispose {
            viewModel.stopDiscovery()
        }
    }

    Scaffold(
        topBar = {
            TopAppBar(
                title = {
                    Text(
                        text = "Devices",
                        fontWeight = FontWeight.Bold
                    )
                },
                actions = {
                    if (isDiscovering) {
                        CircularProgressIndicator(
                            modifier = Modifier
                                .size(24.dp)
                                .padding(end = 16.dp),
                            strokeWidth = 2.dp
                        )
                    } else {
                        IconButton(onClick = { viewModel.startDiscovery() }) {
                            Icon(
                                imageVector = Icons.Outlined.Refresh,
                                contentDescription = "Refresh"
                            )
                        }
                    }

                    IconButton(onClick = onNavigateToPairing) {
                        Icon(
                            imageVector = Icons.Outlined.QrCodeScanner,
                            contentDescription = "Pair new device"
                        )
                    }
                }
            )
        },
        floatingActionButton = {
            ExtendedFloatingActionButton(
                onClick = onNavigateToPairing,
                containerColor = WinuxCyan
            ) {
                Icon(
                    imageVector = Icons.Outlined.Add,
                    contentDescription = null
                )
                Spacer(modifier = Modifier.width(8.dp))
                Text("Pair Device")
            }
        }
    ) { padding ->
        LazyColumn(
            modifier = Modifier
                .fillMaxSize()
                .padding(padding),
            contentPadding = PaddingValues(16.dp),
            verticalArrangement = Arrangement.spacedBy(12.dp)
        ) {
            // Paired Devices Section
            if (pairedDevices.isNotEmpty()) {
                item {
                    Text(
                        text = "Paired Devices",
                        style = MaterialTheme.typography.titleSmall,
                        fontWeight = FontWeight.SemiBold,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                        modifier = Modifier.padding(vertical = 8.dp)
                    )
                }

                items(
                    items = pairedDevices,
                    key = { it.id }
                ) { device ->
                    val deviceState = if (device.id == connectedDeviceId) {
                        connectionState
                    } else {
                        ConnectionState.DISCONNECTED
                    }

                    DeviceCard(
                        deviceWithState = DeviceWithState(
                            device = device,
                            connectionState = deviceState
                        ),
                        onClick = { /* Show device details */ },
                        onConnect = { viewModel.connectToDevice(device) },
                        onDisconnect = { viewModel.disconnect() }
                    )
                }
            }

            // Discovered Devices Section
            if (discoveredDevices.isNotEmpty()) {
                item {
                    Spacer(modifier = Modifier.height(8.dp))
                    Text(
                        text = "Discovered Devices",
                        style = MaterialTheme.typography.titleSmall,
                        fontWeight = FontWeight.SemiBold,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                        modifier = Modifier.padding(vertical = 8.dp)
                    )
                }

                items(
                    items = discoveredDevices.filter { discovered ->
                        pairedDevices.none { it.ipAddress == discovered.ipAddress }
                    },
                    key = { it.id }
                ) { device ->
                    DeviceCard(
                        deviceWithState = DeviceWithState(
                            device = device,
                            connectionState = ConnectionState.DISCONNECTED
                        ),
                        onClick = { /* Navigate to pairing */ },
                        onConnect = { viewModel.connectToDevice(device) },
                        onDisconnect = { }
                    )
                }
            }

            // Empty State
            if (pairedDevices.isEmpty() && discoveredDevices.isEmpty() && !isDiscovering) {
                item {
                    EmptyDevicesState(
                        onScanClick = { viewModel.startDiscovery() },
                        onPairClick = onNavigateToPairing
                    )
                }
            }

            // Scanning State
            if (pairedDevices.isEmpty() && discoveredDevices.isEmpty() && isDiscovering) {
                item {
                    ScanningState()
                }
            }

            // Bottom padding for FAB
            item {
                Spacer(modifier = Modifier.height(80.dp))
            }
        }
    }
}

@Composable
fun EmptyDevicesState(
    onScanClick: () -> Unit,
    onPairClick: () -> Unit
) {
    Column(
        modifier = Modifier
            .fillMaxWidth()
            .padding(32.dp),
        horizontalAlignment = Alignment.CenterHorizontally
    ) {
        Icon(
            imageVector = Icons.Outlined.DevicesOther,
            contentDescription = null,
            modifier = Modifier.size(80.dp),
            tint = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.5f)
        )

        Spacer(modifier = Modifier.height(24.dp))

        Text(
            text = "No Devices Found",
            style = MaterialTheme.typography.titleLarge,
            fontWeight = FontWeight.SemiBold,
            textAlign = TextAlign.Center
        )

        Spacer(modifier = Modifier.height(8.dp))

        Text(
            text = "Make sure your Winux desktop is running and connected to the same network",
            style = MaterialTheme.typography.bodyMedium,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            textAlign = TextAlign.Center
        )

        Spacer(modifier = Modifier.height(24.dp))

        Row(
            horizontalArrangement = Arrangement.spacedBy(12.dp)
        ) {
            OutlinedButton(onClick = onScanClick) {
                Icon(
                    imageVector = Icons.Outlined.Search,
                    contentDescription = null,
                    modifier = Modifier.size(18.dp)
                )
                Spacer(modifier = Modifier.width(8.dp))
                Text("Scan Network")
            }

            Button(onClick = onPairClick) {
                Icon(
                    imageVector = Icons.Outlined.QrCode,
                    contentDescription = null,
                    modifier = Modifier.size(18.dp)
                )
                Spacer(modifier = Modifier.width(8.dp))
                Text("Scan QR Code")
            }
        }
    }
}

@Composable
fun ScanningState() {
    Column(
        modifier = Modifier
            .fillMaxWidth()
            .padding(32.dp),
        horizontalAlignment = Alignment.CenterHorizontally
    ) {
        CircularProgressIndicator(
            modifier = Modifier.size(48.dp),
            color = WinuxCyan
        )

        Spacer(modifier = Modifier.height(24.dp))

        Text(
            text = "Searching for devices...",
            style = MaterialTheme.typography.titleMedium,
            textAlign = TextAlign.Center
        )

        Spacer(modifier = Modifier.height(8.dp))

        Text(
            text = "Looking for Winux desktops on your network",
            style = MaterialTheme.typography.bodyMedium,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            textAlign = TextAlign.Center
        )
    }
}
