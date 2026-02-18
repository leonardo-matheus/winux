//! Sound settings page

use gtk4::prelude::*;
use libadwaita as adw;
use libadwaita::prelude::*;
use tracing::info;

/// Sound settings page
pub struct SoundPage {
    widget: adw::PreferencesPage,
}

impl SoundPage {
    /// Create a new sound settings page
    pub fn new() -> Self {
        let page = adw::PreferencesPage::new();
        page.set_title("Sound");
        page.set_icon_name(Some("audio-speakers-symbolic"));

        // Output group
        let output_group = adw::PreferencesGroup::new();
        output_group.set_title("Output");
        output_group.set_description(Some("Configure audio output devices"));

        // Output device
        let output_device = adw::ComboRow::new();
        output_device.set_title("Output Device");
        output_device.set_subtitle("Select default audio output");
        let devices = gtk4::StringList::new(&[
            "Built-in Speakers",
            "HDMI Output",
            "USB Headset",
            "Bluetooth Headphones",
        ]);
        output_device.set_model(Some(&devices));
        output_group.add(&output_device);

        // Master volume
        let volume_row = adw::ActionRow::new();
        volume_row.set_title("Volume");

        let volume_scale = gtk4::Scale::with_range(gtk4::Orientation::Horizontal, 0.0, 100.0, 1.0);
        volume_scale.set_value(80.0);
        volume_scale.set_draw_value(true);
        volume_scale.set_width_request(200);
        volume_scale.set_hexpand(true);

        let mute_btn = gtk4::ToggleButton::new();
        mute_btn.set_icon_name("audio-volume-high-symbolic");
        mute_btn.connect_toggled(|btn| {
            if btn.is_active() {
                btn.set_icon_name("audio-volume-muted-symbolic");
            } else {
                btn.set_icon_name("audio-volume-high-symbolic");
            }
        });

        let vol_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
        vol_box.append(&volume_scale);
        vol_box.append(&mute_btn);
        volume_row.add_suffix(&vol_box);
        output_group.add(&volume_row);

        // Balance
        let balance_row = adw::ActionRow::new();
        balance_row.set_title("Balance");
        balance_row.set_subtitle("Left/Right audio balance");

        let balance_scale = gtk4::Scale::with_range(gtk4::Orientation::Horizontal, -100.0, 100.0, 1.0);
        balance_scale.set_value(0.0);
        balance_scale.set_draw_value(true);
        balance_scale.set_width_request(200);
        balance_scale.add_mark(-100.0, gtk4::PositionType::Bottom, Some("L"));
        balance_scale.add_mark(0.0, gtk4::PositionType::Bottom, Some("C"));
        balance_scale.add_mark(100.0, gtk4::PositionType::Bottom, Some("R"));
        balance_row.add_suffix(&balance_scale);
        output_group.add(&balance_row);

        page.add(&output_group);

        // Input group
        let input_group = adw::PreferencesGroup::new();
        input_group.set_title("Input");
        input_group.set_description(Some("Configure audio input devices"));

        // Input device
        let input_device = adw::ComboRow::new();
        input_device.set_title("Input Device");
        input_device.set_subtitle("Select default microphone");
        let mics = gtk4::StringList::new(&[
            "Built-in Microphone",
            "USB Headset Microphone",
            "Webcam Microphone",
        ]);
        input_device.set_model(Some(&mics));
        input_group.add(&input_device);

        // Input volume
        let input_vol_row = adw::ActionRow::new();
        input_vol_row.set_title("Input Volume");

        let input_scale = gtk4::Scale::with_range(gtk4::Orientation::Horizontal, 0.0, 100.0, 1.0);
        input_scale.set_value(70.0);
        input_scale.set_draw_value(true);
        input_scale.set_width_request(200);
        input_vol_row.add_suffix(&input_scale);
        input_group.add(&input_vol_row);

        // Input level meter
        let level_row = adw::ActionRow::new();
        level_row.set_title("Input Level");
        level_row.set_subtitle("Current microphone level");

        let level_bar = gtk4::LevelBar::new();
        level_bar.set_min_value(0.0);
        level_bar.set_max_value(1.0);
        level_bar.set_value(0.3);
        level_bar.set_width_request(200);
        level_row.add_suffix(&level_bar);
        input_group.add(&level_row);

        // Noise cancellation
        let noise_row = adw::ActionRow::new();
        noise_row.set_title("Noise Cancellation");
        noise_row.set_subtitle("Reduce background noise from microphone");
        let noise_switch = gtk4::Switch::new();
        noise_switch.set_valign(gtk4::Align::Center);
        noise_row.add_suffix(&noise_switch);
        input_group.add(&noise_row);

        page.add(&input_group);

        // Sound effects group
        let effects_group = adw::PreferencesGroup::new();
        effects_group.set_title("Sound Effects");

        // System sounds
        let system_sounds = adw::ActionRow::new();
        system_sounds.set_title("System Sounds");
        system_sounds.set_subtitle("Play sounds for notifications and actions");
        let system_sounds_switch = gtk4::Switch::new();
        system_sounds_switch.set_active(true);
        system_sounds_switch.set_valign(gtk4::Align::Center);
        system_sounds.add_suffix(&system_sounds_switch);
        effects_group.add(&system_sounds);

        // Alert sound
        let alert_row = adw::ComboRow::new();
        alert_row.set_title("Alert Sound");
        let alerts = gtk4::StringList::new(&[
            "Default",
            "Subtle",
            "Bright",
            "Gaming",
            "None",
        ]);
        alert_row.set_model(Some(&alerts));
        effects_group.add(&alert_row);

        // Startup sound
        let startup_sound = adw::ActionRow::new();
        startup_sound.set_title("Startup Sound");
        startup_sound.set_subtitle("Play sound when Winux starts");
        let startup_sound_switch = gtk4::Switch::new();
        startup_sound_switch.set_valign(gtk4::Align::Center);
        startup_sound.add_suffix(&startup_sound_switch);
        effects_group.add(&startup_sound);

        page.add(&effects_group);

        // Audio profiles group
        let profiles_group = adw::PreferencesGroup::new();
        profiles_group.set_title("Audio Profiles");

        // Profile selection
        let profile_row = adw::ComboRow::new();
        profile_row.set_title("Profile");
        profile_row.set_subtitle("Select audio processing profile");
        let profiles = gtk4::StringList::new(&[
            "Balanced",
            "Gaming",
            "Music",
            "Movie",
            "Voice",
        ]);
        profile_row.set_model(Some(&profiles));
        profiles_group.add(&profile_row);

        // Spatial audio
        let spatial_row = adw::ActionRow::new();
        spatial_row.set_title("Spatial Audio");
        spatial_row.set_subtitle("Enable 3D surround sound for headphones");
        let spatial_switch = gtk4::Switch::new();
        spatial_switch.set_valign(gtk4::Align::Center);
        spatial_row.add_suffix(&spatial_switch);
        profiles_group.add(&spatial_row);

        // Equalizer button
        let eq_row = adw::ActionRow::new();
        eq_row.set_title("Equalizer");
        eq_row.set_subtitle("Customize audio frequency response");
        eq_row.set_activatable(true);

        let eq_arrow = gtk4::Image::from_icon_name("go-next-symbolic");
        eq_row.add_suffix(&eq_arrow);

        eq_row.connect_activated(|_| {
            info!("Opening equalizer");
        });
        profiles_group.add(&eq_row);

        page.add(&profiles_group);

        // Applications group
        let apps_group = adw::PreferencesGroup::new();
        apps_group.set_title("Application Volumes");
        apps_group.set_description(Some("Control volume for individual applications"));

        // Placeholder for running apps
        let app1_row = adw::ActionRow::new();
        app1_row.set_title("Firefox");
        app1_row.set_subtitle("Web Browser");

        let app1_scale = gtk4::Scale::with_range(gtk4::Orientation::Horizontal, 0.0, 100.0, 1.0);
        app1_scale.set_value(100.0);
        app1_scale.set_width_request(150);
        app1_row.add_suffix(&app1_scale);
        apps_group.add(&app1_row);

        let app2_row = adw::ActionRow::new();
        app2_row.set_title("Spotify");
        app2_row.set_subtitle("Music Player");

        let app2_scale = gtk4::Scale::with_range(gtk4::Orientation::Horizontal, 0.0, 100.0, 1.0);
        app2_scale.set_value(80.0);
        app2_scale.set_width_request(150);
        app2_row.add_suffix(&app2_scale);
        apps_group.add(&app2_row);

        page.add(&apps_group);

        SoundPage { widget: page }
    }

    /// Get the page widget
    pub fn widget(&self) -> &adw::PreferencesPage {
        &self.widget
    }
}

impl Default for SoundPage {
    fn default() -> Self {
        Self::new()
    }
}
