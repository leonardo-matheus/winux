//! Media page - Remote media control via MPRIS

use gtk4::prelude::*;
use libadwaita as adw;
use libadwaita::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

use crate::protocol::ConnectionManager;

/// Media player information
#[derive(Clone)]
pub struct MediaPlayer {
    pub name: String,
    pub track_title: String,
    pub track_artist: String,
    pub track_album: String,
    pub album_art: Option<String>,
    pub position: i64,
    pub duration: i64,
    pub is_playing: bool,
    pub volume: f64,
}

impl MediaPlayer {
    pub fn new(
        name: &str,
        track_title: &str,
        track_artist: &str,
        track_album: &str,
    ) -> Self {
        Self {
            name: name.to_string(),
            track_title: track_title.to_string(),
            track_artist: track_artist.to_string(),
            track_album: track_album.to_string(),
            album_art: None,
            position: 0,
            duration: 0,
            is_playing: false,
            volume: 1.0,
        }
    }
}

/// Media control page
pub struct MediaPage {
    widget: gtk4::ScrolledWindow,
    #[allow(dead_code)]
    manager: Rc<RefCell<ConnectionManager>>,
}

impl MediaPage {
    pub fn new(manager: Rc<RefCell<ConnectionManager>>) -> Self {
        let page = adw::PreferencesPage::new();
        page.set_title("Midia");
        page.set_icon_name(Some("multimedia-player-symbolic"));

        // Device selector
        let device_group = adw::PreferencesGroup::builder()
            .title("Dispositivo")
            .build();

        let device_combo = adw::ComboRow::builder()
            .title("Dispositivo")
            .subtitle("Selecione o dispositivo para controlar")
            .build();
        let devices = gtk4::StringList::new(&[
            "Samsung Galaxy S24",
            "iPad Pro",
        ]);
        device_combo.set_model(Some(&devices));
        device_group.add(&device_combo);

        page.add(&device_group);

        // Now Playing
        let now_playing_group = adw::PreferencesGroup::builder()
            .title("Tocando Agora")
            .build();

        // Album art and info
        let player_box = gtk4::Box::new(gtk4::Orientation::Vertical, 16);
        player_box.set_margin_start(24);
        player_box.set_margin_end(24);
        player_box.set_margin_top(16);
        player_box.set_margin_bottom(16);

        // Album art placeholder
        let art_frame = gtk4::Frame::new(None);
        art_frame.set_halign(gtk4::Align::Center);

        let art_box = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        art_box.set_size_request(200, 200);

        let art_icon = gtk4::Image::from_icon_name("audio-x-generic-symbolic");
        art_icon.set_pixel_size(100);
        art_icon.add_css_class("dim-label");
        art_icon.set_valign(gtk4::Align::Center);
        art_icon.set_vexpand(true);
        art_box.append(&art_icon);

        art_frame.set_child(Some(&art_box));
        player_box.append(&art_frame);

        // Track info
        let info_box = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
        info_box.set_halign(gtk4::Align::Center);

        let track_title = gtk4::Label::new(Some("Bohemian Rhapsody"));
        track_title.add_css_class("title-2");
        info_box.append(&track_title);

        let track_artist = gtk4::Label::new(Some("Queen"));
        track_artist.add_css_class("title-4");
        track_artist.add_css_class("dim-label");
        info_box.append(&track_artist);

        let track_album = gtk4::Label::new(Some("A Night at the Opera"));
        track_album.add_css_class("caption");
        track_album.add_css_class("dim-label");
        info_box.append(&track_album);

        player_box.append(&info_box);

        // Progress bar
        let progress_box = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
        progress_box.set_margin_top(8);

        let progress = gtk4::Scale::with_range(gtk4::Orientation::Horizontal, 0.0, 1.0, 0.01);
        progress.set_value(0.35);
        progress.set_draw_value(false);
        progress_box.append(&progress);

        let time_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);

        let time_current = gtk4::Label::new(Some("2:05"));
        time_current.add_css_class("caption");
        time_current.add_css_class("dim-label");
        time_box.append(&time_current);

        let time_spacer = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        time_spacer.set_hexpand(true);
        time_box.append(&time_spacer);

