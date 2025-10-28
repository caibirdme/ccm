use anyhow::{Context, Result};
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
};
use serde_json::Value;
use std::io::IsTerminal;
use std::{fs, io};

use crate::config::{profile_path, profiles_dir};
use crate::profile::{
    add_profile_interactive, get_current_profile, launch_claude_code, remove_profile,
    rename_profile, switch_to_profile,
};

/// Application state for the TUI
pub struct App {
    pub profiles: Vec<String>,
    pub current_profile: Option<String>,
    pub selected_profile: usize,
    pub should_quit: bool,
    pub show_popup: Option<PopupType>,
    pub input_buffer: String,
    pub message: Option<String>,
    pub message_timeout: Option<std::time::Instant>,
    pub popup_selection: bool, // true for Yes, false for No
}

/// Types of popups that can be displayed
#[derive(Debug, Clone)]
pub enum PopupType {
    AddProfile,
    ConfirmDelete(String),
    ConfirmSwitch(String),
    RenameProfile(String),
    ShowProfile(String),
    Message(String),
}

impl App {
    pub fn new() -> Result<Self> {
        let mut app = Self {
            profiles: Vec::new(),
            current_profile: None,
            selected_profile: 0,
            should_quit: false,
            show_popup: None,
            input_buffer: String::new(),
            message: None,
            message_timeout: None,
            popup_selection: true, // default to Yes
        };
        app.refresh_profiles()?;
        Ok(app)
    }

    pub fn refresh_profiles(&mut self) -> Result<()> {
        let dir = profiles_dir()?;
        let mut entries: Vec<_> = fs::read_dir(&dir)?
            .filter_map(|e| e.ok())
            .filter(|e| {
                // Skip hidden files and only include .json files
                e.file_name()
                    .to_str()
                    .map(|name| !name.starts_with('.') && name.ends_with(".json"))
                    .unwrap_or(false)
            })
            .collect();

        entries.sort_by_key(|e| e.file_name());

        self.profiles = entries
            .iter()
            .filter_map(|e| {
                e.path()
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .map(|s| s.to_string())
            })
            .collect();

        self.current_profile = get_current_profile()?;

        // Adjust selected profile if it's out of bounds
        if !self.profiles.is_empty() && self.selected_profile >= self.profiles.len() {
            self.selected_profile = self.profiles.len() - 1;
        }

        Ok(())
    }

    pub fn select_next(&mut self) {
        if !self.profiles.is_empty() {
            self.selected_profile = (self.selected_profile + 1) % self.profiles.len();
        }
    }

    pub fn select_previous(&mut self) {
        if !self.profiles.is_empty() {
            self.selected_profile = if self.selected_profile == 0 {
                self.profiles.len() - 1
            } else {
                self.selected_profile - 1
            };
        }
    }

    pub fn get_selected_profile_name(&self) -> Option<&str> {
        self.profiles.get(self.selected_profile).map(|s| s.as_str())
    }

    pub fn show_message(&mut self, msg: String) {
        self.message = Some(msg.clone());
        self.message_timeout = Some(std::time::Instant::now());
    }

    pub fn update_message_timeout(&mut self) {
        if let Some(timeout) = self.message_timeout
            && timeout.elapsed().as_secs_f64() >= 1.0
        {
            self.message = None;
            self.message_timeout = None;
        }
    }
}

/// TUI rendering and event handling
pub struct TuiApp {
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
    app: App,
}

impl TuiApp {
    pub fn new() -> Result<Self> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;

