use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Margin},
    style::{self, Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{
        Block, BorderType, Borders, List, ListItem, Paragraph, Row, Scrollbar,
        ScrollbarOrientation, ScrollbarState, StatefulWidget, Table, Wrap,
    },
    Frame,
};

use crate::app::{App, Focus, StatefulTable};

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
        .constraints([Constraint::Length(7), Constraint::Percentage(60)])
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

    let details = Table::new(job_details)
        .block(
            Block::default()
                .title("Job details")
                .title_alignment(Alignment::Left)
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        .widths(&[Constraint::Percentage(5), Constraint::Percentage(95)]);

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

    let mut jobs_as_rows = Vec::new();
    for job in &app.slurm_jobs.items {
        let row = Row::new(vec![
            Span::styled(job.job_id.clone(), blue_style),
            Span::styled(job.state.clone(), red_style),
            Span::styled(job.job_name.clone(), light_green_style),
        ]);
        jobs_as_rows.push(row);
    }
    let (job_style, output_style) = match app.focus {
        Focus::JobList => (Style::default().fg(Color::Green), Style::default()),
        Focus::Output => (Style::default(), Style::default().fg(Color::Green)),
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

    let mut scrollbar_state_jobs = ScrollbarState::default()
        .content_length(app.slurm_jobs.len())
        .position(app.selected_index);

    let scrollbar_jobs = Scrollbar::new(ScrollbarOrientation::VerticalRight)
        .begin_symbol(Some("↑"))
        .end_symbol(Some("↓"))
        .thumb_symbol("-");

    // frame.render_widget(scrollbar, subchunks[1]);
    frame.render_stateful_widget(
        scrollbar_jobs,
        subchunks[0].inner(&Margin {
            vertical: 0,
            horizontal: 1,
        }), // using a inner vertical margin of 1 unit makes the scrollbar inside the block
        &mut scrollbar_state_jobs,
    );
    let help_options = vec![
        ("q", "quit"),
        ("⏶/⏷", "navigate"),
        ("shift+up/shift+down", "top/bottom"),
        ("ctrl+up/ctrl+down", "fast scroll"),
        ("esc", "cancel"),
        ("enter", "confirm"),
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

    // let output = Paragraph::new(Text::raw(app.output_file.join("\n")))
    //     .    //     .wrap(Wrap { trim: true });

    let output = Table::new(
        app.output_file
            .items
            .iter()
            .fold(Vec::new(), |mut acc, line| {
                acc.push(Row::new(vec![Span::raw(line)]));
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

    let output_scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
        .begin_symbol(Some("↑"))
        .end_symbol(Some("↓"))
        .thumb_symbol("-");

    let mut scrollbar_state_output = ScrollbarState::default()
        .content_length(app.slurm_jobs.len())
        .position(app.selected_index);

    frame.render_stateful_widget(
        output_scrollbar,
        rhs_subchunks[1].inner(&Margin {
            vertical: 0,
            horizontal: 1,
        }), // using a inner vertical margin of 1 unit makes the scrollbar inside the block
        &mut scrollbar_state_output,
    );
    frame.render_stateful_widget(output, rhs_subchunks[1], &mut app.output_file.state);
}
