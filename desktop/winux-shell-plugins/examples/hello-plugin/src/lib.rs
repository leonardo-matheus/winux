//! Hello World Plugin Example
//!
//! This is a simple example plugin demonstrating the plugin API.
//! It adds a greeting button to the panel and registers some commands.

use gtk4 as gtk;
use gtk::prelude::*;

use winux_shell_plugins::prelude::*;

/// The hello world plugin
pub struct HelloPlugin {
    greeting: String,
    click_count: u32,
}

impl Default for HelloPlugin {
    fn default() -> Self {
        Self {
            greeting: "Hello, Winux!".to_string(),
            click_count: 0,
        }
    }
}

impl Plugin for HelloPlugin {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            id: "org.winux.hello-plugin".into(),
            name: "Hello Plugin".into(),
            version: Version::new(1, 0, 0),
            description: "A simple example plugin that says hello".into(),
            authors: vec!["Winux Team".into()],
            homepage: Some("https://winux.org/plugins/hello".into()),
            license: Some("MIT".into()),
            min_api_version: Version::new(1, 0, 0),
            capabilities: vec![PluginCapability::PanelWidget],
            permissions: {
                let mut perms = PermissionSet::new();
                perms.add(Permission::PanelWidgets);
                perms.add(Permission::NotificationsSend);
                perms
            },
            icon: Some("face-smile-symbolic".into()),
            category: Some("Examples".into()),
            keywords: vec!["hello".into(), "example".into(), "demo".into()],
            ..Default::default()
        }
    }

    fn init(&mut self, ctx: &PluginContext) -> PluginResult<()> {
        log::info!("Hello plugin initialized!");
        ctx.log(log::Level::Info, "Hello from the plugin!");
        Ok(())
    }

    fn shutdown(&mut self) -> PluginResult<()> {
        log::info!("Hello plugin shutting down. Total clicks: {}", self.click_count);
        Ok(())
    }

    fn panel_widget(&self) -> Option<Box<dyn PanelWidget>> {
        Some(Box::new(HelloPanelWidget {
            greeting: self.greeting.clone(),
            click_count: self.click_count,
        }))
    }

    fn command_provider(&self) -> Option<Box<dyn CommandProvider>> {
        Some(Box::new(HelloCommandProvider {
            greeting: self.greeting.clone(),
        }))
    }

    fn launcher_provider(&self) -> Option<Box<dyn LauncherProvider>> {
        Some(Box::new(HelloLauncherProvider))
    }
}

/// Panel widget that shows a greeting
struct HelloPanelWidget {
    greeting: String,
    click_count: u32,
}

impl PanelWidget for HelloPanelWidget {
    fn id(&self) -> &str {
        "hello-widget"
    }

    fn name(&self) -> &str {
        "Hello"
    }

    fn position(&self) -> PanelPosition {
        PanelPosition::Right
    }

    fn size(&self) -> WidgetSize {
        WidgetSize::Small
    }

    fn priority(&self) -> i32 {
        -100 // Low priority, shown at the end
    }

    fn state(&self) -> WidgetState {
        WidgetState::with_icon("face-smile-symbolic")
            .tooltip(&format!("{}\nClicks: {}", self.greeting, self.click_count))
    }

    fn build_widget(&self) -> gtk::Widget {
        let button = gtk::Button::new();
        button.set_has_frame(false);
        button.add_css_class("hello-widget");

        let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 4);

        let icon = gtk::Image::from_icon_name("face-smile-symbolic");
        icon.set_pixel_size(16);
        hbox.append(&icon);

        let label = gtk::Label::new(Some("Hi!"));
        hbox.append(&label);

        button.set_child(Some(&hbox));
        button.set_tooltip_text(Some(&self.greeting));

        button.upcast()
    }

    fn on_click(&mut self) -> WidgetAction {
        self.click_count += 1;
        WidgetAction::Custom {
            name: "greet".to_string(),
            data: self.greeting.clone(),
        }
    }

    fn on_right_click(&mut self) -> WidgetAction {
        WidgetAction::ShowMenu(vec![
            MenuItem::new("say_hello", "Say Hello")
                .with_icon("face-smile-symbolic"),
            MenuItem::new("say_goodbye", "Say Goodbye")
                .with_icon("face-sad-symbolic"),
            MenuItem::separator(),
            MenuItem::new("settings", "Plugin Settings")
                .with_icon("preferences-system-symbolic"),
        ])
    }
}

