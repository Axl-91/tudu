use color_eyre::eyre::Result;
use ratatui::{
    crossterm::event::{self, Event, KeyCode, KeyEvent},
    layout::{Constraint, Layout, Position, Rect},
    style::{Modifier, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    DefaultTerminal, Frame,
};

use crate::tudu::Tudu;

pub struct App {
    tudus: Vec<Tudu>,
    msg_size: usize,
    selected: usize,
    list_state: ListState,
    offset: usize,
    edit_mode: bool,
    input: String,
    exit: bool,
}

impl App {
    pub fn new() -> Self {
        Self {
            input: String::new(),
            msg_size: 0,
            selected: 0,
            list_state: ListState::default(),
            offset: 0,
            edit_mode: false,
            tudus: Vec::new(),
            exit: false,
        }
    }

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
                self.edit_mode = false;
            }
            KeyCode::Backspace => {
                self.input.pop();
            }
            KeyCode::Enter => self.create_tudu(),
            event::KeyCode::Char(char) => self.input.push(char),
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

    pub fn run(&mut self, mut terminal: DefaultTerminal) -> Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.check_input()?;
        }
        Ok(())
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
            .highlight_style(Style::default().fg(ratatui::style::Color::Green))
            .highlight_symbol("➤ ");

        frame.render_stateful_widget(list, area, &mut self.list_state);
    }

    fn draw_input(&mut self, frame: &mut Frame, area: Rect) {
        if self.edit_mode {
            frame.set_cursor_position(Position::new(
                area.x + self.input.len() as u16 + 1,
                area.y + 1,
            ));
        }
        let style_input = if self.edit_mode {
            Style::default().fg(ratatui::style::Color::Red)
        } else {
            Style::default()
        };

        let input = Paragraph::new(self.input.as_str())
            .style(Style::default().fg(ratatui::style::Color::White))
            .block(Block::bordered().title("Input").border_style(style_input));

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
        let vertical = Layout::vertical([
            Constraint::Min(1),
            Constraint::Length(3),
            Constraint::Length(1),
        ]);
        let [messages_area, input_area, help_area] = vertical.areas(frame.area());
        self.msg_size = messages_area.height as usize;

        self.draw_messages(frame, messages_area);

        self.draw_input(frame, input_area);

        self.draw_help(frame, help_area);
    }
}
