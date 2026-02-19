//! Advanced network settings page
//!
//! Features:
//! - DNS servers configuration
//! - Routing table management
//! - Firewall (UFW integration)
//! - Network diagnostics

use gtk4::prelude::*;
use gtk4::{Box, Button, Image, Label, Orientation, ScrolledWindow, TextView};
use libadwaita as adw;
use adw::prelude::*;
use adw::{ActionRow, ComboRow, EntryRow, ExpanderRow, PreferencesGroup, PreferencesPage, SwitchRow};

/// Advanced settings page
pub struct AdvancedPage {
    widget: ScrolledWindow,
}

impl AdvancedPage {
    pub fn new() -> Self {
        let page = PreferencesPage::new();
        page.set_title("Avancado");
        page.set_icon_name(Some("preferences-system-symbolic"));

        // DNS Configuration group
        let dns_group = PreferencesGroup::builder()
            .title("Servidores DNS")
            .description("Configure servidores de resolucao de nomes")
            .build();

        let dns_mode = ComboRow::builder()
            .title("Modo DNS")
            .subtitle("Como obter servidores DNS")
            .build();
        let dns_modes = gtk4::StringList::new(&[
            "Automatico (DHCP)",
            "Manual",
            "DNS over HTTPS (DoH)",
            "DNS over TLS (DoT)",
        ]);
        dns_mode.set_model(Some(&dns_modes));
        dns_group.add(&dns_mode);

        // DNS servers
        let dns_presets = ComboRow::builder()
            .title("Preset")
            .subtitle("Servidores DNS pre-configurados")
            .build();
        let presets = gtk4::StringList::new(&[
            "Personalizado",
            "Google (8.8.8.8)",
            "Cloudflare (1.1.1.1)",
            "Quad9 (9.9.9.9)",
            "OpenDNS (208.67.222.222)",
        ]);
        dns_presets.set_model(Some(&presets));
        dns_group.add(&dns_presets);

        let dns1_entry = EntryRow::builder()
            .title("DNS Primario")
            .text("8.8.8.8")
            .build();
        dns_group.add(&dns1_entry);

        let dns2_entry = EntryRow::builder()
            .title("DNS Secundario")
            .text("8.8.4.4")
            .build();
        dns_group.add(&dns2_entry);

        let dns3_entry = EntryRow::builder()
            .title("DNS Terciario")
            .build();
        dns_group.add(&dns3_entry);

        // DNS over HTTPS
        let doh_expander = ExpanderRow::builder()
            .title("DNS over HTTPS")
            .subtitle("Criptografar consultas DNS")
            .build();

        let doh_switch = SwitchRow::builder()
            .title("Habilitar DoH")
            .active(false)
            .build();
        doh_expander.add_row(&doh_switch);

        let doh_provider = ComboRow::builder()
            .title("Provedor DoH")
            .build();
        let doh_providers = gtk4::StringList::new(&[
            "Cloudflare",
            "Google",
            "Quad9",
            "Personalizado",
        ]);
        doh_provider.set_model(Some(&doh_providers));
        doh_expander.add_row(&doh_provider);

        let doh_url = EntryRow::builder()
            .title("URL DoH")
            .text("https://cloudflare-dns.com/dns-query")
            .build();
        doh_expander.add_row(&doh_url);

        dns_group.add(&doh_expander);

        // Local DNS cache
        let dns_cache = SwitchRow::builder()
            .title("Cache DNS Local")
            .subtitle("Usar systemd-resolved para cache")
            .active(true)
            .build();
        dns_group.add(&dns_cache);

        page.add(&dns_group);

        // Routing group
        let routing_group = PreferencesGroup::builder()
            .title("Roteamento")
            .description("Tabela de rotas de rede")
            .build();

        // Current routes
        let routes = [
            ("default", "192.168.1.1", "enp3s0", "100"),
            ("192.168.1.0/24", "0.0.0.0", "enp3s0", "100"),
            ("172.17.0.0/16", "0.0.0.0", "docker0", "0"),
            ("10.0.0.0/24", "10.0.0.1", "wg0", "50"),
        ];

        for (dest, gateway, iface, metric) in routes {
            let route_row = ActionRow::builder()
                .title(dest)
                .subtitle(&format!("via {} dev {} metric {}", gateway, iface, metric))
                .build();

            route_row.add_prefix(&Image::from_icon_name("network-server-symbolic"));

            let delete_btn = Button::from_icon_name("list-remove-symbolic");
            delete_btn.add_css_class("flat");
            delete_btn.set_valign(gtk4::Align::Center);
            delete_btn.set_tooltip_text(Some("Remover rota"));
            route_row.add_suffix(&delete_btn);

            routing_group.add(&route_row);
        }

        // Add route expander
        let add_route = ExpanderRow::builder()
            .title("Adicionar Rota")
            .subtitle("Criar nova rota estatica")
            .build();

        let route_dest = EntryRow::builder()
            .title("Destino")
            .text("10.10.0.0/24")
            .build();
        add_route.add_row(&route_dest);

        let route_gateway = EntryRow::builder()
            .title("Gateway")
            .text("192.168.1.1")
            .build();
        add_route.add_row(&route_gateway);

        let route_iface = ComboRow::builder()
            .title("Interface")
            .build();
        let ifaces = gtk4::StringList::new(&["enp3s0", "wlan0", "docker0", "wg0"]);
        route_iface.set_model(Some(&ifaces));
        add_route.add_row(&route_iface);

        let route_metric = EntryRow::builder()
            .title("Metrica")
            .text("100")
            .build();
        add_route.add_row(&route_metric);

        let add_route_row = ActionRow::builder().build();
        let add_route_btn = Button::with_label("Adicionar Rota");
        add_route_btn.add_css_class("suggested-action");
        add_route_btn.set_halign(gtk4::Align::Center);
        add_route_btn.set_margin_top(8);
        add_route_btn.set_margin_bottom(8);
        add_route_row.set_child(Some(&add_route_btn));
        add_route.add_row(&add_route_row);

        routing_group.add(&add_route);

        page.add(&routing_group);

        // Firewall group (UFW)
        let firewall_group = PreferencesGroup::builder()
            .title("Firewall")
            .description("Gerenciar regras de firewall (UFW)")
            .build();

        let firewall_switch = SwitchRow::builder()
            .title("Firewall Ativo")
            .subtitle("Proteger contra acessos nao autorizados")
            .active(true)
            .build();

        firewall_switch.connect_active_notify(|switch| {
            if switch.is_active() {
                tracing::info!("Enabling firewall...");
                // Would run: ufw enable
            } else {
                tracing::info!("Disabling firewall...");
                // Would run: ufw disable
            }
        });

        firewall_group.add(&firewall_switch);

        let default_policy = ComboRow::builder()
            .title("Politica Padrao (Entrada)")
            .subtitle("Acao para conexoes de entrada")
            .build();
        let policies = gtk4::StringList::new(&["Negar (deny)", "Permitir (allow)", "Rejeitar (reject)"]);
        default_policy.set_model(Some(&policies));
        firewall_group.add(&default_policy);

        // Firewall rules
        let rules_expander = ExpanderRow::builder()
            .title("Regras do Firewall")
            .subtitle("Gerenciar portas e servicos")
            .build();

        let rules = [
            ("SSH (22/tcp)", "Permitir", true),
            ("HTTP (80/tcp)", "Permitir", true),
            ("HTTPS (443/tcp)", "Permitir", true),
            ("Samba (445/tcp)", "Negar", false),
        ];

        for (port, action, enabled) in rules {
            let rule_row = ActionRow::builder()
                .title(port)
                .subtitle(action)
                .build();

            let rule_switch = gtk4::Switch::new();
            rule_switch.set_active(enabled);
            rule_switch.set_valign(gtk4::Align::Center);
            rule_row.add_suffix(&rule_switch);

            rules_expander.add_row(&rule_row);
        }

        // Add rule
        let add_rule_row = ActionRow::builder()
            .title("Adicionar Regra")
            .activatable(true)
            .build();
        add_rule_row.add_prefix(&Image::from_icon_name("list-add-symbolic"));
        add_rule_row.add_suffix(&Image::from_icon_name("go-next-symbolic"));
        rules_expander.add_row(&add_rule_row);

        firewall_group.add(&rules_expander);

        page.add(&firewall_group);

        // Network diagnostics group
        let diag_group = PreferencesGroup::builder()
            .title("Diagnosticos")
            .description("Ferramentas de diagnostico de rede")
            .build();

        // Ping test
        let ping_row = ActionRow::builder()
            .title("Teste de Ping")
            .subtitle("Verificar conectividade com um host")
            .activatable(true)
            .build();
        ping_row.add_prefix(&Image::from_icon_name("network-transmit-receive-symbolic"));
        ping_row.add_suffix(&Image::from_icon_name("go-next-symbolic"));
        ping_row.connect_activated(|_| {
            tracing::info!("Opening ping dialog...");
        });
        diag_group.add(&ping_row);

        // Traceroute
        let trace_row = ActionRow::builder()
            .title("Traceroute")
            .subtitle("Rastrear rota ate um host")
            .activatable(true)
            .build();
        trace_row.add_prefix(&Image::from_icon_name("mark-location-symbolic"));
        trace_row.add_suffix(&Image::from_icon_name("go-next-symbolic"));
        diag_group.add(&trace_row);

        // DNS lookup
        let lookup_row = ActionRow::builder()
            .title("Consulta DNS")
            .subtitle("Resolver nome de dominio")
            .activatable(true)
            .build();
        lookup_row.add_prefix(&Image::from_icon_name("system-search-symbolic"));
        lookup_row.add_suffix(&Image::from_icon_name("go-next-symbolic"));
        diag_group.add(&lookup_row);

        // Network speed test
        let speed_row = ActionRow::builder()
            .title("Teste de Velocidade")
            .subtitle("Medir velocidade da conexao")
            .activatable(true)
            .build();
        speed_row.add_prefix(&Image::from_icon_name("speedometer-symbolic"));
        speed_row.add_suffix(&Image::from_icon_name("go-next-symbolic"));
        diag_group.add(&speed_row);

        page.add(&diag_group);

        // Network interfaces info
        let interfaces_group = PreferencesGroup::builder()
            .title("Interfaces de Rede")
            .description("Informacoes detalhadas das interfaces")
            .build();

        let interfaces = [
            ("enp3s0", "Ethernet", "1000 Mbps", "Conectado"),
            ("wlan0", "Wi-Fi", "866 Mbps", "Conectado"),
            ("lo", "Loopback", "N/A", "Ativo"),
            ("docker0", "Bridge", "N/A", "Ativo"),
            ("wg0", "WireGuard", "N/A", "Inativo"),
        ];

        for (name, if_type, speed, status) in interfaces {
            let if_expander = ExpanderRow::builder()
                .title(name)
                .subtitle(&format!("{} - {}", if_type, status))
                .build();

            let icon = match if_type {
                "Ethernet" => "network-wired-symbolic",
                "Wi-Fi" => "network-wireless-symbolic",
                "WireGuard" => "network-vpn-symbolic",
                _ => "network-server-symbolic",
            };
            if_expander.add_prefix(&Image::from_icon_name(icon));

            let type_row = ActionRow::builder()
                .title("Tipo")
                .subtitle(if_type)
                .build();
            if_expander.add_row(&type_row);

            let speed_row = ActionRow::builder()
                .title("Velocidade")
                .subtitle(speed)
                .build();
            if_expander.add_row(&speed_row);

            let mtu_row = ActionRow::builder()
                .title("MTU")
                .subtitle("1500")
                .build();
            if_expander.add_row(&mtu_row);

            interfaces_group.add(&if_expander);
        }

        page.add(&interfaces_group);

        // Hostname group
        let hostname_group = PreferencesGroup::builder()
            .title("Identificacao do Sistema")
            .build();

        let hostname_entry = EntryRow::builder()
            .title("Nome do Host")
            .text("winux-desktop")
            .build();
        hostname_group.add(&hostname_entry);

        let apply_hostname_row = ActionRow::builder().build();
        let apply_hostname_btn = Button::with_label("Alterar Hostname");
        apply_hostname_btn.set_halign(gtk4::Align::Center);
        apply_hostname_btn.set_margin_top(8);
        apply_hostname_btn.set_margin_bottom(8);
        apply_hostname_btn.connect_clicked(|_| {
            tracing::info!("Changing hostname...");
        });
        apply_hostname_row.set_child(Some(&apply_hostname_btn));
        hostname_group.add(&apply_hostname_row);

        page.add(&hostname_group);

        let scrolled = ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .child(&page)
            .build();

        Self { widget: scrolled }
    }

    pub fn widget(&self) -> &ScrolledWindow {
        &self.widget
    }
}

impl Default for AdvancedPage {
    fn default() -> Self {
        Self::new()
    }
}