/// Command provider for hello commands
struct HelloCommandProvider {
    greeting: String,
}

impl CommandProvider for HelloCommandProvider {
    fn id(&self) -> &str {
        "hello-commands"
    }

    fn commands(&self) -> Vec<Command> {
        vec![
            Command::new("hello.greet", "Say Hello")
                .with_description("Display a friendly greeting")
                .with_icon("face-smile-symbolic")
                .with_category("Hello Plugin"),
            Command::new("hello.goodbye", "Say Goodbye")
                .with_description("Display a farewell message")
                .with_icon("face-sad-symbolic")
                .with_category("Hello Plugin"),
            Command::new("hello.custom", "Custom Greeting")
                .with_description("Enter a custom greeting message")
                .with_icon("face-wink-symbolic")
                .with_category("Hello Plugin"),
        ]
    }

    fn execute(&mut self, command_id: &str, context: &CommandContext) -> CommandResult {
        match command_id {
            "hello.greet" => {
                CommandResult::Message(self.greeting.clone())
            }
            "hello.goodbye" => {
                CommandResult::Message("Goodbye from Hello Plugin! See you soon!".to_string())
            }
            "hello.custom" => {
                if let Some(input) = &context.input {
                    CommandResult::Message(format!("Custom greeting: {}", input))
                } else {
                    CommandResult::RequiresInput {
                        prompt: "Enter your custom greeting:".to_string(),
                        placeholder: Some("Hello, World!".to_string()),
                    }
                }
            }
            _ => CommandResult::Error(format!("Unknown command: {}", command_id)),
        }
    }
}

/// Launcher provider that adds hello-related search results
struct HelloLauncherProvider;

impl LauncherProvider for HelloLauncherProvider {
    fn id(&self) -> &str {
        "hello-launcher"
    }

    fn name(&self) -> &str {
        "Hello Plugin"
    }

    fn categories(&self) -> Vec<SearchCategory> {
        vec![SearchCategory::Custom("Hello".to_string())]
    }

    fn trigger_prefix(&self) -> Option<&str> {
        Some("hello ")
    }

    fn can_handle(&self, query: &str) -> bool {
        query.to_lowercase().contains("hello") ||
        query.to_lowercase().starts_with("hi") ||
        query.to_lowercase().contains("greet")
    }

    fn search(&self, context: &SearchContext) -> Vec<SearchResult> {
        let query = context.query.to_lowercase();

        let mut results = Vec::new();

        if query.contains("hello") || query.is_empty() {
            results.push(
                SearchResult::new("hello-greet", "Say Hello")
                    .with_subtitle("Display a friendly greeting")
                    .with_icon("face-smile-symbolic")
                    .with_category(SearchCategory::Custom("Hello".to_string()))
                    .with_score(90)
                    .with_action(SearchAction::new("run", "Run Command").default())
            );
        }

        if query.contains("goodbye") || query.contains("bye") {
            results.push(
                SearchResult::new("hello-goodbye", "Say Goodbye")
                    .with_subtitle("Display a farewell message")
                    .with_icon("face-sad-symbolic")
                    .with_category(SearchCategory::Custom("Hello".to_string()))
                    .with_score(85)
            );
        }

        if query.contains("custom") || query.contains("greet") {
            results.push(
                SearchResult::new("hello-custom", "Custom Greeting")
                    .with_subtitle("Enter your own greeting")
                    .with_icon("face-wink-symbolic")
                    .with_category(SearchCategory::Custom("Hello".to_string()))
                    .with_score(80)
            );
        }

        results
    }

    fn activate(&mut self, result_id: &str, _action_id: Option<&str>) -> ActivationResult {
        match result_id {
            "hello-greet" => {
                log::info!("Hello from launcher!");
                ActivationResult::Close
            }
            "hello-goodbye" => {
                log::info!("Goodbye from launcher!");
                ActivationResult::Close
            }
            "hello-custom" => {
                // Would show input dialog
                ActivationResult::Refresh
            }
            _ => ActivationResult::None,
        }
    }

    fn suggestions(&self) -> Vec<SearchResult> {
        vec![
            SearchResult::new("hello-greet", "Say Hello")
                .with_subtitle("Display a friendly greeting")
                .with_icon("face-smile-symbolic")
                .with_category(SearchCategory::Custom("Hello".to_string()))
        ]
    }
}

// Plugin entry point - this macro creates the necessary FFI functions
winux_shell_plugins::declare_plugin!(HelloPlugin, HelloPlugin::default);
