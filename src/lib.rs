use app::StatefulTable;
use crossterm::event::KeyEvent;
use slurm::SlurmJob;

/// Application.
pub mod app;

/// Terminal events handler.
pub mod event;

/// Widget renderer.
pub mod ui;

/// Terminal user interface.
pub mod tui;

/// Event handler.
pub mod handler;

pub mod slurm;

pub mod file_watcher;
pub mod job_watcher;

