use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, Borders, List, ListItem, Paragraph, Row, Table},
    Frame,
};

use crate::app::App;

/// Renders the user interface widgets.
pub fn render(app: &mut App, frame: &mut Frame) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(25), Constraint::Percentage(75)])
        .split(frame.size());
    // This is where you add new widgets.
    // See the following resources:
    // - https://docs.rs/ratatui/latest/ratatui/widgets/index.html
    // - https://github.com/ratatui-org/ratatui/tree/master/examples
    frame.render_widget(
        Paragraph::new(format!(
            "Futility: A terminal program to monitor your SLURM jobs.\n\
                Press `Esc`, `Ctrl-C` or `q` to stop running.\n\
                Press left and right to increment and decrement the counter respectively.\n\
                Found information on: {} jobs",
            app.slurm_jobs.len()
        ))
        .block(
            Block::default()
                .title("Template")
                .title_alignment(Alignment::Center)
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        .style(Style::default().fg(Color::Cyan).bg(Color::Black))
        .alignment(Alignment::Center),
        chunks[0],
    );
    // let mut jobs_as_rows = Vec::new();
    // for job in app.slurm_jobs {
    //     let row = Row::new(vec![
    //         job.job_id.clone(),
    //         job.partition.clone(),
    //         job.job_name.clone(),
    //         job.start.clone(),
    //         job.work_dir.clone(),
    //         job.state.clone(),
    //     ]);
    //     jobs_as_rows.push(row);
    // }
    let jobs: Vec<ListItem> = app
        .slurm_jobs
        .items
        .iter()
        .map(|job| ListItem::new(job.job_id.clone()))
        .collect();
    let list = List::new(jobs)
        .block(
            Block::default()
                .title("SLURM Job List")
                .title_alignment(Alignment::Center)
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        .style(Style::default().fg(Color::Cyan).bg(Color::Black))
        .highlight_style(Style::default().add_modifier(Modifier::BOLD))
        .highlight_symbol(">>");

    frame.render_stateful_widget(list, chunks[1], &mut app.slurm_jobs.state)
}
