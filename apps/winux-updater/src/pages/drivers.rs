//! Drivers page - Manage hardware drivers

use gtk4::prelude::*;
use gtk4::{Box, Button, Label, Orientation, ProgressBar, ScrolledWindow, Spinner};
use libadwaita as adw;
use adw::prelude::*;
use adw::{ActionRow, PreferencesGroup, PreferencesPage, StatusPage, ExpanderRow, SwitchRow};
use std::cell::RefCell;
use std::rc::Rc;

use crate::backend::UpdateManager;

pub struct DriversPage {
    widget: Box,
    content_stack: gtk4::Stack,
    update_manager: Rc<RefCell<UpdateManager>>,
}

impl DriversPage {
    pub fn new(update_manager: Rc<RefCell<UpdateManager>>) -> Self {
        let widget = Box::new(Orientation::Vertical, 0);

        // Stack for different states
        let content_stack = gtk4::Stack::new();
        content_stack.set_vexpand(true);

        // Loading state
        let loading_page = Self::create_loading_state();
        content_stack.add_named(&loading_page, Some("loading"));

        // Drivers list
        let drivers_content = Self::create_drivers_list();
        content_stack.add_named(&drivers_content, Some("drivers"));

        // Default to drivers list
        content_stack.set_visible_child_name("drivers");

        widget.append(&content_stack);

        Self {
            widget,
            content_stack,
            update_manager,
        }
    }

    fn create_loading_state() -> StatusPage {
        let spinner = Spinner::new();
        spinner.set_size_request(48, 48);
        spinner.start();

        StatusPage::builder()
            .icon_name("emblem-synchronizing-symbolic")
            .title("Detectando Hardware")
            .description("Buscando drivers disponiveis...")
            .child(&spinner)
            .build()
    }

