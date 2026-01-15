//! Core application state and Iced Application implementation
//!
//! Uses Iced 0.13 API with polished visual design inspired by Gauntlet/Raycast.

use iced::widget::{container, text_input, column, row, text, scrollable, Space, image};
use iced::{Element, Length, Theme, Subscription, keyboard, Event, Task, Border, Background, Color, Padding, window};
use iced::keyboard::Key;

use crate::backend::api::{BackendClient, ChatRequest};
use crate::native::apps::AppIndexer;
use crate::hotkey;
use crate::commands::Command;

// ============================================================================
// Theme Colors (Raycast/Gauntlet inspired)
// ============================================================================

mod colors {
    use iced::Color;
    
    pub const BACKGROUND: Color = Color::from_rgb(0.09, 0.09, 0.11);
    pub const SURFACE: Color = Color::from_rgb(0.12, 0.12, 0.14);
    pub const SURFACE_HIGHLIGHT: Color = Color::from_rgb(0.18, 0.18, 0.22);
    pub const BORDER: Color = Color::from_rgb(0.25, 0.25, 0.28);
    pub const PRIMARY: Color = Color::from_rgb(0.4, 0.55, 1.0);
    pub const TEXT: Color = Color::from_rgb(0.95, 0.95, 0.95);
    pub const TEXT_MUTED: Color = Color::from_rgb(0.55, 0.55, 0.6);
    pub const TEXT_PLACEHOLDER: Color = Color::from_rgb(0.4, 0.4, 0.45);
    pub const SELECTION: Color = Color::from_rgb(0.2, 0.25, 0.35);
}

// ============================================================================
// UI State Types
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum UIMode {
    #[default]
    Search,
    Results,
    Chat,
    Settings,
}

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub id: String,
    pub title: String,
    pub subtitle: String,
    pub icon: Option<String>,
    pub category: ResultCategory,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResultCategory {
    App,
    File,
    Command,
    AI,
    Clipboard,
}

// ============================================================================
// Application State
// ============================================================================

pub struct Ruty {
    prompt: String,
    results: Vec<SearchResult>,
    selected_index: usize,
    mode: UIMode,
    loading: bool,
    ai_status: String,
    ai_response: String,
    tools_used: Vec<String>,
    backend: BackendClient,
    app_indexer: AppIndexer,
    visible: bool,
    focused: bool,
    session_id: String,
}

#[derive(Debug, Clone)]
pub enum Message {
    PromptChanged(String),
    PromptSubmit,
    SelectNext,
    SelectPrevious,
    ExecuteSelected,
    Escape,
    SearchComplete(Vec<SearchResult>),
    AIResponseChunk(String),
    AIResponseWithTools { response: String, tools: Vec<String> },
    AIResponseComplete,
    AIError(String),
    Tick,
    WindowFocusLost,
    HotkeyPressed,
    IcedEvent(Event),
}

impl Default for Ruty {
    fn default() -> Self {
        Self {
            prompt: String::new(),
            results: Vec::new(),
            selected_index: 0,
            mode: UIMode::Search,
            loading: false,
            ai_status: String::new(),
            ai_response: String::new(),
            tools_used: Vec::new(),
            backend: BackendClient::new(),
            app_indexer: AppIndexer::new(),
            visible: true,
            focused: true,
            session_id: uuid::Uuid::new_v4().to_string(),
        }
    }
}

impl Ruty {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn title(&self) -> String {
        String::from("Ruty")
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::PromptChanged(new_prompt) => {
                self.prompt = new_prompt.clone();
                
                // Clear results when prompt is empty
                if new_prompt.is_empty() {
                    self.results.clear();
                    self.mode = UIMode::Search;
                }
                // Only show results preview for /app command
                else if new_prompt.starts_with("/app ") {
                    let query = new_prompt.strip_prefix("/app ").unwrap_or("");
                    if !query.is_empty() {
                        self.search(query);
                    }
                }
                
                Task::none()
            }
            
