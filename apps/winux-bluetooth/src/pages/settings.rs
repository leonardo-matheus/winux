//! Bluetooth settings page

use gtk4::prelude::*;
use libadwaita as adw;
use libadwaita::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

use crate::bluez::BluetoothManager;

/// Settings page for Bluetooth configuration
pub struct SettingsPage {
    widget: gtk4::ScrolledWindow,
    manager: Rc<RefCell<BluetoothManager>>,
}

impl SettingsPage {
    pub fn new(manager: Rc<RefCell<BluetoothManager>>) -> Self {
        let page = adw::PreferencesPage::new();
        page.set_title("Configuracoes");
        page.set_icon_name(Some("emblem-system-symbolic"));

        // Device identity group
        let identity_group = adw::PreferencesGroup::builder()
            .title("Identidade do Dispositivo")
            .description("Como este computador aparece para outros dispositivos")
            .build();

        // Device name
        let name_row = adw::EntryRow::builder()
            .title("Nome do Dispositivo")
            .text("Winux Desktop")
            .build();
        identity_group.add(&name_row);

        // MAC Address (read-only)
        let mac_row = adw::ActionRow::builder()
            .title("Endereco MAC")
            .subtitle("00:1A:7D:DA:71:13")
            .build();
        mac_row.add_prefix(&gtk4::Image::from_icon_name("network-wired-symbolic"));

        let copy_btn = gtk4::Button::from_icon_name("edit-copy-symbolic");
        copy_btn.add_css_class("flat");
        copy_btn.set_valign(gtk4::Align::Center);
        copy_btn.set_tooltip_text(Some("Copiar endereco"));
        mac_row.add_suffix(&copy_btn);
        identity_group.add(&mac_row);

        page.add(&identity_group);

        // Visibility group
        let visibility_group = adw::PreferencesGroup::builder()
            .title("Visibilidade")
            .build();

        let visible_row = adw::SwitchRow::builder()
            .title("Visivel para outros dispositivos")
            .subtitle("Permite que dispositivos proximos encontrem este computador")
            .active(false)
            .build();
        visibility_group.add(&visible_row);

        let timeout_row = adw::ComboRow::builder()
            .title("Tempo de Visibilidade")
            .subtitle("Desligar visibilidade automaticamente apos")
            .build();
        let timeouts = gtk4::StringList::new(&[
            "1 minuto",
            "2 minutos",
            "5 minutos",
            "10 minutos",
            "Sempre visivel",
        ]);
        timeout_row.set_model(Some(&timeouts));
        timeout_row.set_selected(2);
        visibility_group.add(&timeout_row);

        page.add(&visibility_group);

        // Pairing group
        let pairing_group = adw::PreferencesGroup::builder()
            .title("Pareamento")
            .build();

        let pairable_row = adw::SwitchRow::builder()
            .title("Permitir Pareamento")
            .subtitle("Aceitar solicitacoes de pareamento de outros dispositivos")
            .active(true)
            .build();
        pairing_group.add(&pairable_row);

        let pairable_timeout_row = adw::ComboRow::builder()
            .title("Tempo para Pareamento")
            .subtitle("Janela de tempo para aceitar pareamentos")
            .build();
        let pair_timeouts = gtk4::StringList::new(&[
            "30 segundos",
            "1 minuto",
            "2 minutos",
            "5 minutos",
            "Sem limite",
        ]);
        pairable_timeout_row.set_model(Some(&pair_timeouts));
        pairable_timeout_row.set_selected(1);
        pairing_group.add(&pairable_timeout_row);

        let confirm_row = adw::SwitchRow::builder()
            .title("Confirmar Pareamentos")
            .subtitle("Sempre pedir confirmacao antes de parear")
            .active(true)
            .build();
        pairing_group.add(&confirm_row);

        page.add(&pairing_group);

        // Audio profiles group
        let audio_group = adw::PreferencesGroup::builder()
            .title("Perfis de Audio")
            .description("Configuracoes para dispositivos de audio Bluetooth")
            .build();

        let a2dp_row = adw::SwitchRow::builder()
            .title("A2DP (Audio de Alta Qualidade)")
            .subtitle("Perfil para streaming de musica")
            .active(true)
            .build();
        audio_group.add(&a2dp_row);

        let hfp_row = adw::SwitchRow::builder()
            .title("HFP (Hands-Free)")
            .subtitle("Perfil para chamadas com microfone")
            .active(true)
            .build();
        audio_group.add(&hfp_row);

        let hsp_row = adw::SwitchRow::builder()
            .title("HSP (Headset)")
            .subtitle("Perfil basico de headset")
            .active(true)
            .build();
        audio_group.add(&hsp_row);

        let codec_row = adw::ComboRow::builder()
            .title("Codec Preferido")
            .subtitle("Codec de audio para A2DP")
            .build();
        let codecs = gtk4::StringList::new(&[
            "Automatico",
            "LDAC (Alta Qualidade)",
            "aptX HD",
            "aptX",
            "AAC",
            "SBC",
        ]);
        codec_row.set_model(Some(&codecs));
        audio_group.add(&codec_row);

        page.add(&audio_group);

        // Input devices group
        let input_group = adw::PreferencesGroup::builder()
            .title("Dispositivos de Entrada")
            .description("Configuracoes para teclados e mouses")
            .build();

        let hid_row = adw::SwitchRow::builder()
            .title("HID (Human Interface Device)")
            .subtitle("Suporte para teclados e mouses Bluetooth")
            .active(true)
            .build();
        input_group.add(&hid_row);

        let reconnect_row = adw::SwitchRow::builder()
            .title("Reconectar Automaticamente")
            .subtitle("Conectar automaticamente a dispositivos HID conhecidos")
            .active(true)
            .build();
        input_group.add(&reconnect_row);

        page.add(&input_group);

        // Power management group
        let power_group = adw::PreferencesGroup::builder()
            .title("Energia")
            .build();

        let auto_off_row = adw::SwitchRow::builder()
            .title("Desligar Bluetooth quando ocioso")
            .subtitle("Economiza bateria quando nao ha dispositivos conectados")
            .active(false)
            .build();
        power_group.add(&auto_off_row);

        let auto_off_timeout_row = adw::ComboRow::builder()
            .title("Tempo de Inatividade")
            .subtitle("Desligar Bluetooth apos")
            .build();
        let idle_timeouts = gtk4::StringList::new(&[
            "5 minutos",
            "10 minutos",
            "30 minutos",
            "1 hora",
        ]);
        auto_off_timeout_row.set_model(Some(&idle_timeouts));
        power_group.add(&auto_off_timeout_row);

        page.add(&power_group);

        // Advanced group
        let advanced_group = adw::PreferencesGroup::builder()
            .title("Avancado")
            .build();

        let adapter_row = adw::ComboRow::builder()
            .title("Adaptador Bluetooth")
            .subtitle("Selecione o adaptador a usar")
            .build();
        let adapters = gtk4::StringList::new(&["hci0 - Intel AX211 Bluetooth"]);
        adapter_row.set_model(Some(&adapters));
        advanced_group.add(&adapter_row);

        let reset_row = adw::ActionRow::builder()
            .title("Resetar Adaptador")
            .subtitle("Reiniciar o servico Bluetooth")
            .activatable(true)
            .build();
        reset_row.add_prefix(&gtk4::Image::from_icon_name("view-refresh-symbolic"));
        advanced_group.add(&reset_row);

        let logs_row = adw::ActionRow::builder()
            .title("Ver Logs do Bluetooth")
            .subtitle("Abrir logs do BlueZ para diagnostico")
            .activatable(true)
            .build();
        logs_row.add_prefix(&gtk4::Image::from_icon_name("utilities-terminal-symbolic"));
        logs_row.add_suffix(&gtk4::Image::from_icon_name("go-next-symbolic"));
        advanced_group.add(&logs_row);

        page.add(&advanced_group);

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
