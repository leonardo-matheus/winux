// Game card component - Steam Deck inspired
// Large clickable cards with cover art and quick actions

use gtk4::prelude::*;
use gtk4::{Box, Button, Frame, Image, Label, Orientation, Overlay};

/// Game platform enumeration
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Platform {
    Steam,
    GOG,
    Epic,
    Lutris,
    Native,
    Emulator,
}

impl Platform {
    pub fn display_name(&self) -> &str {
        match self {
            Self::Steam => "Steam",
            Self::GOG => "GOG",
            Self::Epic => "Epic",
            Self::Lutris => "Lutris",
            Self::Native => "Nativo",
            Self::Emulator => "Emulador",
        }
    }

    pub fn css_class(&self) -> &str {
        match self {
            Self::Steam => "platform-steam",
            Self::GOG => "platform-gog",
            Self::Epic => "platform-epic",
            Self::Lutris => "platform-lutris",
            Self::Native => "platform-native",
            Self::Emulator => "platform-emulator",
        }
    }

    pub fn icon_name(&self) -> &str {
        match self {
            Self::Steam => "steam-symbolic",
            Self::GOG => "folder-games-symbolic",
            Self::Epic => "gamepad-symbolic",
            Self::Lutris => "wine-symbolic",
            Self::Native => "tux-symbolic",
            Self::Emulator => "media-optical-symbolic",
        }
    }
}

/// Game information for displaying in cards
#[derive(Debug, Clone)]
pub struct GameInfo {
    pub id: String,
    pub name: String,
    pub platform: Platform,
    pub installed: bool,
    pub playtime_hours: f64,
    pub last_played: Option<String>,
    pub cover_icon: String, // Placeholder icon text
    pub native: bool,
}

impl GameInfo {
    pub fn playtime_formatted(&self) -> String {
        if self.playtime_hours < 1.0 {
            format!("{:.0}m", self.playtime_hours * 60.0)
        } else if self.playtime_hours < 100.0 {
            format!("{:.1}h", self.playtime_hours)
        } else {
            format!("{:.0}h", self.playtime_hours)
        }
    }
}

/// Create a game card widget
pub fn create_game_card(game: &GameInfo) -> Frame {
    let card = Frame::builder()
        .css_classes(vec!["card", "game-card"])
        .build();

    // Use overlay for play button on hover
    let overlay = Overlay::new();

    let content = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(8)
        .margin_start(12)
        .margin_end(12)
        .margin_top(12)
        .margin_bottom(12)
        .width_request(180)
        .build();

    // Cover image placeholder
    let cover_box = Box::builder()
        .width_request(160)
        .height_request(200)
        .css_classes(vec!["card"])
        .halign(gtk4::Align::Center)
        .valign(gtk4::Align::Center)
        .build();

    let cover_label = Label::builder()
        .label(&game.cover_icon)
        .css_classes(vec!["title-1"])
        .halign(gtk4::Align::Center)
        .valign(gtk4::Align::Center)
        .build();
    cover_box.append(&cover_label);

    // Platform badge overlay on cover
    let platform_badge = Label::builder()
        .label(game.platform.display_name())
        .css_classes(vec!["platform-badge", game.platform.css_class()])
        .halign(gtk4::Align::Start)
        .valign(gtk4::Align::Start)
        .margin_start(8)
        .margin_top(8)
        .build();

    let cover_overlay = Overlay::new();
    cover_overlay.set_child(Some(&cover_box));
    cover_overlay.add_overlay(&platform_badge);

    // Native badge if applicable
    if game.native {
        let native_badge = Label::builder()
            .label("Linux")
            .css_classes(vec!["platform-badge", "platform-native"])
            .halign(gtk4::Align::End)
            .valign(gtk4::Align::Start)
            .margin_end(8)
            .margin_top(8)
            .build();
        cover_overlay.add_overlay(&native_badge);
    }

    content.append(&cover_overlay);

    // Game title
    let title = Label::builder()
        .label(&game.name)
        .css_classes(vec!["game-title"])
        .halign(gtk4::Align::Center)
        .ellipsize(gtk4::pango::EllipsizeMode::End)
        .max_width_chars(20)
        .build();
    content.append(&title);

    // Playtime and last played
    if game.installed {
        let info_box = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(8)
            .halign(gtk4::Align::Center)
            .build();

        if game.playtime_hours > 0.0 {
            let playtime = Label::builder()
                .label(&game.playtime_formatted())
                .css_classes(vec!["dim-label", "caption"])
                .build();
            info_box.append(&playtime);
        }

        if let Some(ref last_played) = game.last_played {
            if game.playtime_hours > 0.0 {
                let separator = Label::builder()
                    .label("|")
                    .css_classes(vec!["dim-label", "caption"])
                    .build();
                info_box.append(&separator);
            }

            let last_played_label = Label::builder()
                .label(last_played)
                .css_classes(vec!["dim-label", "caption"])
                .build();
            info_box.append(&last_played_label);
        }

        content.append(&info_box);
    }

    // Action button
    let action_btn = if game.installed {
        Button::builder()
            .icon_name("media-playback-start-symbolic")
            .css_classes(vec!["suggested-action", "circular"])
            .tooltip_text("Jogar")
            .halign(gtk4::Align::Center)
            .build()
    } else {
        Button::builder()
            .label("Instalar")
            .css_classes(vec!["install-button"])
            .halign(gtk4::Align::Center)
            .build()
    };
    content.append(&action_btn);

    overlay.set_child(Some(&content));

    card.set_child(Some(&overlay));
    card
}