            Message::PromptSubmit => {
                let prompt = self.prompt.clone();
                
                if prompt.is_empty() {
                    return Task::none();
                }
                
                // Parse command
                match Command::parse(&prompt) {
                    Command::App { query } => {
                        // Search for apps and switch to results mode
                        self.search(&query);
                        self.mode = UIMode::Results;
                        return Task::none();
                    }
                    Command::Context { path } => {
                        self.loading = true;
                        self.mode = UIMode::Chat;
                        let backend = self.backend.clone();
                        let session_id = self.session_id.clone();
                        return Task::perform(
                            async move {
                                backend.load_context(&session_id, &path).await
                            },
                            |result| match result {
                                Ok(resp) => Message::AIResponseChunk(resp.message),
                                Err(e) => Message::AIError(e),
                            }
                        ).chain(Task::done(Message::AIResponseComplete));
                    }
                    Command::Clear => {
                        self.prompt.clear();
                        self.ai_response.clear();
                        self.results.clear();
                        self.mode = UIMode::Search;
                        return Task::none();
                    }
                    Command::Providers { provider, model } => {
                        if provider.is_some() {
                            // Update provider
                            self.ai_response = "Provider switching not yet implemented".to_string();
                            self.mode = UIMode::Chat;
                        } else {
                            // List providers
                            self.loading = true;
                            self.mode = UIMode::Chat;
                            let backend = self.backend.clone();
                            return Task::perform(
                                async move {
                                    backend.get_providers().await
                                },
                                |result| match result {
                                    Ok(resp) => {
                                        let provider_list = resp.providers.iter()
                                            .map(|p| format!("• {} ({})", p.display_name, p.models.join(", ")))
                                            .collect::<Vec<_>>()
                                            .join("\n");
                                        Message::AIResponseChunk(format!(
                                            "Available Providers:\n{}\n\nCurrent: {} / {}",
                                            provider_list, resp.current_provider, resp.current_model
                                        ))
                                    }
                                    Err(e) => Message::AIError(e),
                                }
                            ).chain(Task::done(Message::AIResponseComplete));
                        }
                        return Task::none();
                    }
                    Command::Help => {
                        self.ai_response = Command::help_text().to_string();
                        self.mode = UIMode::Chat;
                        return Task::none();
                    }
                    Command::Settings => {
                        self.ai_response = "Settings not yet implemented".to_string();
                        self.mode = UIMode::Chat;
                        return Task::none();
                    }
                    Command::Chat { message } => {
                        // Regular chat - send to AI
                        if !self.results.is_empty() {
                            // If there are search results, execute selected instead
                            self.execute_selected();
                            return Task::none();
                        }
                        
                        self.loading = true;
                        self.ai_status = "🤔 Thinking...".to_string();
                        self.ai_response.clear();
                        self.tools_used.clear();
                        self.mode = UIMode::Chat;
                        
                        let backend = self.backend.clone();
                        let session_id = self.session_id.clone();
                        return Task::perform(
                            async move {
                                let request = ChatRequest {
                                    message,
                                    session_id,
                                    local_context: None,
                                    api_keys: None,
                                };
                                backend.chat(request).await
                            },
                            |result| match result {
                                Ok(resp) => Message::AIResponseWithTools {
                                    response: resp.response,
                                    tools: resp.tools_used,
                                },
                                Err(e) => Message::AIError(e),
                            }
                        ).chain(Task::done(Message::AIResponseComplete));
                    }
                }
            }
            
            Message::SelectNext => {
                if !self.results.is_empty() {
                    self.selected_index = (self.selected_index + 1) % self.results.len();
                }
                Task::none()
            }
            
            Message::SelectPrevious => {
                if !self.results.is_empty() {
                    self.selected_index = if self.selected_index == 0 {
                        self.results.len() - 1
                    } else {
                        self.selected_index - 1
                    };
                }
                Task::none()
            }
            
            Message::ExecuteSelected => {
                self.execute_selected();
                Task::none()
            }
            
            Message::Escape => {
                if self.mode == UIMode::Chat {
                    self.mode = UIMode::Search;
                    self.ai_response.clear();
                } else {
                    self.prompt.clear();
                    self.results.clear();
                    self.mode = UIMode::Search;
                }
                Task::none()
            }
            
            Message::SearchComplete(results) => {
                self.results = results;
                self.selected_index = 0;
                self.mode = UIMode::Results;
                self.loading = false;
                Task::none()
            }
            
            Message::AIResponseChunk(chunk) => {
                self.ai_response.push_str(&chunk);
                Task::none()
            }
            
