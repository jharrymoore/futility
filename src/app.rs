use crate::slurm::{refresh_job_list, SlurmJob};
use ratatui::widgets::*;
use std::error;

/// Application result type.
pub type AppResult<T> = std::result::Result<T, Box<dyn error::Error>>;

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
}

impl App {
    /// Constructs a new instance of [`App`].
    pub fn new(user: String, time_period: usize) -> Self {
        let running = true;
        let slurm_jobs = refresh_job_list(&user, time_period);
        Self {
            user,
            time_period,
            running,
            slurm_jobs,
            selected_index: 0,
        }
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
        self.slurm_jobs.previous();
        if self.selected_index > 0 {
            self.selected_index = self.selected_index.saturating_sub(1);
        }
    }

    pub fn on_down(&mut self) {
        self.slurm_jobs.next();
        if self.selected_index < self.slurm_jobs.len() - 1 {
            self.selected_index = self.selected_index.saturating_add(1);
        }
    }
    pub fn on_c(&mut self) {
        // cancel the currently selected job
        let job = &self.slurm_jobs.items[self.selected_index];

        job.cancel_job();
    }
}
