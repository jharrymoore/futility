use clap::Parser;
use crossbeam::channel::{unbounded, Sender};
use crossterm::event::{self, Event};
use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use futility::app::{App, AppResult};
// use futility::event::{Even t, EventHandler};
use ratatui::backend::{Backend, CrosstermBackend};
use ratatui::Terminal;
use std::{io, thread};

#[derive(Parser)]
struct CLIArgs {
    #[clap(short, long, default_value = "24")]
    time_period: usize,
    #[clap(short, long, default_value = "")]
    user: String,
}

fn main() -> AppResult<()> {
    let args = CLIArgs::parse();
    // Create an application.

    // let (sender, receiver) = unbounded();

    // Initialize the terminal user interface.
    // let events = EventHandler::new(250);
    // let mut tui = Tui::new(terminal, events);
    // tui.init()?;
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(io::stderr());
    let mut terminal = Terminal::new(backend)?;
    let _ = run_app(&mut terminal, args);

    // create a channel to communicate between watcher threads and main application thread.

    // Start the main loop.
    // while app.running {
    //     // Render the user interface.
    //     tui.draw(&mut app)?;
    //     // Handle events.
    //     match tui.events.next()? {
    //         Event::Tick => app.tick(),
    //         Event::Key(key_event) => handle_key_events(key_event, &mut app)?,
    //         Event::Mouse(_) => {}
    //         Event::Resize(_, _) => {}
    //     }
    //     // check time elapsed since last update time - if more than 10 s, update
    //     // if app.last_update_time + Duration::seconds(10) < Local::now().time() {
    //     //     app.slurm_jobs = refresh_job_list(&app.user, app.time_period);
    //     //     app.last_update_time = Local::now().time();
    //     // }
    // }

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    // Exit the user interface.
    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, args: CLIArgs) -> io::Result<()> {
    let (input_tx, input_rx) = unbounded();
    let slurm_refresh_rate = 10;
    let file_refresh_rate = 10;
    let mut app = App::new(
        input_rx,
        args.user,
        args.time_period,
        slurm_refresh_rate,
        file_refresh_rate,
    );

    thread::spawn(move || input_loop(input_tx));
    app.run(terminal);
    Ok(())
}

fn input_loop(tx: Sender<io::Result<Event>>) {
    loop {
        tx.send(event::read()).unwrap();
    }
}
