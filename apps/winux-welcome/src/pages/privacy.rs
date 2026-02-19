// Winux Welcome - Privacy Settings
// Configure telemetry and crash reports

use gtk4::prelude::*;
use gtk4::{Box, Label, Orientation};
use libadwaita as adw;
use adw::prelude::*;
use adw::{PreferencesGroup, PreferencesPage, ActionRow, SwitchRow};
use std::cell::RefCell;
use std::rc::Rc;

use crate::window::WelcomeState;

pub fn create_page(state: Rc<RefCell<WelcomeState>>) -> gtk4::ScrolledWindow {
    let page = PreferencesPage::new();

    // Header
    let header_box = Box::new(Orientation::Vertical, 8);
    header_box.set_margin_top(24);
    header_box.set_margin_bottom(12);
    header_box.set_halign(gtk4::Align::Center);

    let icon = gtk4::Image::from_icon_name("security-high-symbolic");
    icon.set_pixel_size(64);
    icon.add_css_class("accent");
    header_box.append(&icon);

    let title = Label::new(Some("Privacidade"));
    title.add_css_class("title-1");
    header_box.append(&title);

    let subtitle = Label::new(Some("Voce tem controle total sobre seus dados"));
    subtitle.add_css_class("dim-label");
    header_box.append(&subtitle);

    let header_group = PreferencesGroup::new();
    header_group.add(&header_box);
    page.add(&header_group);

    // Privacy philosophy
    let philosophy_group = PreferencesGroup::builder()
        .title("Nossa Filosofia")
        .build();

    let philosophy = ActionRow::builder()
        .title("Privacidade em Primeiro Lugar")
        .subtitle("O Winux foi projetado com privacidade como prioridade. \
                   Nenhum dado e coletado sem seu consentimento explicito.")
        .build();
    philosophy.add_prefix(&gtk4::Image::from_icon_name("emblem-ok-symbolic"));
    philosophy_group.add(&philosophy);

    page.add(&philosophy_group);

    // Telemetry section
    let telemetry_group = PreferencesGroup::builder()
        .title("Telemetria")
        .description("Ajude a melhorar o Winux enviando dados anonimos de uso")
        .build();

    let telemetry_switch = SwitchRow::builder()
        .title("Estatisticas de uso")
        .subtitle("Envia dados anonimos sobre quais funcionalidades voce usa mais. \
                   Isso nos ajuda a priorizar melhorias.")
        .active(false)
        .build();

    // Connect to state
    let state_clone = state.clone();
    telemetry_switch.connect_active_notify(move |switch| {
        state_clone.borrow_mut().telemetry_enabled = switch.is_active();
    });

    telemetry_group.add(&telemetry_switch);

    // Data collected info
    let data_collected = ActionRow::builder()
        .title("O que e coletado?")
        .subtitle("- Versao do sistema\n\
                   - Hardware basico (CPU, RAM, GPU)\n\
                   - Apps mais usados (sem identificacao)\n\
                   - Erros de sistema anonimizados")
        .build();
    data_collected.add_prefix(&gtk4::Image::from_icon_name("dialog-information-symbolic"));
    telemetry_group.add(&data_collected);

    page.add(&telemetry_group);

    // Crash reports section
    let crash_group = PreferencesGroup::builder()
        .title("Relatorios de Erro")
        .description("Envie relatorios quando algo der errado")
        .build();

    let crash_switch = SwitchRow::builder()
        .title("Relatorios de crash automaticos")
        .subtitle("Quando um aplicativo trava, envia um relatorio anonimo para ajudar \
                   os desenvolvedores a corrigir o problema.")
        .active(true)
        .build();

    // Connect to state
    let state_clone = state.clone();
    crash_switch.connect_active_notify(move |switch| {
        state_clone.borrow_mut().crash_reports_enabled = switch.is_active();
    });
    state.borrow_mut().crash_reports_enabled = true;

    crash_group.add(&crash_switch);

    let crash_info = ActionRow::builder()
        .title("O que e incluido no relatorio?")
        .subtitle("- Stack trace do erro\n\
                   - Versao do aplicativo\n\
                   - Configuracao basica do sistema\n\
                   - NUNCA inclui dados pessoais ou arquivos")
        .build();
    crash_info.add_prefix(&gtk4::Image::from_icon_name("dialog-information-symbolic"));
    crash_group.add(&crash_info);

    page.add(&crash_group);

    // Location section
    let location_group = PreferencesGroup::builder()
        .title("Localizacao")
        .description("Servicos que usam sua localizacao")
        .build();

    let location_switch = SwitchRow::builder()
        .title("Servicos de localizacao")
        .subtitle("Permite que aplicativos solicitem sua localizacao. \
                   Voce sera perguntado cada vez que um app quiser acessar.")
        .active(true)
        .build();
    location_group.add(&location_switch);

    let weather_row = ActionRow::builder()
        .title("Clima automatico")
        .subtitle("Mostra previsao do tempo baseada em sua localizacao")
        .build();
    let weather_switch = gtk4::Switch::new();
    weather_switch.set_active(true);
    weather_switch.set_valign(gtk4::Align::Center);
    weather_row.add_suffix(&weather_switch);
    location_group.add(&weather_row);

    page.add(&location_group);

    // Online accounts section
    let accounts_group = PreferencesGroup::builder()
        .title("Contas Online")
        .description("Integracao com servicos externos")
        .build();

    let online_search = SwitchRow::builder()
        .title("Busca online")
        .subtitle("Inclui resultados da web nas buscas do sistema")
        .active(false)
        .build();
    accounts_group.add(&online_search);

    let cloud_sync = SwitchRow::builder()
        .title("Sincronizacao de configuracoes")
        .subtitle("Sincroniza suas configuracoes entre dispositivos Winux")
        .active(false)
        .build();
    accounts_group.add(&cloud_sync);

    page.add(&accounts_group);

    // Transparency note
    let transparency_group = PreferencesGroup::new();
    let transparency_note = Label::new(Some(
        "Todas as configuracoes de privacidade podem ser alteradas a qualquer momento \
         em Configuracoes > Privacidade. O Winux e 100% codigo aberto - voce pode \
         verificar exatamente o que cada funcionalidade faz."
    ));
    transparency_note.add_css_class("dim-label");
    transparency_note.add_css_class("caption");
    transparency_note.set_wrap(true);
    transparency_note.set_margin_top(12);
    transparency_note.set_margin_bottom(12);
    transparency_group.add(&transparency_note);
    page.add(&transparency_group);

    let scrolled = gtk4::ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .child(&page)
        .build();

    scrolled
}
