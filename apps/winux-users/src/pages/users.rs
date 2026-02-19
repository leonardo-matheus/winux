//! Users listing page

use gtk4::prelude::*;
use gtk4::{Box, Button, Orientation, ScrolledWindow};
use libadwaita as adw;
use adw::prelude::*;
use adw::{ActionRow, PreferencesGroup, PreferencesPage, StatusPage};

use crate::backend::AccountsService;
use crate::ui::UserRow;

use std::cell::RefCell;
use std::rc::Rc;

/// Users listing page
pub struct UsersPage {
    widget: ScrolledWindow,
    accounts_service: Rc<RefCell<AccountsService>>,
}

impl UsersPage {
    pub fn new(accounts_service: Rc<RefCell<AccountsService>>) -> Self {
        let page = PreferencesPage::new();
        page.set_title("Usuarios");
        page.set_icon_name(Some("system-users-symbolic"));

        // Header section
        let header_group = PreferencesGroup::new();

        let status = StatusPage::builder()
            .icon_name("system-users-symbolic")
            .title("Gerenciamento de Usuarios")
            .description("Gerencie usuarios, permissoes e configuracoes de conta")
            .build();

        let header_box = Box::new(Orientation::Vertical, 12);
        header_box.append(&status);

        // Add user button
        let add_btn = Button::with_label("Adicionar Usuario");
        add_btn.add_css_class("suggested-action");
        add_btn.add_css_class("pill");
        add_btn.set_halign(gtk4::Align::Center);
        add_btn.connect_clicked(|_| {
            tracing::info!("Add user clicked");
            // TODO: Show add user dialog
        });
        header_box.append(&add_btn);
        header_box.set_margin_bottom(24);

        header_group.add(&header_box);
        page.add(&header_group);

        // Current user section
        let current_group = PreferencesGroup::builder()
            .title("Seu Usuario")
            .description("Conta atualmente conectada")
            .build();

        // Get current user info
        let current_user = std::env::var("USER").unwrap_or_else(|_| "usuario".to_string());
        let current_row = Self::create_user_row(
            &current_user,
            &Self::get_real_name(&current_user),
            true,
            true,
        );
        current_group.add(&current_row);
        page.add(&current_group);

        // Other users section
        let others_group = PreferencesGroup::builder()
            .title("Outros Usuarios")
            .description("Contas de usuario no sistema")
            .build();

        // Get list of users from /etc/passwd
        let users = Self::get_system_users();
        let current_user_clone = current_user.clone();

        for user in users {
            if user.username != current_user_clone && user.uid >= 1000 && user.uid < 65534 {
                let row = Self::create_user_row(
                    &user.username,
                    &user.real_name,
                    user.is_admin,
                    false,
                );
                others_group.add(&row);
            }
        }

        // If no other users, show message
        let no_users_row = ActionRow::builder()
            .title("Nenhum outro usuario")
            .subtitle("Clique em 'Adicionar Usuario' para criar uma nova conta")
            .build();
        no_users_row.add_prefix(&gtk4::Image::from_icon_name("dialog-information-symbolic"));

        if Self::get_system_users().iter().filter(|u| u.uid >= 1000 && u.uid < 65534).count() <= 1 {
            others_group.add(&no_users_row);
        }

        page.add(&others_group);

        let scrolled = ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .child(&page)
            .build();

        UsersPage {
            widget: scrolled,
            accounts_service,
        }
    }

    fn create_user_row(username: &str, real_name: &str, is_admin: bool, is_current: bool) -> ActionRow {
        let row = ActionRow::builder()
            .title(if real_name.is_empty() { username } else { real_name })
            .subtitle(if is_admin {
                format!("{} - Administrador", username)
            } else {
                format!("{} - Usuario Padrao", username)
            })
            .activatable(true)
            .build();

        // Avatar
        let avatar = adw::Avatar::new(48, Some(real_name), true);
        row.add_prefix(&avatar);

        // Current user indicator
        if is_current {
            let badge = gtk4::Label::new(Some("Voce"));
            badge.add_css_class("dim-label");
            row.add_suffix(&badge);
        }

        // Admin badge
        if is_admin {
            let admin_icon = gtk4::Image::from_icon_name("security-high-symbolic");
            admin_icon.set_tooltip_text(Some("Administrador"));
            row.add_suffix(&admin_icon);
        }

        // Edit button
        let edit_btn = Button::from_icon_name("document-edit-symbolic");
        edit_btn.set_tooltip_text(Some("Editar usuario"));
        edit_btn.add_css_class("flat");
        edit_btn.set_valign(gtk4::Align::Center);
        row.add_suffix(&edit_btn);

        row.add_suffix(&gtk4::Image::from_icon_name("go-next-symbolic"));

        row
    }

    fn get_real_name(username: &str) -> String {
        // Try to get from GECOS field in /etc/passwd
        if let Ok(content) = std::fs::read_to_string("/etc/passwd") {
            for line in content.lines() {
                let parts: Vec<&str> = line.split(':').collect();
                if parts.len() >= 5 && parts[0] == username {
                    let gecos = parts[4];
                    // GECOS format: Real Name,Room,Phone,Other
                    let real_name = gecos.split(',').next().unwrap_or("");
                    if !real_name.is_empty() {
                        return real_name.to_string();
                    }
                }
            }
        }
        username.to_string()
    }

    fn get_system_users() -> Vec<UserInfo> {
        let mut users = Vec::new();

        if let Ok(content) = std::fs::read_to_string("/etc/passwd") {
            for line in content.lines() {
                let parts: Vec<&str> = line.split(':').collect();
                if parts.len() >= 7 {
                    let username = parts[0].to_string();
                    let uid: u32 = parts[2].parse().unwrap_or(0);
                    let gid: u32 = parts[3].parse().unwrap_or(0);
                    let gecos = parts[4];
                    let real_name = gecos.split(',').next().unwrap_or("").to_string();
                    let shell = parts[6];

                    // Check if user has a valid shell (not nologin)
                    let has_login = !shell.contains("nologin") && !shell.contains("false");

                    // Check if admin (member of wheel or sudo group)
                    let is_admin = Self::is_user_admin(&username);

                    if has_login {
                        users.push(UserInfo {
                            username,
                            real_name,
                            uid,
                            gid,
                            is_admin,
                        });
                    }
                }
            }
        }

        users
    }

    fn is_user_admin(username: &str) -> bool {
        // Check /etc/group for wheel or sudo membership
        if let Ok(content) = std::fs::read_to_string("/etc/group") {
            for line in content.lines() {
                let parts: Vec<&str> = line.split(':').collect();
                if parts.len() >= 4 {
                    let group_name = parts[0];
                    let members = parts[3];

                    if (group_name == "wheel" || group_name == "sudo") {
                        if members.split(',').any(|m| m == username) {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    pub fn widget(&self) -> &ScrolledWindow {
        &self.widget
    }

    pub fn refresh(&self) {
        // Refresh user list
        tracing::info!("Refreshing user list");
    }
}

struct UserInfo {
    username: String,
    real_name: String,
    uid: u32,
    gid: u32,
    is_admin: bool,
}
