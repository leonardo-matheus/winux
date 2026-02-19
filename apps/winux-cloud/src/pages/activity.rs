//! Activity page - Sync activity log

use gtk4::prelude::*;
use gtk4::{Box, Button, Orientation, SearchEntry, DropDown, StringList};
use libadwaita as adw;
use adw::prelude::*;
use adw::{ActionRow, PreferencesGroup, PreferencesPage, StatusPage};

/// Activity log page
pub struct ActivityPage {
    widget: gtk4::ScrolledWindow,
}

impl ActivityPage {
    pub fn new() -> Self {
        let content = Box::new(Orientation::Vertical, 0);

        // Status header
        let status = StatusPage::builder()
            .icon_name("view-list-symbolic")
            .title("Atividade")
            .description("Historico de sincronizacao")
            .build();
        content.append(&status);

        // Filters toolbar
        let toolbar = Box::new(Orientation::Horizontal, 8);
        toolbar.set_margin_start(16);
        toolbar.set_margin_end(16);
        toolbar.set_margin_top(8);
        toolbar.set_margin_bottom(8);

        let search = SearchEntry::builder()
            .placeholder_text("Buscar atividade...")
            .hexpand(true)
            .build();
        toolbar.append(&search);

        // Activity type filter
        let types = StringList::new(&["Todas", "Uploads", "Downloads", "Deletados", "Conflitos", "Erros"]);
        let type_dropdown = DropDown::builder()
            .model(&types)
            .build();
        toolbar.append(&type_dropdown);

        // Time period filter
        let periods = StringList::new(&["Hoje", "Esta Semana", "Este Mes", "Tudo"]);
        let period_dropdown = DropDown::builder()
            .model(&periods)
            .build();
        toolbar.append(&period_dropdown);

        content.append(&toolbar);

        let page = PreferencesPage::new();

        // Today's Activity Group
        let today_group = PreferencesGroup::builder()
            .title("Hoje")
            .description("19 de Fevereiro de 2026")
            .build();

        let today_activities = [
            ("Relatorio_Final.pdf", "Upload concluido", "14:32", "go-up-symbolic", "success"),
            ("Planilha_Orcamento.xlsx", "Download concluido", "14:28", "go-down-symbolic", "success"),
            ("foto_equipe.jpg", "Sincronizado", "13:45", "emblem-synchronizing-symbolic", "success"),
            ("documento_antigo.docx", "Movido para lixeira", "12:30", "user-trash-symbolic", "warning"),
            ("notas_reuniao.txt", "Conflito resolvido (manter ambos)", "11:15", "dialog-warning-symbolic", "warning"),
            ("backup_dados.zip", "Upload falhou - Sem espaco", "10:00", "dialog-error-symbolic", "error"),
        ];

        for (file, action, time, icon, status_type) in today_activities {
            let row = ActionRow::builder()
                .title(file)
                .subtitle(&format!("{} - {}", action, time))
                .activatable(true)
                .build();

            let icon_widget = gtk4::Image::from_icon_name(icon);
            if status_type == "error" {
                icon_widget.add_css_class("error");
            } else if status_type == "warning" {
                icon_widget.add_css_class("warning");
            }
            row.add_prefix(&icon_widget);

            if status_type == "error" {
                let retry_btn = Button::with_label("Tentar Novamente");
                retry_btn.add_css_class("flat");
                retry_btn.set_valign(gtk4::Align::Center);
                row.add_suffix(&retry_btn);
            }

            row.add_suffix(&gtk4::Image::from_icon_name("go-next-symbolic"));
            today_group.add(&row);
        }

        page.add(&today_group);

        // Yesterday's Activity Group
        let yesterday_group = PreferencesGroup::builder()
            .title("Ontem")
            .description("18 de Fevereiro de 2026")
            .build();

        let yesterday_activities = [
            ("Apresentacao.pptx", "Upload concluido", "18:45", "go-up-symbolic", "success"),
            ("Projeto_Final/", "234 arquivos sincronizados", "16:30", "folder-symbolic", "success"),
            ("video_reuniao.mp4", "Download concluido", "15:20", "go-down-symbolic", "success"),
            ("Compartilhado: Proposta.pdf", "Link criado", "14:00", "emblem-shared-symbolic", "success"),
        ];

        for (file, action, time, icon, _status_type) in yesterday_activities {
            let row = ActionRow::builder()
                .title(file)
                .subtitle(&format!("{} - {}", action, time))
                .activatable(true)
                .build();
            row.add_prefix(&gtk4::Image::from_icon_name(icon));
            row.add_suffix(&gtk4::Image::from_icon_name("go-next-symbolic"));
            yesterday_group.add(&row);
        }

        page.add(&yesterday_group);

        // This Week Group
        let week_group = PreferencesGroup::builder()
            .title("Esta Semana")
            .build();

        let week_summary = ActionRow::builder()
            .title("Resumo da Semana")
            .subtitle("1,234 arquivos sincronizados, 4.5 GB transferidos")
            .build();
        week_summary.add_prefix(&gtk4::Image::from_icon_name("view-list-bullet-symbolic"));
        week_group.add(&week_summary);

        let uploads_row = ActionRow::builder()
            .title("Uploads")
            .subtitle("567 arquivos, 2.1 GB")
            .build();
        uploads_row.add_prefix(&gtk4::Image::from_icon_name("go-up-symbolic"));
        week_group.add(&uploads_row);

        let downloads_row = ActionRow::builder()
            .title("Downloads")
            .subtitle("667 arquivos, 2.4 GB")
            .build();
        downloads_row.add_prefix(&gtk4::Image::from_icon_name("go-down-symbolic"));
        week_group.add(&downloads_row);

        let conflicts_row = ActionRow::builder()
            .title("Conflitos Resolvidos")
            .subtitle("12 conflitos")
            .build();
        conflicts_row.add_prefix(&gtk4::Image::from_icon_name("dialog-warning-symbolic"));
        week_group.add(&conflicts_row);

        let errors_row = ActionRow::builder()
            .title("Erros")
            .subtitle("3 erros (2 resolvidos)")
            .activatable(true)
            .build();
        errors_row.add_prefix(&gtk4::Image::from_icon_name("dialog-error-symbolic"));
        errors_row.add_suffix(&gtk4::Image::from_icon_name("go-next-symbolic"));
        week_group.add(&errors_row);

        page.add(&week_group);

        // Version History Group
        let versions_group = PreferencesGroup::builder()
            .title("Historico de Versoes")
            .description("Versoes anteriores de arquivos modificados")
            .build();

        let version_files = [
            ("Relatorio_Final.pdf", "5 versoes", "Ultima: Hoje 14:32"),
            ("Planilha_Orcamento.xlsx", "12 versoes", "Ultima: Hoje 10:15"),
            ("Contrato_2026.docx", "3 versoes", "Ultima: 15/02/2026"),
        ];

        for (file, versions, last_mod) in version_files {
            let row = ActionRow::builder()
                .title(file)
                .subtitle(&format!("{} - {}", versions, last_mod))
                .activatable(true)
                .build();
            row.add_prefix(&gtk4::Image::from_icon_name("document-open-recent-symbolic"));

            let restore_btn = Button::with_label("Restaurar");
            restore_btn.add_css_class("flat");
            restore_btn.set_valign(gtk4::Align::Center);
            row.add_suffix(&restore_btn);

            row.add_suffix(&gtk4::Image::from_icon_name("go-next-symbolic"));
            versions_group.add(&row);
        }

        page.add(&versions_group);

        // Actions Group
        let actions_group = PreferencesGroup::builder()
            .title("Acoes")
            .build();

        let export_row = ActionRow::builder()
            .title("Exportar Historico")
            .subtitle("Exportar log de atividade para arquivo")
            .activatable(true)
            .build();
        export_row.add_prefix(&gtk4::Image::from_icon_name("document-save-symbolic"));

        let export_btn = Button::with_label("Exportar");
        export_btn.add_css_class("flat");
        export_btn.set_valign(gtk4::Align::Center);
        export_row.add_suffix(&export_btn);
        actions_group.add(&export_row);

        let clear_row = ActionRow::builder()
            .title("Limpar Historico")
            .subtitle("Remover historico de atividade antigo")
            .activatable(true)
            .build();
        clear_row.add_prefix(&gtk4::Image::from_icon_name("edit-clear-symbolic"));

        let clear_btn = Button::with_label("Limpar");
        clear_btn.add_css_class("flat");
        clear_btn.set_valign(gtk4::Align::Center);
        clear_row.add_suffix(&clear_btn);
        actions_group.add(&clear_row);

        page.add(&actions_group);

        content.append(&page);

        let scrolled = gtk4::ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .child(&content)
            .build();

        Self { widget: scrolled }
    }

    pub fn widget(&self) -> &gtk4::ScrolledWindow {
        &self.widget
    }
}

impl Default for ActivityPage {
    fn default() -> Self {
        Self::new()
    }
}
