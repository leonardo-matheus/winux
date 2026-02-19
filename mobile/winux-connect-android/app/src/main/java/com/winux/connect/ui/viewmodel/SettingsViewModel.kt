package com.winux.connect.ui.viewmodel

import android.content.Context
import android.content.Intent
import android.net.Uri
import android.os.PowerManager
import android.provider.Settings
import androidx.datastore.core.DataStore
import androidx.datastore.preferences.core.*
import androidx.datastore.preferences.preferencesDataStore
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.google.android.gms.oss.licenses.OssLicensesMenuActivity
import dagger.hilt.android.lifecycle.HiltViewModel
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.flow.*
import kotlinx.coroutines.launch
import javax.inject.Inject

private val Context.dataStore: DataStore<Preferences> by preferencesDataStore(name = "settings")

@HiltViewModel
class SettingsViewModel @Inject constructor(
    @ApplicationContext private val context: Context
) : ViewModel() {

    private object PreferencesKeys {
        val SYNC_NOTIFICATIONS = booleanPreferencesKey("sync_notifications")
        val SYNC_CLIPBOARD = booleanPreferencesKey("sync_clipboard")
        val AUTO_CONNECT = booleanPreferencesKey("auto_connect")
        val BATTERY_SYNC = booleanPreferencesKey("battery_sync")
        val MEDIA_CONTROLS = booleanPreferencesKey("media_controls")
        val DARK_MODE = booleanPreferencesKey("dark_mode")
    }

    val syncNotifications: StateFlow<Boolean> = context.dataStore.data
        .map { it[PreferencesKeys.SYNC_NOTIFICATIONS] ?: true }
        .stateIn(viewModelScope, SharingStarted.WhileSubscribed(5000), true)

    val syncClipboard: StateFlow<Boolean> = context.dataStore.data
        .map { it[PreferencesKeys.SYNC_CLIPBOARD] ?: true }
        .stateIn(viewModelScope, SharingStarted.WhileSubscribed(5000), true)

    val autoConnect: StateFlow<Boolean> = context.dataStore.data
        .map { it[PreferencesKeys.AUTO_CONNECT] ?: true }
        .stateIn(viewModelScope, SharingStarted.WhileSubscribed(5000), true)

    val batterySync: StateFlow<Boolean> = context.dataStore.data
        .map { it[PreferencesKeys.BATTERY_SYNC] ?: true }
        .stateIn(viewModelScope, SharingStarted.WhileSubscribed(5000), true)

    val mediaControls: StateFlow<Boolean> = context.dataStore.data
        .map { it[PreferencesKeys.MEDIA_CONTROLS] ?: true }
        .stateIn(viewModelScope, SharingStarted.WhileSubscribed(5000), true)

    val darkMode: StateFlow<Boolean> = context.dataStore.data
        .map { it[PreferencesKeys.DARK_MODE] ?: true }
        .stateIn(viewModelScope, SharingStarted.WhileSubscribed(5000), true)

    fun setSyncNotifications(enabled: Boolean) {
        viewModelScope.launch {
            context.dataStore.edit {
                it[PreferencesKeys.SYNC_NOTIFICATIONS] = enabled
            }
        }
    }

    fun setSyncClipboard(enabled: Boolean) {
        viewModelScope.launch {
            context.dataStore.edit {
                it[PreferencesKeys.SYNC_CLIPBOARD] = enabled
            }
        }
    }

    fun setAutoConnect(enabled: Boolean) {
        viewModelScope.launch {
            context.dataStore.edit {
                it[PreferencesKeys.AUTO_CONNECT] = enabled
            }
        }
    }

    fun setBatterySync(enabled: Boolean) {
        viewModelScope.launch {
            context.dataStore.edit {
                it[PreferencesKeys.BATTERY_SYNC] = enabled
            }
        }
    }

    fun setMediaControls(enabled: Boolean) {
        viewModelScope.launch {
            context.dataStore.edit {
                it[PreferencesKeys.MEDIA_CONTROLS] = enabled
            }
        }
    }

    fun setDarkMode(enabled: Boolean) {
        viewModelScope.launch {
            context.dataStore.edit {
                it[PreferencesKeys.DARK_MODE] = enabled
            }
        }
    }

    fun requestBatteryOptimization(context: Context) {
        try {
            val powerManager = context.getSystemService(Context.POWER_SERVICE) as PowerManager
            if (!powerManager.isIgnoringBatteryOptimizations(context.packageName)) {
                val intent = Intent(Settings.ACTION_REQUEST_IGNORE_BATTERY_OPTIMIZATIONS).apply {
                    data = Uri.parse("package:${context.packageName}")
                    addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
                }
                context.startActivity(intent)
            }
        } catch (e: Exception) {
            // Open battery settings instead
            val intent = Intent(Settings.ACTION_BATTERY_SAVER_SETTINGS).apply {
                addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
            }
            context.startActivity(intent)
        }
    }

    fun showLicenses(context: Context) {
        try {
            val intent = Intent(context, OssLicensesMenuActivity::class.java).apply {
                addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
            }
            context.startActivity(intent)
        } catch (e: Exception) {
            // OSS licenses activity not available
        }
    }

    fun sendFeedback(context: Context) {
        val intent = Intent(Intent.ACTION_SENDTO).apply {
            data = Uri.parse("mailto:")
            putExtra(Intent.EXTRA_EMAIL, arrayOf("feedback@winux.org"))
            putExtra(Intent.EXTRA_SUBJECT, "Winux Connect Feedback")
            addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
        }
        try {
            context.startActivity(intent)
        } catch (e: Exception) {
            // No email app available
        }
    }
}
