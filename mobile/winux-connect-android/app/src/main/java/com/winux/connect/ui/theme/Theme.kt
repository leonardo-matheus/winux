package com.winux.connect.ui.theme

import android.app.Activity
import android.os.Build
import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.darkColorScheme
import androidx.compose.material3.dynamicDarkColorScheme
import androidx.compose.material3.dynamicLightColorScheme
import androidx.compose.material3.lightColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.SideEffect
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.toArgb
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalView
import androidx.core.view.WindowCompat

private val DarkColorScheme = darkColorScheme(
    primary = WinuxCyan,
    onPrimary = Color.Black,
    primaryContainer = WinuxCyanDark.copy(alpha = 0.2f),
    onPrimaryContainer = WinuxCyan,

    secondary = WinuxMagenta,
    onSecondary = Color.Black,
    secondaryContainer = WinuxMagentaDark.copy(alpha = 0.2f),
    onSecondaryContainer = WinuxMagenta,

    tertiary = WinuxGreen,
    onTertiary = Color.Black,
    tertiaryContainer = WinuxGreenDark.copy(alpha = 0.2f),
    onTertiaryContainer = WinuxGreen,

    background = SurfaceDark,
    onBackground = OnSurfaceDark,

    surface = SurfaceDark,
    onSurface = OnSurfaceDark,
    surfaceVariant = SurfaceVariantDark,
    onSurfaceVariant = OnSurfaceVariantDark,
    surfaceContainerLow = SurfaceContainerDark,
    surfaceContainer = SurfaceContainerDark,
    surfaceContainerHigh = SurfaceContainerHighDark,

    error = ErrorColor,
    onError = Color.White,

    outline = OnSurfaceVariantDark.copy(alpha = 0.5f),
    outlineVariant = OnSurfaceVariantDark.copy(alpha = 0.25f)
)

private val LightColorScheme = lightColorScheme(
    primary = WinuxCyanLight,
    onPrimary = Color.White,
    primaryContainer = WinuxCyan.copy(alpha = 0.15f),
    onPrimaryContainer = WinuxCyanLight,

    secondary = WinuxMagentaLight,
    onSecondary = Color.White,
    secondaryContainer = WinuxMagenta.copy(alpha = 0.15f),
    onSecondaryContainer = WinuxMagentaLight,

    tertiary = WinuxGreenLight,
    onTertiary = Color.White,
    tertiaryContainer = WinuxGreen.copy(alpha = 0.15f),
    onTertiaryContainer = WinuxGreenLight,

    background = SurfaceLight,
    onBackground = OnSurfaceLight,

    surface = SurfaceLight,
    onSurface = OnSurfaceLight,
    surfaceVariant = SurfaceVariantLight,
    onSurfaceVariant = OnSurfaceVariantLight,
    surfaceContainerLow = SurfaceContainerLight,
    surfaceContainer = SurfaceContainerLight,
    surfaceContainerHigh = SurfaceContainerHighLight,

    error = ErrorColor,
    onError = Color.White,

    outline = OnSurfaceVariantLight.copy(alpha = 0.5f),
    outlineVariant = OnSurfaceVariantLight.copy(alpha = 0.25f)
)

@Composable
fun WinuxConnectTheme(
    darkTheme: Boolean = isSystemInDarkTheme(),
    dynamicColor: Boolean = false, // Disabled to use Winux branding
    content: @Composable () -> Unit
) {
    val colorScheme = when {
        dynamicColor && Build.VERSION.SDK_INT >= Build.VERSION_CODES.S -> {
            val context = LocalContext.current
            if (darkTheme) dynamicDarkColorScheme(context) else dynamicLightColorScheme(context)
        }
        darkTheme -> DarkColorScheme
        else -> LightColorScheme
    }

    val view = LocalView.current
    if (!view.isInEditMode) {
        SideEffect {
            val window = (view.context as Activity).window
            window.statusBarColor = colorScheme.surface.toArgb()
            window.navigationBarColor = colorScheme.surface.toArgb()
            WindowCompat.getInsetsController(window, view).apply {
                isAppearanceLightStatusBars = !darkTheme
                isAppearanceLightNavigationBars = !darkTheme
            }
        }
    }

    MaterialTheme(
        colorScheme = colorScheme,
        content = content
    )
}
