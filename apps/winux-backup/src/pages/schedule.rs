//! Schedule page - Backup scheduling and retention

use gtk4::prelude::*;
use gtk4::{Box, Button, CheckButton, Label, Orientation};
use libadwaita as adw;
use adw::prelude::*;
use adw::{
    ActionRow, ComboRow, ExpanderRow, PreferencesGroup, PreferencesPage,
    SpinRow, SwitchRow,
};

/// Schedule page
pub struct SchedulePage {
    widget: gtk4::ScrolledWindow,
}

impl SchedulePage {
    pub fn new() -> Self {
        let page = PreferencesPage::new();

        // Scheduled Backups Group
        let scheduled_group = PreferencesGroup::builder()
            .title("Backups Agendados")
            .description("Gerencie suas tarefas de backup automatico")
            .build();

        // System backup schedule
        let system_schedule = ExpanderRow::builder()
            .title("Sistema Completo")
            .subtitle("Semanal - Domingos as 03:00")
            .show_enable_switch(true)
            .enable_expansion(true)
            .build();
        system_schedule.add_prefix(&gtk4::Image::from_icon_name("computer-symbolic"));

        let sys_freq = ComboRow::builder()
            .title("Frequencia")
            .build();
        let frequencies = gtk4::StringList::new(&[
            "Diario",
            "Semanal",
            "Quinzenal",
            "Mensal",
        ]);
        sys_freq.set_model(Some(&frequencies));
        sys_freq.set_selected(1);
        system_schedule.add_row(&sys_freq);

        let sys_day = ComboRow::builder()
            .title("Dia da Semana")
            .build();
        let days = gtk4::StringList::new(&[
            "Segunda",
            "Terca",
            "Quarta",
            "Quinta",
            "Sexta",
            "Sabado",
            "Domingo",
        ]);
        sys_day.set_model(Some(&days));
        sys_day.set_selected(6);
        system_schedule.add_row(&sys_day);

        let sys_time = ComboRow::builder()
            .title("Horario")
            .build();
        let times = gtk4::StringList::new(&[
            "00:00", "01:00", "02:00", "03:00", "04:00", "05:00",
            "06:00", "12:00", "18:00", "22:00", "23:00",
        ]);
        sys_time.set_model(Some(&times));
        sys_time.set_selected(3);
        system_schedule.add_row(&sys_time);

        scheduled_group.add(&system_schedule);

        // Home backup schedule
        let home_schedule = ExpanderRow::builder()
            .title("Home Folder")
            .subtitle("Diario - 03:00")
            .show_enable_switch(true)
            .enable_expansion(true)
            .build();
        home_schedule.add_prefix(&gtk4::Image::from_icon_name("user-home-symbolic"));

        let home_freq = ComboRow::builder()
            .title("Frequencia")
            .build();
        home_freq.set_model(Some(&frequencies));
        home_freq.set_selected(0);
        home_schedule.add_row(&home_freq);

        let home_time = ComboRow::builder()
            .title("Horario")
            .build();
        home_time.set_model(Some(&times));
        home_time.set_selected(3);
        home_schedule.add_row(&home_time);

        scheduled_group.add(&home_schedule);

        // Config backup schedule
        let config_schedule = ExpanderRow::builder()
            .title("Configuracoes de Apps")
            .subtitle("Diario - 10:00")
            .show_enable_switch(true)
            .enable_expansion(true)
            .build();
        config_schedule.add_prefix(&gtk4::Image::from_icon_name("applications-system-symbolic"));

        let config_freq = ComboRow::builder()
            .title("Frequencia")
            .build();
        config_freq.set_model(Some(&frequencies));
        config_freq.set_selected(0);
        config_schedule.add_row(&config_freq);

        let config_time = ComboRow::builder()
            .title("Horario")
            .build();
        config_time.set_model(Some(&times));
        config_time.set_selected(5);
        config_schedule.add_row(&config_time);

        scheduled_group.add(&config_schedule);

        // Add new schedule
        let add_schedule = ActionRow::builder()
            .title("Adicionar Agendamento...")
            .activatable(true)
            .build();
        add_schedule.add_prefix(&gtk4::Image::from_icon_name("list-add-symbolic"));
        add_schedule.add_suffix(&gtk4::Image::from_icon_name("go-next-symbolic"));
        scheduled_group.add(&add_schedule);

        page.add(&scheduled_group);

        // Retention Policy Group
        let retention_group = PreferencesGroup::builder()
            .title("Politica de Retencao")
            .description("Quantos backups manter de cada tipo")
            .build();

        let keep_daily = SpinRow::builder()
            .title("Backups Diarios")
            .subtitle("Manter ultimos N backups diarios")
            .adjustment(&gtk4::Adjustment::new(7.0, 1.0, 30.0, 1.0, 5.0, 0.0))
            .build();
        retention_group.add(&keep_daily);

        let keep_weekly = SpinRow::builder()
            .title("Backups Semanais")
            .subtitle("Manter ultimos N backups semanais")
            .adjustment(&gtk4::Adjustment::new(4.0, 1.0, 12.0, 1.0, 2.0, 0.0))
            .build();
        retention_group.add(&keep_weekly);

        let keep_monthly = SpinRow::builder()
            .title("Backups Mensais")
            .subtitle("Manter ultimos N backups mensais")
            .adjustment(&gtk4::Adjustment::new(6.0, 1.0, 24.0, 1.0, 3.0, 0.0))
            .build();
        retention_group.add(&keep_monthly);

        let keep_yearly = SpinRow::builder()
            .title("Backups Anuais")
            .subtitle("Manter ultimos N backups anuais")
            .adjustment(&gtk4::Adjustment::new(2.0, 0.0, 10.0, 1.0, 1.0, 0.0))
            .build();
        retention_group.add(&keep_yearly);

        let auto_prune = SwitchRow::builder()
            .title("Remover Automaticamente")
            .subtitle("Excluir backups antigos automaticamente")
            .active(true)
            .build();
        retention_group.add(&auto_prune);

        page.add(&retention_group);

        // Scheduling Options Group
        let options_group = PreferencesGroup::builder()
            .title("Opcoes de Agendamento")
            .build();

        let wake_for_backup = SwitchRow::builder()
            .title("Acordar para Backup")
            .subtitle("Despertar o computador para executar backup agendado")
            .active(false)
            .build();
        options_group.add(&wake_for_backup);

        let run_on_ac = SwitchRow::builder()
            .title("Apenas na Energia")
            .subtitle("Executar backups apenas quando conectado a energia")
            .active(true)
            .build();
        options_group.add(&run_on_ac);

        let skip_metered = SwitchRow::builder()
            .title("Pular em Rede Limitada")
            .subtitle("Nao executar backups remotos em conexoes limitadas")
            .active(true)
            .build();
        options_group.add(&skip_metered);

        let retry_failed = SwitchRow::builder()
            .title("Tentar Novamente")
            .subtitle("Reagendar backups que falharam")
            .active(true)
            .build();
        options_group.add(&retry_failed);

        let retry_delay = SpinRow::builder()
            .title("Atraso para Retentativa")
            .subtitle("Minutos ate tentar novamente")
            .adjustment(&gtk4::Adjustment::new(30.0, 5.0, 120.0, 5.0, 15.0, 0.0))
            .build();
        options_group.add(&retry_delay);

        page.add(&options_group);

        // Notifications Group
        let notify_group = PreferencesGroup::builder()
            .title("Notificacoes")
            .build();

        let notify_start = SwitchRow::builder()
            .title("Notificar Inicio")
            .subtitle("Notificar quando um backup iniciar")
            .active(false)
            .build();
        notify_group.add(&notify_start);

        let notify_complete = SwitchRow::builder()
            .title("Notificar Conclusao")
            .subtitle("Notificar quando um backup terminar")
            .active(true)
            .build();
        notify_group.add(&notify_complete);

        let notify_error = SwitchRow::builder()
            .title("Notificar Erros")
            .subtitle("Notificar quando um backup falhar")
            .active(true)
            .build();
        notify_group.add(&notify_error);

        let email_report = SwitchRow::builder()
            .title("Relatorio por Email")
            .subtitle("Enviar resumo semanal por email")
            .active(false)
            .build();
        notify_group.add(&email_report);

        page.add(&notify_group);

        // Upcoming Backups Group
        let upcoming_group = PreferencesGroup::builder()
            .title("Proximos Backups")
            .build();

        let upcoming = [
            ("Sistema Completo", "Dom, 23/02/2026 03:00"),
            ("Home Folder", "Qui, 20/02/2026 03:00"),
            ("Configuracoes de Apps", "Qui, 20/02/2026 10:00"),
        ];

        for (name, datetime) in upcoming {
            let row = ActionRow::builder()
                .title(name)
                .subtitle(datetime)
                .build();
            row.add_prefix(&gtk4::Image::from_icon_name("alarm-symbolic"));
            upcoming_group.add(&row);
        }

        page.add(&upcoming_group);

        let scrolled = gtk4::ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .child(&page)
            .build();

        Self { widget: scrolled }
    }

    pub fn widget(&self) -> &gtk4::ScrolledWindow {
        &self.widget
    }
}

impl Default for SchedulePage {
    fn default() -> Self {
        Self::new()
    }
}
