// Main Application Window

use crate::ai::AzureOpenAIClient;
use crate::chat::Conversation;
use crate::database::ConversationDatabase;
use crate::ui::{ChatView, InputArea, Sidebar};
use gdk4;
use gtk4::prelude::*;
use gtk4::{gio, glib, ApplicationWindow, Box as GtkBox, Orientation, Paned};
use libadwaita as adw;
use libadwaita::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use parking_lot::RwLock;

pub struct AiWindow {
    pub window: adw::ApplicationWindow,
    pub client: Arc<AzureOpenAIClient>,
    pub database: Arc<ConversationDatabase>,
    pub current_conversation: Rc<RefCell<Option<Conversation>>>,
    pub chat_view: ChatView,
    pub input_area: InputArea,
    pub sidebar: Sidebar,
}

impl AiWindow {
    pub fn new(app: &gtk4::Application) -> adw::ApplicationWindow {
        // Initialize database
        let database = Arc::new(ConversationDatabase::new().expect("Failed to initialize database"));

        // Initialize Azure OpenAI client
        let client = Arc::new(AzureOpenAIClient::new());

        // Create window
        let window = adw::ApplicationWindow::builder()
            .application(app)
            .title("Winux AI Assistant")
            .default_width(1200)
            .default_height(800)
            .build();

        // Apply CSS
        Self::load_css();

        // Create main layout
        let main_box = GtkBox::new(Orientation::Vertical, 0);

        // Create header bar
        let header = Self::create_header_bar();
        main_box.append(&header);

        // Create paned container for sidebar and chat
        let paned = Paned::new(Orientation::Horizontal);
        paned.set_shrink_start_child(false);
        paned.set_shrink_end_child(false);
        paned.set_position(280);

        // Create sidebar
        let sidebar = Sidebar::new(database.clone());
        paned.set_start_child(Some(&sidebar.widget));

        // Create chat area
        let chat_container = GtkBox::new(Orientation::Vertical, 0);
        chat_container.set_hexpand(true);
        chat_container.set_vexpand(true);

        // Create chat view
        let chat_view = ChatView::new();
        chat_container.append(&chat_view.widget);

        // Create input area
        let input_area = InputArea::new();
        chat_container.append(&input_area.widget);

        paned.set_end_child(Some(&chat_container));
        main_box.append(&paned);

        window.set_content(Some(&main_box));

        // Current conversation state
        let current_conversation: Rc<RefCell<Option<Conversation>>> = Rc::new(RefCell::new(None));

        // Setup actions
        Self::setup_actions(&window, client.clone(), database.clone(),
                           chat_view.clone(), input_area.clone(),
                           sidebar.clone(), current_conversation.clone());

        // Connect input area send button
        let client_clone = client.clone();
        let chat_view_clone = chat_view.clone();
        let database_clone = database.clone();
        let current_conv_clone = current_conversation.clone();

        input_area.connect_send(move |message, attachments| {
            Self::handle_send_message(
                &client_clone,
                &chat_view_clone,
                &database_clone,
                &current_conv_clone,
                message,
                attachments,
            );
        });

        // Connect sidebar conversation selection
        let chat_view_clone = chat_view.clone();
        let database_clone = database.clone();
        let current_conv_clone = current_conversation.clone();

        sidebar.connect_conversation_selected(move |conv_id| {
            if let Some(conversation) = database_clone.load_conversation(&conv_id) {
                chat_view_clone.load_conversation(&conversation);
                *current_conv_clone.borrow_mut() = Some(conversation);
            }
        });

        // Load recent conversations
        sidebar.refresh();

        // Start new conversation by default
        let new_conv = Conversation::new();
        *current_conversation.borrow_mut() = Some(new_conv);

        window
    }

