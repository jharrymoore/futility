use std::collections::HashMap;

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    prelude::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Padding, Paragraph, Row, Table},
    Frame,
};

use crate::app::{App, Focus};

/// Renders the user interface widgets.
pub fn render(app: &mut App, frame: &mut Frame) {
    let blue_style = Style::default().fg(Color::Blue);
    let light_green_style = Style::default().fg(Color::LightGreen);
    let red_style = Style::default().fg(Color::LightRed);
    let orange_style = Style::default().fg(Color::Yellow);
    let white_style = Style::default().fg(Color::White);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(85), Constraint::Max(3)])
        .split(frame.size());

    let subchunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(25), Constraint::Percentage(75)])
        .split(chunks[0]);

    let rhs_subchunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(11),
            // Constraint::Length(3),
            Constraint::Percentage(60),
        ])
        .split(subchunks[1]);

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

    // construct detailed job info
    let mut job_details = Vec::new();
    let selected_job = &app.slurm_jobs.items.get(app.selected_index).clone();
    if let Some(selected_job) = selected_job {
        job_details.push(Row::new(vec![
            Span::styled("STATE".to_string(), blue_style),
            Span::styled(
                selected_job.state.to_owned(),
                *job_status_map
                    .get(&selected_job.state.as_str())
                    .unwrap_or(&white_style),
            ),
        ]));
        job_details.push(Row::new(vec![
            Span::styled("JOB ID".to_string(), blue_style),
            Span::styled(selected_job.job_id.to_owned(), white_style),
        ]));
        job_details.push(Row::new(vec![
            Span::styled("JOB NAME".to_string(), blue_style),
            Span::styled(selected_job.job_name.to_owned(), white_style),
        ]));
        job_details.push(Row::new(vec![
            Span::styled("NODE".to_string(), blue_style),
            Span::styled(selected_job.node_list.to_owned(), white_style),
        ]));
        job_details.push(Row::new(vec![
            Span::styled("WORK DIR".to_string(), blue_style),
            Span::styled(selected_job.work_dir.to_owned(), white_style),
        ]));
        job_details.push(Row::new(vec![
            Span::styled("ACCOUNT".to_string(), blue_style),
            Span::styled(selected_job.account.to_owned(), white_style),
        ]));
        job_details.push(Row::new(vec![
            Span::styled("SUBMIT".to_string(), blue_style),
            Span::styled(selected_job.submit.to_owned(), white_style),
        ]));
        job_details.push(Row::new(vec![
            Span::styled("START".to_string(), blue_style),
            Span::styled(selected_job.start.to_owned(), white_style),
        ]));
        job_details.push(Row::new(vec![
            Span::styled("ELAPSED".to_string(), blue_style),
            Span::styled(
                selected_job.elapsed_time.to_owned() + " / " + &selected_job.time_limit,
                white_style,
            ),
        ]));
    }

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
            Constraint::Percentage(30),
            Constraint::Percentage(5),
            Constraint::Percentage(65),
        ]);

    frame.render_stateful_widget(table, subchunks[0], &mut app.slurm_jobs.state);

    let help_options = vec![
        ("q, ctrl+c", "quit"),
        ("⏶/⏷", "navigate"),
        ("t/b", "top/bottom"),
        ("shift+up/shift+down", "fast scroll"),
        ("esc", "cancel"),
        ("c", "cancel job"),
        ("r", "requeue job"),
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

    frame.render_stateful_widget(output, rhs_subchunks[1], &mut app.job_output.state);

    // if we are in the process of cancelling a job, render a central box over the top,
    if app.cancelling {
        let area = centered_rect(60, 20, frame.size());
        let text = format!(
            "Cancelling job: {}",
            app.slurm_jobs.items[app.selected_index].job_id
        );
        let cancel_box = Paragraph::new(text)
            .block(
                Block::default()
                    .title_alignment(Alignment::Center)
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .style(Style::default().fg(Color::Red)),
            )
            .alignment(Alignment::Center);
        frame.render_widget(Clear, area);
        frame.render_widget(cancel_box, area);
    }
    if app.requeueing {
        let area = centered_rect(60, 20, frame.size());
        let text = format!(
            "Requeueing job: {}",
            app.slurm_jobs.items[app.selected_index].job_id
        );
        let cancel_box = Paragraph::new(text)
            .block(
                Block::default()
                    .title_alignment(Alignment::Center)
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .style(Style::default().fg(Color::Red)),
            )
            .alignment(Alignment::Center);
        frame.render_widget(Clear, area);
        frame.render_widget(cancel_box, area);
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
