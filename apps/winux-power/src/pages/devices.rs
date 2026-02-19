// USB and peripheral devices power management page

use gtk4::prelude::*;
use gtk4::{Box, Orientation, Label};
use libadwaita as adw;
use libadwaita::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

use crate::backend::PowerManager;

pub struct DevicesPage {
    container: gtk4::ScrolledWindow,
}

impl DevicesPage {
    pub fn new(manager: Rc<RefCell<PowerManager>>) -> Self {
        let page = adw::PreferencesPage::new();

        // USB Power Management Group
        let usb_group = adw::PreferencesGroup::builder()
            .title("Gerenciamento de Energia USB")
            .description("Configure economia de energia para dispositivos USB")
            .build();

        // USB autosuspend
        let autosuspend_row = adw::SwitchRow::builder()
            .title("Suspensao Automatica USB")
            .subtitle("Suspende dispositivos USB inativos")
            .active(true)
            .build();
        usb_group.add(&autosuspend_row);

        // Autosuspend timeout
        let timeout_row = adw::ComboRow::builder()
            .title("Tempo para Suspensao")
            .subtitle("Segundos de inatividade")
            .build();
        let timeouts = gtk4::StringList::new(&[
            "1 segundo",
            "2 segundos",
            "5 segundos",
            "10 segundos",
            "30 segundos",
        ]);
        timeout_row.set_model(Some(&timeouts));
        timeout_row.set_selected(1);
        usb_group.add(&timeout_row);

        // Wake on USB
        let wake_row = adw::SwitchRow::builder()
            .title("Acordar por USB")
            .subtitle("Permite dispositivos USB acordarem o sistema")
            .active(true)
            .build();
        usb_group.add(&wake_row);

        page.add(&usb_group);

        // Connected USB Devices Group
        let connected_group = adw::PreferencesGroup::builder()
            .title("Dispositivos USB Conectados")
            .description("Dispositivos atualmente conectados")
            .build();

        // Example devices
        let devices = [
            ("USB Mouse", "Logitech MX Master 3", "input-mouse-symbolic", true, "0.5 W"),
            ("USB Keyboard", "Keychron K2", "input-keyboard-symbolic", true, "0.3 W"),
            ("USB Hub", "Hub USB 3.0", "drive-harddisk-usb-symbolic", false, "2.1 W"),
            ("Webcam", "Logitech C920", "camera-web-symbolic", true, "0.8 W"),
            ("External Drive", "Samsung T7 1TB", "drive-harddisk-symbolic", true, "1.2 W"),
        ];

        for (name, model, icon, can_suspend, power) in devices {
            let row = adw::ExpanderRow::builder()
                .title(name)
                .subtitle(model)
                .build();
            row.add_prefix(&gtk4::Image::from_icon_name(icon));

            // Power consumption badge
            let power_label = Label::new(Some(power));
            power_label.add_css_class("dim-label");
            row.add_suffix(&power_label);

            // Power saving toggle
            if can_suspend {
                let power_save_row = adw::SwitchRow::builder()
                    .title("Economia de Energia")
                    .subtitle("Permitir suspensao automatica")
                    .active(true)
                    .build();
                row.add_row(&power_save_row);
            } else {
                let info_row = adw::ActionRow::builder()
                    .title("Hub USB")
                    .subtitle("Nao suporta suspensao automatica")
                    .build();
                row.add_row(&info_row);
            }

            // Detailed info row
            let details_row = adw::ActionRow::builder()
                .title("Detalhes")
                .subtitle("Ver informacoes detalhadas do dispositivo")
                .activatable(true)
                .build();
            details_row.add_suffix(&gtk4::Image::from_icon_name("go-next-symbolic"));
            row.add_row(&details_row);

            connected_group.add(&row);
        }

        page.add(&connected_group);

        // Bluetooth Power Management Group
        let bt_group = adw::PreferencesGroup::builder()
            .title("Bluetooth")
            .description("Gerenciamento de energia Bluetooth")
            .build();

        // Bluetooth power management
        let bt_power_row = adw::SwitchRow::builder()
            .title("Economia de Energia Bluetooth")
            .subtitle("Reduz consumo do adaptador Bluetooth")
            .active(true)
            .build();
        bt_power_row.add_prefix(&gtk4::Image::from_icon_name("bluetooth-symbolic"));
        bt_group.add(&bt_power_row);

        // Disable Bluetooth on suspend
        let bt_suspend_row = adw::SwitchRow::builder()
            .title("Desligar ao Suspender")
            .subtitle("Desabilita Bluetooth durante suspensao")
            .active(false)
            .build();
        bt_group.add(&bt_suspend_row);

        page.add(&bt_group);

        // PCI Devices Group
        let pci_group = adw::PreferencesGroup::builder()
            .title("Dispositivos PCI")
            .description("Gerenciamento de energia para placa de rede, etc")
            .build();

        // WiFi power management
        let wifi_power_row = adw::SwitchRow::builder()
            .title("WiFi - Economia de Energia")
            .subtitle("Pode aumentar latencia de rede")
            .active(true)
            .build();
        wifi_power_row.add_prefix(&gtk4::Image::from_icon_name("network-wireless-symbolic"));
        pci_group.add(&wifi_power_row);

        // Ethernet power management
        let eth_power_row = adw::SwitchRow::builder()
            .title("Ethernet - Wake on LAN")
            .subtitle("Permite acordar o sistema via rede")
            .active(false)
            .build();
        eth_power_row.add_prefix(&gtk4::Image::from_icon_name("network-wired-symbolic"));
        pci_group.add(&eth_power_row);

        // Audio power management
        let audio_power_row = adw::SwitchRow::builder()
            .title("Audio - Economia de Energia")
            .subtitle("Desliga audio quando inativo")
            .active(true)
            .build();
        audio_power_row.add_prefix(&gtk4::Image::from_icon_name("audio-speakers-symbolic"));
        pci_group.add(&audio_power_row);

        // Audio timeout
        let audio_timeout_row = adw::ComboRow::builder()
            .title("Timeout de Audio")
            .subtitle("Tempo para desligar audio")
            .build();
        let audio_times = gtk4::StringList::new(&[
            "1 segundo",
            "5 segundos",
            "10 segundos",
            "30 segundos",
            "1 minuto",
        ]);
        audio_timeout_row.set_model(Some(&audio_times));
        audio_timeout_row.set_selected(2);
        pci_group.add(&audio_timeout_row);

        page.add(&pci_group);

        // GPU Power Management Group
        let gpu_group = adw::PreferencesGroup::builder()
            .title("Placa de Video")
            .description("Configuracoes de energia da GPU")
            .build();

        // GPU power profile
        let gpu_profile_row = adw::ComboRow::builder()
            .title("Perfil de Energia")
            .subtitle("Modo de operacao da GPU")
            .build();
        gpu_profile_row.add_prefix(&gtk4::Image::from_icon_name("video-display-symbolic"));
        let gpu_profiles = gtk4::StringList::new(&[
            "Automatico",
            "Baixo Consumo",
            "Desempenho",
        ]);
        gpu_profile_row.set_model(Some(&gpu_profiles));
        gpu_profile_row.set_selected(0);
        gpu_group.add(&gpu_profile_row);

        // Switchable graphics (if available)
        let switchable_row = adw::ComboRow::builder()
            .title("Graficos Hibridos")
            .subtitle("Para notebooks com GPU dedicada")
            .build();
        let switchable_modes = gtk4::StringList::new(&[
            "Hibrido (Automatico)",
            "Integrada Apenas",
            "Dedicada Apenas",
        ]);
        switchable_row.set_model(Some(&switchable_modes));
        switchable_row.set_selected(0);
        gpu_group.add(&switchable_row);

        // DRM power management
        let drm_row = adw::SwitchRow::builder()
            .title("Runtime PM da GPU")
            .subtitle("Desliga GPU quando nao em uso")
            .active(true)
            .build();
        gpu_group.add(&drm_row);

        page.add(&gpu_group);

        // SATA/NVMe Power Management
        let storage_group = adw::PreferencesGroup::builder()
            .title("Armazenamento")
            .description("Economia de energia para discos")
            .build();

        // SATA power management
        let sata_row = adw::ComboRow::builder()
            .title("AHCI Link Power Management")
            .subtitle("Modo de economia para SATA")
            .build();
        sata_row.add_prefix(&gtk4::Image::from_icon_name("drive-harddisk-symbolic"));
        let sata_modes = gtk4::StringList::new(&[
            "max_performance",
            "med_power_with_dipm",
            "min_power",
        ]);
        sata_row.set_model(Some(&sata_modes));
        sata_row.set_selected(1);
        storage_group.add(&sata_row);

        // NVMe power management
        let nvme_row = adw::ComboRow::builder()
            .title("NVMe Power State")
            .subtitle("Estado de energia para SSDs NVMe")
            .build();
        nvme_row.add_prefix(&gtk4::Image::from_icon_name("drive-harddisk-solidstate-symbolic"));
        let nvme_states = gtk4::StringList::new(&[
            "Automatico",
            "PS0 (Maximo Desempenho)",
            "PS3 (Economia)",
        ]);
        nvme_row.set_model(Some(&nvme_states));
        nvme_row.set_selected(0);
        storage_group.add(&nvme_row);

        // Spindown time for HDDs
        let spindown_row = adw::ComboRow::builder()
            .title("Tempo para Desligar HDD")
            .subtitle("Parar rotacao de discos magneticos")
            .build();
        let spindown_times = gtk4::StringList::new(&[
            "5 minutos",
            "10 minutos",
            "20 minutos",
            "30 minutos",
            "1 hora",
            "Nunca",
        ]);
        spindown_row.set_model(Some(&spindown_times));
        spindown_row.set_selected(2);
        storage_group.add(&spindown_row);

        page.add(&storage_group);

        let container = gtk4::ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .child(&page)
            .build();

        Self { container }
    }

    pub fn widget(&self) -> &gtk4::ScrolledWindow {
        &self.container
    }
}
