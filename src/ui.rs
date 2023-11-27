use std::collections::HashMap;

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Gauge, Paragraph, Row, Table},
    Frame,
};

use crate::app::{App, Focus};

/// Renders the user interface widgets.
pub fn render(app: &mut App, frame: &mut Frame) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(85), Constraint::Max(3)])
        .split(frame.size());

    let subchunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(20), Constraint::Percentage(80)])
        .split(chunks[0]);

    let rhs_subchunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(7),
            Constraint::Length(3),
            Constraint::Percentage(60),
        ])
        .split(subchunks[1]);

    // construct detailed job info
    let mut job_details = Vec::new();
    let selected_job = &app.slurm_jobs.items[app.selected_index].clone();

    job_details.push(Row::new(vec![
        "STATE".to_string(),
        selected_job.state.to_owned(),
    ]));
    job_details.push(Row::new(vec![
        "JOB ID".to_string(),
        selected_job.job_id.to_owned(),
    ]));
    job_details.push(Row::new(vec![
        "JOB NAME".to_string(),
        selected_job.job_name.to_owned(),
    ]));
    job_details.push(Row::new(vec![
        "WORK DIR".to_string(),
        selected_job.work_dir.to_owned(),
    ]));
    job_details.push(Row::new(vec![
        "STDOUT".to_string(),
        selected_job.stdout.to_owned().unwrap_or("".to_string()),
    ]));

    let active_job_percent =
        app.slurm_jobs.items[app.selected_index].get_percent_completed() as u16;

    let prog_gauge = Gauge::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Progress")
                .border_type(BorderType::Rounded),
        )
        .gauge_style(
            Style::default()
                .fg(Color::White)
                .bg(Color::Black)
                .add_modifier(Modifier::ITALIC),
        )
        .percent(active_job_percent);
    frame.render_widget(prog_gauge, rhs_subchunks[1]);

    let details = Table::new(job_details)
        .block(
            Block::default()
                .title("Job details")
                .title_alignment(Alignment::Left)
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        .widths(&[Constraint::Length(10), Constraint::Percentage(95)]);

    frame.render_widget(details, rhs_subchunks[0]);

    // add a details block

    // This is where you add new widgets.
    // See the following resources:
    // - https://docs.rs/ratatui/latest/ratatui/widgets/index.html
    // - https://github.com/ratatui-org/ratatui/tree/master/examples
    // let jobs: Vec<ListItem> = app
    //     .slurm_jobs
    //     .items
    //     .iter()
    //     .map(|job| ListItem::new(job.job_id.clone()))
    //     .collect();

    let blue_style = Style::default().fg(Color::Blue);
    let light_green_style = Style::default().fg(Color::LightGreen);
    let red_style = Style::default().fg(Color::LightRed);
    let orange_style = Style::default().fg(Color::Yellow);
    let white_style = Style::default().fg(Color::White);

    let job_status_map = HashMap::from([
        ("F", red_style),
        ("PD", orange_style),
        ("R", light_green_style),
        ("CD", light_green_style),
        ("CA", orange_style),
        ("TO", light_green_style),
        ("PR", orange_style),
        ("NF", orange_style),
        ("RV", orange_style),
        ("S", orange_style),
    ]);

    let mut jobs_as_rows = Vec::new();
    for job in &app.slurm_jobs.items {
        let status_style = job_status_map
            .get(&job.state.as_str())
            .unwrap_or(&red_style);
        let row = Row::new(vec![
            Span::styled(job.job_id.clone(), blue_style),
            Span::styled(job.state.clone(), *status_style),
            Span::styled(job.job_name.clone(), white_style),
        ]);
        jobs_as_rows.push(row);
    }
    let (job_style, output_style) = match app.focus {
        Focus::JobList => (
            Style::default().fg(Color::Green),
            Style::default().fg(Color::White),
        ),
        Focus::Output => (
            Style::default().fg(Color::White),
            Style::default().fg(Color::Green),
        ),
    };

    let table = Table::new(jobs_as_rows)
        .block(
            Block::default()
                .title("SLURM Job List")
                .title_alignment(Alignment::Left)
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .style(job_style),
        )
        .highlight_style(
            Style::default()
                .bg(Color::Green)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        )
        .widths(&[
            Constraint::Percentage(20),
            Constraint::Max(3),
            Constraint::Percentage(70),
        ]);

    frame.render_stateful_widget(table, subchunks[0], &mut app.slurm_jobs.state);

    let help_options = vec![
        ("q, ctrl+c", "quit"),
        ("⏶/⏷", "navigate"),
        ("t/b", "top/bottom"),
        ("shift+up/shift+down", "fast scroll"),
        ("esc", "cancel"),
        ("c", "cancel job"),
        // ("o", "toggle stdout/stderr"),
    ];

    let help = Line::from(
        help_options
            .iter()
            .fold(Vec::new(), |mut acc, (key, description)| {
                if !acc.is_empty() {
                    acc.push(Span::raw(" | "));
                }
                acc.push(Span::styled(*key, blue_style));
                acc.push(Span::raw(": "));
                acc.push(Span::styled(*description, light_green_style));
                acc
            }),
    );

    let help = Paragraph::new(help).block(
        Block::default()
            .title("Commands")
            .title_alignment(Alignment::Left)
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded),
    );
    frame.render_widget(help, chunks[1]);

    let output = Table::new(
        app.job_output
            .items
            .iter()
            .fold(Vec::new(), |mut acc, line| {
                acc.push(Row::new(vec![Span::styled(line, white_style)]));
                acc
            }),
    )
    .widths(&[Constraint::Percentage(100)])
    .block(
        Block::default()
            .title("Output")
            .title_alignment(Alignment::Left)
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .style(output_style),
    )
    .highlight_style(
        Style::default()
            .bg(Color::Blue)
            .fg(Color::Black)
            .add_modifier(Modifier::BOLD),
    );

    frame.render_stateful_widget(output, rhs_subchunks[2], &mut app.job_output.state);
}
