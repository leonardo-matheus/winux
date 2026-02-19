// Window module - Main application window

use gtk4::prelude::*;
use gtk4::{Application, Box, Orientation};
use libadwaita as adw;
use adw::prelude::*;
use adw::{ApplicationWindow, HeaderBar, ViewStack, ViewSwitcher};

use crate::pages::{UsersPage, UserEditPage, GroupsPage, LoginPage};
use crate::backend::AccountsService;

use std::cell::RefCell;
use std::rc::Rc;

/// Main window for user management
pub struct UsersWindow {
    window: ApplicationWindow,
    stack: ViewStack,
    accounts_service: Rc<RefCell<AccountsService>>,
}

impl UsersWindow {
    pub fn new(app: &Application) -> Self {
        let header = HeaderBar::new();

        let stack = ViewStack::new();
        stack.set_vexpand(true);

        let accounts_service = Rc::new(RefCell::new(AccountsService::new()));

        // Users Page
        let users_page = UsersPage::new(accounts_service.clone());
        stack.add_titled(users_page.widget(), Some("users"), "Usuarios")
            .set_icon_name(Some("system-users-symbolic"));

        // User Edit Page (hidden by default, shown when editing)
        let user_edit_page = UserEditPage::new(accounts_service.clone());
        stack.add_titled(user_edit_page.widget(), Some("user-edit"), "Editar Usuario")
            .set_icon_name(Some("user-info-symbolic"));

        // Groups Page
        let groups_page = GroupsPage::new(accounts_service.clone());
        stack.add_titled(groups_page.widget(), Some("groups"), "Grupos")
            .set_icon_name(Some("system-users-symbolic"));

        // Login Options Page
        let login_page = LoginPage::new(accounts_service.clone());
        stack.add_titled(login_page.widget(), Some("login"), "Login")
            .set_icon_name(Some("preferences-system-login-symbolic"));

        let switcher = ViewSwitcher::builder()
            .stack(&stack)
            .policy(adw::ViewSwitcherPolicy::Wide)
            .build();

        header.set_title_widget(Some(&switcher));

        let main_box = Box::new(Orientation::Vertical, 0);
        main_box.append(&stack);

        let window = ApplicationWindow::builder()
            .application(app)
            .title("Gerenciamento de Usuarios")
            .default_width(900)
            .default_height(650)
            .content(&main_box)
            .build();

        window.set_titlebar(Some(&header));

        UsersWindow {
            window,
            stack,
            accounts_service,
        }
    }

    pub fn present(&self) {
        self.window.present();
    }

    pub fn show_user_edit(&self, username: &str) {
        self.stack.set_visible_child_name("user-edit");
    }

    pub fn show_users_list(&self) {
        self.stack.set_visible_child_name("users");
    }
}