/// Create a compact game card for lists
pub fn create_game_row(game: &GameInfo) -> Box {
    let row = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(12)
        .margin_start(12)
        .margin_end(12)
        .margin_top(8)
        .margin_bottom(8)
        .build();

    // Small cover icon
    let icon_box = Box::builder()
        .width_request(48)
        .height_request(48)
        .css_classes(vec!["card"])
        .halign(gtk4::Align::Center)
        .valign(gtk4::Align::Center)
        .build();

    let icon_label = Label::builder()
        .label(&game.cover_icon)
        .css_classes(vec!["title-4"])
        .build();
    icon_box.append(&icon_label);
    row.append(&icon_box);

    // Game info
    let info_box = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(2)
        .hexpand(true)
        .valign(gtk4::Align::Center)
        .build();

    let title = Label::builder()
        .label(&game.name)
        .css_classes(vec!["game-title"])
        .halign(gtk4::Align::Start)
        .ellipsize(gtk4::pango::EllipsizeMode::End)
        .build();
    info_box.append(&title);

    let subtitle_box = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(8)
        .build();

    let platform_label = Label::builder()
        .label(game.platform.display_name())
        .css_classes(vec!["dim-label", "caption"])
        .build();
    subtitle_box.append(&platform_label);

    if game.playtime_hours > 0.0 {
        let sep = Label::builder()
            .label("|")
            .css_classes(vec!["dim-label", "caption"])
            .build();
        subtitle_box.append(&sep);

        let playtime = Label::builder()
            .label(&game.playtime_formatted())
            .css_classes(vec!["dim-label", "caption"])
            .build();
        subtitle_box.append(&playtime);
    }

    info_box.append(&subtitle_box);
    row.append(&info_box);

    // Status indicator
    if game.installed {
        let status = Image::from_icon_name("emblem-ok-symbolic");
        status.add_css_class("success");
        status.set_tooltip_text(Some("Instalado"));
        row.append(&status);
    }

    // Play/Install button
    let action_btn = if game.installed {
        Button::builder()
            .icon_name("media-playback-start-symbolic")
            .css_classes(vec!["suggested-action", "circular"])
            .valign(gtk4::Align::Center)
            .build()
    } else {
        Button::builder()
            .label("Instalar")
            .css_classes(vec!["flat"])
            .valign(gtk4::Align::Center)
            .build()
    };
    row.append(&action_btn);

    // More options button
    let more_btn = Button::builder()
        .icon_name("view-more-symbolic")
        .css_classes(vec!["flat"])
        .valign(gtk4::Align::Center)
        .build();
    row.append(&more_btn);

    row
}

/// Create a featured game banner
pub fn create_featured_banner(game: &GameInfo) -> Frame {
    let banner = Frame::builder()
        .css_classes(vec!["card", "featured-banner"])
        .build();

    let content = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(30)
        .margin_start(30)
        .margin_end(30)
        .margin_top(30)
        .margin_bottom(30)
        .build();

    // Large cover placeholder
    let cover = Box::builder()
        .width_request(200)
        .height_request(250)
        .css_classes(vec!["card"])
        .halign(gtk4::Align::Center)
        .valign(gtk4::Align::Center)
        .build();

    let cover_label = Label::builder()
        .label(&game.cover_icon)
        .css_classes(vec!["title-1"])
        .build();
    cover.append(&cover_label);
    content.append(&cover);

    // Game info
    let info = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(12)
        .valign(gtk4::Align::Center)
        .hexpand(true)
        .build();

    let title = Label::builder()
        .label(&game.name)
        .css_classes(vec!["title-1"])
        .halign(gtk4::Align::Start)
        .build();
    info.append(&title);

    let platform_box = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(8)
        .build();

    let platform_badge = Label::builder()
        .label(game.platform.display_name())
        .css_classes(vec!["platform-badge", game.platform.css_class()])
        .build();
    platform_box.append(&platform_badge);

    if game.native {
        let native_badge = Label::builder()
            .label("Native Linux")
            .css_classes(vec!["platform-badge", "platform-native"])
            .build();
        platform_box.append(&native_badge);
    }

    info.append(&platform_box);

    // Playtime
    if game.playtime_hours > 0.0 {
        let playtime = Label::builder()
            .label(&format!("Tempo de jogo: {}", game.playtime_formatted()))
            .css_classes(vec!["dim-label"])
            .halign(gtk4::Align::Start)
            .build();
        info.append(&playtime);
    }

    // Buttons
    let buttons_box = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(12)
        .margin_top(12)
        .build();

    let play_btn = Button::builder()
        .label(if game.installed { "Jogar" } else { "Instalar" })
        .css_classes(vec!["play-button"])
        .build();
    buttons_box.append(&play_btn);

    let details_btn = Button::builder()
        .label("Detalhes")
        .css_classes(vec!["flat"])
        .build();
    buttons_box.append(&details_btn);

    info.append(&buttons_box);
    content.append(&info);

    banner.set_child(Some(&content));
    banner
}
