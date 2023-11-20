use clap::Parser;
use futility::app::{App, AppResult};
use futility::event::{Event, EventHandler};
use futility::handler::handle_key_events;
use futility::tui::Tui;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io;

#[derive(Parser)]
struct Args {
    #[clap(short, long, default_value = "24")]
    time_period: usize,
    #[clap(short, long, default_value = "")]
    user: String,
}

fn main() -> AppResult<()> {
    let args = Args::parse();
    // Create an application.
    let mut app = App::new(args.user, args.time_period);

    // Initialize the terminal user interface.
    let backend = CrosstermBackend::new(io::stderr());
    let mut terminal = Terminal::new(backend)?;
    let events = EventHandler::new(250);
    let mut tui = Tui::new(terminal, events);
    tui.init()?;

    // Start the main loop.
    while app.running {
        // Render the user interface.
        tui.draw(&mut app)?;
        // Handle events.
        match tui.events.next()? {
            Event::Tick => app.tick(),
            Event::Key(key_event) => handle_key_events(key_event, &mut app)?,
            Event::Mouse(_) => {}
            Event::Resize(_, _) => {}
        }
    }

    // Exit the user interface.
    tui.exit()?;
    Ok(())
}
