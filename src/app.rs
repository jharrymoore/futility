use crate::{
    file_watcher::file_watcher,
    slurm::{refresh_job_list, SlurmJob},
};
use ratatui::widgets::*;
use std::{error, path::PathBuf};

/// Application result type.
pub type AppResult<T> = std::result::Result<T, Box<dyn error::Error>>;

#[derive(Debug)]
pub enum Focus {
    JobList,
    Output,
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

#[derive(Debug)]
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
            None => 0,
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
    pub user: String,
    pub time_period: usize,
    pub running: bool,
    pub slurm_jobs: StatefulTable<SlurmJob>,
    pub selected_index: usize,
    pub output_file: StatefulTable<String>,
    pub focus: Focus,
    pub output_line_index: usize,
}

impl App {
    /// Constructs a new instance of [`App`].
    pub fn new(user: String, time_period: usize) -> Self {
        let running = true;
        let slurm_jobs = refresh_job_list(&user, time_period);
        let output_file_path = format!(
            "{}/slurm-{}.out",
            slurm_jobs.items[0].work_dir.clone(),
            slurm_jobs.items[0].job_id.clone()
        );
        let output_file = file_watcher(PathBuf::from(&output_file_path));
        let output_file_items = match output_file {
            Some(contents) => contents,
            None => vec![format!("Could not read file: {}", &output_file_path).to_string()],
        };
        let output_file = StatefulTable::with_items(output_file_items);
        Self {
            user,
            time_period,
            running,
            slurm_jobs,
            selected_index: 0,
            output_file,
            focus: Focus::JobList,
            output_line_index: 0,
        }
    }

    pub fn get_output_file_contents(&mut self) {
        let output_file_path = format!(
            "{}/slurm-{}.out",
            self.slurm_jobs.items[self.selected_index].work_dir.clone(),
            self.slurm_jobs.items[self.selected_index].job_id.clone()
        );
        let output_file = file_watcher(PathBuf::from(&output_file_path));
        let output_file = match output_file {
            Some(contents) => contents,
            None => vec![format!("Could not read file: {}", &output_file_path).to_string()],
        };

        self.output_file.items = output_file;
    }

    /// Handles the tick event of the terminal.
    pub fn tick(&self) {}

    /// Set running to false to quit the application.
    pub fn quit(&mut self) {
        self.running = false;
    }

    pub fn refresh_slurm_jobs(&mut self) {
        self.slurm_jobs = refresh_job_list(&self.user, self.time_period);
    }

    pub fn on_up(&mut self) {
        match self.focus {
            Focus::JobList => {
                self.slurm_jobs.previous();
                if self.selected_index > 0 {
                    self.selected_index = self.selected_index.saturating_sub(1);
                    self.get_output_file_contents();
                }
            }
            Focus::Output => {
                // now this should just scroll up on the output text
                self.output_file.previous();
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
                    self.get_output_file_contents();
                }
            }
            Focus::Output => {
                // now this should just scroll down on the output text
                self.output_file.next();
                if self.output_line_index < self.output_file.len() - 1 {
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
                self.output_file.state.select(Some(
                    self.output_file
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
                if self.output_line_index < self.output_file.len() - 5 {
                    self.output_line_index = self.output_line_index.saturating_add(5);
                } else {
                    self.output_line_index = self.output_file.len() - 1;
                }
                // select the new line
                self.output_file
                    .state
                    .select(Some(self.output_line_index));
            }
        }
    }

    pub fn on_t(&mut self) {
        match self.focus {
            Focus::JobList => {
                self.selected_index = 0;
                self.get_output_file_contents();
                self.slurm_jobs.top()
            }
            Focus::Output => {
                self.output_line_index = 0;
                self.output_file.top();
            }
        }
    }

    pub fn on_b(&mut self) {
        match self.focus {
            Focus::JobList => {
                self.selected_index = self.slurm_jobs.len() - 1;
                self.slurm_jobs.bottom();
                self.get_output_file_contents();
            }
            Focus::Output => {
                self.output_line_index = self.output_file.len() - 1;
                self.output_file.bottom();
            }
        }
    }
}
