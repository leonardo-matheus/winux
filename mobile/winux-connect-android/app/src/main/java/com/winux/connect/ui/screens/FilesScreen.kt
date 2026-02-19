package com.winux.connect.ui.screens

import android.net.Uri
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
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
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import com.winux.connect.data.ConnectionState
import com.winux.connect.services.FileTransfer
import com.winux.connect.services.TransferDirection
import com.winux.connect.services.TransferState
import com.winux.connect.ui.theme.*
import com.winux.connect.ui.viewmodel.FilesViewModel

/**
 * Screen for managing file transfers
 */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun FilesScreen(
    viewModel: FilesViewModel = hiltViewModel()
) {
    val transfers by viewModel.transfers.collectAsState()
    val connectionState by viewModel.connectionState.collectAsState()
    val isConnected = connectionState == ConnectionState.CONNECTED

    val filePicker = rememberLauncherForActivityResult(
        contract = ActivityResultContracts.GetContent()
    ) { uri: Uri? ->
        uri?.let { viewModel.sendFile(it) }
    }

    val multiFilePicker = rememberLauncherForActivityResult(
        contract = ActivityResultContracts.GetMultipleContents()
    ) { uris: List<Uri> ->
        uris.forEach { viewModel.sendFile(it) }
    }

    Scaffold(
        topBar = {
            TopAppBar(
                title = {
                    Text(
                        text = "File Transfer",
                        fontWeight = FontWeight.Bold
                    )
                }
            )
        },
        floatingActionButton = {
            if (isConnected) {
                ExtendedFloatingActionButton(
                    onClick = { filePicker.launch("*/*") },
                    containerColor = WinuxCyan
                ) {
                    Icon(
                        imageVector = Icons.Outlined.Upload,
                        contentDescription = null
                    )
                    Spacer(modifier = Modifier.width(8.dp))
                    Text("Send File")
                }
            }
        }
    ) { padding ->
        if (!isConnected) {
            NotConnectedState(
                modifier = Modifier
                    .fillMaxSize()
                    .padding(padding)
            )
        } else if (transfers.isEmpty()) {
            EmptyTransfersState(
                onSendFile = { filePicker.launch("*/*") },
                modifier = Modifier
                    .fillMaxSize()
                    .padding(padding)
            )
        } else {
            LazyColumn(
                modifier = Modifier
                    .fillMaxSize()
                    .padding(padding),
                contentPadding = PaddingValues(16.dp),
                verticalArrangement = Arrangement.spacedBy(12.dp)
            ) {
                // Active Transfers
                val activeTransfers = transfers.filter {
                    it.state == TransferState.IN_PROGRESS || it.state == TransferState.PENDING
                }

                if (activeTransfers.isNotEmpty()) {
                    item {
                        Text(
                            text = "Active Transfers",
                            style = MaterialTheme.typography.titleSmall,
                            fontWeight = FontWeight.SemiBold,
                            color = MaterialTheme.colorScheme.onSurfaceVariant
                        )
                    }

                    items(activeTransfers, key = { it.id }) { transfer ->
                        FileTransferCard(
                            transfer = transfer,
                            onCancel = { viewModel.cancelTransfer(transfer.id) }
                        )
                    }
                }

                // Completed Transfers
                val completedTransfers = transfers.filter {
                    it.state == TransferState.COMPLETED || it.state == TransferState.FAILED
                }

                if (completedTransfers.isNotEmpty()) {
                    item {
                        if (activeTransfers.isNotEmpty()) {
                            Spacer(modifier = Modifier.height(8.dp))
                        }
                        Text(
                            text = "Recent Transfers",
                            style = MaterialTheme.typography.titleSmall,
                            fontWeight = FontWeight.SemiBold,
                            color = MaterialTheme.colorScheme.onSurfaceVariant
                        )
                    }

                    items(completedTransfers, key = { it.id }) { transfer ->
                        FileTransferCard(
                            transfer = transfer,
                            onOpen = transfer.localPath?.let { { viewModel.openFile(it) } }
                        )
                    }
                }

                // Bottom padding for FAB
                item {
                    Spacer(modifier = Modifier.height(80.dp))
                }
            }
        }
    }
}

