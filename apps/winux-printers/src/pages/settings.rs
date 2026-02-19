//! Printer settings page

use gtk4::prelude::*;
use libadwaita as adw;
use libadwaita::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

use crate::cups::CupsManager;

/// Settings page for printer and CUPS configuration
pub struct SettingsPage {
    widget: gtk4::ScrolledWindow,
    #[allow(dead_code)]
    manager: Rc<RefCell<CupsManager>>,
}

impl SettingsPage {
    pub fn new(manager: Rc<RefCell<CupsManager>>) -> Self {
        let page = adw::PreferencesPage::new();
        page.set_title("Configuracoes");
        page.set_icon_name(Some("emblem-system-symbolic"));

        // Paper settings group
        let paper_group = adw::PreferencesGroup::builder()
            .title("Papel")
            .description("Configuracoes padrao de papel")
            .build();

        let paper_size_row = adw::ComboRow::builder()
            .title("Tamanho do Papel")
            .subtitle("Tamanho padrao para novos trabalhos")
            .build();
        let paper_sizes = gtk4::StringList::new(&[
            "A4 (210 x 297 mm)",
            "Letter (8.5 x 11 in)",
            "Legal (8.5 x 14 in)",
            "A3 (297 x 420 mm)",
            "A5 (148 x 210 mm)",
            "Oficio (215.9 x 340 mm)",
            "Envelope C5",
            "Envelope DL",
            "Personalizado...",
        ]);
        paper_size_row.set_model(Some(&paper_sizes));
        paper_group.add(&paper_size_row);

        let orientation_row = adw::ComboRow::builder()
            .title("Orientacao")
            .subtitle("Orientacao padrao do papel")
            .build();
        let orientations = gtk4::StringList::new(&["Retrato", "Paisagem", "Retrato Invertido", "Paisagem Invertida"]);
        orientation_row.set_model(Some(&orientations));
        paper_group.add(&orientation_row);

        let source_row = adw::ComboRow::builder()
            .title("Bandeja de Papel")
            .subtitle("Origem padrao do papel")
            .build();
        let sources = gtk4::StringList::new(&[
            "Automatica",
            "Bandeja 1 (Principal)",
            "Bandeja 2",
            "Alimentador Manual",
            "Envelope",
        ]);
        source_row.set_model(Some(&sources));
        paper_group.add(&source_row);

        page.add(&paper_group);

        // Quality settings group
        let quality_group = adw::PreferencesGroup::builder()
            .title("Qualidade de Impressao")
            .build();

        let quality_row = adw::ComboRow::builder()
            .title("Qualidade")
            .subtitle("Resolucao de impressao")
            .build();
        let qualities = gtk4::StringList::new(&[
            "Rascunho (Rapido)",
            "Normal (600 dpi)",
            "Alta (1200 dpi)",
            "Maxima (2400 dpi)",
        ]);
        quality_row.set_model(Some(&qualities));
        quality_row.set_selected(1);
        quality_group.add(&quality_row);

        let color_row = adw::ComboRow::builder()
            .title("Cor")
            .subtitle("Modo de cor padrao")
            .build();
        let colors = gtk4::StringList::new(&[
            "Automatico",
            "Colorido",
            "Escala de Cinza",
            "Preto e Branco",
        ]);
        color_row.set_model(Some(&colors));
        quality_group.add(&color_row);

        let grayscale_row = adw::SwitchRow::builder()
            .title("Economizar Tinta Colorida")
            .subtitle("Imprimir em escala de cinza quando possivel")
            .active(false)
            .build();
        quality_group.add(&grayscale_row);

        page.add(&quality_group);

        // Duplex settings group
        let duplex_group = adw::PreferencesGroup::builder()
            .title("Impressao Duplex")
            .description("Imprimir em ambos os lados do papel")
            .build();

        let duplex_row = adw::ComboRow::builder()
            .title("Duplex")
            .subtitle("Modo de impressao frente e verso")
            .build();
        let duplex_modes = gtk4::StringList::new(&[
            "Desligado (Apenas um lado)",
            "Borda Longa (Livro)",
            "Borda Curta (Bloco)",
        ]);
        duplex_row.set_model(Some(&duplex_modes));
        duplex_group.add(&duplex_row);

        let auto_duplex_row = adw::SwitchRow::builder()
            .title("Duplex Automatico por Padrao")
            .subtitle("Usar duplex sempre que a impressora suportar")
            .active(false)
            .build();
        duplex_group.add(&auto_duplex_row);

        page.add(&duplex_group);

        // Layout settings group
        let layout_group = adw::PreferencesGroup::builder()
            .title("Layout")
            .build();

        let pages_per_sheet_row = adw::ComboRow::builder()
            .title("Paginas por Folha")
            .subtitle("Numero de paginas em cada folha")
            .build();
        let pages_per_sheet = gtk4::StringList::new(&[
            "1 (Normal)",
            "2 (2-up)",
            "4 (4-up)",
            "6 (6-up)",
            "9 (9-up)",
            "16 (16-up)",
        ]);
        pages_per_sheet_row.set_model(Some(&pages_per_sheet));
        layout_group.add(&pages_per_sheet_row);

        let border_row = adw::SwitchRow::builder()
            .title("Bordas nas Paginas")
            .subtitle("Adicionar bordas ao redor de cada pagina")
            .active(false)
            .build();
        layout_group.add(&border_row);

        let collate_row = adw::SwitchRow::builder()
            .title("Agrupar")
            .subtitle("Agrupar paginas ao imprimir multiplas copias")
            .active(true)
            .build();
        layout_group.add(&collate_row);

        let reverse_row = adw::SwitchRow::builder()
            .title("Ordem Reversa")
            .subtitle("Imprimir da ultima pagina para a primeira")
            .active(false)
            .build();
        layout_group.add(&reverse_row);

        page.add(&layout_group);

        // CUPS settings group
        let cups_group = adw::PreferencesGroup::builder()
            .title("Servico CUPS")
            .description("Configuracoes do servidor de impressao")
            .build();

        let server_row = adw::EntryRow::builder()
            .title("Servidor CUPS")
            .text("localhost:631")
            .build();
        cups_group.add(&server_row);

        let status_row = adw::ActionRow::builder()
            .title("Status do CUPS")
            .subtitle("Servico em execucao")
            .build();
        status_row.add_prefix(&gtk4::Image::from_icon_name("emblem-ok-symbolic"));

        let status_icon = gtk4::Image::from_icon_name("emblem-ok-symbolic");
        status_icon.add_css_class("success");
        status_row.add_suffix(&status_icon);
        cups_group.add(&status_row);

        let web_row = adw::ActionRow::builder()
            .title("Interface Web do CUPS")
            .subtitle("Abrir gerenciamento avancado no navegador")
            .activatable(true)
            .build();
        web_row.add_prefix(&gtk4::Image::from_icon_name("applications-internet-symbolic"));
        web_row.add_suffix(&gtk4::Image::from_icon_name("go-next-symbolic"));
        cups_group.add(&web_row);

        let logs_row = adw::ActionRow::builder()
            .title("Ver Logs do CUPS")
            .subtitle("Diagnostico e solucao de problemas")
            .activatable(true)
            .build();
        logs_row.add_prefix(&gtk4::Image::from_icon_name("utilities-terminal-symbolic"));
        logs_row.add_suffix(&gtk4::Image::from_icon_name("go-next-symbolic"));
        cups_group.add(&logs_row);

        page.add(&cups_group);

        // Sharing settings group
        let sharing_group = adw::PreferencesGroup::builder()
            .title("Compartilhamento")
            .description("Compartilhar impressoras na rede")
            .build();

        let share_printers_row = adw::SwitchRow::builder()
            .title("Compartilhar Impressoras")
            .subtitle("Permitir que outros computadores imprimam nas suas impressoras")
            .active(false)
            .build();
        sharing_group.add(&share_printers_row);

        let publish_row = adw::SwitchRow::builder()
            .title("Publicar via Bonjour/Avahi")
            .subtitle("Anunciar impressoras na rede local")
            .active(true)
            .build();
        sharing_group.add(&publish_row);

        let auth_row = adw::SwitchRow::builder()
            .title("Exigir Autenticacao")
            .subtitle("Solicitar credenciais para impressao remota")
            .active(true)
            .build();
        sharing_group.add(&auth_row);

        page.add(&sharing_group);

        // Advanced settings group
        let advanced_group = adw::PreferencesGroup::builder()
            .title("Avancado")
            .build();

        let debug_row = adw::SwitchRow::builder()
            .title("Modo de Depuracao")
            .subtitle("Habilitar logs detalhados para diagnostico")
            .active(false)
            .build();
        advanced_group.add(&debug_row);

        let job_history_row = adw::SpinRow::builder()
            .title("Historico de Trabalhos")
            .subtitle("Dias para manter historico de impressao")
            .build();
        job_history_row.set_adjustment(&gtk4::Adjustment::new(7.0, 0.0, 365.0, 1.0, 7.0, 0.0));
        advanced_group.add(&job_history_row);

        let max_jobs_row = adw::SpinRow::builder()
            .title("Limite de Trabalhos")
            .subtitle("Numero maximo de trabalhos na fila")
            .build();
        max_jobs_row.set_adjustment(&gtk4::Adjustment::new(100.0, 10.0, 1000.0, 10.0, 100.0, 0.0));
        advanced_group.add(&max_jobs_row);

        let restart_row = adw::ActionRow::builder()
            .title("Reiniciar Servico CUPS")
            .subtitle("Reiniciar o servidor de impressao")
            .activatable(true)
            .build();
        restart_row.add_prefix(&gtk4::Image::from_icon_name("view-refresh-symbolic"));
        advanced_group.add(&restart_row);

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
