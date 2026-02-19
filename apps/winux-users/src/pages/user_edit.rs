//! User editing page

use gtk4::prelude::*;
use gtk4::{Box, Button, Entry, Orientation, ScrolledWindow, PasswordEntry};
use libadwaita as adw;
use adw::prelude::*;
use adw::{ActionRow, ComboRow, EntryRow, PreferencesGroup, PreferencesPage, SwitchRow, ExpanderRow};

use crate::backend::AccountsService;
use crate::ui::{AvatarPicker, PasswordDialog};

use std::cell::RefCell;
use std::rc::Rc;

/// User edit page
pub struct UserEditPage {
    widget: ScrolledWindow,
    accounts_service: Rc<RefCell<AccountsService>>,
    username: Rc<RefCell<String>>,
}

impl UserEditPage {
    pub fn new(accounts_service: Rc<RefCell<AccountsService>>) -> Self {
        let page = PreferencesPage::new();
        page.set_title("Editar Usuario");
        page.set_icon_name(Some("user-info-symbolic"));

        let username = Rc::new(RefCell::new(String::new()));

        // Avatar section
        let avatar_group = PreferencesGroup::builder()
            .title("Avatar")
            .description("Foto de perfil do usuario")
            .build();

        let avatar_row = ActionRow::builder()
            .title("Foto do Usuario")
            .subtitle("Clique para alterar")
            .activatable(true)
            .build();

        let avatar = adw::Avatar::new(80, Some("Usuario"), true);
        avatar_row.add_prefix(&avatar);

        let change_avatar_btn = Button::with_label("Alterar");
        change_avatar_btn.add_css_class("flat");
        change_avatar_btn.set_valign(gtk4::Align::Center);
        change_avatar_btn.connect_clicked(|_| {
            tracing::info!("Change avatar clicked");
            // TODO: Show avatar picker dialog
        });
        avatar_row.add_suffix(&change_avatar_btn);

        let remove_avatar_btn = Button::from_icon_name("user-trash-symbolic");
        remove_avatar_btn.set_tooltip_text(Some("Remover foto"));
        remove_avatar_btn.add_css_class("flat");
        remove_avatar_btn.set_valign(gtk4::Align::Center);
        avatar_row.add_suffix(&remove_avatar_btn);

        avatar_group.add(&avatar_row);
        page.add(&avatar_group);

        // Basic info section
        let info_group = PreferencesGroup::builder()
            .title("Informacoes Basicas")
            .build();

        let name_row = EntryRow::builder()
            .title("Nome Completo")
            .text("Nome do Usuario")
            .build();
        info_group.add(&name_row);

        let username_row = ActionRow::builder()
            .title("Nome de Usuario")
            .subtitle("usuario")
            .build();
        let username_label = gtk4::Label::new(Some("usuario"));
        username_label.add_css_class("dim-label");
        username_row.add_suffix(&username_label);
        info_group.add(&username_row);

        let email_row = EntryRow::builder()
            .title("Email")
            .text("")
            .build();
        info_group.add(&email_row);

        page.add(&info_group);

        // Account type section
        let type_group = PreferencesGroup::builder()
            .title("Tipo de Conta")
            .description("Nivel de acesso do usuario")
            .build();

        let account_type = ComboRow::builder()
            .title("Tipo")
            .subtitle("Determina as permissoes do usuario")
            .build();
        let types = gtk4::StringList::new(&["Usuario Padrao", "Administrador"]);
        account_type.set_model(Some(&types));
        type_group.add(&account_type);

        let admin_info = ActionRow::builder()
            .title("Sobre tipos de conta")
            .subtitle("Administradores podem instalar software, alterar configuracoes do sistema e gerenciar outros usuarios")
            .build();
        admin_info.add_prefix(&gtk4::Image::from_icon_name("dialog-information-symbolic"));
        type_group.add(&admin_info);

        page.add(&type_group);

        // Password section
        let password_group = PreferencesGroup::builder()
            .title("Senha")
            .build();

        let password_row = ActionRow::builder()
            .title("Senha")
            .subtitle("Ultima alteracao: Desconhecido")
            .activatable(true)
            .build();
        password_row.add_prefix(&gtk4::Image::from_icon_name("dialog-password-symbolic"));

        let change_pwd_btn = Button::with_label("Alterar Senha");
        change_pwd_btn.add_css_class("flat");
        change_pwd_btn.set_valign(gtk4::Align::Center);
        change_pwd_btn.connect_clicked(|btn| {
            tracing::info!("Change password clicked");
            // Show password dialog
            if let Some(window) = btn.root().and_then(|r| r.downcast::<gtk4::Window>().ok()) {
                PasswordDialog::show(&window, "usuario", |old, new| {
                    tracing::info!("Password change requested");
                    // TODO: Call AccountsService to change password
                });
            }
        });
        password_row.add_suffix(&change_pwd_btn);
        password_row.add_suffix(&gtk4::Image::from_icon_name("go-next-symbolic"));
        password_group.add(&password_row);

        page.add(&password_group);

        // Groups section
        let groups_group = PreferencesGroup::builder()
            .title("Grupos")
            .description("Grupos aos quais o usuario pertence")
            .build();

        let groups_expander = ExpanderRow::builder()
            .title("Grupos do Usuario")
            .subtitle("5 grupos")
            .build();

        // Common groups
        let common_groups = [
            ("wheel", "Administradores (sudo)", true),
            ("users", "Usuarios", true),
            ("audio", "Audio", true),
            ("video", "Video", true),
            ("storage", "Armazenamento", false),
            ("network", "Rede", false),
            ("docker", "Docker", false),
            ("libvirt", "Virtualizacao", false),
        ];

        for (group, description, is_member) in common_groups {
            let row = SwitchRow::builder()
                .title(group)
                .subtitle(description)
                .active(is_member)
                .build();
            groups_expander.add_row(&row);
        }

        groups_group.add(&groups_expander);

        let manage_groups_row = ActionRow::builder()
            .title("Gerenciar Todos os Grupos")
            .activatable(true)
            .build();
        manage_groups_row.add_suffix(&gtk4::Image::from_icon_name("go-next-symbolic"));
        groups_group.add(&manage_groups_row);

        page.add(&groups_group);

        // Language section
        let lang_group = PreferencesGroup::builder()
            .title("Idioma")
            .build();

        let language_row = ComboRow::builder()
            .title("Idioma")
            .subtitle("Idioma preferido do usuario")
            .build();
        let languages = gtk4::StringList::new(&[
            "Portugues (Brasil)",
            "English (US)",
            "Espanol",
            "Deutsch",
            "Francais",
        ]);
        language_row.set_model(Some(&languages));
        lang_group.add(&language_row);

        page.add(&lang_group);

        // Danger zone
        let danger_group = PreferencesGroup::builder()
            .title("Zona de Perigo")
            .build();

        let disable_row = ActionRow::builder()
            .title("Desativar Conta")
            .subtitle("Impede o usuario de fazer login")
            .activatable(true)
            .build();
        disable_row.add_prefix(&gtk4::Image::from_icon_name("action-unavailable-symbolic"));
        let disable_switch = gtk4::Switch::new();
        disable_switch.set_valign(gtk4::Align::Center);
        disable_row.add_suffix(&disable_switch);
        danger_group.add(&disable_row);

        let delete_row = ActionRow::builder()
            .title("Excluir Usuario")
            .subtitle("Remove permanentemente a conta e todos os arquivos")
            .activatable(true)
            .build();
        delete_row.add_css_class("error");
        delete_row.add_prefix(&gtk4::Image::from_icon_name("user-trash-symbolic"));

        let delete_btn = Button::with_label("Excluir");
        delete_btn.add_css_class("destructive-action");
        delete_btn.set_valign(gtk4::Align::Center);
        delete_btn.connect_clicked(|_| {
            tracing::info!("Delete user clicked");
            // TODO: Show confirmation dialog
        });
        delete_row.add_suffix(&delete_btn);
        danger_group.add(&delete_row);

        page.add(&danger_group);

        // Apply button
        let apply_group = PreferencesGroup::new();

        let button_box = Box::new(Orientation::Horizontal, 12);
        button_box.set_halign(gtk4::Align::Center);
        button_box.set_margin_top(24);
        button_box.set_margin_bottom(24);

        let cancel_btn = Button::with_label("Cancelar");
        cancel_btn.add_css_class("pill");
        button_box.append(&cancel_btn);

        let apply_btn = Button::with_label("Aplicar Alteracoes");
        apply_btn.add_css_class("suggested-action");
        apply_btn.add_css_class("pill");
        apply_btn.connect_clicked(|_| {
            tracing::info!("Apply changes clicked");
            // TODO: Apply changes via AccountsService
        });
        button_box.append(&apply_btn);

        apply_group.add(&button_box);
        page.add(&apply_group);

        let scrolled = ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .child(&page)
            .build();

        UserEditPage {
            widget: scrolled,
            accounts_service,
            username,
        }
    }

    pub fn widget(&self) -> &ScrolledWindow {
        &self.widget
    }

    pub fn load_user(&self, username: &str) {
        *self.username.borrow_mut() = username.to_string();
        tracing::info!("Loading user: {}", username);
        // TODO: Load user data from AccountsService
    }
}