        let app = App::new()?;
        Ok(Self { terminal, app })
    }

    pub fn run(&mut self) -> Result<()> {
        while !self.app.should_quit {
            {
                let app = &self.app;
                self.terminal.draw(|f| {
                    Self::render_ui_static(app, f);
                })?;
            }

            self.handle_events()?;
            self.app.update_message_timeout();
        }

        Ok(())
    }

    fn render_ui_static(app: &App, f: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Header
                Constraint::Min(0),    // Main content
                Constraint::Length(3), // Footer
            ])
            .split(f.area());

        // Render header
        Self::render_header(f, chunks[0]);

        // Render main content
        Self::render_main_content(app, f, chunks[1]);

        // Render footer
        Self::render_footer(f, chunks[2]);

        // Render popup if active
        if let Some(ref popup) = app.show_popup {
            Self::render_popup(app, f, popup);
        }

        // Render temporary message if active
        if let Some(ref message) = app.message {
            Self::render_message(f, message);
        }
    }

    fn render_header(f: &mut Frame, area: Rect) {
        let header_text = vec![Line::from(vec![
            Span::styled(
                "Claude Config Manager",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" - ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                "TUI Mode",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
        ])];

        let header = Paragraph::new(header_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(Color::Blue))
                    .title(" 🎛️  Profile Manager "),
            )
            .style(Style::default().fg(Color::White))
            .alignment(ratatui::layout::Alignment::Center);

        f.render_widget(header, area);
    }

    fn render_main_content(app: &App, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
            .split(area);

        Self::render_profile_list(app, f, chunks[0]);
        Self::render_profile_details(app, f, chunks[1]);
    }

    fn render_profile_list(app: &App, f: &mut Frame, area: Rect) {
        let items: Vec<ListItem> = app
            .profiles
            .iter()
            .enumerate()
            .map(|(i, name)| {
                let is_current = app.current_profile.as_deref() == Some(name);
                let is_selected = i == app.selected_profile;

                let mut style = Style::default();
                if is_selected {
                    // 使用半透明背景色，保持文字可见性
                    style = style.fg(Color::White).bg(Color::Blue);
                } else if is_current {
                    style = style.fg(Color::Green);
                } else {
                    style = style.fg(Color::Gray);
                }

                let prefix = if is_current { "📍 " } else { "  " };
                let content = format!("{}{}", prefix, name);

                ListItem::new(content).style(style)
            })
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title(" Profiles ")
                    .border_style(Style::default().fg(Color::Blue)),
            )
            .highlight_style(
                Style::default()
                    .bg(Color::Rgb(50, 50, 100)) // 半透明深蓝色背景
                    .add_modifier(Modifier::BOLD)
                    .fg(Color::White),
            );

        let mut list_state = ListState::default();
        list_state.select(Some(app.selected_profile));

        f.render_stateful_widget(list, area, &mut list_state);
    }

    fn render_profile_details(app: &App, f: &mut Frame, area: Rect) {
        if let Some(profile_name) = app.profiles.get(app.selected_profile) {
            let details = match Self::get_profile_details_static(profile_name) {
                Ok(details) => details,
                Err(_) => vec![Line::from("Failed to load profile details")],
            };

            let paragraph = Paragraph::new(details)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .title(format!(" Details: {} ", profile_name))
                        .border_style(Style::default().fg(Color::Blue)),
                )
                .wrap(Wrap { trim: false }) // Keep original formatting and indentation
                .scroll((0, 0)); // Enable scrolling if content is too large

            f.render_widget(paragraph, area);
        } else {
            let paragraph = Paragraph::new("No profile selected").block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title(" Details ")
                    .border_style(Style::default().fg(Color::Blue)),
            );

            f.render_widget(paragraph, area);
        }
    }

    fn render_footer(f: &mut Frame, area: Rect) {
        let help_text = vec![
            Line::from(vec![
                Span::styled(
                    "↑/↓",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(": Navigate ", Style::default().fg(Color::Gray)),
                Span::styled(
                    "Enter",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(": Switch ", Style::default().fg(Color::Gray)),
                Span::styled(
                    "a",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(": Add ", Style::default().fg(Color::Gray)),
                Span::styled(
                    "d",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(": Delete ", Style::default().fg(Color::Gray)),
                Span::styled(
                    "r",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(": Rename ", Style::default().fg(Color::Gray)),
                Span::styled(
                    "Q",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                ),
                Span::styled(": quit ", Style::default().fg(Color::Gray)),
            ]),
            Line::from(vec![
                Span::styled(
                    "s",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(": Show ", Style::default().fg(Color::Gray)),
                Span::styled(
                    "i",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(": Import ", Style::default().fg(Color::Gray)),
                Span::styled(
                    "l",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(": Launch ", Style::default().fg(Color::Gray)),
            ]),
            Line::from(vec![
                Span::styled("按 ", Style::default().fg(Color::Gray)),
                Span::styled(
                    "Q",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                ),
                Span::styled(" 退出TUI模式", Style::default().fg(Color::Gray)),
            ]),
        ];

        let footer = Paragraph::new(help_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(Color::Blue)),
            )
            .style(Style::default().fg(Color::White))
            .alignment(ratatui::layout::Alignment::Center);

        f.render_widget(footer, area);
    }

    fn render_popup(app: &App, f: &mut Frame, popup_type: &PopupType) {
        let popup_area = Self::centered_rect(60, 40, f.area());
        f.render_widget(Clear, popup_area);

        match popup_type {
            PopupType::AddProfile => {
                Self::render_add_profile_popup(app, f, popup_area);
            }
            PopupType::ConfirmDelete(profile_name) => {
                Self::render_confirm_delete_popup(f, popup_area, profile_name);
            }
            PopupType::ConfirmSwitch(_) => {
                Self::render_confirm_switch_popup(f, popup_area, app);
            }
            PopupType::RenameProfile(profile_name) => {
                Self::render_rename_profile_popup(app, f, popup_area, profile_name);
            }
            PopupType::ShowProfile(profile_name) => {
                Self::render_show_profile_popup(f, popup_area, profile_name);
            }
            PopupType::Message(msg) => {
                Self::render_message_popup(f, popup_area, msg);
            }
        }
    }

    fn render_message(f: &mut Frame, message: &str) {
        let message_area = Self::centered_rect(50, 20, f.area());
        f.render_widget(Clear, message_area);

        let paragraph = Paragraph::new(message)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .style(Style::default().fg(Color::Green)),
            )
            .style(Style::default().fg(Color::White).bg(Color::DarkGray))
            .alignment(ratatui::layout::Alignment::Center);

        f.render_widget(paragraph, message_area);
    }

    fn render_add_profile_popup(app: &App, f: &mut Frame, area: Rect) {
        let content = vec![
            Line::from("Add New Profile"),
            Line::from(""),
            Line::from(format!("Profile name: {}", app.input_buffer)),
            Line::from(""),
            Line::from("Enter profile name, then press Enter to confirm"),
            Line::from("Press Esc to cancel"),
        ];

        let paragraph = Paragraph::new(content)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title(" Add Profile ")
                    .border_style(Style::default().fg(Color::Yellow)),
            )
            .style(Style::default().fg(Color::White))
            .alignment(ratatui::layout::Alignment::Center)
            .wrap(Wrap { trim: true });

        f.render_widget(paragraph, area);
    }

    fn render_confirm_delete_popup(f: &mut Frame, area: Rect, profile_name: &str) {
        let content = vec![
            Line::from("Confirm Delete"),
            Line::from(""),
            Line::from(format!("Delete profile '{}'?", profile_name)),
            Line::from(""),
            Line::from("y: Yes  n: No  Esc: Cancel"),
        ];

        let paragraph = Paragraph::new(content)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title(" Confirm Delete ")
                    .border_style(Style::default().fg(Color::Red)),
            )
            .style(Style::default().fg(Color::White))
            .alignment(ratatui::layout::Alignment::Center)
            .wrap(Wrap { trim: true });

        f.render_widget(paragraph, area);
    }

    fn render_confirm_switch_popup(f: &mut Frame, area: Rect, app: &App) {
        // Create the popup with borders first
        let popup_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(" Confirm Switch ")
            .border_style(Style::default().fg(Color::Yellow));

        f.render_widget(popup_block, area);

        // Create inner area inside the popup (subtract borders)
        let inner = area.inner(Margin::new(1, 1)); // 1 char margin for borders

        // Split inner area into content area (80%) and button area (20%)
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(80), // Content area
                Constraint::Percentage(20), // Button area
            ])
            .split(inner);

        // Content area - question text
        let content = vec![
            Line::from("Confirm Switch"),
            Line::from(""),
            Line::from(format!(
                "Switch to profile '{}'?",
                if let Some(PopupType::ConfirmSwitch(name)) = &app.show_popup {
                    name.as_str()
                } else {
                    "unknown"
                }
            )),
        ];

        let paragraph = Paragraph::new(content)
            .style(Style::default().fg(Color::White))
            .alignment(ratatui::layout::Alignment::Center)
            .wrap(Wrap { trim: true });

        f.render_widget(paragraph, layout[0]);

        // Button area - center the buttons horizontally
        let buttons_row = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Min(0),    // Left margin (flexible)
                Constraint::Length(8), // Yes button
                Constraint::Length(4), // Gap between buttons
                Constraint::Length(8), // No button
                Constraint::Min(0),    // Right margin (flexible)
            ])
            .split(layout[1]);

        // Button styles - web-like rectangular buttons
        let yes_style = if app.popup_selection {
            Style::default()
                .fg(Color::Black)
                .bg(Color::Rgb(220, 220, 220)) // Light gray background
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
                .fg(Color::White)
                .bg(Color::Rgb(100, 100, 100)) // Dark gray background
        };

        let no_style = if !app.popup_selection {
            Style::default()
                .fg(Color::Black)
                .bg(Color::Rgb(220, 220, 220)) // Light gray background
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
                .fg(Color::White)
                .bg(Color::Rgb(100, 100, 100)) // Dark gray background
        };

        // Create rectangular buttons with Paragraph and blocks
        let yes_button = Paragraph::new("Yes")
            .style(yes_style)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Gray)),
            )
            .alignment(ratatui::layout::Alignment::Center);

        let no_button = Paragraph::new("No")
            .style(no_style)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Gray)),
            )
            .alignment(ratatui::layout::Alignment::Center);

        f.render_widget(yes_button, buttons_row[1]);
        f.render_widget(no_button, buttons_row[3]);
    }

    fn render_rename_profile_popup(app: &App, f: &mut Frame, area: Rect, profile_name: &str) {
        let content = vec![
            Line::from("Rename Profile"),
            Line::from(""),
            Line::from(format!("From: {}", profile_name)),
            Line::from(format!("To: {}", app.input_buffer)),
            Line::from(""),
            Line::from("Enter new name, then press Enter to confirm"),
            Line::from("Press Esc to cancel"),
        ];

        let paragraph = Paragraph::new(content)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title(" Rename Profile ")
                    .border_style(Style::default().fg(Color::Cyan)),
            )
            .style(Style::default().fg(Color::White))
            .alignment(ratatui::layout::Alignment::Center)
            .wrap(Wrap { trim: true });

        f.render_widget(paragraph, area);
    }

    fn render_show_profile_popup(f: &mut Frame, area: Rect, profile_name: &str) {
        let content = match Self::get_profile_details_static(profile_name) {
            Ok(details) => details,
            Err(_) => vec![Line::from("Failed to load profile details")],
        };

        let paragraph = Paragraph::new(content)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title(format!(" Profile: {} ", profile_name))
                    .border_style(Style::default().fg(Color::Green)),
            )
            .style(Style::default().fg(Color::White))
            .wrap(Wrap { trim: true });

        f.render_widget(paragraph, area);
    }

    fn render_message_popup(f: &mut Frame, area: Rect, message: &str) {
        let content = vec![Line::from(message)];

        let paragraph = Paragraph::new(content)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title(" Message ")
                    .border_style(Style::default().fg(Color::Green)),
            )
            .style(Style::default().fg(Color::White))
            .alignment(ratatui::layout::Alignment::Center)
            .wrap(Wrap { trim: true });

        f.render_widget(paragraph, area);
    }

    fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
        let popup_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ])
            .split(r);

        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ])
            .split(popup_layout[1])[1]
    }

    pub fn get_profile_details_static(profile_name: &str) -> Result<Vec<Line<'static>>> {
        let profile_path = profile_path(profile_name)?;
        let content = fs::read_to_string(&profile_path)
            .with_context(|| format!("reading profile {}", profile_path.display()))?;

        let json: Value = serde_json::from_str(&content)?;

        // 创建一个JSON副本，用于隐藏敏感信息
        let mut display_json = json.clone();

        // 隐藏token
        if let Some(env_obj) = display_json.get_mut("env").and_then(|v| v.as_object_mut()) {
            for (key, value) in env_obj.iter_mut() {
                if key.contains("AUTH_TOKEN") || key.contains("TOKEN") {
                    *value = Value::String("••••••••••••••••".to_string());
                }
            }
        }

        // 将处理后的JSON格式化为pretty printed string
        let display_content = serde_json::to_string_pretty(&display_json)
            .with_context(|| "failed to serialize json")?;

        // 解析JSON并添加语法高亮
        let lines = Self::highlight_json(&display_content);

        Ok(lines)
    }

    fn highlight_json(json_str: &str) -> Vec<Line<'static>> {
        let mut lines = Vec::new();

        for line in json_str.lines() {
            let mut spans = Vec::new();
            let mut chars = line.chars().peekable();

            while let Some(ch) = chars.next() {
                if ch == '"' {
                    // 处理字符串
                    let mut string_content = String::new();
                    string_content.push('"'); // 添加开始的引号

                    // 收集字符串内容直到结束的引号
                    while let Some(&next_ch) = chars.peek() {
                        if next_ch == '"' {
                            break;
                        }
                        string_content.push(chars.next().unwrap());
                    }

                    // 添加结束的引号
                    if let Some('"') = chars.next() {
                        string_content.push('"');
                    }

                    // 判断是否是键名还是值
                    let remaining: String = chars.clone().collect();
                    let is_key = remaining.trim().starts_with(':');

                    // 检查是否是隐藏的token
                    let is_token = string_content.contains("•••");

                    let style = if is_key {
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD)
                    } else if is_token {
                        Style::default().fg(Color::Red).add_modifier(Modifier::DIM)
                    } else if string_content.contains("http") {
                        Style::default().fg(Color::Green)
                    } else {
                        Style::default().fg(Color::Magenta)
                    };

                    spans.push(Span::styled(string_content, style));
                } else if ch.is_ascii_digit() {
                    // 处理数字
                    let mut number = ch.to_string();
                    while let Some(&next_ch) = chars.peek() {
                        if next_ch.is_ascii_digit() || next_ch == '.' {
                            number.push(chars.next().unwrap());
                        } else {
                            break;
                        }
                    }
                    spans.push(Span::styled(number, Style::default().fg(Color::Blue)));
                } else if ch.is_ascii_alphabetic() {
                    // 处理布尔值和null
                    let mut word = ch.to_string();
                    while let Some(&next_ch) = chars.peek() {
                        if next_ch.is_ascii_alphabetic() {
                            word.push(chars.next().unwrap());
                        } else {
                            break;
                        }
                    }

                    let style = match word.as_str() {
                        "true" | "false" => Style::default()
                            .fg(Color::Blue)
                            .add_modifier(Modifier::BOLD),
                        "null" => Style::default()
                            .fg(Color::Gray)
                            .add_modifier(Modifier::ITALIC),
                        _ => Style::default().fg(Color::White),
                    };

                    spans.push(Span::styled(word, style));
                } else {
                    // 处理其他字符（括号、冒号、逗号、空格等）
                    let style = match ch {
                        '{' | '}' | '[' | ']' => Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                        ':' | ',' => Style::default().fg(Color::White),
                        _ => Style::default().fg(Color::White),
                    };
                    spans.push(Span::styled(ch.to_string(), style));
                }
            }

            lines.push(Line::from(spans));
        }

        lines
    }

    fn handle_events(&mut self) -> Result<()> {
        if event::poll(std::time::Duration::from_millis(100))?
            && let Event::Key(key) = event::read()?
        {
            // Handle popup-specific input first
            let popup = self.app.show_popup.clone();
            if let Some(popup) = popup {
                self.handle_popup_input(key, &popup)?;
                return Ok(());
            }

            // Handle main app input
            self.handle_main_input(key)?;
        }
        Ok(())
    }

    fn handle_popup_input(&mut self, key: KeyEvent, popup_type: &PopupType) -> Result<()> {
        match popup_type {
            PopupType::AddProfile => {
                match key.code {
                    KeyCode::Enter => {
                        if !self.app.input_buffer.is_empty() {
                            let profile_name = self.app.input_buffer.clone();
                            // Add profile using existing function
                            if let Err(e) = add_profile_interactive(&profile_name, &[]) {
                                self.app
                                    .show_message(format!("Failed to add profile: {}", e));
                            } else {
                                self.app.show_message(format!(
                                    "Profile '{}' added successfully",
                                    profile_name
                                ));
                                self.app.refresh_profiles()?;
                            }
                            self.app.input_buffer.clear();
                            self.app.show_popup = None;
                        }
                    }
                    KeyCode::Char(c) => {
                        self.app.input_buffer.push(c);
                    }
                    KeyCode::Backspace => {
                        self.app.input_buffer.pop();
                    }
                    KeyCode::Esc => {
                        self.app.input_buffer.clear();
                        self.app.show_popup = None;
                    }
                    _ => {}
                }
            }
            PopupType::ConfirmDelete(profile_name) => match key.code {
                KeyCode::Char('y') | KeyCode::Char('Y') => {
                    if let Err(e) = remove_profile(profile_name) {
                        self.app
                            .show_message(format!("Failed to delete profile: {}", e));
                    } else {
                        self.app
                            .show_message(format!("Profile '{}' deleted", profile_name));
                        self.app.refresh_profiles()?;
                    }
                    self.app.show_popup = None;
                }
                KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                    self.app.show_popup = None;
                }
                _ => {}
            },
            PopupType::ConfirmSwitch(profile_name) => {
                match key.code {
                    KeyCode::Left => {
                        self.app.popup_selection = true; // Select Yes
                    }
                    KeyCode::Right => {
                        self.app.popup_selection = false; // Select No
                    }
                    KeyCode::Enter => {
                        if self.app.popup_selection {
                            // Yes selected
                            if let Err(e) = switch_to_profile(profile_name) {
                                self.app
                                    .show_message(format!("Failed to switch profile: {}", e));
                            } else {
                                self.app.show_message(format!(
                                    "Switched to profile '{}'",
                                    profile_name
                                ));
                                self.app.refresh_profiles()?;
                            }
                        }
                        self.app.show_popup = None;
                        self.app.popup_selection = true; // Reset to default
                    }
                    KeyCode::Esc => {
                        self.app.show_popup = None;
                        self.app.popup_selection = true; // Reset to default
                    }
                    _ => {}
                }
            }
            PopupType::RenameProfile(profile_name) => match key.code {
                KeyCode::Enter => {
                    if !self.app.input_buffer.is_empty() {
                        let new_name = self.app.input_buffer.clone();
                        if let Err(e) = rename_profile(profile_name, &new_name) {
                            self.app
                                .show_message(format!("Failed to rename profile: {}", e));
                        } else {
                            self.app
                                .show_message(format!("Profile renamed to '{}'", new_name));
                            self.app.refresh_profiles()?;
                        }
                        self.app.input_buffer.clear();
                        self.app.show_popup = None;
                    }
                }
                KeyCode::Char(c) => {
                    self.app.input_buffer.push(c);
                }
                KeyCode::Backspace => {
                    self.app.input_buffer.pop();
                }
                KeyCode::Esc => {
                    self.app.input_buffer.clear();
                    self.app.show_popup = None;
                }
                _ => {}
            },
            PopupType::ShowProfile(_) => match key.code {
                KeyCode::Esc | KeyCode::Enter => {
                    self.app.show_popup = None;
                }
                _ => {}
            },
            PopupType::Message(_) => {
                self.app.show_popup = None;
            }
        }
        Ok(())
    }

    fn handle_main_input(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                self.app.select_previous();
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.app.select_next();
            }
            KeyCode::Enter => {
                let selected_profile = self.app.profiles.get(self.app.selected_profile).cloned();
                let is_current = self.app.current_profile.as_deref();

                if let Some(profile_name) = selected_profile {
                    // 如果是当前profile，不需要确认
                    if is_current == Some(profile_name.as_str()) {
                        self.app
                            .show_message(format!("Already using profile '{}'", profile_name));
                    } else {
                        self.app.popup_selection = true; // Reset to Yes
                        self.app.show_popup = Some(PopupType::ConfirmSwitch(profile_name));
                    }
                }
            }
            KeyCode::Char('a') => {
                self.app.show_popup = Some(PopupType::AddProfile);
                self.app.input_buffer.clear();
            }
            KeyCode::Char('d') => {
                if let Some(profile_name) = self.app.get_selected_profile_name() {
                    self.app.show_popup = Some(PopupType::ConfirmDelete(profile_name.to_string()));
                }
            }
            KeyCode::Char('r') => {
                if let Some(profile_name) = self.app.get_selected_profile_name() {
                    self.app.show_popup = Some(PopupType::RenameProfile(profile_name.to_string()));
                    self.app.input_buffer.clear();
                }
            }
            KeyCode::Char('s') => {
                if let Some(profile_name) = self.app.get_selected_profile_name() {
                    self.app.show_popup = Some(PopupType::ShowProfile(profile_name.to_string()));
                }
            }
            KeyCode::Char('i') => {
                self.app.input_buffer.clear();
                self.app.show_popup = Some(PopupType::AddProfile);
            }
            KeyCode::Char('l') => {
                if let Err(e) = launch_claude_code() {
                    self.app
                        .show_message(format!("Failed to launch Claude Code: {}", e));
                }
            }
            KeyCode::Char('q') => {
                self.app.should_quit = true;
            }
            _ => {}
        }
        Ok(())
    }
}

