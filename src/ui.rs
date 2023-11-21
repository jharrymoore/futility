use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Margin},
    style::{Color, Modifier, Style},
    widgets::{
        Block, BorderType, Borders, List, ListItem, Paragraph, Row, Scrollbar,
        ScrollbarOrientation, ScrollbarState, StatefulWidget, Table,
    },
    Frame,
};

use crate::app::App;

/// Renders the user interface widgets.
pub fn render(app: &mut App, frame: &mut Frame) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(20),
            Constraint::Percentage(65),
            Constraint::Percentage(15),
        ])
        .split(frame.size());

    // This is where you add new widgets.
    // See the following resources:
    // - https://docs.rs/ratatui/latest/ratatui/widgets/index.html
    // - https://github.com/ratatui-org/ratatui/tree/master/examples
    frame.render_widget(
        Paragraph::new(format!(
            "Futility: A terminal program to monitor your SLURM jobs.\n\
                Found information on: {} jobs, selected_index: {}",
            app.slurm_jobs.len(),
            app.selected_index
        ))
        .block(
            Block::default()
                .title("Futil")
                .title_alignment(Alignment::Center)
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        // .style(Style::default().fg(Color::Cyan).bg(Color::Black))
        .alignment(Alignment::Center),
        chunks[0],
    );
    let jobs: Vec<ListItem> = app
        .slurm_jobs
        .items
        .iter()
        .map(|job| ListItem::new(job.job_id.clone()))
        .collect();

    let mut jobs_as_rows = Vec::new();
    for job in &app.slurm_jobs.items {
        let row = Row::new(vec![
            job.job_id.clone(),
            job.partition.clone(),
            job.job_name.clone(),
            job.start.clone(),
            job.work_dir.clone(),
            job.state.clone(),
        ]);
        jobs_as_rows.push(row);
    }

    let table = Table::new(jobs_as_rows)
        .block(
            Block::default()
                .title("SLURM Job List")
                .title_alignment(Alignment::Center)
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        .header(
            Row::new(vec![
                "Job ID",
                "Partition",
                "Job Name",
                "Start",
                "Work Dir",
                "State",
            ])
            .style(Style::default().fg(Color::Yellow))
            .bottom_margin(1),
        )
        .highlight_style(Style::default().add_modifier(Modifier::BOLD))
        .highlight_symbol(">>")
        .widths(&[
            Constraint::Percentage(10),
            Constraint::Percentage(5),
            Constraint::Percentage(10),
            Constraint::Percentage(10),
            Constraint::Percentage(40),
            Constraint::Percentage(10),
        ]);

    frame.render_stateful_widget(table, chunks[1], &mut app.slurm_jobs.state);

    let mut scrollbar_state = ScrollbarState::default()
        .content_length(app.slurm_jobs.len())
        .position(app.selected_index);

    let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
        .begin_symbol(Some("↑"))
        .end_symbol(Some("↓"))
        .thumb_symbol("-");

    // frame.render_widget(scrollbar, subchunks[1]);
    frame.render_stateful_widget(
        scrollbar,
        chunks[1].inner(&Margin {
            vertical: 0,
            horizontal: 1,
        }), // using a inner vertical margin of 1 unit makes the scrollbar inside the block
        &mut scrollbar_state,
    );

    let bottom_bar = Block::default()
        .title("Commands")
        .title_alignment(Alignment::Center)
        .borders(Borders::TOP)
        .border_type(BorderType::Rounded);

    frame.render_widget(bottom_bar, chunks[2]);
}
