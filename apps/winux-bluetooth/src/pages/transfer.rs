//! File transfer page (OBEX)

use gtk4::prelude::*;
use libadwaita as adw;
use libadwaita::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

use crate::bluez::BluetoothManager;

/// File transfer page using OBEX protocol
pub struct TransferPage {
    widget: gtk4::ScrolledWindow,
    manager: Rc<RefCell<BluetoothManager>>,
}

impl TransferPage {
    pub fn new(manager: Rc<RefCell<BluetoothManager>>) -> Self {
        let page = adw::PreferencesPage::new();
        page.set_title("Transferir");
        page.set_icon_name(Some("folder-download-symbolic"));

        // Send file group
        let send_group = adw::PreferencesGroup::builder()
            .title("Enviar Arquivo")
            .description("Envie arquivos para dispositivos pareados")
            .build();

        // Device selection
        let device_row = adw::ComboRow::builder()
            .title("Dispositivo de Destino")
            .subtitle("Selecione o dispositivo para enviar")
            .build();
        let devices = gtk4::StringList::new(&[
            "Fones Bluetooth XM5",
            "iPhone 15",
            "Galaxy Buds Pro",
        ]);
        device_row.set_model(Some(&devices));
        send_group.add(&device_row);

        // File selection
        let file_row = adw::ActionRow::builder()
            .title("Arquivo Selecionado")
            .subtitle("Nenhum arquivo selecionado")
            .activatable(true)
            .build();
        file_row.add_prefix(&gtk4::Image::from_icon_name("document-open-symbolic"));

        let select_btn = gtk4::Button::with_label("Selecionar");
        select_btn.add_css_class("flat");
        select_btn.set_valign(gtk4::Align::Center);
        file_row.add_suffix(&select_btn);
        send_group.add(&file_row);

        // Send button
        let send_action_row = adw::ActionRow::new();
        let send_btn = gtk4::Button::with_label("Enviar Arquivo");
        send_btn.add_css_class("suggested-action");
        send_btn.add_css_class("pill");
        send_btn.set_halign(gtk4::Align::Center);
        send_btn.set_margin_top(8);
        send_btn.set_margin_bottom(8);
        send_action_row.set_child(Some(&send_btn));
        send_group.add(&send_action_row);

        page.add(&send_group);

        // Active transfers group
        let active_group = adw::PreferencesGroup::builder()
            .title("Transferencias Ativas")
            .build();

        // Sample active transfer
        let transfer_row = adw::ActionRow::builder()
            .title("documento.pdf")
            .subtitle("Enviando para iPhone 15 - 45%")
            .build();
        transfer_row.add_prefix(&gtk4::Image::from_icon_name("document-send-symbolic"));

        let progress = gtk4::ProgressBar::new();
        progress.set_fraction(0.45);
        progress.set_valign(gtk4::Align::Center);
        progress.set_size_request(100, -1);
        transfer_row.add_suffix(&progress);

        let cancel_btn = gtk4::Button::from_icon_name("process-stop-symbolic");
        cancel_btn.add_css_class("flat");
        cancel_btn.set_valign(gtk4::Align::Center);
        cancel_btn.set_tooltip_text(Some("Cancelar transferencia"));
        transfer_row.add_suffix(&cancel_btn);

        active_group.add(&transfer_row);

        page.add(&active_group);

        // Receive files group
        let receive_group = adw::PreferencesGroup::builder()
            .title("Receber Arquivos")
            .description("Configure como receber arquivos via Bluetooth")
            .build();

        let accept_row = adw::SwitchRow::builder()
            .title("Aceitar Arquivos Automaticamente")
            .subtitle("Aceitar transferencias de dispositivos pareados")
            .active(false)
            .build();
        receive_group.add(&accept_row);

        let folder_row = adw::ActionRow::builder()
            .title("Pasta de Download")
            .subtitle("~/Downloads/Bluetooth")
            .activatable(true)
            .build();
        folder_row.add_prefix(&gtk4::Image::from_icon_name("folder-symbolic"));
        folder_row.add_suffix(&gtk4::Image::from_icon_name("go-next-symbolic"));
        receive_group.add(&folder_row);

        let notify_row = adw::SwitchRow::builder()
            .title("Notificar ao Receber")
            .subtitle("Mostrar notificacao quando um arquivo for recebido")
            .active(true)
            .build();
        receive_group.add(&notify_row);

        page.add(&receive_group);

        // Transfer history group
        let history_group = adw::PreferencesGroup::builder()
            .title("Historico de Transferencias")
            .build();

        let history_items = [
            ("foto.jpg", "Recebido de iPhone 15", "Ontem 14:30", true),
            ("musica.mp3", "Enviado para Galaxy Buds", "Ontem 10:15", true),
            ("video.mp4", "Falhou - Conexao perdida", "18/02/2026", false),
            ("documento.docx", "Recebido de Laptop", "17/02/2026", true),
        ];

        for (name, description, time, success) in history_items {
            let row = adw::ActionRow::builder()
                .title(name)
                .subtitle(&format!("{} - {}", description, time))
                .activatable(true)
                .build();

            let icon_name = if success {
                "emblem-ok-symbolic"
            } else {
                "dialog-error-symbolic"
            };
            let icon = gtk4::Image::from_icon_name(icon_name);
            if success {
                icon.add_css_class("success");
            } else {
                icon.add_css_class("error");
            }
            row.add_prefix(&icon);

            row.add_suffix(&gtk4::Image::from_icon_name("go-next-symbolic"));
            history_group.add(&row);
        }

        let clear_row = adw::ActionRow::builder()
            .title("Limpar Historico")
            .activatable(true)
            .build();
        clear_row.add_prefix(&gtk4::Image::from_icon_name("user-trash-symbolic"));
        history_group.add(&clear_row);

        page.add(&history_group);

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