impl Drop for TuiApp {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(self.terminal.backend_mut(), LeaveAlternateScreen);
        let _ = self.terminal.show_cursor();
    }
}

/// Public function to launch the TUI
pub fn launch_tui() -> Result<()> {
    // Check if we're in a proper terminal environment
    if !std::io::stderr().is_terminal() {
        anyhow::bail!("TUI mode requires a terminal. Please run in a proper terminal environment.");
    }

    // Check terminal size
    if let Ok((width, height)) = crossterm::terminal::size()
        && (width < 80 || height < 24)
    {
        eprintln!(
            "Warning: Terminal size ({}x{}) is smaller than recommended (80x24)",
            width, height
        );
    }

    // Initialize color-eyre for better error reporting
    let _ = color_eyre::install();

    // Try to initialize TUI with better error handling
    let mut tui_app = match TuiApp::new() {
        Ok(app) => app,
        Err(e) => {
            anyhow::bail!(
                "Failed to initialize TUI: {}. Make sure you're running in a proper terminal.",
                e
            );
        }
    };

    // Run the TUI application
    match tui_app.run() {
        Ok(_) => Ok(()),
        Err(e) => {
            anyhow::bail!("TUI error: {}", e);
        }
    }
}

/// Demo function to show TUI capabilities in non-terminal environments
pub fn demo_tui() -> Result<()> {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║             Claude Config Manager - TUI Demo                ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();

    // Create app to show profile information
    let app = App::new()?;

    println!("📋 Profiles Found: {}", app.profiles.len());
    if let Some(ref current) = app.current_profile {
        println!("📍 Current Profile: {}", current);
    }
    println!();

    // Show a sample profile with JSON highlighting (simulated)
    if !app.profiles.is_empty() {
        let profile_name = &app.profiles[0];
        println!("📄 Sample Profile: {}", profile_name);
        println!("{}", "─".repeat(60));

        // Show the original JSON for comparison
        if let Ok(content) = std::fs::read_to_string(profile_path(profile_name)?) {
            // Process JSON to hide tokens
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                let mut display_json = json.clone();
                if let Some(env_obj) = display_json.get_mut("env").and_then(|v| v.as_object_mut()) {
                    for (key, value) in env_obj.iter_mut() {
                        if key.contains("AUTH_TOKEN") || key.contains("TOKEN") {
                            *value = serde_json::Value::String("••••••••••••••••".to_string());
                        }
                    }
                }

                if let Ok(pretty_json) = serde_json::to_string_pretty(&display_json) {
                    // Simulate syntax highlighting with ANSI colors
                    let highlighted = simulate_syntax_highlighting(&pretty_json);
                    println!("{}", highlighted);
                }
            }
        }
    }

    println!();
    println!("🎮 TUI Controls:");
    println!("   ↑/↓ or j/k: Navigate profiles");
    println!("   Enter: Switch profile (with confirmation)");
    println!("   ←/→: Select Yes/No in dialogs");
    println!("   s: Show profile details (with full syntax highlighting)");
    println!("   a: Add profile, d: Delete, r: Rename, i: Import, l: Launch");
    println!("   q: Quit");
    println!();
    println!("🎨 Features:");
    println!("   ✓ JSON syntax highlighting (cyan keys, magenta strings, blue numbers)");
    println!("   ✓ Interactive confirmation dialogs");
    println!("   ✓ Semi-transparent selection highlighting");
    println!("   ✓ Token masking for security");
    println!("   ✓ Complete profile management");
    println!();
    println!("💡 To use the full TUI mode, run in a real terminal:");
    println!("   ./target/release/ccm ui");
    println!();

    Ok(())
}

