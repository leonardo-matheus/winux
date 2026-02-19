package com.winux.connect.protocol

import android.content.Context
import android.net.nsd.NsdManager
import android.net.nsd.NsdServiceInfo
import android.util.Log
import com.winux.connect.data.Device
import com.winux.connect.data.DeviceType
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.channels.awaitClose
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.callbackFlow
import javax.inject.Inject
import javax.inject.Singleton

/**
 * Network Service Discovery for finding Winux desktops on the network
 */
@Singleton
class Discovery @Inject constructor(
    @ApplicationContext private val context: Context
) {
    private val nsdManager: NsdManager by lazy {
        context.getSystemService(Context.NSD_SERVICE) as NsdManager
    }

    private var discoveryListener: NsdManager.DiscoveryListener? = null
    private var isDiscovering = false

    companion object {
        private const val TAG = "WinuxDiscovery"
        const val SERVICE_TYPE = "_winux._tcp."
        const val SERVICE_NAME_PREFIX = "Winux Desktop"
    }

    /**
     * Start discovering Winux desktops on the network
     * Returns a Flow of discovered devices
     */
    fun discoverDevices(): Flow<DiscoveryEvent> = callbackFlow {
        if (isDiscovering) {
            stopDiscovery()
        }

        discoveryListener = object : NsdManager.DiscoveryListener {
            override fun onDiscoveryStarted(serviceType: String) {
                Log.d(TAG, "Discovery started: $serviceType")
                trySend(DiscoveryEvent.Started)
            }

            override fun onServiceFound(serviceInfo: NsdServiceInfo) {
                Log.d(TAG, "Service found: ${serviceInfo.serviceName}")

                // Resolve the service to get IP and port
                resolveService(serviceInfo) { device ->
                    device?.let {
                        trySend(DiscoveryEvent.DeviceFound(it))
                    }
                }
            }

            override fun onServiceLost(serviceInfo: NsdServiceInfo) {
                Log.d(TAG, "Service lost: ${serviceInfo.serviceName}")
                trySend(DiscoveryEvent.DeviceLost(serviceInfo.serviceName))
            }

            override fun onDiscoveryStopped(serviceType: String) {
                Log.d(TAG, "Discovery stopped: $serviceType")
                trySend(DiscoveryEvent.Stopped)
                isDiscovering = false
            }

            override fun onStartDiscoveryFailed(serviceType: String, errorCode: Int) {
                Log.e(TAG, "Start discovery failed: $errorCode")
                trySend(DiscoveryEvent.Error("Failed to start discovery: $errorCode"))
                isDiscovering = false
            }

            override fun onStopDiscoveryFailed(serviceType: String, errorCode: Int) {
                Log.e(TAG, "Stop discovery failed: $errorCode")
                isDiscovering = false
            }
        }

        try {
            nsdManager.discoverServices(SERVICE_TYPE, NsdManager.PROTOCOL_DNS_SD, discoveryListener)
            isDiscovering = true
        } catch (e: Exception) {
            Log.e(TAG, "Failed to start discovery", e)
            trySend(DiscoveryEvent.Error(e.message ?: "Unknown error"))
        }

        awaitClose {
            stopDiscovery()
        }
    }

    /**
     * Resolve a discovered service to get detailed information
     */
    private fun resolveService(serviceInfo: NsdServiceInfo, callback: (Device?) -> Unit) {
        val resolveListener = object : NsdManager.ResolveListener {
            override fun onServiceResolved(resolvedInfo: NsdServiceInfo) {
                Log.d(TAG, "Service resolved: ${resolvedInfo.serviceName} @ ${resolvedInfo.host?.hostAddress}:${resolvedInfo.port}")

                val device = Device(
                    name = resolvedInfo.serviceName,
                    hostname = resolvedInfo.host?.hostName ?: resolvedInfo.serviceName,
                    ipAddress = resolvedInfo.host?.hostAddress ?: "",
                    port = resolvedInfo.port,
                    deviceType = parseDeviceType(resolvedInfo),
                    capabilities = parseCapabilities(resolvedInfo),
                    osVersion = parseOsVersion(resolvedInfo)
                )

                callback(device)
            }

            override fun onResolveFailed(serviceInfo: NsdServiceInfo, errorCode: Int) {
                Log.e(TAG, "Resolve failed for ${serviceInfo.serviceName}: $errorCode")
                callback(null)
            }
        }

        try {
            nsdManager.resolveService(serviceInfo, resolveListener)
        } catch (e: Exception) {
            Log.e(TAG, "Failed to resolve service", e)
            callback(null)
        }
    }

    /**
     * Stop service discovery
     */
    fun stopDiscovery() {
        if (isDiscovering && discoveryListener != null) {
            try {
                nsdManager.stopServiceDiscovery(discoveryListener)
            } catch (e: Exception) {
                Log.e(TAG, "Failed to stop discovery", e)
            }
            isDiscovering = false
            discoveryListener = null
        }
    }

    /**
     * Parse device type from service TXT records
     */
    private fun parseDeviceType(serviceInfo: NsdServiceInfo): DeviceType {
        val attributes = serviceInfo.attributes
        val typeStr = attributes["type"]?.let { String(it) } ?: "desktop"

        return when (typeStr.lowercase()) {
            "laptop" -> DeviceType.LAPTOP
            "server" -> DeviceType.SERVER
            else -> DeviceType.DESKTOP
        }
    }

    /**
     * Parse capabilities from service TXT records
     */
    private fun parseCapabilities(serviceInfo: NsdServiceInfo): List<String> {
        val attributes = serviceInfo.attributes
        val capsStr = attributes["capabilities"]?.let { String(it) } ?: ""

        return if (capsStr.isNotEmpty()) {
            capsStr.split(",").map { it.trim() }
        } else {
            listOf("notifications", "clipboard", "files", "media")
        }
    }

    /**
     * Parse OS version from service TXT records
     */
    private fun parseOsVersion(serviceInfo: NsdServiceInfo): String {
        val attributes = serviceInfo.attributes
        return attributes["os"]?.let { String(it) } ?: "Winux"
    }
}

/**
 * Discovery events
 */
sealed class DiscoveryEvent {
    data object Started : DiscoveryEvent()
    data object Stopped : DiscoveryEvent()
    data class DeviceFound(val device: Device) : DiscoveryEvent()
    data class DeviceLost(val serviceName: String) : DiscoveryEvent()
    data class Error(val message: String) : DiscoveryEvent()
}