    fn create_drivers_list() -> ScrolledWindow {
        let page = PreferencesPage::new();

        // GPU Drivers
        let gpu_group = PreferencesGroup::builder()
            .title("Placa de Video")
            .description("Drivers graficos detectados")
            .build();

        // Current GPU info
        let gpu_info = ActionRow::builder()
            .title("NVIDIA GeForce RTX 4070")
            .subtitle("Dispositivo PCI detectado")
            .build();
        gpu_info.add_prefix(&gtk4::Image::from_icon_name("video-display-symbolic"));
        gpu_group.add(&gpu_info);

        // NVIDIA proprietary driver
        let nvidia_expander = ExpanderRow::builder()
            .title("nvidia-driver-550 (Proprietario, Recomendado)")
            .subtitle("Versao 550.67.01 - Driver oficial NVIDIA")
            .build();
        nvidia_expander.add_prefix(&gtk4::Image::from_icon_name("emblem-default-symbolic"));

        let nvidia_radio = gtk4::CheckButton::new();
        nvidia_radio.set_active(true);
        nvidia_radio.set_valign(gtk4::Align::Center);
        nvidia_expander.add_suffix(&nvidia_radio);

        let nvidia_info1 = ActionRow::builder()
            .title("Suporte CUDA")
            .subtitle("Sim - 12.4")
            .build();
        nvidia_expander.add_row(&nvidia_info1);

        let nvidia_info2 = ActionRow::builder()
            .title("Vulkan")
            .subtitle("1.3.277")
            .build();
        nvidia_expander.add_row(&nvidia_info2);

        let nvidia_info3 = ActionRow::builder()
            .title("Status")
            .subtitle("Instalado e em uso")
            .build();
        nvidia_info3.add_suffix(&gtk4::Image::from_icon_name("emblem-ok-symbolic"));
        nvidia_expander.add_row(&nvidia_info3);

        gpu_group.add(&nvidia_expander);

        // Nouveau (open source)
        let nouveau_expander = ExpanderRow::builder()
            .title("nouveau (Open Source)")
            .subtitle("Driver open source da comunidade")
            .build();

        let nouveau_radio = gtk4::CheckButton::new();
        nouveau_radio.set_group(Some(&nvidia_radio));
        nouveau_radio.set_valign(gtk4::Align::Center);
        nouveau_expander.add_suffix(&nouveau_radio);

        let nouveau_info1 = ActionRow::builder()
            .title("Suporte CUDA")
            .subtitle("Nao")
            .build();
        nouveau_expander.add_row(&nouveau_info1);

        let nouveau_info2 = ActionRow::builder()
            .title("Desempenho")
            .subtitle("Limitado - sem reclocking")
            .build();
        nouveau_expander.add_row(&nouveau_info2);

        gpu_group.add(&nouveau_expander);

        page.add(&gpu_group);

        // Wireless drivers
        let wifi_group = PreferencesGroup::builder()
            .title("Rede Wireless")
            .build();

        let wifi_info = ActionRow::builder()
            .title("Intel Wi-Fi 6 AX201")
            .subtitle("iwlwifi - Driver Intel incluido no kernel")
            .build();
        wifi_info.add_prefix(&gtk4::Image::from_icon_name("network-wireless-symbolic"));

        let wifi_status = Label::new(Some("Em uso"));
        wifi_status.add_css_class("success");
        wifi_status.set_valign(gtk4::Align::Center);
        wifi_info.add_suffix(&wifi_status);
        wifi_group.add(&wifi_info);

        // Firmware
        let firmware_row = ActionRow::builder()
            .title("Firmware")
            .subtitle("iwlwifi-ty-a0-gf-a0-72.ucode")
            .build();
        firmware_row.add_prefix(&gtk4::Image::from_icon_name("application-x-firmware-symbolic"));
        wifi_group.add(&firmware_row);

        page.add(&wifi_group);

        // Bluetooth
        let bt_group = PreferencesGroup::builder()
            .title("Bluetooth")
            .build();

        let bt_info = ActionRow::builder()
            .title("Intel Bluetooth AX201")
            .subtitle("btusb + btintel - Drivers incluidos")
            .build();
        bt_info.add_prefix(&gtk4::Image::from_icon_name("bluetooth-active-symbolic"));

        let bt_status = Label::new(Some("Em uso"));
        bt_status.add_css_class("success");
        bt_status.set_valign(gtk4::Align::Center);
        bt_info.add_suffix(&bt_status);
        bt_group.add(&bt_info);

        page.add(&bt_group);

        // Audio
        let audio_group = PreferencesGroup::builder()
            .title("Audio")
            .build();

        let audio_info = ActionRow::builder()
            .title("Intel High Definition Audio")
            .subtitle("snd_hda_intel - Driver ALSA")
            .build();
        audio_info.add_prefix(&gtk4::Image::from_icon_name("audio-card-symbolic"));

        let audio_status = Label::new(Some("Em uso"));
        audio_status.add_css_class("success");
        audio_status.set_valign(gtk4::Align::Center);
        audio_info.add_suffix(&audio_status);
        audio_group.add(&audio_info);

        let pipewire_row = ActionRow::builder()
            .title("PipeWire")
            .subtitle("Servidor de audio ativo")
            .build();
        pipewire_row.add_prefix(&gtk4::Image::from_icon_name("audio-speakers-symbolic"));
        audio_group.add(&pipewire_row);

        page.add(&audio_group);

        // Printers
        let printer_group = PreferencesGroup::builder()
            .title("Impressoras")
            .build();

        let printer_row = ActionRow::builder()
            .title("HP LaserJet Pro M404dn")
            .subtitle("hplip - Drivers HP instalados")
            .build();
        printer_row.add_prefix(&gtk4::Image::from_icon_name("printer-symbolic"));

        let printer_status = Label::new(Some("Pronto"));
        printer_status.add_css_class("success");
        printer_status.set_valign(gtk4::Align::Center);
        printer_row.add_suffix(&printer_status);
        printer_group.add(&printer_row);

        let scanner_row = ActionRow::builder()
            .title("Scanner")
            .subtitle("SANE + hpaio")
            .build();
        scanner_row.add_prefix(&gtk4::Image::from_icon_name("scanner-symbolic"));
        printer_group.add(&scanner_row);

        page.add(&printer_group);

        // Additional drivers
        let additional_group = PreferencesGroup::builder()
            .title("Drivers Adicionais Disponiveis")
            .description("Drivers opcionais que podem ser instalados")
            .build();

        // NVIDIA CUDA toolkit
        let cuda_row = ActionRow::builder()
            .title("nvidia-cuda-toolkit")
            .subtitle("Ferramentas de desenvolvimento CUDA 12.4")
            .activatable(true)
            .build();
        cuda_row.add_prefix(&gtk4::Image::from_icon_name("applications-science-symbolic"));

        let cuda_btn = Button::with_label("Instalar");
        cuda_btn.add_css_class("flat");
        cuda_btn.set_valign(gtk4::Align::Center);
        cuda_row.add_suffix(&cuda_btn);
        additional_group.add(&cuda_row);

        // Intel microcode
        let microcode_row = ActionRow::builder()
            .title("intel-microcode")
            .subtitle("Atualizacoes de microcódigo do processador")
            .build();
        microcode_row.add_prefix(&gtk4::Image::from_icon_name("cpu-symbolic"));

        let microcode_status = Label::new(Some("Instalado"));
        microcode_status.add_css_class("success");
        microcode_status.set_valign(gtk4::Align::Center);
        microcode_row.add_suffix(&microcode_status);
        additional_group.add(&microcode_row);

        // Fingerprint reader
        let fingerprint_row = ActionRow::builder()
            .title("fprintd + libfprint")
            .subtitle("Suporte a leitor de digitais")
            .activatable(true)
            .build();
        fingerprint_row.add_prefix(&gtk4::Image::from_icon_name("fingerprint-symbolic"));

        let fp_btn = Button::with_label("Instalar");
        fp_btn.add_css_class("flat");
        fp_btn.set_valign(gtk4::Align::Center);
        fingerprint_row.add_suffix(&fp_btn);
        additional_group.add(&fingerprint_row);

        // Thunderbolt
        let thunderbolt_row = ActionRow::builder()
            .title("bolt")
            .subtitle("Gerenciamento de dispositivos Thunderbolt")
            .build();
        thunderbolt_row.add_prefix(&gtk4::Image::from_icon_name("thunderbolt-symbolic"));

        let tb_status = Label::new(Some("Instalado"));
        tb_status.add_css_class("success");
        tb_status.set_valign(gtk4::Align::Center);
        thunderbolt_row.add_suffix(&tb_status);
        additional_group.add(&thunderbolt_row);

        page.add(&additional_group);

        // Settings
        let settings_group = PreferencesGroup::builder()
            .title("Configuracoes de Drivers")
            .build();

        let secure_boot = SwitchRow::builder()
            .title("Secure Boot")
            .subtitle("Drivers assinados necessarios")
            .active(false)
            .build();
        settings_group.add(&secure_boot);

        let auto_detect = SwitchRow::builder()
            .title("Detectar Hardware Automaticamente")
            .subtitle("Buscar drivers ao conectar novos dispositivos")
            .active(true)
            .build();
        settings_group.add(&auto_detect);

        let dkms = SwitchRow::builder()
            .title("DKMS")
            .subtitle("Recompilar drivers automaticamente ao atualizar kernel")
            .active(true)
            .build();
        settings_group.add(&dkms);

        page.add(&settings_group);

        // Hardware info
        let hw_group = PreferencesGroup::builder()
            .title("Informacoes do Sistema")
            .build();

        let lspci_row = ActionRow::builder()
            .title("Listar Hardware PCI")
            .subtitle("Executar lspci -v")
            .activatable(true)
            .build();
        lspci_row.add_prefix(&gtk4::Image::from_icon_name("computer-symbolic"));
        lspci_row.add_suffix(&gtk4::Image::from_icon_name("go-next-symbolic"));
        hw_group.add(&lspci_row);

        let lsusb_row = ActionRow::builder()
            .title("Listar Dispositivos USB")
            .subtitle("Executar lsusb")
            .activatable(true)
            .build();
        lsusb_row.add_prefix(&gtk4::Image::from_icon_name("drive-removable-media-symbolic"));
        lsusb_row.add_suffix(&gtk4::Image::from_icon_name("go-next-symbolic"));
        hw_group.add(&lsusb_row);

        let dmesg_row = ActionRow::builder()
            .title("Ver Mensagens do Kernel")
            .subtitle("Executar dmesg (ultimas mensagens)")
            .activatable(true)
            .build();
        dmesg_row.add_prefix(&gtk4::Image::from_icon_name("utilities-terminal-symbolic"));
        dmesg_row.add_suffix(&gtk4::Image::from_icon_name("go-next-symbolic"));
        hw_group.add(&dmesg_row);

        page.add(&hw_group);

        let scrolled = ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .child(&page)
            .build();

        scrolled
    }

    pub fn widget(&self) -> &Box {
        &self.widget
    }

    pub fn refresh(&self) {
        self.content_stack.set_visible_child_name("loading");
        // Would trigger async hardware detection
    }
}
