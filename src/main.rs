use color_eyre::eyre::Result;
use ratatui::{
    crossterm::{
        event::{self, Event},
        style::Color,
    },
    layout::{Constraint, Layout},
    style::Stylize,
    widgets::{Block, List, ListItem, Paragraph, Widget},
    DefaultTerminal, Frame,
};

#[derive(Debug, Default)]
struct TuduApp {
    tudus: Vec<Tudu>,
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

fn run(mut terminal: DefaultTerminal, app: &mut TuduApp) -> Result<()> {
    loop {
        terminal.draw(|f| render(f, app))?;
        if let Event::Key(key) = event::read()? {
            match key.code {
                event::KeyCode::Esc => {
                    break;
                }
                _ => {}
            }
        }
    }
    Ok(())
}

fn render(frame: &mut Frame, app: &mut TuduApp) {
    Paragraph::new("Welcome to TuDu").render(frame.area(), frame.buffer_mut());

    let [border_area] = Layout::vertical([Constraint::Fill(1)])
        .margin(1)
        .areas(frame.area());

    let [list_area] = Layout::vertical([Constraint::Fill(1)])
        .margin(1)
        .areas(border_area);

    Block::bordered()
        .border_type(ratatui::widgets::BorderType::Rounded)
        .fg(Color::DarkGrey)
        .render(border_area, frame.buffer_mut());

    List::new(
        app.tudus
            .iter()
            .map(|x| ListItem::from(x.description.clone())),
    )
    .render(list_area, frame.buffer_mut());
}
