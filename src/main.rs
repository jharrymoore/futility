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
use fern;
use ratatui::backend::{Backend, CrosstermBackend};
use ratatui::Terminal;
use std::time::SystemTime;
use std::{io, thread};

#[derive(Parser)]
struct CLIArgs {
    #[clap(short, long, default_value = "24")]
    time_period: usize,
    #[clap(short, long, default_value = "")]
    user: String,
}



fn setup_logger() -> Result<(), fern::InitError> {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{} {} {}] {}",
                humantime::format_rfc3339_seconds(SystemTime::now()),
                record.level(),
                record.target(),
                message
            ))
        })
        .level(log::LevelFilter::Debug)
        .chain(std::io::stdout())
        .chain(fern::log_file("output.log")?)
        .apply()?;
    Ok(())
}

fn main() -> AppResult<()> {
    let args = CLIArgs::parse();
    // Create an application.
    setup_logger()?;

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(io::stderr());
    let mut terminal = Terminal::new(backend)?;
    run_app(&mut terminal, args)?;

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
    app.run(terminal)?;
    Ok(())
}

fn input_loop(tx: Sender<io::Result<Event>>) {
    loop {
        tx.send(event::read()).unwrap();
    }
}
