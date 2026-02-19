package com.winux.connect.ui.screens

import android.graphics.Bitmap
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.*
import androidx.compose.material.icons.outlined.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.asImageBitmap
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import com.winux.connect.protocol.PairingState
import com.winux.connect.ui.theme.WinuxCyan
import com.winux.connect.ui.theme.WinuxMagenta
import com.winux.connect.ui.viewmodel.PairingViewModel

/**
 * Screen for pairing with a Winux desktop
 */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun PairingScreen(
    onNavigateBack: () -> Unit,
    onPairingComplete: () -> Unit,
    viewModel: PairingViewModel = hiltViewModel()
) {
    val pairingState by viewModel.pairingState.collectAsState()
    val qrCodeBitmap by viewModel.qrCodeBitmap.collectAsState()
    val pin by viewModel.pin.collectAsState()
    var enteredPin by remember { mutableStateOf("") }

    LaunchedEffect(Unit) {
        viewModel.generateQRCode()
    }

    LaunchedEffect(pairingState) {
        if (pairingState is PairingState.Paired) {
            onPairingComplete()
        }
    }

    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text("Pair Device") },
                navigationIcon = {
                    IconButton(onClick = {
                        viewModel.cancelPairing()
                        onNavigateBack()
                    }) {
                        Icon(
                            imageVector = Icons.Filled.ArrowBack,
                            contentDescription = "Back"
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
            Spacer(modifier = Modifier.height(24.dp))

            when (pairingState) {
                is PairingState.Idle, is PairingState.WaitingForConnection -> {
                    PairingQRContent(
                        qrCodeBitmap = qrCodeBitmap,
                        pin = pin,
                        onManualConnect = { viewModel.startManualPairing() }
                    )
                }
                is PairingState.SendingRequest, is PairingState.WaitingForConfirmation -> {
                    PairingProgressContent()
                }
                is PairingState.VerifyingPin -> {
                    PinVerificationContent(
                        enteredPin = enteredPin,
                        onPinChange = { enteredPin = it },
                        onVerify = { viewModel.verifyPin(enteredPin) },
                        onCancel = { viewModel.cancelPairing() }
                    )
                }
                is PairingState.Paired -> {
                    PairingSuccessContent()
                }
                is PairingState.Failed -> {
                    PairingFailedContent(
                        reason = (pairingState as PairingState.Failed).reason,
                        onRetry = { viewModel.generateQRCode() },
                        onCancel = onNavigateBack
                    )
                }
            }
        }
    }
}

@Composable
fun PairingQRContent(
    qrCodeBitmap: Bitmap?,
    pin: String?,
    onManualConnect: () -> Unit
) {
    Text(
        text = "Scan QR Code",
        style = MaterialTheme.typography.headlineSmall,
        fontWeight = FontWeight.Bold
    )

    Spacer(modifier = Modifier.height(8.dp))

    Text(
        text = "Open Winux Connect on your desktop and scan this code",
        style = MaterialTheme.typography.bodyMedium,
        color = MaterialTheme.colorScheme.onSurfaceVariant,
        textAlign = TextAlign.Center,
        modifier = Modifier.padding(horizontal = 32.dp)
    )

    Spacer(modifier = Modifier.height(32.dp))

    // QR Code
    Box(
        modifier = Modifier
            .size(280.dp)
            .clip(RoundedCornerShape(16.dp))
            .background(MaterialTheme.colorScheme.surfaceContainerHigh),
        contentAlignment = Alignment.Center
    ) {
        qrCodeBitmap?.let { bitmap ->
            Image(
                bitmap = bitmap.asImageBitmap(),
                contentDescription = "Pairing QR Code",
                modifier = Modifier
                    .size(256.dp)
                    .clip(RoundedCornerShape(8.dp))
            )
        } ?: CircularProgressIndicator(color = WinuxCyan)
    }

    Spacer(modifier = Modifier.height(24.dp))

    // PIN Display
    pin?.let { pinValue ->
        Text(
            text = "Or enter this PIN:",
            style = MaterialTheme.typography.bodyMedium,
            color = MaterialTheme.colorScheme.onSurfaceVariant
        )

        Spacer(modifier = Modifier.height(8.dp))

        Card(
            colors = CardDefaults.cardColors(
                containerColor = MaterialTheme.colorScheme.primaryContainer
            )
        ) {
            Text(
                text = pinValue,
                style = MaterialTheme.typography.headlineMedium,
                fontWeight = FontWeight.Bold,
                letterSpacing = 8.sp,
                color = WinuxCyan,
                modifier = Modifier.padding(horizontal = 24.dp, vertical = 12.dp)
            )
        }
    }

    Spacer(modifier = Modifier.height(32.dp))

    // Instructions
    Card(
        modifier = Modifier
            .fillMaxWidth()
            .padding(horizontal = 24.dp),
        colors = CardDefaults.cardColors(
            containerColor = MaterialTheme.colorScheme.surfaceContainerLow
        )
    ) {
        Column(
            modifier = Modifier.padding(16.dp)
        ) {
            InstructionItem(
                number = "1",
                text = "Open Winux desktop settings"
            )
            Spacer(modifier = Modifier.height(8.dp))
            InstructionItem(
                number = "2",
                text = "Go to 'Mobile Connect'"
            )
            Spacer(modifier = Modifier.height(8.dp))
            InstructionItem(
                number = "3",
                text = "Click 'Pair Phone' and scan the QR code"
            )
        }
    }

    Spacer(modifier = Modifier.height(24.dp))

    TextButton(onClick = onManualConnect) {
        Icon(
            imageVector = Icons.Outlined.Keyboard,
            contentDescription = null,
            modifier = Modifier.size(18.dp)
        )
        Spacer(modifier = Modifier.width(8.dp))
        Text("Connect Manually")
    }
}