@Composable
fun FileTransferCard(
    transfer: FileTransfer,
    onCancel: (() -> Unit)? = null,
    onOpen: (() -> Unit)? = null
) {
    Card(
        modifier = Modifier.fillMaxWidth()
    ) {
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .padding(16.dp)
        ) {
            Row(
                verticalAlignment = Alignment.CenterVertically
            ) {
                // File icon
                Icon(
                    imageVector = getFileIcon(transfer.mimeType),
                    contentDescription = null,
                    tint = when (transfer.state) {
                        TransferState.COMPLETED -> WinuxGreen
                        TransferState.FAILED -> ErrorColor
                        else -> WinuxCyan
                    },
                    modifier = Modifier.size(40.dp)
                )

                Spacer(modifier = Modifier.width(12.dp))

                // File info
                Column(modifier = Modifier.weight(1f)) {
                    Text(
                        text = transfer.fileName,
                        style = MaterialTheme.typography.titleSmall,
                        fontWeight = FontWeight.Medium,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis
                    )

                    Text(
                        text = formatFileSize(transfer.fileSize),
                        style = MaterialTheme.typography.bodySmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant
                    )
                }

                // Direction icon
                Icon(
                    imageVector = if (transfer.direction == TransferDirection.INCOMING) {
                        Icons.Outlined.Download
                    } else {
                        Icons.Outlined.Upload
                    },
                    contentDescription = null,
                    modifier = Modifier.size(20.dp),
                    tint = MaterialTheme.colorScheme.onSurfaceVariant
                )
            }

            // Progress bar for active transfers
            if (transfer.state == TransferState.IN_PROGRESS) {
                Spacer(modifier = Modifier.height(12.dp))

                LinearProgressIndicator(
                    progress = { transfer.progress / 100f },
                    modifier = Modifier.fillMaxWidth(),
                    color = WinuxCyan
                )

                Spacer(modifier = Modifier.height(8.dp))

                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.SpaceBetween
                ) {
                    Text(
                        text = "${transfer.progress}%",
                        style = MaterialTheme.typography.bodySmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant
                    )

                    Text(
                        text = "${formatFileSize(transfer.bytesTransferred)} / ${formatFileSize(transfer.fileSize)}",
                        style = MaterialTheme.typography.bodySmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant
                    )
                }

                onCancel?.let {
                    Spacer(modifier = Modifier.height(8.dp))
                    TextButton(
                        onClick = it,
                        modifier = Modifier.align(Alignment.End)
                    ) {
                        Text("Cancel")
                    }
                }
            }

            // Status for completed/failed transfers
            if (transfer.state == TransferState.COMPLETED || transfer.state == TransferState.FAILED) {
                Spacer(modifier = Modifier.height(8.dp))

                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.SpaceBetween,
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    Row(verticalAlignment = Alignment.CenterVertically) {
                        Icon(
                            imageVector = if (transfer.state == TransferState.COMPLETED) {
                                Icons.Filled.CheckCircle
                            } else {
                                Icons.Filled.Error
                            },
                            contentDescription = null,
                            modifier = Modifier.size(16.dp),
                            tint = if (transfer.state == TransferState.COMPLETED) {
                                WinuxGreen
                            } else {
                                ErrorColor
                            }
                        )
                        Spacer(modifier = Modifier.width(4.dp))
                        Text(
                            text = if (transfer.state == TransferState.COMPLETED) {
                                "Completed"
                            } else {
                                transfer.errorMessage ?: "Failed"
                            },
                            style = MaterialTheme.typography.bodySmall,
                            color = if (transfer.state == TransferState.COMPLETED) {
                                WinuxGreen
                            } else {
                                ErrorColor
                            }
                        )
                    }

                    if (transfer.state == TransferState.COMPLETED && onOpen != null) {
                        TextButton(onClick = onOpen) {
                            Text("Open")
                        }
                    }
                }
            }

            // Pending state
            if (transfer.state == TransferState.PENDING) {
                Spacer(modifier = Modifier.height(8.dp))
                Row(
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    CircularProgressIndicator(
                        modifier = Modifier.size(16.dp),
                        strokeWidth = 2.dp
                    )
                    Spacer(modifier = Modifier.width(8.dp))
                    Text(
                        text = "Waiting for acceptance...",
                        style = MaterialTheme.typography.bodySmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant
                    )
                }
            }
        }
    }
}