            Message::AIResponseWithTools { response, tools } => {
                self.ai_response = response;
                self.tools_used = tools.clone();
                
                // Format tools used for status
                if !tools.is_empty() {
                    let tool_icons = tools.iter().map(|t| {
                        match t.as_str() {
                            "search_memory" | "query_supermemory" => "🔍 Searched memory",
                            "add_memory" => "💾 Saved to memory",
                            "open_url" | "open_browser" => "🌐 Opened browser",
                            "run_shell" | "run_command" => "⚙️ Ran command",
                            "get_system_info" => "💻 Got system info",
                            _ => "🔧 Used tool",
                        }
                    }).collect::<Vec<_>>().join(", ");
                    self.ai_status = tool_icons;
                } else {
                    self.ai_status.clear();
                }
                Task::none()
            }
            
            Message::AIResponseComplete => {
                self.loading = false;
                Task::none()
            }
            
            Message::AIError(err) => {
                self.ai_response = format!("Error: {}", err);
                self.loading = false;
                Task::none()
            }
            
            Message::IcedEvent(event) => {
                match event {
                    Event::Keyboard(keyboard::Event::KeyPressed { key, .. }) => {
                        match key {
                            Key::Named(keyboard::key::Named::ArrowDown) => {
                                return self.update(Message::SelectNext);
                            }
                            Key::Named(keyboard::key::Named::ArrowUp) => {
                                return self.update(Message::SelectPrevious);
                            }
                            Key::Named(keyboard::key::Named::Escape) => {
                                return self.update(Message::Escape);
                            }
                            _ => {}
                        }
                    }
                    Event::Window(window::Event::Focused) => {
                        self.focused = true;
                    }
                    Event::Window(window::Event::Unfocused) => {
                        self.focused = false;
                        return self.update(Message::WindowFocusLost);
                    }
                    _ => {}
                }
                Task::none()
            }
            Message::Tick => {
                // Check RPC WindowController for toggle requests
                if let Some(controller) = crate::get_window_controller() {
                    use std::sync::atomic::Ordering;
                    
                    // Check for quit
                    if controller.quit_requested.swap(false, Ordering::SeqCst) {
                        tracing::info!("Quit requested via RPC");
                        std::process::exit(0);
                    }
                    
                    // Check for visibility toggle
                    if controller.toggle_requested.swap(false, Ordering::SeqCst) {
                        let visible = controller.visible.load(Ordering::SeqCst);
                        tracing::info!("Window visibility change via RPC: {}", visible);
                        
                        // Toggle window visibility using resize (Wayland doesn't support move_to)
                        return if visible {
                            // Show: resize to full size and try to bring to front
                            // Show: resize to full size and try to bring to front
                            window::get_oldest().and_then(|id| {
                                Task::batch([
                                    // Reset level to force WM to re-evaluate
                                    window::change_level(id, window::Level::Normal), 
                                    window::resize(id, iced::Size::new(700.0, 400.0)),
                                    window::gain_focus(id),
                                    window::request_user_attention(id, Some(window::UserAttention::Critical)),
                                    // Set AlwaysOnTop LAST (and after a level reset) to be aggressive
                                    window::change_level(id, window::Level::AlwaysOnTop),
                                ])
                            })
                        } else {
                            // Hide: shrink to minimal size and set normal level
                            window::get_oldest().and_then(|id| {
                                Task::batch([
                                    window::resize(id, iced::Size::new(1.0, 1.0)),
                                    window::change_level(id, window::Level::Normal),
                                ])
                            })
                        };
                    }
                }
                
                // Check if hotkey was pressed (X11 or SIGUSR1)
                // Check if hotkey was pressed (X11 or SIGUSR1)
                if hotkey::check_hotkey_pressed() {
                    tracing::info!("Hotkey detected - smart toggling window");
                    if let Some(controller) = crate::get_window_controller() {
                        use std::sync::atomic::Ordering;
                        
                        // Smart Toggle Logic:
                        // If window is FOCUSED, then Hide.
                        // If window is HIDDEN or NOT FOCUSED, then Show.
                        let should_show = !self.focused;
                        
                        controller.visible.store(should_show, Ordering::SeqCst);
                        
                        // We set toggle_requested to true to trigger the actual window update in the block above
                        // But wait, the block above (lines 383+) runs on toggle_requested.
                        // We need to ensure it runs with the NEW visibility state.
                        // Since we just set 'visible', we can set toggle_requested=true and it will be picked up
                        // in the NEXT tick (or we can handle it now if we refactor).
                        // For simplicity, we'll let the next tick handle it, BUT we need to ensure the logic matches.
                        
                        controller.toggle_requested.store(true, Ordering::SeqCst);
                    }
                }
                Task::none()
            }
            