        let time_total = gtk4::Label::new(Some("5:55"));
        time_total.add_css_class("caption");
        time_total.add_css_class("dim-label");
        time_box.append(&time_total);

        progress_box.append(&time_box);
        player_box.append(&progress_box);

        // Playback controls
        let controls_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 16);
        controls_box.set_halign(gtk4::Align::Center);
        controls_box.set_margin_top(16);

        let shuffle_btn = gtk4::Button::from_icon_name("media-playlist-shuffle-symbolic");
        shuffle_btn.add_css_class("flat");
        shuffle_btn.add_css_class("circular");
        controls_box.append(&shuffle_btn);

        let prev_btn = gtk4::Button::from_icon_name("media-skip-backward-symbolic");
        prev_btn.add_css_class("flat");
        prev_btn.add_css_class("circular");
        controls_box.append(&prev_btn);

        let play_btn = gtk4::Button::from_icon_name("media-playback-pause-symbolic");
        play_btn.add_css_class("circular");
        play_btn.add_css_class("suggested-action");
        play_btn.set_size_request(48, 48);
        controls_box.append(&play_btn);

        let next_btn = gtk4::Button::from_icon_name("media-skip-forward-symbolic");
        next_btn.add_css_class("flat");
        next_btn.add_css_class("circular");
        controls_box.append(&next_btn);

        let repeat_btn = gtk4::Button::from_icon_name("media-playlist-repeat-symbolic");
        repeat_btn.add_css_class("flat");
        repeat_btn.add_css_class("circular");
        controls_box.append(&repeat_btn);

        player_box.append(&controls_box);

        // Volume control
        let volume_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
        volume_box.set_margin_top(16);

        let volume_icon = gtk4::Image::from_icon_name("audio-volume-high-symbolic");
        volume_box.append(&volume_icon);

        let volume_scale = gtk4::Scale::with_range(gtk4::Orientation::Horizontal, 0.0, 1.0, 0.05);
        volume_scale.set_value(0.75);
        volume_scale.set_draw_value(false);
        volume_scale.set_hexpand(true);
        volume_box.append(&volume_scale);

        player_box.append(&volume_box);

        let player_row = adw::ActionRow::new();
        player_row.set_child(Some(&player_box));
        now_playing_group.add(&player_row);

        page.add(&now_playing_group);

        // Control PC media from phone
        let reverse_group = adw::PreferencesGroup::builder()
            .title("Controle Remoto")
            .description("Controle a midia do PC pelo telefone")
            .build();

        let pc_control_switch = adw::SwitchRow::builder()
            .title("Permitir controle remoto")
            .subtitle("O telefone pode controlar players de midia do PC")
            .active(true)
            .build();
        reverse_group.add(&pc_control_switch);

        let players_row = adw::ActionRow::builder()
            .title("Players disponiveis")
            .subtitle("Spotify, VLC, Firefox")
            .build();
        players_row.add_prefix(&gtk4::Image::from_icon_name("applications-multimedia-symbolic"));
        reverse_group.add(&players_row);

        page.add(&reverse_group);

        // Queue / playlist
        let queue_group = adw::PreferencesGroup::builder()
            .title("Fila de Reproducao")
            .build();

        let queue_items = vec![
            ("Don't Stop Me Now", "Queen"),
            ("Under Pressure", "Queen & David Bowie"),
            ("We Will Rock You", "Queen"),
            ("Another One Bites the Dust", "Queen"),
        ];

        for (i, (title, artist)) in queue_items.iter().enumerate() {
            let row = adw::ActionRow::builder()
                .title(*title)
                .subtitle(*artist)
                .activatable(true)
                .build();

            let num_label = gtk4::Label::new(Some(&(i + 2).to_string()));
            num_label.add_css_class("dim-label");
            row.add_prefix(&num_label);

            row.add_suffix(&gtk4::Image::from_icon_name("list-remove-symbolic"));

            queue_group.add(&row);
        }

        page.add(&queue_group);

        let scrolled = gtk4::ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .child(&page)
            .build();

        Self {
            widget: scrolled,
            manager,
        }
    }

    pub fn widget(&self) -> &gtk4::ScrolledWindow {
        &self.widget
    }
}
