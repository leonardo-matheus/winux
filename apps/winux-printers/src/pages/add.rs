//! Add printer page - discovery and setup

use gtk4::prelude::*;
use libadwaita as adw;
use libadwaita::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

use crate::cups::{CupsManager, DiscoveredPrinter, ConnectionType};

/// Add printer page - discover and configure new printers
pub struct AddPrinterPage {
    widget: gtk4::ScrolledWindow,
    #[allow(dead_code)]
    manager: Rc<RefCell<CupsManager>>,
}

impl AddPrinterPage {
    pub fn new(manager: Rc<RefCell<CupsManager>>) -> Self {
        let page = adw::PreferencesPage::new();
        page.set_title("Adicionar Impressora");
        page.set_icon_name(Some("list-add-symbolic"));

        // Discovery group
        let discovery_group = adw::PreferencesGroup::builder()
            .title("Impressoras Encontradas")
            .description("Impressoras descobertas automaticamente na rede")
            .build();

        // Scanning indicator
        let scanning_row = adw::ActionRow::builder()
            .title("Buscando impressoras...")
            .subtitle("Aguarde enquanto a rede e escaneada")
            .build();
        let spinner = gtk4::Spinner::new();
        spinner.start();
        spinner.set_valign(gtk4::Align::Center);
        scanning_row.add_prefix(&spinner);

        // Refresh button
        let refresh_btn = gtk4::Button::from_icon_name("view-refresh-symbolic");
        refresh_btn.add_css_class("flat");
        refresh_btn.set_valign(gtk4::Align::Center);
        refresh_btn.set_tooltip_text(Some("Buscar novamente"));
        scanning_row.add_suffix(&refresh_btn);
        discovery_group.add(&scanning_row);

        // Sample discovered printers
        let discovered = vec![
            DiscoveredPrinter::new(
                "HP LaserJet Pro M404dn",
                "192.168.1.100",
                ConnectionType::Ipp,
                Some("HP Inc."),
            ),
            DiscoveredPrinter::new(
                "Brother MFC-L2750DW",
                "192.168.1.101",
                ConnectionType::IppEverywhere,
                Some("Brother"),
            ),
            DiscoveredPrinter::new(
                "Epson EcoTank ET-4760",
                "Epson EcoTank._ipp._tcp.local",
                ConnectionType::Dnssd,
                Some("Epson"),
            ),
        ];

        for printer in &discovered {
            let row = Self::create_discovered_row(printer);
            discovery_group.add(&row);
        }

        page.add(&discovery_group);

        // Manual configuration group
        let manual_group = adw::PreferencesGroup::builder()
            .title("Configuracao Manual")
            .description("Adicionar impressora inserindo os dados manualmente")
            .build();

        // Connection type
        let conn_type_row = adw::ComboRow::builder()
            .title("Tipo de Conexao")
            .subtitle("Selecione como a impressora esta conectada")
            .build();
        let conn_types = gtk4::StringList::new(&[
            "IPP (Internet Printing Protocol)",
            "IPP Everywhere",
            "LPD (Line Printer Daemon)",
            "Socket/JetDirect (AppSocket)",
            "USB",
            "Windows (SMB/CIFS)",
        ]);
        conn_type_row.set_model(Some(&conn_types));
        manual_group.add(&conn_type_row);

        // Host/Address
        let host_row = adw::EntryRow::builder()
            .title("Endereco")
            .text("")
            .build();
        host_row.set_input_hints(gtk4::InputHints::NO_SPELLCHECK);
        manual_group.add(&host_row);

        // Port
        let port_row = adw::SpinRow::builder()
            .title("Porta")
            .subtitle("Porta de conexao (padrao: 631 para IPP)")
            .build();
        port_row.set_adjustment(&gtk4::Adjustment::new(631.0, 1.0, 65535.0, 1.0, 100.0, 0.0));
        manual_group.add(&port_row);

        // Queue name
        let queue_row = adw::EntryRow::builder()
            .title("Nome da Fila")
            .text("printers/")
            .build();
        queue_row.set_input_hints(gtk4::InputHints::NO_SPELLCHECK);
        manual_group.add(&queue_row);

        page.add(&manual_group);

        // Driver selection group
        let driver_group = adw::PreferencesGroup::builder()
            .title("Driver de Impressao")
            .description("Selecione o driver apropriado para sua impressora")
            .build();

        // Auto-detect
        let auto_driver_row = adw::SwitchRow::builder()
            .title("Detectar Driver Automaticamente")
            .subtitle("Usar IPP Everywhere ou driverless quando possivel")
            .active(true)
            .build();
        driver_group.add(&auto_driver_row);

        // Manufacturer
        let make_row = adw::ComboRow::builder()
            .title("Fabricante")
            .subtitle("Selecione o fabricante da impressora")
            .build();
        let manufacturers = gtk4::StringList::new(&[
            "Selecionar...",
            "Brother",
            "Canon",
            "Epson",
            "HP",
            "Kyocera",
            "Lexmark",
            "OKI",
            "Ricoh",
            "Samsung",
            "Xerox",
            "Generico",
        ]);
        make_row.set_model(Some(&manufacturers));
        driver_group.add(&make_row);

        // Model
        let model_row = adw::ComboRow::builder()
            .title("Modelo")
            .subtitle("Selecione o modelo da impressora")
            .sensitive(false)
            .build();
        let models = gtk4::StringList::new(&["Selecione primeiro o fabricante..."]);
        model_row.set_model(Some(&models));
        driver_group.add(&model_row);

        // PPD file
        let ppd_row = adw::ActionRow::builder()
            .title("Arquivo PPD Personalizado")
            .subtitle("Usar um arquivo PPD fornecido pelo fabricante")
            .activatable(true)
            .build();
        ppd_row.add_prefix(&gtk4::Image::from_icon_name("document-open-symbolic"));
        ppd_row.add_suffix(&gtk4::Image::from_icon_name("go-next-symbolic"));
        driver_group.add(&ppd_row);

        page.add(&driver_group);

        // Printer name group
        let name_group = adw::PreferencesGroup::builder()
            .title("Identificacao")
            .build();

        let name_row = adw::EntryRow::builder()
            .title("Nome da Impressora")
            .text("Nova-Impressora")
            .build();
        name_group.add(&name_row);

        let description_row = adw::EntryRow::builder()
            .title("Descricao")
            .text("")
            .build();
        description_row.set_input_hints(gtk4::InputHints::NO_SPELLCHECK);
        name_group.add(&description_row);

        let location_row = adw::EntryRow::builder()
            .title("Localizacao")
            .text("")
            .build();
        location_row.set_input_hints(gtk4::InputHints::NO_SPELLCHECK);
        name_group.add(&location_row);

        let shared_row = adw::SwitchRow::builder()
            .title("Compartilhar esta Impressora")
            .subtitle("Permitir que outros computadores imprimam nela")
            .active(false)
            .build();
        name_group.add(&shared_row);

        page.add(&name_group);

        // Actions group
        let actions_group = adw::PreferencesGroup::new();

        let add_btn = gtk4::Button::builder()
            .label("Adicionar Impressora")
            .css_classes(vec!["suggested-action".to_string(), "pill".to_string()])
            .halign(gtk4::Align::Center)
            .build();

        let btn_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        btn_box.set_halign(gtk4::Align::Center);
        btn_box.set_margin_top(12);
        btn_box.set_margin_bottom(12);
        btn_box.append(&add_btn);
        actions_group.add(&btn_box);

        page.add(&actions_group);

        let scrolled = gtk4::ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .child(&page)
            .build();

        Self {
            widget: scrolled,
            manager,
        }
    }

