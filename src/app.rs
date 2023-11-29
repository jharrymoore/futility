use crate::{
    file_watcher::{FileWatcherError, FileWatcherHandle},
    job_watcher::JobWatcherHandle,
    slurm::SlurmJob,
    ui::render,
};
use crossbeam::{
    channel::{unbounded, Receiver},
    select,
};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::{backend::Backend, widgets::*, Terminal};
use std::{error, io, path::PathBuf, time::Duration};

/// Application result type.
pub type AppResult<T> = std::result::Result<T, Box<dyn error::Error>>;

#[derive(Debug)]
pub enum Focus {
    JobList,
    Output,
}

pub enum AppMessage {
    // if job list is not empty, return the vec, otherwise None
    JobList(Option<Vec<SlurmJob>>),
    // Just return the string, split it later
    OutputFile(Result<String, FileWatcherError>),
    Key(KeyEvent),
}

#[derive(Debug)]
pub struct StatefulList<T> {
    pub state: ListState,
    pub items: Vec<T>,
}

impl<T> StatefulList<T> {
    pub fn with_items(items: Vec<T>) -> StatefulList<T> {
        StatefulList {
            state: ListState::default(),
            items,
        }
    }

    pub fn top(&mut self) {
        self.state.select(Some(0));
    }

    pub fn bottom(&mut self) {
        self.state.select(Some(self.items.len() - 1));
    }

    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }
}

impl IntoIterator for StatefulList<SlurmJob> {
    type Item = SlurmJob;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.into_iter()
    }
}

#[derive(Debug, Default)]
pub struct StatefulTable<T> {
    pub state: TableState,
    pub items: Vec<T>,
}

impl<T> StatefulTable<T> {
    pub fn with_items(items: Vec<T>) -> StatefulTable<T> {
        StatefulTable {
            state: TableState::default(),
            items,
        }
    }

    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    self.items.len() - 1
                } else {
                    i + 1
                }
            }
            // the only case where it is None is when it is initialised, next should skip to one.
            None => 1,
        };
        self.state.select(Some(i));
    }

    pub fn top(&mut self) {
        self.state.select(Some(0));
    }

    pub fn bottom(&mut self) {
        self.state.select(Some(self.items.len() - 1));
    }

    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    0
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }
}

impl IntoIterator for StatefulTable<SlurmJob> {
    type Item = SlurmJob;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.into_iter()
    }
}

/// Application.
#[derive(Debug)]
pub struct App {
    /// Is the application running?
    user: String,
    time_period: usize,
    pub running: bool,
    pub slurm_jobs: StatefulTable<SlurmJob>,
    pub selected_index: usize,
    // pub output_file: StatefulTable<String>,
    pub job_output: StatefulTable<String>,
    pub focus: Focus,
    pub output_line_index: usize,
    receiver: Receiver<AppMessage>,
    input_receiver: Receiver<io::Result<Event>>,
    job_watcher_handle: JobWatcherHandle,
    file_watcher_handle: FileWatcherHandle,
}

impl App {
    /// Constructs a new instance of [`App`].
    pub fn new(
        input_rx: Receiver<io::Result<Event>>,
        user: String,
        time_period: usize,
        slurm_refresh: u64,
        file_refresh_rate: u64,
    ) -> Self {
        let (sender, receiver) = unbounded();
        // sender gets used for the job watcher and slurm watcher threads.
        let running = true;
        let slurm_jobs = StatefulTable::<SlurmJob>::default();
        let job_output = StatefulTable::<String>::default();
        let job_watcher_handle = JobWatcherHandle::new(
            sender.clone(),
            Duration::from_secs(slurm_refresh),
            user.clone(),
            time_period,
        );
        let file_watcher_handle =
            FileWatcherHandle::new(sender.clone(), Duration::from_secs(file_refresh_rate));

        Self {
            user,
            time_period,
            running,
            slurm_jobs,
            selected_index: 0,
            focus: Focus::JobList,
            output_line_index: 0,
            job_output,
            receiver,
            input_receiver: input_rx,
            job_watcher_handle,
            file_watcher_handle,
        }
    }

    pub fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> io::Result<()> {
        loop {
            select! {
                // it we get something from the receiver thread:
                    recv(self.receiver) -> event => {
                    self.handle(event.unwrap());
                }
                // from the input_recv channel, handle key presses
                recv(self.input_receiver) -> input_res => {
                    match input_res.unwrap().unwrap() {
                        Event::Key(key_event) => {
                            if (key_event.code == KeyCode::Char('c') && key_event.modifiers == KeyModifiers::CONTROL) || key_event.code == KeyCode::Char('q') {
                                return Ok(());
                            } else {
                                self.handle(AppMessage::Key(key_event));
                            }
                }
                        // resize, anything else, continue
                        Event::Resize(_,_) => {},
                        _ => {}
                    }
                }
            };
            terminal.draw(|f| render(self, f)).unwrap();
        }
    }

