use crate::app::{App, AppResult};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Handles the key events and updates the state of [`App`].
pub fn handle_key_events(key_event: KeyEvent, app: &mut App) -> AppResult<()> {
    match key_event.code {
        // Exit application on `ESC` or `q`
        KeyCode::Esc | KeyCode::Char('q') => {
            app.quit();
        }
        // Exit application on `Ctrl-C`
        KeyCode::Char('c') | KeyCode::Char('C') => {
            if key_event.modifiers == KeyModifiers::CONTROL {
                app.quit();
            } else {
                app.on_c();
            }
        }
        // Counter handlers
        KeyCode::Char('r') => {
            // TODO: this should be done periodically anyway
            app.refresh_slurm_jobs();
        }
        KeyCode::Down => {
            if key_event.modifiers == KeyModifiers::SHIFT {
                app.on_shift_down();
            } else {
                app.on_down();
            }
        }
        KeyCode::Char('t') => {
            app.on_t();
        }
        KeyCode::Char('b') => {
            app.on_b();
        }
        KeyCode::Up => {
            if key_event.modifiers == KeyModifiers::SHIFT {
                app.on_shift_up();
            } else {
                app.on_up();
            }
        }
        KeyCode::Tab => {
            app.toggle_focus();
        }
        _ => {}
    }
    Ok(())
}
