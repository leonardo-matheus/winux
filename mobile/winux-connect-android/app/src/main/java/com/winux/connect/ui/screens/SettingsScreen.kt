package com.winux.connect.ui.screens

import androidx.compose.foundation.clickable
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
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import com.winux.connect.services.NotificationListener
import com.winux.connect.ui.viewmodel.SettingsViewModel

/**
 * Settings screen for app configuration
 */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun SettingsScreen(
    viewModel: SettingsViewModel = hiltViewModel()
) {
    val context = LocalContext.current

    val syncNotifications by viewModel.syncNotifications.collectAsState()
    val syncClipboard by viewModel.syncClipboard.collectAsState()
    val autoConnect by viewModel.autoConnect.collectAsState()
    val batterySync by viewModel.batterySync.collectAsState()
    val mediaControls by viewModel.mediaControls.collectAsState()
    val darkMode by viewModel.darkMode.collectAsState()

    val notificationPermissionGranted = remember {
        NotificationListener.isPermissionGranted(context)
    }

    Scaffold(
        topBar = {
            TopAppBar(
                title = {
                    Text(
                        text = "Settings",
                        fontWeight = FontWeight.Bold
                    )
                }
            )
        }
    ) { padding ->
        Column(
            modifier = Modifier
                .fillMaxSize()
                .padding(padding)
                .verticalScroll(rememberScrollState())
        ) {
            // Connection Settings
            SettingsSection(title = "Connection") {
                SwitchSettingItem(
                    icon = Icons.Outlined.WifiFind,
                    title = "Auto-connect",
                    subtitle = "Automatically connect to paired devices",
                    checked = autoConnect,
                    onCheckedChange = { viewModel.setAutoConnect(it) }
                )
            }

            // Sync Settings
            SettingsSection(title = "Sync") {
                SwitchSettingItem(
                    icon = Icons.Outlined.Notifications,
                    title = "Notification sync",
                    subtitle = if (notificationPermissionGranted) {
                        "Forward phone notifications to PC"
                    } else {
                        "Tap to grant notification access"
                    },
                    checked = syncNotifications && notificationPermissionGranted,
                    onCheckedChange = {
                        if (notificationPermissionGranted) {
                            viewModel.setSyncNotifications(it)
                        } else {
                            NotificationListener.requestPermission(context)
                        }
                    }
                )

                HorizontalDivider(modifier = Modifier.padding(horizontal = 16.dp))

                SwitchSettingItem(
                    icon = Icons.Outlined.ContentPaste,
                    title = "Clipboard sync",
                    subtitle = "Share clipboard between devices",
                    checked = syncClipboard,
                    onCheckedChange = { viewModel.setSyncClipboard(it) }
                )

                HorizontalDivider(modifier = Modifier.padding(horizontal = 16.dp))

                SwitchSettingItem(
                    icon = Icons.Outlined.BatteryStd,
                    title = "Battery sync",
                    subtitle = "Show phone battery on PC",
                    checked = batterySync,
                    onCheckedChange = { viewModel.setBatterySync(it) }
                )

                HorizontalDivider(modifier = Modifier.padding(horizontal = 16.dp))

                SwitchSettingItem(
                    icon = Icons.Outlined.MusicNote,
                    title = "Media controls",
                    subtitle = "Control PC media from phone",
                    checked = mediaControls,
                    onCheckedChange = { viewModel.setMediaControls(it) }
                )
            }

            // Appearance
            SettingsSection(title = "Appearance") {
                SwitchSettingItem(
                    icon = Icons.Outlined.DarkMode,
                    title = "Dark mode",
                    subtitle = "Use dark theme",
                    checked = darkMode,
                    onCheckedChange = { viewModel.setDarkMode(it) }
                )
            }

            // Permissions
            SettingsSection(title = "Permissions") {
                ClickableSettingItem(
                    icon = Icons.Outlined.Notifications,
                    title = "Notification access",
                    subtitle = if (notificationPermissionGranted) "Granted" else "Not granted",
                    onClick = { NotificationListener.requestPermission(context) }
                )

                HorizontalDivider(modifier = Modifier.padding(horizontal = 16.dp))

                ClickableSettingItem(
                    icon = Icons.Outlined.BatteryAlert,
                    title = "Battery optimization",
                    subtitle = "Disable for reliable background sync",
                    onClick = { viewModel.requestBatteryOptimization(context) }
                )
            }

            // About
            SettingsSection(title = "About") {
                ClickableSettingItem(
                    icon = Icons.Outlined.Info,
                    title = "Version",
                    subtitle = "1.0.0",
                    onClick = { }
                )

                HorizontalDivider(modifier = Modifier.padding(horizontal = 16.dp))

                ClickableSettingItem(
                    icon = Icons.Outlined.Code,
                    title = "Open source licenses",
                    subtitle = "View third-party licenses",
                    onClick = { viewModel.showLicenses(context) }
                )

                HorizontalDivider(modifier = Modifier.padding(horizontal = 16.dp))

                ClickableSettingItem(
                    icon = Icons.Outlined.Feedback,
                    title = "Send feedback",
                    subtitle = "Report bugs or suggest features",
                    onClick = { viewModel.sendFeedback(context) }
                )
            }

            Spacer(modifier = Modifier.height(32.dp))
        }
    }
}