    pub fn handle(&mut self, msg: AppMessage) {
        match msg {
            // If we have a refreshed job list, update the slurm jobs in place
            AppMessage::JobList(job_list) => {
                match job_list {
                    Some(job_list) => {
                        self.slurm_jobs.items = job_list;
                        if self.selected_index > self.slurm_jobs.len() - 1 {
                            self.selected_index = self.slurm_jobs.len() - 1;
                        }
                    }
                    None => {}
                }
                // self.slurm_jobs.items = job_list;
                // if self.selected_index > self.slurm_jobs.len() - 1 {
                //     self.selected_index = self.slurm_jobs.len() - 1;
                // }
            }
            // if we have an updated output file, update the output in place
            AppMessage::OutputFile(output_file) => {
                self.job_output.items = match output_file {
                    Ok(contents) => contents.lines().map(|s| s.to_string()).collect(),
                    Err(e) => vec![e.to_string()],
                };
            }
            AppMessage::Key(key_event) => match key_event.code {
                KeyCode::Char('c') | KeyCode::Char('C') => {
                    self.on_c();
                }
                KeyCode::Down => {
                    if key_event.modifiers == KeyModifiers::SHIFT {
                        self.on_shift_down();
                    } else {
                        self.on_down();
                    }
                }
                KeyCode::Char('t') => {
                    self.on_t();
                }
                KeyCode::Char('b') => {
                    self.on_b();
                }
                KeyCode::Up => {
                    if key_event.modifiers == KeyModifiers::SHIFT {
                        self.on_shift_up();
                    } else {
                        self.on_up();
                    }
                }
                KeyCode::Tab => {
                    self.toggle_focus();
                }
                _ => {}
            },
        }
        // update the job watcher
        let curr_output_file = self.get_output_file_path();
        self.file_watcher_handle.set_file_path(curr_output_file);
    }

    // TODO: this function is to go now§
    pub fn get_output_file_path(&mut self) -> Option<PathBuf> {
        // check if stdout is an existing file
        let current_job = &self.slurm_jobs.items.get(self.selected_index);
        if let Some(job) = current_job {
            if let Some(stdout) = &job.stdout {
                return Some(PathBuf::from(stdout));
            } else {
                return Some(PathBuf::from(format!(
                    "{}/slurm-{}.out",
                    job.work_dir.clone(),
                    job.job_id.clone()
                )))
            };
        }
        None
    }

    /// Handles the tick event of the terminal.
    pub fn tick(&self) {}

    pub fn on_up(&mut self) {
        match self.focus {
            Focus::JobList => {
                self.slurm_jobs.previous();
                if self.selected_index > 0 {
                    self.selected_index = self.selected_index.saturating_sub(1);
                    // self.get_output_file_contents();
                }
            }
            Focus::Output => {
                // now this should just scroll up on the output text
                self.job_output.previous();
                if self.output_line_index > 0 {
                    self.output_line_index = self.output_line_index.saturating_sub(1);
                }
            }
        }
    }

    pub fn on_down(&mut self) {
        match self.focus {
            Focus::JobList => {
                self.slurm_jobs.next();
                if self.selected_index < self.slurm_jobs.len() - 1 {
                    self.selected_index = self.selected_index.saturating_add(1);
                    // self.get_output_file_contents();
                }
            }
            Focus::Output => {
                // now this should just scroll down on the output text
                self.job_output.next();
                if self.output_line_index < self.job_output.len() - 1 {
                    self.output_line_index = self.output_line_index.saturating_add(1);
                }
            }
        }
    }
    pub fn on_c(&mut self) {
        // cancel the currently selected job
        let job = &self.slurm_jobs.items[self.selected_index];

        job.cancel();
    }

    pub fn toggle_focus(&mut self) {
        match self.focus {
            Focus::JobList => {
                self.focus = Focus::Output;
            }
            Focus::Output => {
                self.focus = Focus::JobList;
            }
        }
    }

    pub fn on_shift_up(&mut self) {
        match self.focus {
            Focus::JobList => {
                // move up 10 lines
                self.slurm_jobs.state.select(Some(
                    self.slurm_jobs
                        .state
                        .selected()
                        .unwrap_or(0)
                        .saturating_sub(10),
                ));
                self.selected_index = self.selected_index.saturating_sub(10);
            }
            Focus::Output => {
                // move up 10 lines
                self.job_output.state.select(Some(
                    self.job_output
                        .state
                        .selected()
                        .unwrap_or(0)
                        .saturating_sub(10),
                ));
                self.output_line_index = self.output_line_index.saturating_sub(10);
            }
        }
    }

    pub fn on_shift_down(&mut self) {
        match self.focus {
            Focus::JobList => {
                // step down by at most 5 jobs
                if self.selected_index < self.slurm_jobs.len() - 5 {
                    self.selected_index = self.selected_index.saturating_add(5);
                } else {
                    self.selected_index = self.slurm_jobs.len() - 1;
                }
                self.slurm_jobs.state.select(Some(self.selected_index));
            }
            Focus::Output => {
                if self.output_line_index < self.job_output.len() - 5 {
                    self.output_line_index = self.output_line_index.saturating_add(5);
                } else {
                    self.output_line_index = self.job_output.len() - 1;
                }
                // select the new line
                self.job_output.state.select(Some(self.output_line_index));
            }
        }
    }

    pub fn on_t(&mut self) {
        match self.focus {
            Focus::JobList => {
                self.selected_index = 0;
                // self.get_output_file_contents();
                self.slurm_jobs.top()
            }
            Focus::Output => {
                self.output_line_index = 0;
                self.job_output.top();
            }
        }
    }

    pub fn on_b(&mut self) {
        match self.focus {
            Focus::JobList => {
                self.selected_index = self.slurm_jobs.len() - 1;
                self.slurm_jobs.bottom();
                // self.get_output_file_contents();
            }
            Focus::Output => {
                self.output_line_index = self.job_output.len() - 1;
                self.job_output.bottom();
            }
        }
    }
}
