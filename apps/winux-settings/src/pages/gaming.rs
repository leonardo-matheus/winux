//! Gaming settings page

use gtk4::prelude::*;
use libadwaita as adw;
use libadwaita::prelude::*;
use tracing::info;

/// Gaming settings page
pub struct GamingPage {
    widget: adw::PreferencesPage,
}

impl GamingPage {
    /// Create a new gaming settings page
    pub fn new() -> Self {
        let page = adw::PreferencesPage::new();
        page.set_title("Gaming");
        page.set_icon_name(Some("input-gaming-symbolic"));

        // Game Mode group
        let gamemode_group = adw::PreferencesGroup::new();
        gamemode_group.set_title("Game Mode");
        gamemode_group.set_description(Some(
            "Optimize system performance while gaming",
        ));

        // Game Mode toggle
        let gamemode_row = adw::SwitchRow::new();
        gamemode_row.set_title("Game Mode");
        gamemode_row.set_subtitle("Automatically optimize performance when games are running");
        gamemode_group.add(&gamemode_row);

        // CPU optimization
        let cpu_row = adw::SwitchRow::new();
        cpu_row.set_title("CPU Governor");
        cpu_row.set_subtitle("Set CPU to performance mode while gaming");
        cpu_row.set_active(true);
        gamemode_group.add(&cpu_row);

        // GPU optimization
        let gpu_row = adw::SwitchRow::new();
        gpu_row.set_title("GPU Performance Mode");
        gpu_row.set_subtitle("Maximize GPU clock speeds");
        gpu_row.set_active(true);
        gamemode_group.add(&gpu_row);

        // Compositor
        let compositor_row = adw::SwitchRow::new();
        compositor_row.set_title("Disable Compositor");
        compositor_row.set_subtitle("Reduce input latency by disabling compositing");
        gamemode_group.add(&compositor_row);

        // Screen tearing
        let tearing_row = adw::SwitchRow::new();
        tearing_row.set_title("Allow Screen Tearing");
        tearing_row.set_subtitle("Disable VSync for lower input latency");
        gamemode_group.add(&tearing_row);

        page.add(&gamemode_group);

        // Wine/Proton group
        let wine_group = adw::PreferencesGroup::new();
        wine_group.set_title("Wine & Proton");
        wine_group.set_description(Some("Windows game compatibility layer settings"));

        // Default Wine version
        let wine_version = adw::ComboRow::new();
        wine_version.set_title("Default Wine Version");
        let versions = gtk4::StringList::new(&[
            "Proton Experimental",
            "Proton 8.0",
            "Proton 7.0",
            "GE-Proton Latest",
            "Wine Staging 9.0",
            "Wine 9.0",
        ]);
        wine_version.set_model(Some(&versions));
        wine_group.add(&wine_version);

        // DXVK
        let dxvk_row = adw::SwitchRow::new();
        dxvk_row.set_title("DXVK");
        dxvk_row.set_subtitle("Vulkan-based DirectX 9/10/11 translation");
        dxvk_row.set_active(true);
        wine_group.add(&dxvk_row);

        // VKD3D
        let vkd3d_row = adw::SwitchRow::new();
        vkd3d_row.set_title("VKD3D-Proton");
        vkd3d_row.set_subtitle("Vulkan-based DirectX 12 translation");
        vkd3d_row.set_active(true);
        wine_group.add(&vkd3d_row);

        // FSR
        let fsr_row = adw::ComboRow::new();
        fsr_row.set_title("AMD FSR");
        fsr_row.set_subtitle("FidelityFX Super Resolution upscaling");
        let fsr_modes = gtk4::StringList::new(&[
            "Disabled",
            "Ultra Quality",
            "Quality",
            "Balanced",
            "Performance",
        ]);
        fsr_row.set_model(Some(&fsr_modes));
        wine_group.add(&fsr_row);

        // Shader cache
        let shader_row = adw::SwitchRow::new();
        shader_row.set_title("Shader Pre-caching");
        shader_row.set_subtitle("Download pre-compiled shaders when available");
        shader_row.set_active(true);
        wine_group.add(&shader_row);

        page.add(&wine_group);

        // Performance monitoring group
        let perf_group = adw::PreferencesGroup::new();
        perf_group.set_title("Performance Monitoring");

        // MangoHud
        let mangohud_row = adw::SwitchRow::new();
        mangohud_row.set_title("MangoHud");
        mangohud_row.set_subtitle("Show performance overlay in games");
        perf_group.add(&mangohud_row);

        // MangoHud position
        let hud_position = adw::ComboRow::new();
        hud_position.set_title("Overlay Position");
        let positions = gtk4::StringList::new(&[
            "Top Left",
            "Top Right",
            "Bottom Left",
            "Bottom Right",
            "Top Center",
            "Bottom Center",
        ]);
        hud_position.set_model(Some(&positions));
        perf_group.add(&hud_position);

        // What to show
        let show_fps = adw::SwitchRow::new();
        show_fps.set_title("Show FPS");
        show_fps.set_active(true);
        perf_group.add(&show_fps);

        let show_frametime = adw::SwitchRow::new();
        show_frametime.set_title("Show Frame Time");
        show_frametime.set_active(true);
        perf_group.add(&show_frametime);

        let show_cpu = adw::SwitchRow::new();
        show_cpu.set_title("Show CPU Usage");
        show_cpu.set_active(true);
        perf_group.add(&show_cpu);

        let show_gpu = adw::SwitchRow::new();
        show_gpu.set_title("Show GPU Usage");
        show_gpu.set_active(true);
        perf_group.add(&show_gpu);

        let show_vram = adw::SwitchRow::new();
        show_vram.set_title("Show VRAM Usage");
        perf_group.add(&show_vram);

        let show_ram = adw::SwitchRow::new();
        show_ram.set_title("Show RAM Usage");
        perf_group.add(&show_ram);

        page.add(&perf_group);

        // Controllers group
        let controller_group = adw::PreferencesGroup::new();
        controller_group.set_title("Controllers");

        // Detected controllers
        let controller_row = adw::ActionRow::new();
        controller_row.set_title("Xbox Wireless Controller");
        controller_row.set_subtitle("Connected via Bluetooth");
        controller_row.add_prefix(&gtk4::Image::from_icon_name("input-gaming-symbolic"));

        let config_btn = gtk4::Button::with_label("Configure");
        config_btn.add_css_class("flat");
        controller_row.add_suffix(&config_btn);
        controller_group.add(&controller_row);

        // Controller deadzone
        let deadzone_row = adw::ActionRow::new();
        deadzone_row.set_title("Stick Deadzone");
        deadzone_row.set_subtitle("Adjust analog stick sensitivity");

        let deadzone_scale = gtk4::Scale::with_range(gtk4::Orientation::Horizontal, 0.0, 30.0, 1.0);
        deadzone_scale.set_value(10.0);
        deadzone_scale.set_draw_value(true);
        deadzone_scale.set_width_request(150);
        deadzone_row.add_suffix(&deadzone_scale);
        controller_group.add(&deadzone_row);

        // Vibration
        let vibration_row = adw::SwitchRow::new();
        vibration_row.set_title("Vibration");
        vibration_row.set_subtitle("Enable controller rumble");
        vibration_row.set_active(true);
        controller_group.add(&vibration_row);

        page.add(&controller_group);

        // Steam group
        let steam_group = adw::PreferencesGroup::new();
        steam_group.set_title("Steam Integration");

        // Steam Runtime
        let runtime_row = adw::ComboRow::new();
        runtime_row.set_title("Steam Runtime");
        let runtimes = gtk4::StringList::new(&[
            "Steam Linux Runtime 3.0 (Sniper)",
            "Steam Linux Runtime 2.0 (Soldier)",
            "Native",
        ]);
        runtime_row.set_model(Some(&runtimes));
        steam_group.add(&runtime_row);

        // Enable Steam Play for all titles
        let steamplay_row = adw::SwitchRow::new();
        steamplay_row.set_title("Steam Play for All Titles");
        steamplay_row.set_subtitle("Enable Proton for all Windows games");
        steamplay_row.set_active(true);
        steam_group.add(&steamplay_row);

        page.add(&steam_group);

        // System tweaks group
        let tweaks_group = adw::PreferencesGroup::new();
        tweaks_group.set_title("System Tweaks");

        // Swappiness
        let swap_row = adw::ActionRow::new();
        swap_row.set_title("Swappiness");
        swap_row.set_subtitle("Lower values keep more in RAM (recommended: 10)");

        let swap_spin = gtk4::SpinButton::with_range(0.0, 100.0, 5.0);
        swap_spin.set_value(10.0);
        swap_row.add_suffix(&swap_spin);
        tweaks_group.add(&swap_row);

        // Split lock detection
        let splitlock_row = adw::SwitchRow::new();
        splitlock_row.set_title("Disable Split Lock Detection");
        splitlock_row.set_subtitle("May improve performance in some games");
        tweaks_group.add(&splitlock_row);

        // Watch dogs
        let watchdog_row = adw::SwitchRow::new();
        watchdog_row.set_title("Disable NMI Watchdog");
        watchdog_row.set_subtitle("Reduce CPU overhead");
        tweaks_group.add(&watchdog_row);

        page.add(&tweaks_group);

        GamingPage { widget: page }
    }

    /// Get the page widget
    pub fn widget(&self) -> &adw::PreferencesPage {
        &self.widget
    }
}

impl Default for GamingPage {
    fn default() -> Self {
        Self::new()
    }
}
