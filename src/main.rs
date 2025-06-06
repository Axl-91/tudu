use color_eyre::eyre::Result;
use ratatui::{
    crossterm::{
        event::{self, Event, KeyEvent},
        style::Color,
    },
    layout::{Constraint, Layout},
    style::{Style, Stylize},
    text::{Line, ToSpan},
    widgets::{block, Block, List, ListState, Padding, Paragraph, Widget},
    DefaultTerminal, Frame,
};

enum FormAction {
    None,
    Submit,
    Escape,
}

#[derive(Debug, Default)]
struct TuduApp {
    tudus: Vec<Tudu>,
    list_state: ListState,
    is_add_new: bool,
    input_value: String,
}

#[derive(Debug, Default)]
struct Tudu {
    completed: bool,
    description: String,
}

fn init_tudu_app() -> TuduApp {
    let mut app = TuduApp::default();

    app.tudus.push(Tudu {
        completed: false,
        description: "Finish TuDu App".to_owned(),
    });
    app.tudus.push(Tudu {
        completed: false,
        description: "Watch Tutorial".to_owned(),
    });
    app.tudus.push(Tudu {
        completed: true,
        description: "Create Struct TuDu".to_owned(),
    });

    return app;
}

fn main() -> Result<()> {
    color_eyre::install()?;

    let mut app = init_tudu_app();

    let terminal = ratatui::init();
    let result = run(terminal, &mut app);

    ratatui::restore();
    result
}

fn handle_key_press(key: KeyEvent, app: &mut TuduApp) -> bool {
    match key.code {
        event::KeyCode::Esc => return true,
        event::KeyCode::Enter => {
            if let Some(index) = app.list_state.selected() {
                if let Some(tudu) = app.tudus.get_mut(index) {
                    tudu.completed = !tudu.completed;
                }
            }
        }
        event::KeyCode::Char(char) => match char {
            'q' => return true,
            'k' => app.list_state.select_next(),
            'j' => app.list_state.select_previous(),
            'A' => app.is_add_new = true,
            'D' => {
                if let Some(index) = app.list_state.selected() {
                    app.tudus.remove(index);
                }
            }
            _ => {}
        },
        _ => {}
    }
    false
}

fn handle_add_new(key: KeyEvent, app: &mut TuduApp) -> FormAction {
    match key.code {
        event::KeyCode::Esc => return FormAction::Escape,
        event::KeyCode::Enter => return FormAction::Submit,
        event::KeyCode::Char(char) => app.input_value.push(char),
        event::KeyCode::Backspace => {
            app.input_value.pop();
        }
        _ => {}
    }
    FormAction::None
}

fn run(mut terminal: DefaultTerminal, app: &mut TuduApp) -> Result<()> {
    loop {
        terminal.draw(|f| render(f, app))?;
        if let Event::Key(key) = event::read()? {
            if app.is_add_new {
                match handle_add_new(key, app) {
                    FormAction::None => {}
                    FormAction::Submit => {
                        app.is_add_new = false;
                        app.tudus.push(Tudu {
                            completed: false,
                            description: app.input_value.clone(),
                        });
                        app.input_value.clear();
                    }
                    FormAction::Escape => {
                        app.is_add_new = false;
                        app.input_value.clear();
                    }
                }
            } else {
                if handle_key_press(key, app) {
                    break;
                }
            }
        }
    }
    Ok(())
}

fn render(frame: &mut Frame, app: &mut TuduApp) {
    let [border_area] = Layout::vertical([Constraint::Fill(1)])
        .margin(1)
        .areas(frame.area());

    if app.is_add_new {
        Paragraph::new(app.input_value.as_str())
            .block(
                Block::bordered()
                    .title("Create a new TuDu".to_span().into_centered_line())
                    .fg(Color::DarkGreen)
                    .padding(Padding::uniform(1))
                    .border_type(block::BorderType::Rounded),
            )
            .render(border_area, frame.buffer_mut());
    } else {
        let [list_area] = Layout::vertical([Constraint::Fill(1)])
            .margin(1)
            .areas(border_area);

        Block::bordered()
            .title("List of TuDus".to_span().into_centered_line())
            .border_type(ratatui::widgets::BorderType::Rounded)
            .fg(Color::DarkMagenta)
            .render(border_area, frame.buffer_mut());

        let style = Style::new().magenta();

        let list = List::new(app.tudus.iter().map(|x| {
            if x.completed {
                Line::styled(format!(" ☐ {}", x.description), style)
            } else {
                Line::styled(format!(" ✓ {}", x.description), style)
            }
        }))
        .highlight_symbol(">")
        .highlight_style(Style::default().fg(ratatui::style::Color::Green));

        frame.render_stateful_widget(list, list_area, &mut app.list_state);
    }
}