            Message::HotkeyPressed => {
                tracing::info!("Global hotkey pressed: Super+Space");
                Task::none()
            }
            
            Message::WindowFocusLost => {
                if let Some(controller) = crate::get_window_controller() {
                    use std::sync::atomic::Ordering;
                    // Update controller state if currently visible
                    if controller.visible.load(Ordering::SeqCst) {
                        tracing::info!("Focus lost - auto-hiding window");
                        controller.visible.store(false, Ordering::SeqCst);
                        
                        return window::get_oldest().and_then(|id| {
                            Task::batch([
                                window::resize(id, iced::Size::new(1.0, 1.0)),
                                window::change_level(id, window::Level::Normal),
                            ])
                        });
                    }
                }
                Task::none()
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        // Search bar with styling
        let search_bar = container(
            text_input("Ask Ruty anything...", &self.prompt)
                .on_input(Message::PromptChanged)
                .on_submit(Message::PromptSubmit)
                .padding(Padding::new(16.0))
                .size(20)
                .style(|_theme, status| {
                    text_input::Style {
                        background: Background::Color(Color::TRANSPARENT),
                        border: Border::default(),
                        icon: colors::TEXT_MUTED,
                        placeholder: colors::TEXT_PLACEHOLDER,
                        value: colors::TEXT,
                        selection: colors::PRIMARY,
                    }
                })
        )
        .padding(Padding::from([8.0, 16.0]))
        .width(Length::Fill)
        .style(|_theme| container::Style {
            background: Some(Background::Color(colors::SURFACE)),
            border: Border {
                color: colors::BORDER,
                width: 1.0,
                radius: 12.0.into(),
            },
            ..Default::default()
        });

        // Build content based on mode
        let content: Element<'_, Message> = match self.mode {
            UIMode::Search => {
                // Just the search bar with hint text below
                column![
                    search_bar,
                    Space::with_height(16),
                    container(
                        text("Type to search apps, files, or ask AI...")
                            .size(14)
                            .color(colors::TEXT_MUTED)
                    )
                    .width(Length::Fill)
                    .center_x(Length::Fill)
                ]
                .spacing(0)
                .into()
            }
            UIMode::Results => {
                let results_list = self.view_results();
                column![
                    search_bar,
                    Space::with_height(12),
                    results_list
                ]
                .spacing(0)
                .into()
            }
            UIMode::Chat => {
                // Status line (thinking, tools used)
                let status_text = if self.loading {
                    text(&self.ai_status).size(13).color(colors::TEXT_MUTED)
                } else if !self.ai_status.is_empty() {
                    text(&self.ai_status).size(13).color(colors::PRIMARY)
                } else {
                    text("").size(13)
                };
                
                let response_view = container(
                    scrollable(
                        container(
                            text(&self.ai_response)
                                .size(15)
                                .color(colors::TEXT)
                        )
                        .padding(16)
                    )
                    .height(Length::Fill)
                )
                .width(Length::Fill)
                .height(Length::Fill)
                .style(|_theme| container::Style {
                    background: Some(Background::Color(colors::SURFACE)),
                    border: Border::default().rounded(8),
                    ..Default::default()
                });
                
                column![
                    search_bar,
                    Space::with_height(8),
                    status_text,
                    Space::with_height(4),
                    response_view
                ]
                .spacing(0)
                .into()
            }
            UIMode::Settings => {
                column![
                    search_bar,
                    text("Settings - Coming Soon").color(colors::TEXT_MUTED)
                ]
                .into()
            }
        };

        // Main container with rounded corners and proper background
        container(
            container(content)
                .padding(16)
                .width(Length::Fill)
                .height(Length::Fill)
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_theme| container::Style {
            background: Some(Background::Color(colors::BACKGROUND)),
            border: Border {
                color: colors::BORDER,
                width: 1.0,
                radius: 16.0.into(),
            },
            ..Default::default()
        })
        .into()
    }