fn simulate_syntax_highlighting(json_str: &str) -> String {
    let mut result = String::new();
    let mut chars = json_str.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '"' {
            // Handle strings
            let mut string_content = String::new();
            string_content.push('"');

            while let Some(&next_ch) = chars.peek() {
                if next_ch == '"' {
                    break;
                }
                string_content.push(chars.next().unwrap());
            }

            if let Some('"') = chars.next() {
                string_content.push('"');
            }

            let remaining: String = chars.clone().collect();
            let is_key = remaining.trim().starts_with(':');
            let is_token = string_content.contains("•••");

            // Add ANSI colors
            if is_key {
                result.push_str("\x1b[36;1m"); // Cyan bold
            } else if is_token {
                result.push_str("\x1b[31;2m"); // Red dim
            } else if string_content.contains("http") {
                result.push_str("\x1b[32m"); // Green
            } else {
                result.push_str("\x1b[35m"); // Magenta
            }

            result.push_str(&string_content);
            result.push_str("\x1b[0m"); // Reset
        } else if ch.is_ascii_digit() {
            // Handle numbers
            let mut number = ch.to_string();
            while let Some(&next_ch) = chars.peek() {
                if next_ch.is_ascii_digit() || next_ch == '.' {
                    number.push(chars.next().unwrap());
                } else {
                    break;
                }
            }
            result.push_str(&format!("\x1b[34m{}\x1b[0m", number)); // Blue
        } else if ch.is_ascii_alphabetic() {
            // Handle booleans and null
            let mut word = ch.to_string();
            while let Some(&next_ch) = chars.peek() {
                if next_ch.is_ascii_alphabetic() {
                    word.push(chars.next().unwrap());
                } else {
                    break;
                }
            }

            match word.as_str() {
                "true" | "false" => {
                    result.push_str(&format!("\x1b[34;1m{}\x1b[0m", word)); // Blue bold
                }
                "null" => {
                    result.push_str(&format!("\x1b[37;3m{}\x1b[0m", word)); // Gray italic
                }
                _ => {
                    result.push_str(&word);
                }
            }
        } else {
            // Handle other characters
            let style = match ch {
                '{' | '}' | '[' | ']' => "\x1b[33;1m", // Yellow bold
                ':' | ',' => "\x1b[37m",               // White
                _ => "\x1b[37m",                       // White
            };
            result.push_str(&format!("{}{}\x1b[0m", style, ch));
        }
    }

    result
}

/// Debug function to test TUI components without full terminal
pub fn test_tui_components() -> Result<()> {
    println!("=== Testing TUI Components ===");

    // Test App creation
    println!("1. Testing App creation...");
    let app = App::new()?;
    println!("   ✓ App created successfully");
    println!("   ✓ Found {} profiles", app.profiles.len());
    println!("   ✓ Current profile: {:?}", app.current_profile);

    // Test profile details
    if !app.profiles.is_empty() {
        let profile_name = &app.profiles[0];
        println!("2. Testing JSON highlighting for profile: {}", profile_name);

        let lines = TuiApp::get_profile_details_static(profile_name)?;
        println!("   ✓ Generated {} lines of highlighted JSON", lines.len());

        // Show first few lines
        for (i, line) in lines.iter().take(3).enumerate() {
            println!("   Line {}: {:?}", i, line);
        }
    }

    println!("=== All TUI Components Working ===");
    Ok(())
}
