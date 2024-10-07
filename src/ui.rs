use crate::app::{App, CurrentScreen, CurrentlyEditing};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Clear, HighlightSpacing, List, ListItem, Paragraph, Wrap};
use ratatui::Frame;
use ratatui::prelude::Alignment;

pub const BG_COLOR: Color = Color::DarkGray;
pub const NORMAL_ROW_COLOR: Color = Color::DarkGray;
pub const ALT_ROW_COLOR: Color = Color::Blue;
pub const TEXT_COLOR: Color = Color::LightGreen;
pub const SELECTION_STYLE_FG: Color = Color::LightGreen;
pub const SELECTION_HEADER_BG: Color = Color::DarkGray;


pub fn ui(frame: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(3),
        ])
        .split(frame.area());

    render_header(frame, chunks[0]);
    render_schedule(frame, app, chunks[1]);
    render_footer(frame, app, chunks[2]);

    if app.currently_editing.is_some() {
        render_editor(frame, app);
    }

    if let CurrentScreen::Exiting = app.current_screen {
        render_exit_screen(frame);
    }
}

fn render_header(frame: &mut Frame, area: Rect) {
    let title_block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default());

    let title = Paragraph::new(Text::styled(
        "Modify Borealis schedule",
        Style::default().fg(Color::Green),
    ))
        .block(title_block);

    frame.render_widget(title, area);
}

fn render_schedule(frame: &mut Frame, app: &mut App, area: Rect) {
    // We create two blocks, one is for the header (outer) and the other is for list (inner).
    let outer_block = Block::default()
        .borders(Borders::NONE)
        .fg(TEXT_COLOR)
        .bg(SELECTION_HEADER_BG)
        .title("Schedule Lines")
        .title_alignment(Alignment::Center);
    let inner_block = Block::default()
        .borders(Borders::NONE)
        .fg(TEXT_COLOR)
        .bg(NORMAL_ROW_COLOR);

    // We get the inner area from outer_block. We'll use this area later to render the table.
    let outer_area = area;
    let inner_area = outer_block.inner(outer_area);

    // We can render the header in outer_block.
    frame.render_widget(outer_block, area);

    // Iterate through all elements in the `mode_list` and stylize them.
    let items: Vec<ListItem> = app
        .schedule_list
        .lines
        .iter()
        .enumerate()
        .map(|(i, schedule_item)| schedule_item.to_list_item(i))
        .collect();

    // Create a List from all list items and highlight the currently selected one
    let items = List::new(items)
        .block(inner_block)
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .add_modifier(Modifier::REVERSED)
                .fg(SELECTION_STYLE_FG),
        )
        .highlight_symbol(">")
        .highlight_spacing(HighlightSpacing::Always);

    // We can now render the item list
    frame.render_stateful_widget(items, inner_area, &mut app.schedule_list.state);
}

fn render_footer(frame: &mut Frame, app: &mut App, area: Rect) {
    let current_navigation_text = vec![
        // The first half of the text
        match app.current_screen {
            CurrentScreen::Main => Span::styled("Normal Mode", Style::default().fg(Color::Green)),
            CurrentScreen::Adding | CurrentScreen::Selecting => {
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
                        Span::styled("Selecting Experiment", Style::default().fg(Color::Green))
                    }
                    CurrentlyEditing::SchedulingMode => Span::styled(
                        "Selecting Scheduling Mode",
                        Style::default().fg(Color::LightGreen),
                    ),
                    CurrentlyEditing::Kwargs => Span::styled(
                        "Editing Keyword Arguments",
                        Style::default().fg(Color::Green),
                    ),
                    CurrentlyEditing::Done => {
                        Span::styled("Confirm entry", Style::default().fg(Color::Green))
                    }
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
                Style::default().fg(Color::LightGreen),
            ),
            CurrentScreen::Adding => Span::styled(
                "(ESC) to cancel / (Tab) or ↑↓ to switch field / enter to complete",
                Style::default().fg(Color::LightGreen),
            ),
            CurrentScreen::Exiting => Span::styled(
                "(q) to quit / (a) to add a schedule line / (r) to remove a schedule line",
                Style::default().fg(Color::LightGreen),
            ),
            CurrentScreen::Selecting | CurrentScreen::Removing => Span::styled(
                "Use ↓↑ to move, g/G to go top/bottom, enter to select",
                Style::default().fg(Color::LightGreen),
            ),
        }
    };

    let key_notes_footer =
        Paragraph::new(Line::from(current_keys_hint)).block(Block::default().borders(Borders::ALL));

    let footer_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    frame.render_widget(mode_footer, footer_chunks[0]);
    frame.render_widget(key_notes_footer, footer_chunks[1]);
}