    fn create_discovered_row(printer: &DiscoveredPrinter) -> adw::ActionRow {
        let connection_text = match printer.connection_type {
            ConnectionType::Ipp => "IPP",
            ConnectionType::IppEverywhere => "IPP Everywhere",
            ConnectionType::Lpd => "LPD",
            ConnectionType::Socket => "Socket",
            ConnectionType::Usb => "USB",
            ConnectionType::Dnssd => "mDNS/DNS-SD",
            ConnectionType::Smb => "Windows (SMB)",
        };

        let subtitle = format!("{} - {}", printer.address, connection_text);

        let row = adw::ActionRow::builder()
            .title(&printer.name)
            .subtitle(&subtitle)
            .activatable(true)
            .build();

        // Icon based on manufacturer
        let icon_name = match printer.manufacturer.as_deref() {
            Some("HP") | Some("HP Inc.") => "printer-symbolic",
            Some("Canon") => "printer-symbolic",
            Some("Epson") => "printer-symbolic",
            Some("Brother") => "printer-symbolic",
            _ => "printer-symbolic",
        };
        row.add_prefix(&gtk4::Image::from_icon_name(icon_name));

        // Add button
        let add_btn = gtk4::Button::from_icon_name("list-add-symbolic");
        add_btn.add_css_class("flat");
        add_btn.set_valign(gtk4::Align::Center);
        add_btn.set_tooltip_text(Some("Adicionar esta impressora"));
        row.add_suffix(&add_btn);

        // Driverless badge if supported
        if matches!(printer.connection_type, ConnectionType::IppEverywhere | ConnectionType::Dnssd) {
            let badge = gtk4::Label::new(Some("Driverless"));
            badge.add_css_class("success");
            badge.add_css_class("caption");
            badge.set_valign(gtk4::Align::Center);
            badge.set_margin_end(8);
            row.add_suffix(&badge);
        }

        row
    }

    pub fn widget(&self) -> &gtk4::ScrolledWindow {
        &self.widget
    }
}