    fn load_css() {
        let provider = gtk4::CssProvider::new();
        provider.load_from_string(include_str!("ui/style.css"));

        gtk4::style_context_add_provider_for_display(
            &gdk4::Display::default().expect("Could not get default display"),
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }

    fn create_header_bar() -> adw::HeaderBar {
        let header = adw::HeaderBar::new();

        // New conversation button
        let new_btn = gtk4::Button::builder()
            .icon_name("list-add-symbolic")
            .tooltip_text("New Conversation (Ctrl+N)")
            .build();
        new_btn.set_action_name(Some("win.new-conversation"));
        header.pack_start(&new_btn);

        // Model selector
        let model_dropdown = gtk4::DropDown::from_strings(&["GPT-4o", "o1"]);
        model_dropdown.set_selected(0);
        model_dropdown.set_tooltip_text(Some("Select AI Model"));
        header.pack_start(&model_dropdown);

        // Title
        let title = adw::WindowTitle::new("Winux AI Assistant", "Azure OpenAI");
        header.set_title_widget(Some(&title));

        // Settings button
        let settings_btn = gtk4::Button::builder()
            .icon_name("emblem-system-symbolic")
            .tooltip_text("Settings")
            .build();
        settings_btn.set_action_name(Some("win.settings"));
        header.pack_end(&settings_btn);

        // Export button
        let export_btn = gtk4::Button::builder()
            .icon_name("document-save-symbolic")
            .tooltip_text("Export Conversation")
            .build();
        export_btn.set_action_name(Some("win.export"));
        header.pack_end(&export_btn);

        header
    }

    fn setup_actions(
        window: &adw::ApplicationWindow,
        client: Arc<AzureOpenAIClient>,
        database: Arc<ConversationDatabase>,
        chat_view: ChatView,
        input_area: InputArea,
        sidebar: Sidebar,
        current_conversation: Rc<RefCell<Option<Conversation>>>,
    ) {
        let action_group = gio::SimpleActionGroup::new();

        // New conversation action
        let chat_view_clone = chat_view.clone();
        let current_conv_clone = current_conversation.clone();
        let sidebar_clone = sidebar.clone();
        let new_action = gio::SimpleAction::new("new-conversation", None);
        new_action.connect_activate(move |_, _| {
            let new_conv = Conversation::new();
            *current_conv_clone.borrow_mut() = Some(new_conv);
            chat_view_clone.clear();
            sidebar_clone.refresh();
        });
        action_group.add_action(&new_action);

        // Settings action
        let window_clone = window.clone();
        let settings_action = gio::SimpleAction::new("settings", None);
        settings_action.connect_activate(move |_, _| {
            Self::show_settings_dialog(&window_clone);
        });
        action_group.add_action(&settings_action);

        // Export action
        let window_clone = window.clone();
        let current_conv_clone = current_conversation.clone();
        let export_action = gio::SimpleAction::new("export", None);
        export_action.connect_activate(move |_, _| {
            if let Some(conv) = current_conv_clone.borrow().as_ref() {
                Self::export_conversation(&window_clone, conv);
            }
        });
        action_group.add_action(&export_action);

        // Delete conversation action
        let database_clone = database.clone();
        let sidebar_clone = sidebar.clone();
        let chat_view_clone = chat_view.clone();
        let current_conv_clone = current_conversation.clone();
        let delete_action = gio::SimpleAction::new("delete-conversation", Some(&String::static_variant_type()));
        delete_action.connect_activate(move |_, param| {
            if let Some(id) = param.and_then(|p| p.get::<String>()) {
                database_clone.delete_conversation(&id);
                sidebar_clone.refresh();

                // If deleting current conversation, clear chat
                if let Some(conv) = current_conv_clone.borrow().as_ref() {
                    if conv.id == id {
                        *current_conv_clone.borrow_mut() = Some(Conversation::new());
                        chat_view_clone.clear();
                    }
                }
            }
        });
        action_group.add_action(&delete_action);

        window.insert_action_group("win", Some(&action_group));

        // Keyboard shortcuts
        let app = window.application().unwrap();
        app.set_accels_for_action("win.new-conversation", &["<Ctrl>n"]);
    }

    fn handle_send_message(
        client: &Arc<AzureOpenAIClient>,
        chat_view: &ChatView,
        database: &Arc<ConversationDatabase>,
        current_conversation: &Rc<RefCell<Option<Conversation>>>,
        message: String,
        attachments: Vec<crate::ui::Attachment>,
    ) {
        use crate::chat::{Message, MessageRole};

        // Ensure we have a conversation
        let mut conv_borrow = current_conversation.borrow_mut();
        if conv_borrow.is_none() {
            *conv_borrow = Some(Conversation::new());
        }
        let conversation = conv_borrow.as_mut().unwrap();

        // Add user message
        let user_message = Message::new(MessageRole::User, message.clone());
        conversation.add_message(user_message.clone());
        chat_view.add_message(&user_message);

        // Handle attachments
        for attachment in &attachments {
            match attachment {
                crate::ui::Attachment::Image(path) => {
                    // Add image to message for vision
                    let img_message = Message::with_image(
                        MessageRole::User,
                        format!("[Image: {}]", path),
                        path.clone(),
                    );
                    conversation.add_message(img_message);
                }
                crate::ui::Attachment::File(path) => {
                    // Read file content and add to context
                    if let Ok(content) = std::fs::read_to_string(path) {
                        let file_message = Message::new(
                            MessageRole::User,
                            format!("File content from '{}':\n```\n{}\n```", path, content),
                        );
                        conversation.add_message(file_message);
                    }
                }
            }
        }

        // Show typing indicator
        chat_view.show_typing_indicator();

        // Clone for async
        let client = client.clone();
        let chat_view = chat_view.clone();
        let database = database.clone();
        let conv_id = conversation.id.clone();
        let messages = conversation.get_messages_for_api();

        // Send to API in background
        glib::spawn_future_local(async move {
            match client.chat_completion_stream(messages).await {
                Ok(mut stream) => {
                    chat_view.hide_typing_indicator();

                    let mut full_response = String::new();
                    let response_message = Message::new(MessageRole::Assistant, String::new());
                    chat_view.start_streaming_message(&response_message);

                    while let Some(chunk) = stream.recv().await {
                        full_response.push_str(&chunk);
                        chat_view.append_to_streaming_message(&chunk);
                    }

                    chat_view.finish_streaming_message();

                    // Save to database
                    let assistant_message = Message::new(MessageRole::Assistant, full_response);
                    database.save_message(&conv_id, &assistant_message);
                }
                Err(e) => {
                    chat_view.hide_typing_indicator();
                    chat_view.show_error(&format!("Error: {}", e));
                }
            }
        });

        // Update conversation title if first message
        if conversation.messages.len() == 1 {
            // Use first few words as title
            let title: String = message.split_whitespace().take(5).collect::<Vec<_>>().join(" ");
            conversation.title = if title.len() > 50 {
                format!("{}...", &title[..50])
            } else {
                title
            };
        }

        // Save conversation
        database.save_conversation(conversation);
    }

    fn show_settings_dialog(window: &adw::ApplicationWindow) {
        use crate::ai::Settings;

        let dialog = adw::PreferencesWindow::builder()
            .title("AI Settings")
            .transient_for(window)
            .modal(true)
            .build();

        let page = adw::PreferencesPage::new();

        // Model settings group
        let model_group = adw::PreferencesGroup::builder()
            .title("Model Settings")
            .build();

        // Default model
        let model_row = adw::ComboRow::builder()
            .title("Default Model")
            .subtitle("Select the default AI model")
            .build();
        model_row.set_model(Some(&gtk4::StringList::new(&["GPT-4o", "o1"])));
        model_group.add(&model_row);

        // Temperature
        let temp_row = adw::SpinRow::builder()
            .title("Temperature")
            .subtitle("Controls randomness (0.0 - 2.0)")
            .adjustment(&gtk4::Adjustment::new(0.7, 0.0, 2.0, 0.1, 0.5, 0.0))
            .build();
        model_group.add(&temp_row);

        // Max tokens
        let tokens_row = adw::SpinRow::builder()
            .title("Max Tokens")
            .subtitle("Maximum response length")
            .adjustment(&gtk4::Adjustment::new(4096.0, 256.0, 128000.0, 256.0, 1024.0, 0.0))
            .build();
        model_group.add(&tokens_row);

        page.add(&model_group);

        // System prompt group
        let prompt_group = adw::PreferencesGroup::builder()
            .title("System Prompt")
            .build();

        let prompt_expander = adw::ExpanderRow::builder()
            .title("Custom System Prompt")
            .subtitle("Customize the AI behavior")
            .build();

        let prompt_text = gtk4::TextView::builder()
            .left_margin(12)
            .right_margin(12)
            .top_margin(8)
            .bottom_margin(8)
            .wrap_mode(gtk4::WrapMode::Word)
            .build();
        prompt_text.buffer().set_text("You are a helpful AI assistant integrated into Winux OS. You help users with coding, system administration, file management, and general questions. Be concise but thorough.");

        let prompt_scroll = gtk4::ScrolledWindow::builder()
            .min_content_height(150)
            .child(&prompt_text)
            .build();

        prompt_expander.add_row(&adw::ActionRow::builder().child(&prompt_scroll).build());
        prompt_group.add(&prompt_expander);
        page.add(&prompt_group);

        // Features group
        let features_group = adw::PreferencesGroup::builder()
            .title("Features")
            .build();

        let streaming_row = adw::SwitchRow::builder()
            .title("Streaming Responses")
            .subtitle("Show responses as they're generated")
            .active(true)
            .build();
        features_group.add(&streaming_row);

        let system_info_row = adw::SwitchRow::builder()
            .title("Include System Context")
            .subtitle("Share system information with AI")
            .active(true)
            .build();
        features_group.add(&system_info_row);

        let code_exec_row = adw::SwitchRow::builder()
            .title("Terminal Integration")
            .subtitle("Allow AI to suggest terminal commands")
            .active(true)
            .build();
        features_group.add(&code_exec_row);

        page.add(&features_group);

        dialog.add(&page);
        dialog.present();
    }

    fn export_conversation(window: &adw::ApplicationWindow, conversation: &Conversation) {
        let dialog = gtk4::FileDialog::builder()
            .title("Export Conversation")
            .initial_name(&format!("{}.md", conversation.title.replace(" ", "_")))
            .build();

        let conv_clone = conversation.clone();
        dialog.save(Some(window), None::<&gio::Cancellable>, move |result| {
            if let Ok(file) = result {
                if let Some(path) = file.path() {
                    let markdown = conv_clone.to_markdown();
                    let _ = std::fs::write(path, markdown);
                }
            }
        });
    }
}