@Composable
fun InstructionItem(number: String, text: String) {
    Row(
        verticalAlignment = Alignment.CenterVertically
    ) {
        Box(
            modifier = Modifier
                .size(24.dp)
                .clip(RoundedCornerShape(12.dp))
                .background(WinuxCyan),
            contentAlignment = Alignment.Center
        ) {
            Text(
                text = number,
                style = MaterialTheme.typography.labelMedium,
                fontWeight = FontWeight.Bold,
                color = MaterialTheme.colorScheme.surface
            )
        }
        Spacer(modifier = Modifier.width(12.dp))
        Text(
            text = text,
            style = MaterialTheme.typography.bodyMedium
        )
    }
}

@Composable
fun PairingProgressContent() {
    Spacer(modifier = Modifier.height(48.dp))

    CircularProgressIndicator(
        modifier = Modifier.size(64.dp),
        color = WinuxMagenta
    )

    Spacer(modifier = Modifier.height(24.dp))

    Text(
        text = "Pairing...",
        style = MaterialTheme.typography.headlineSmall,
        fontWeight = FontWeight.Bold
    )

    Spacer(modifier = Modifier.height(8.dp))

    Text(
        text = "Please wait while we establish a secure connection",
        style = MaterialTheme.typography.bodyMedium,
        color = MaterialTheme.colorScheme.onSurfaceVariant,
        textAlign = TextAlign.Center,
        modifier = Modifier.padding(horizontal = 32.dp)
    )
}

@Composable
fun PinVerificationContent(
    enteredPin: String,
    onPinChange: (String) -> Unit,
    onVerify: () -> Unit,
    onCancel: () -> Unit
) {
    Spacer(modifier = Modifier.height(32.dp))

    Icon(
        imageVector = Icons.Outlined.Pin,
        contentDescription = null,
        modifier = Modifier.size(64.dp),
        tint = WinuxCyan
    )

    Spacer(modifier = Modifier.height(24.dp))

    Text(
        text = "Enter PIN",
        style = MaterialTheme.typography.headlineSmall,
        fontWeight = FontWeight.Bold
    )

    Spacer(modifier = Modifier.height(8.dp))

    Text(
        text = "Enter the PIN shown on your desktop",
        style = MaterialTheme.typography.bodyMedium,
        color = MaterialTheme.colorScheme.onSurfaceVariant
    )

    Spacer(modifier = Modifier.height(32.dp))

    OutlinedTextField(
        value = enteredPin,
        onValueChange = { if (it.length <= 6) onPinChange(it) },
        modifier = Modifier.width(200.dp),
        textStyle = MaterialTheme.typography.headlineMedium.copy(
            textAlign = TextAlign.Center,
            letterSpacing = 8.sp
        ),
        keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Number),
        singleLine = true
    )

    Spacer(modifier = Modifier.height(32.dp))

    Row(
        horizontalArrangement = Arrangement.spacedBy(12.dp)
    ) {
        OutlinedButton(onClick = onCancel) {
            Text("Cancel")
        }

        Button(
            onClick = onVerify,
            enabled = enteredPin.length == 6
        ) {
            Text("Verify")
        }
    }
}

@Composable
fun PairingSuccessContent() {
    Spacer(modifier = Modifier.height(48.dp))

    Icon(
        imageVector = Icons.Filled.CheckCircle,
        contentDescription = null,
        modifier = Modifier.size(80.dp),
        tint = WinuxCyan
    )

    Spacer(modifier = Modifier.height(24.dp))

    Text(
        text = "Paired Successfully!",
        style = MaterialTheme.typography.headlineSmall,
        fontWeight = FontWeight.Bold
    )

    Spacer(modifier = Modifier.height(8.dp))

    Text(
        text = "Your phone is now connected to your Winux desktop",
        style = MaterialTheme.typography.bodyMedium,
        color = MaterialTheme.colorScheme.onSurfaceVariant,
        textAlign = TextAlign.Center,
        modifier = Modifier.padding(horizontal = 32.dp)
    )
}

@Composable
fun PairingFailedContent(
    reason: String,
    onRetry: () -> Unit,
    onCancel: () -> Unit
) {
    Spacer(modifier = Modifier.height(48.dp))

    Icon(
        imageVector = Icons.Outlined.Error,
        contentDescription = null,
        modifier = Modifier.size(80.dp),
        tint = MaterialTheme.colorScheme.error
    )

    Spacer(modifier = Modifier.height(24.dp))

    Text(
        text = "Pairing Failed",
        style = MaterialTheme.typography.headlineSmall,
        fontWeight = FontWeight.Bold
    )

    Spacer(modifier = Modifier.height(8.dp))

    Text(
        text = reason,
        style = MaterialTheme.typography.bodyMedium,
        color = MaterialTheme.colorScheme.onSurfaceVariant,
        textAlign = TextAlign.Center,
        modifier = Modifier.padding(horizontal = 32.dp)
    )

    Spacer(modifier = Modifier.height(32.dp))

    Row(
        horizontalArrangement = Arrangement.spacedBy(12.dp)
    ) {
        OutlinedButton(onClick = onCancel) {
            Text("Cancel")
        }

        Button(onClick = onRetry) {
            Icon(
                imageVector = Icons.Outlined.Refresh,
                contentDescription = null,
                modifier = Modifier.size(18.dp)
            )
            Spacer(modifier = Modifier.width(8.dp))
            Text("Try Again")
        }
    }
}

private val Int.sp: androidx.compose.ui.unit.TextUnit
    get() = androidx.compose.ui.unit.TextUnit(this.toFloat(), androidx.compose.ui.unit.TextUnitType.Sp)