fn render_editor(frame: &mut Frame, app: &mut App) {
    let popup_block = Block::default()
        .title("Create a schedule entry")
        .borders(Borders::ALL)
        .style(Style::default().bg(BG_COLOR));

    let area = centered_rect(60, 25, frame.area());

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
    let mut mode_block = if let Some(i) = app.mode_list.state.selected() {
        Paragraph::new(format!("Scheduling Mode: {}", app.mode_list.modes[i].clone()))
    } else { Paragraph::new("Scheduling Mode: ") };

    let mut kwargs_block = Paragraph::new(format!("Kwargs: {}", app.kwarg_input.clone()));
    let mut done_block = Paragraph::new("Enter");

    let active_style = Style::default().bg(Color::LightYellow).fg(Color::Black);

    match app.currently_editing.unwrap_or_else(|| CurrentlyEditing::Year) {
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

    frame.render_widget(popup_block, area);

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

    if let CurrentScreen::Selecting = app.current_screen {
        // We create two blocks, one is for the header (outer) and the other is for list (inner).
        let outer_block = Block::default()
            .borders(Borders::NONE)
            .fg(TEXT_COLOR)
            .bg(SELECTION_HEADER_BG)
            .title("Scheduling Modes")
            .title_alignment(Alignment::Center);
        let inner_block = Block::default()
            .borders(Borders::NONE)
            .fg(TEXT_COLOR)
            .bg(NORMAL_ROW_COLOR);

        // We get the inner area from outer_block. We'll use this area later to render the table.
        let outer_area = popup_chunks[1];
        let inner_area = outer_block.inner(outer_area);

        // We can render the header in outer_area.
        frame.render_widget(outer_block, outer_area);

        // Iterate through all elements in the `mode_list` and stylize them.
        let items: Vec<ListItem> = app
            .mode_list
            .modes
            .iter()
            .map(|mode_item| mode_item.to_list_item())
            .collect();

        // Create a List from all list items and highlight the currently selected one
        let items = List::new(items)
            .block(inner_block)
            .highlight_style(
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .add_modifier(Modifier::REVERSED)
                    .fg(SELECTION_STYLE_FG),
            )
            .highlight_symbol(">")
            .highlight_spacing(HighlightSpacing::Always);

        // We can now render the item list
        frame.render_stateful_widget(items, inner_area, &mut app.mode_list.state);
    }

    if app.last_err.is_some() {
        let error_block = Block::default()
            .title("Error")
            .borders(Borders::ALL)
            .style(Style::default().bg(Color::LightRed).fg(Color::Black));
        let error_text = Paragraph::new(format!("{:?}", app.last_err.clone().unwrap()))
            .block(error_block)
            .wrap(Wrap { trim: true });
        frame.render_widget(error_text, popup_chunks[1]);
    }
}

fn render_exit_screen(frame: &mut Frame) {
    frame.render_widget(Clear, frame.area()); //this clears the entire screen and anything already drawn
    let popup_block = Block::default()
        .title("Confirm")
        .borders(Borders::NONE)
        .style(Style::default().bg(BG_COLOR));

    let exit_text = Text::styled(
        "Would you like to write the new schedule? (y/n)",
        Style::default().fg(Color::LightBlue),
    );
    // the `trim: false` will stop the text from being cut off when over the edge of the block
    let exit_paragraph = Paragraph::new(exit_text)
        .block(popup_block)
        .wrap(Wrap { trim: false });

    let area = centered_rect(60, 25, frame.area());
    frame.render_widget(exit_paragraph, area);
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