    fn view_results(&self) -> Element<'_, Message> {
        let items: Vec<Element<'_, Message>> = self
            .results
            .iter()
            .enumerate()
            .map(|(i, result)| {
                let is_selected = i == self.selected_index;
                
                // Render icon: use actual image if available, fallback to text symbol
                let icon_element: Element<'_, Message> = if let Some(ref icon_path) = result.icon {
                    container(
                        image(icon_path.as_str())
                            .width(24)
                            .height(24)
                    )
                    .width(36)
                    .center_x(36)
                    .into()
                } else {
                    // Fallback symbol based on category
                    let symbol = match result.category {
                        ResultCategory::App => "●",
                        ResultCategory::File => "◆",
                        ResultCategory::Command => "»",
                        ResultCategory::AI => "◎",
                        ResultCategory::Clipboard => "▢",
                    };
                    container(
                        text(symbol).size(20).color(colors::PRIMARY)
                    )
                    .width(36)
                    .center_x(36)
                    .into()
                };
                
                let item_content = row![
                    // Icon (image or fallback)
                    icon_element,
                    
                    // Title and subtitle
                    column![
                        text(&result.title)
                            .size(15)
                            .color(if is_selected { colors::TEXT } else { colors::TEXT }),
                        text(&result.subtitle)
                            .size(12)
                            .color(colors::TEXT_MUTED)
                    ]
                    .spacing(2),
                    
                    // Spacer
                    Space::with_width(Length::Fill),
                    
                    // Keyboard hint for selected item
                    text(if is_selected { "↵" } else { "" })
                        .size(12)
                        .color(colors::TEXT_MUTED)
                ]
                .spacing(12)
                .align_y(iced::Alignment::Center);

                container(item_content)
                    .padding(Padding::from([10.0, 12.0]))
                    .width(Length::Fill)
                    .style(move |_theme| container::Style {
                        background: Some(Background::Color(
                            if is_selected { colors::SELECTION } else { Color::TRANSPARENT }
                        )),
                        border: Border::default().rounded(8),
                        ..Default::default()
                    })
                    .into()
            })
            .collect();

        container(
            scrollable(
                column(items).spacing(4)
            )
            .height(Length::Fill)
        )
        .height(Length::FillPortion(1))
        .into()
    }

    pub fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([
            iced::event::listen().map(Message::IcedEvent),
            hotkey::hotkey_tick_subscription().map(|_| Message::Tick),
        ])
    }

    pub fn theme(&self) -> Theme {
        Theme::Dark
    }

    // ========================================================================
    // Business Logic
    // ========================================================================

    fn handle_command(&mut self, input: &str) {
        let parts: Vec<&str> = input.split_whitespace().collect();
        let cmd = parts.first().copied().unwrap_or("");
        let args = parts.get(1..).unwrap_or(&[]).join(" ");

        match cmd {
            "/app" => self.search_apps(&args),
            "/file" => self.search_files(&args),
            "/clip" => self.show_clipboard(),
            "/quit" => std::process::exit(0),
            _ => {}
        }
    }

    fn search(&mut self, query: &str) {
        let app_results: Vec<SearchResult> = self
            .app_indexer
            .search(query)
            .into_iter()
            .take(8)
            .map(|app| SearchResult {
                id: app.id.clone(),
                title: app.name.clone(),
                subtitle: app.categories.first().cloned().unwrap_or_default(),
                icon: app.icon_path().map(|p| p.to_string_lossy().to_string()),
                category: ResultCategory::App,
            })
            .collect();

        self.results = app_results;
        self.selected_index = 0;
        self.mode = if self.results.is_empty() {
            UIMode::Search
        } else {
            UIMode::Results
        };
    }

    fn search_apps(&mut self, query: &str) {
        self.search(query);
    }

    fn search_files(&mut self, _query: &str) {
        // TODO: Implement file search
    }

    fn show_clipboard(&mut self) {
        // TODO: Implement clipboard display
    }

    fn execute_selected(&mut self) {
        if let Some(result) = self.results.get(self.selected_index) {
            match result.category {
                ResultCategory::App => {
                    let _ = self.app_indexer.launch(&result.id);
                }
                _ => {}
            }
        }
    }

    fn send_to_ai(&mut self) {
        self.mode = UIMode::Chat;
        self.loading = true;
        self.ai_response = String::from("Thinking...");
        // TODO: Async call to backend
    }
}
