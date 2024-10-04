use crate::app::{App, CurrentScreen, CurrentlyEditing};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap};
use ratatui::Frame;

pub fn ui(frame: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(3),
        ])
        .split(frame.area());

    let title_block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default());

    let title = Paragraph::new(Text::styled(
        "Modify Borealis schedule",
        Style::default().fg(Color::Green),
    ))
    .block(title_block);

    frame.render_widget(title, chunks[0]);

    let mut list_items = Vec::<ListItem>::new();

    for line in &app.schedule_lines {
        list_items.push(ListItem::new(Line::from(Span::styled(
            line.format(),
            Style::default().fg(Color::Yellow),
        ))));
    }

    let list = List::new(list_items);

    frame.render_widget(list, chunks[1]);

    let current_navigation_text = vec![
        // The first half of the text
        match app.current_screen {
            CurrentScreen::Main => Span::styled("Normal Mode", Style::default().fg(Color::Green)),
            CurrentScreen::Adding => {
                Span::styled("Adding Mode", Style::default().fg(Color::Yellow))
            }
            CurrentScreen::Removing => {
                Span::styled("Removing Mode", Style::default().fg(Color::Yellow))
            }
            CurrentScreen::Exiting => Span::styled("Exiting", Style::default().fg(Color::LightRed)),
        }
        .to_owned(),
        // A white divider bar to separate the two sections
        Span::styled(" | ", Style::default().fg(Color::White)),
        // The final section of the text, with hints on what the user is editing
        {
            if let Some(editing) = &app.currently_editing {
                match editing {
                    CurrentlyEditing::Year => {
                        Span::styled("Editing Year", Style::default().fg(Color::Green))
                    }
                    CurrentlyEditing::Month => {
                        Span::styled("Editing Month", Style::default().fg(Color::Green))
                    }
                    CurrentlyEditing::Day => {
                        Span::styled("Editing Day", Style::default().fg(Color::Green))
                    }
                    CurrentlyEditing::Hour => {
                        Span::styled("Editing Hour", Style::default().fg(Color::Green))
                    }
                    CurrentlyEditing::Minute => {
                        Span::styled("Editing Minute", Style::default().fg(Color::Green))
                    }
                    CurrentlyEditing::Duration => {
                        Span::styled("Editing Duration", Style::default().fg(Color::LightGreen))
                    }
                    CurrentlyEditing::Priority => {
                        Span::styled("Editing Priority", Style::default().fg(Color::Green))
                    }
                    CurrentlyEditing::Experiment => {
                        Span::styled("Editing Experiment", Style::default().fg(Color::Green))
                    }
                    CurrentlyEditing::SchedulingMode => Span::styled(
                        "Editing Scheduling Mode",
                        Style::default().fg(Color::LightGreen),
                    ),
                    CurrentlyEditing::Kwargs => Span::styled(
                        "Editing Keyword Arguments",
                        Style::default().fg(Color::Green),
                    ),
                    CurrentlyEditing::Done => Span::styled(
                        "Confirm entry",
                        Style::default().fg(Color::Green),
                    ),
                }
            } else {
                Span::styled("Not Editing Anything", Style::default().fg(Color::DarkGray))
            }
        },
    ];

    let mode_footer = Paragraph::new(Line::from(current_navigation_text))
        .block(Block::default().borders(Borders::ALL));

    let current_keys_hint = {
        match app.current_screen {
            CurrentScreen::Main => Span::styled(
                "(q) to quit / (a) to add a schedule line / (r) to remove a schedule line",
                Style::default().fg(Color::Red),
            ),
            CurrentScreen::Adding | CurrentScreen::Removing => Span::styled(
                "(ESC) to cancel/(Tab) to switch boxes/enter to complete",
                Style::default().fg(Color::Red),
            ),
            CurrentScreen::Exiting => Span::styled(
                "(q) to quit / (a) to add a schedule line / (r) to remove a schedule line",
                Style::default().fg(Color::Red),
            ),
        }
    };

    let key_notes_footer =
        Paragraph::new(Line::from(current_keys_hint)).block(Block::default().borders(Borders::ALL));

    let footer_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[2]);

    frame.render_widget(mode_footer, footer_chunks[0]);
    frame.render_widget(key_notes_footer, footer_chunks[1]);

    if let Some(editing) = &app.currently_editing {
        let popup_block = Block::default()
            .title("Create a schedule entry")
            .borders(Borders::ALL)
            .style(Style::default().bg(Color::DarkGray));

        let area = centered_rect(60, 25, frame.area());
        frame.render_widget(popup_block, area);

        let popup_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .margin(1)
            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
            .split(area);

        let line_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
            ])
            .split(popup_chunks[0]);

        let selection_block = Block::default().title("Selection").borders(Borders::ALL);
        let mut year_block = Paragraph::new(format!("Year: {}", app.year_input.clone()));
        let mut month_block = Paragraph::new(format!("Month: {}", app.month_input.clone()));
        let mut day_block = Paragraph::new(format!("Day: {}", app.day_input.clone()));
        let mut hour_block = Paragraph::new(format!("Hour: {}", app.hour_input.clone()));
        let mut minute_block = Paragraph::new(format!("Minute: {}", app.minute_input.clone()));
        let mut duration_block =
            Paragraph::new(format!("Duration: {}", app.duration_input.clone()));
        let mut priority_block =
            Paragraph::new(format!("Priority: {}", app.priority_input.clone()));
        let mut experiment_block =
            Paragraph::new(format!("Experiment: {}", app.experiment_input.clone()));
        let mut mode_block = Paragraph::new(format!("Scheduling Mode: {}", app.mode_input.clone()));
        let mut kwargs_block = Paragraph::new(format!("Kwargs: {}", app.kwarg_input.clone()));
        let mut done_block = Paragraph::new("Enter");

        let active_style = Style::default().bg(Color::LightYellow).fg(Color::Black);

        match editing {
            CurrentlyEditing::Year => year_block = year_block.style(active_style),
            CurrentlyEditing::Month => month_block = month_block.style(active_style),
            CurrentlyEditing::Day => day_block = day_block.style(active_style),
            CurrentlyEditing::Hour => hour_block = hour_block.style(active_style),
            CurrentlyEditing::Minute => minute_block = minute_block.style(active_style),
            CurrentlyEditing::Duration => duration_block = duration_block.style(active_style),
            CurrentlyEditing::Priority => priority_block = priority_block.style(active_style),
            CurrentlyEditing::Experiment => experiment_block = experiment_block.style(active_style),
            CurrentlyEditing::SchedulingMode => mode_block = mode_block.style(active_style),
            CurrentlyEditing::Kwargs => kwargs_block = kwargs_block.style(active_style),
            CurrentlyEditing::Done => done_block = done_block.style(active_style),
        };

        frame.render_widget(year_block, line_chunks[0]);
        frame.render_widget(month_block, line_chunks[1]);
        frame.render_widget(day_block, line_chunks[2]);
        frame.render_widget(hour_block, line_chunks[3]);
        frame.render_widget(minute_block, line_chunks[4]);
        frame.render_widget(duration_block, line_chunks[5]);
        frame.render_widget(priority_block, line_chunks[6]);
        frame.render_widget(experiment_block, line_chunks[7]);
        frame.render_widget(mode_block, line_chunks[8]);
        frame.render_widget(kwargs_block, line_chunks[9]);
        frame.render_widget(done_block, line_chunks[10]);

        let selection_text = Paragraph::new("Possible selections").block(selection_block);
        frame.render_widget(selection_text, popup_chunks[1]);
    }

    if let CurrentScreen::Exiting = app.current_screen {
        frame.render_widget(Clear, frame.area()); //this clears the entire screen and anything already drawn
        let popup_block = Block::default()
            .title("Y/N")
            .borders(Borders::NONE)
            .style(Style::default().bg(Color::DarkGray));

        let exit_text = Text::styled(
            "Would you like to write the new schedule? (y/n)",
            Style::default().fg(Color::Red),
        );
        // the `trim: false` will stop the text from being cut off when over the edge of the block
        let exit_paragraph = Paragraph::new(exit_text)
            .block(popup_block)
            .wrap(Wrap { trim: false });

        let area = centered_rect(60, 25, frame.area());
        frame.render_widget(exit_paragraph, area);
    }
}

/// helper function to create a centered rect using up certain percentage of the available rect `r`
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    // Cut the given rectangle into three vertical pieces
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    // Then cut the middle vertical piece into three width-wise pieces
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1] // Return the middle chunk
}
