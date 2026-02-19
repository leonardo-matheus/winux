package com.winux.connect.ui.viewmodel

import android.graphics.Bitmap
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.winux.connect.data.Device
import com.winux.connect.protocol.Pairing
import com.winux.connect.protocol.PairingState
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.*
import kotlinx.coroutines.launch
import javax.inject.Inject

@HiltViewModel
class PairingViewModel @Inject constructor(
    private val pairing: Pairing
) : ViewModel() {

    val pairingState: StateFlow<PairingState> = pairing.pairingState

    private val _qrCodeBitmap = MutableStateFlow<Bitmap?>(null)
    val qrCodeBitmap: StateFlow<Bitmap?> = _qrCodeBitmap.asStateFlow()

    private val _pin = MutableStateFlow<String?>(null)
    val pin: StateFlow<String?> = _pin.asStateFlow()

    fun generateQRCode() {
        viewModelScope.launch {
            _pin.value = pairing.generatePin()
            _qrCodeBitmap.value = pairing.generateQRCodeBitmap()
        }
    }

    fun startPairing(device: Device) {
        viewModelScope.launch {
            pairing.startPairing(device)
        }
    }

    fun startManualPairing() {
        // TODO: Implement manual pairing flow
    }

    fun verifyPin(enteredPin: String) {
        if (pairing.verifyPin(enteredPin)) {
            // PIN verified, continue pairing
        } else {
            // Invalid PIN
        }
    }

    fun cancelPairing() {
        pairing.cancelPairing()
        pairing.reset()
    }
}
