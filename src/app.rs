use color_eyre::eyre::Result;
use ratatui::{
    crossterm::event::{self, Event, KeyCode, KeyEvent},
    layout::{Constraint, Direction, Layout, Position, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
    DefaultTerminal, Frame,
};

use crate::tudu::Tudu;

#[derive(Debug, Default)]
struct Cursor {
    init_x: u16,
    x: u16,
    y: u16,
    limit: u16,
}

pub struct App {
    tudus: Vec<Tudu>,
    cursor: Cursor,
    msg_size: usize,
    selected: usize,
    list_state: ListState,
    offset: usize,
    edit_mode: bool,
    input: String,
    exit: bool,
}

impl App {
    fn previous_selected(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
            self.adjust_messages();
            self.list_state.select_previous();
        }
    }

    fn next_selected(&mut self) {
        if self.selected + 1 < self.tudus.len() {
            self.selected += 1;
            self.adjust_messages();
            self.list_state.select_next();
        }
    }

    fn adjust_messages(&mut self) {
        if self.selected < self.offset {
            self.offset = self.selected;
        } else if self.selected >= self.offset + self.msg_size {
            self.offset = self.selected - self.msg_size + 1;
        }
    }

    fn default_input(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('q') => self.exit = true,
            KeyCode::Char('e') => self.edit_mode = true,
            KeyCode::Char('E') => self.edit_mode = true,
            KeyCode::Char('D') => {
                if let Some(index) = self.list_state.selected() {
                    self.tudus.remove(index);
                }
            }
            KeyCode::Up => self.previous_selected(),
            KeyCode::Down => self.next_selected(),
            KeyCode::Enter => {
                if let Some(index) = self.list_state.selected() {
                    if let Some(tudu) = self.tudus.get_mut(index) {
                        tudu.change_state();
                    }
                }
            }
            KeyCode::Esc => self.exit = true,
            _ => {}
        }
    }

    fn create_tudu(&mut self) {
        let new_ingress = self.input.clone();
        let new_tudu = Tudu::new(new_ingress);
        self.tudus.push(new_tudu);

        if self.tudus.len() == 1 {
            self.list_state.select_next();
        }

        self.input.clear();
        self.edit_mode = false;
    }

    fn edit_input(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.input.clear();
                self.cursor.x = 0;
                self.edit_mode = false;
            }
            KeyCode::Backspace => {
                if !self.input.is_empty() {
                    self.input.pop();
                    if self.cursor.x == self.cursor.init_x {
                        self.cursor.x = self.cursor.limit;
                        self.cursor.y -= 1;
                    } else {
                        self.cursor.x -= 1;
                    }
                }
            }
            KeyCode::Enter => self.create_tudu(),
            event::KeyCode::Char(char) => {
                if self.input.len() < 250 {
                    self.input.push(char);
                    if self.cursor.x == self.cursor.limit {
                        self.cursor.x = self.cursor.init_x;
                        self.cursor.y += 1;
                    } else {
                        self.cursor.x += 1;
                    }
                }
            }
            _ => {}
        }
    }

    fn check_input(&mut self) -> Result<()> {
        if let Event::Key(key) = event::read()? {
            if self.edit_mode {
                self.edit_input(key);
            } else {
                self.default_input(key);
            }
        }
        Ok(())
    }

    fn get_help_message(&mut self) -> Vec<Span<'_>> {
        if self.edit_mode {
            vec![
                "Go back ".into(),
                "<Esc>".blue().bold(),
                " Add ".into(),
                "<Enter>".blue().bold(),
            ]
        } else {
            vec![
                "Exit ".into(),
                "<Esc/q>".blue().bold(),
                " New ".into(),
                "<e/E>".blue().bold(),
                " Navigate ".into(),
                "<Up/Down>".blue().bold(),
                " Delete ".into(),
                "<D>".blue().bold(),
                " Mark Completed ".into(),
                "<Enter>".blue().bold(),
            ]
        }
    }

    fn get_visible_items(&self) -> &[Tudu] {
        let end = usize::min(self.tudus.len(), self.offset + self.msg_size);
        &self.tudus[self.offset..end]
    }

    fn popup_rect(&self, percent_x: u16, percent_y: u16, r: Rect) -> Rect {
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
            .split(popup_layout[1])[1] // Return the middle chunk
    }

    fn draw_messages(&mut self, frame: &mut Frame, area: Rect) {
        let items: Vec<ListItem> = self
            .get_visible_items()
            .iter()
            .map(|m| {
                let content = Line::from(Span::raw(format!("{m}")));
                ListItem::new(content)
            })
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("TuDu")
                    .title_alignment(ratatui::layout::Alignment::Center),
            )
            .highlight_style(Style::default().fg(Color::Green))
            .highlight_symbol("➤ ");

        frame.render_stateful_widget(list, area, &mut self.list_state);
    }

    fn draw_input(&mut self, frame: &mut Frame) {
        let popup_block = Block::default()
            .title("Create a new TuDu")
            .borders(Borders::ALL)
            .style(Style::default().bg(Color::Black).fg(Color::Red));

        let input = Paragraph::new(self.input.as_str())
            .wrap(Wrap { trim: false })
            .style(Style::default().fg(Color::White))
            .block(popup_block);

        let area = self.popup_rect(75, 15, frame.area());

        if self.cursor.x == 0 {
            self.cursor.init_x = area.x + 1;
            self.cursor.x = area.x + 1;
            self.cursor.y = area.y + 1;
            self.cursor.limit = (area.x - 1) + (area.width - 1);
        }
        let cursor_position = Position::new(self.cursor.x, self.cursor.y);
        frame.set_cursor_position(cursor_position);

        frame.render_widget(input, area);
    }

    fn draw_help(&mut self, frame: &mut Frame, area: Rect) {
        let msg = self.get_help_message();

        let style = Style::default().add_modifier(Modifier::RAPID_BLINK);

        let text = Text::from(Line::from(msg)).patch_style(style);
        let help_message = Paragraph::new(text).centered();
        frame.render_widget(help_message, area);
    }

    fn draw(&mut self, frame: &mut Frame) {
        let vertical = Layout::vertical([Constraint::Min(1), Constraint::Length(1)]);
        let [messages_area, help_area] = vertical.areas(frame.area());
        self.msg_size = messages_area.height as usize;

        self.draw_messages(frame, messages_area);

        self.draw_help(frame, help_area);

        if self.edit_mode {
            self.draw_input(frame);
        }
    }

    pub fn new() -> Self {
        Self {
            input: String::new(),
            cursor: Cursor::default(),
            msg_size: 0,
            selected: 0,
            list_state: ListState::default(),
            offset: 0,
            edit_mode: false,
            tudus: Vec::new(),
            exit: false,
        }
    }

    pub fn run(&mut self, mut terminal: DefaultTerminal) -> Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.check_input()?;
        }
        Ok(())
    }
}