@Composable
fun NotConnectedState(modifier: Modifier = Modifier) {
    Column(
        modifier = modifier.padding(32.dp),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.Center
    ) {
        Icon(
            imageVector = Icons.Outlined.CloudOff,
            contentDescription = null,
            modifier = Modifier.size(80.dp),
            tint = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.5f)
        )

        Spacer(modifier = Modifier.height(24.dp))

        Text(
            text = "Not Connected",
            style = MaterialTheme.typography.titleLarge,
            fontWeight = FontWeight.SemiBold,
            textAlign = TextAlign.Center
        )

        Spacer(modifier = Modifier.height(8.dp))

        Text(
            text = "Connect to a Winux desktop to transfer files",
            style = MaterialTheme.typography.bodyMedium,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            textAlign = TextAlign.Center
        )
    }
}

@Composable
fun EmptyTransfersState(
    onSendFile: () -> Unit,
    modifier: Modifier = Modifier
) {
    Column(
        modifier = modifier.padding(32.dp),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.Center
    ) {
        Icon(
            imageVector = Icons.Outlined.FolderOpen,
            contentDescription = null,
            modifier = Modifier.size(80.dp),
            tint = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.5f)
        )

        Spacer(modifier = Modifier.height(24.dp))

        Text(
            text = "No Transfers",
            style = MaterialTheme.typography.titleLarge,
            fontWeight = FontWeight.SemiBold,
            textAlign = TextAlign.Center
        )

        Spacer(modifier = Modifier.height(8.dp))

        Text(
            text = "Send files to your desktop or receive files from it",
            style = MaterialTheme.typography.bodyMedium,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            textAlign = TextAlign.Center
        )

        Spacer(modifier = Modifier.height(24.dp))

        Button(onClick = onSendFile) {
            Icon(
                imageVector = Icons.Outlined.Upload,
                contentDescription = null,
                modifier = Modifier.size(18.dp)
            )
            Spacer(modifier = Modifier.width(8.dp))
            Text("Send File")
        }
    }
}

fun getFileIcon(mimeType: String): androidx.compose.ui.graphics.vector.ImageVector {
    return when {
        mimeType.startsWith("image/") -> Icons.Outlined.Image
        mimeType.startsWith("video/") -> Icons.Outlined.VideoFile
        mimeType.startsWith("audio/") -> Icons.Outlined.AudioFile
        mimeType.startsWith("text/") -> Icons.Outlined.Description
        mimeType.contains("pdf") -> Icons.Outlined.PictureAsPdf
        mimeType.contains("zip") || mimeType.contains("archive") -> Icons.Outlined.FolderZip
        else -> Icons.Outlined.InsertDriveFile
    }
}

fun formatFileSize(bytes: Long): String {
    return when {
        bytes < 1024 -> "$bytes B"
        bytes < 1024 * 1024 -> "${bytes / 1024} KB"
        bytes < 1024 * 1024 * 1024 -> String.format("%.1f MB", bytes / (1024.0 * 1024))
        else -> String.format("%.2f GB", bytes / (1024.0 * 1024 * 1024))
    }
}