@Composable
fun SettingsSection(
    title: String,
    content: @Composable ColumnScope.() -> Unit
) {
    Column {
        Text(
            text = title,
            style = MaterialTheme.typography.titleSmall,
            fontWeight = FontWeight.SemiBold,
            color = MaterialTheme.colorScheme.primary,
            modifier = Modifier.padding(horizontal = 16.dp, vertical = 12.dp)
        )

        Card(
            modifier = Modifier
                .fillMaxWidth()
                .padding(horizontal = 16.dp),
            colors = CardDefaults.cardColors(
                containerColor = MaterialTheme.colorScheme.surfaceContainerLow
            )
        ) {
            Column(content = content)
        }

        Spacer(modifier = Modifier.height(16.dp))
    }
}

@Composable
fun SwitchSettingItem(
    icon: ImageVector,
    title: String,
    subtitle: String,
    checked: Boolean,
    onCheckedChange: (Boolean) -> Unit
) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .clickable { onCheckedChange(!checked) }
            .padding(16.dp),
        verticalAlignment = Alignment.CenterVertically
    ) {
        Icon(
            imageVector = icon,
            contentDescription = null,
            tint = MaterialTheme.colorScheme.onSurfaceVariant,
            modifier = Modifier.size(24.dp)
        )

        Spacer(modifier = Modifier.width(16.dp))

        Column(modifier = Modifier.weight(1f)) {
            Text(
                text = title,
                style = MaterialTheme.typography.bodyLarge
            )
            Text(
                text = subtitle,
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant
            )
        }

        Switch(
            checked = checked,
            onCheckedChange = onCheckedChange
        )
    }
}

@Composable
fun ClickableSettingItem(
    icon: ImageVector,
    title: String,
    subtitle: String,
    onClick: () -> Unit
) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .clickable(onClick = onClick)
            .padding(16.dp),
        verticalAlignment = Alignment.CenterVertically
    ) {
        Icon(
            imageVector = icon,
            contentDescription = null,
            tint = MaterialTheme.colorScheme.onSurfaceVariant,
            modifier = Modifier.size(24.dp)
        )

        Spacer(modifier = Modifier.width(16.dp))

        Column(modifier = Modifier.weight(1f)) {
            Text(
                text = title,
                style = MaterialTheme.typography.bodyLarge
            )
            Text(
                text = subtitle,
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant
            )
        }

        Icon(
            imageVector = Icons.Filled.ChevronRight,
            contentDescription = null,
            tint = MaterialTheme.colorScheme.onSurfaceVariant
        )
    }
}
