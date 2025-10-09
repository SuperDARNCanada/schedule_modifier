use crate::app::{App, CurrentScreen, CurrentlyEditing};
use ratatui::layout::{Constraint, Direction, Flex, Layout, Rect};
use ratatui::prelude::Alignment;
use ratatui::style::palette::tailwind;
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::symbols;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, HighlightSpacing, List, ListItem, Padding, Paragraph, Tabs, Wrap};
use ratatui::Frame;

pub const BG_COLOR: Color = Color::DarkGray;
pub const NORMAL_ROW_COLOR: Color = Color::DarkGray;
pub const ALT_ROW_COLOR: Color = Color::Indexed(237);
pub const TEXT_COLOR: Color = Color::Indexed(10);
pub const SELECTION_STYLE_FG: Color = Color::LightGreen;
pub const KEY_COLOR: Color = Color::LightYellow;
pub const HINT_COLOR: Color = Color::LightBlue;

pub fn ui(frame: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(3),
        ])
        .split(frame.area());

    render_header(frame, app, chunks[0]);
    if let CurrentScreen::Exiting = app.current_screen {
        render_exit_screen(frame, app, chunks[1]);
    } else {
        render_schedule(frame, app, chunks[1]);
        if app.currently_editing.is_some() {
            render_editor(frame, app);
        }
    }
    render_footer(frame, app, chunks[2]);
}

fn render_header(frame: &mut Frame, app: &mut App, area: Rect) {
    let current_navigation_text = vec![
        // The first half of the text
        match app.current_screen {
            CurrentScreen::Main => Span::styled("Normal Mode", Style::default().fg(Color::Green).bold()),
            CurrentScreen::Adding | CurrentScreen::Selecting => {
                Span::styled("Adding Mode", Style::default().fg(Color::Yellow).bold())
            }
            CurrentScreen::Removing => {
                Span::styled("Removing Mode", Style::default().fg(Color::Yellow).bold())
            }
            CurrentScreen::Exiting => Span::styled("Review and Save", Style::default().fg(Color::Cyan).bold()),
        }
            .to_owned(),
        // A white divider bar to separate the two sections
        Span::styled(" | ", Style::default().fg(Color::White)),
        // The final section of the text, with hints on what the user is editing
        {
            let style = Style::default().fg(Color::Green);
            if let Some(editing) = &app.currently_editing {
                match editing {
                    CurrentlyEditing::Year => Span::styled("Editing Year", style),
                    CurrentlyEditing::Month => Span::styled("Editing Month", style),
                    CurrentlyEditing::Day => Span::styled("Editing Day", style),
                    CurrentlyEditing::Hour => Span::styled("Editing Hour", style),
                    CurrentlyEditing::Minute => Span::styled("Editing Minute", style),
                    CurrentlyEditing::Duration => Span::styled("Editing Duration", style),
                    CurrentlyEditing::Priority => Span::styled("Editing Priority", style),
                    CurrentlyEditing::Experiment => Span::styled("Selecting Experiment", style),
                    CurrentlyEditing::SchedulingMode => {
                        Span::styled("Selecting Scheduling Mode", style)
                    }
                    CurrentlyEditing::Kwargs => Span::styled("Editing Keyword Arguments", style),
                    CurrentlyEditing::Add => Span::styled("Confirm entry", style),
                    CurrentlyEditing::AddAll => Span::styled("Save to all schedules", style),
                    CurrentlyEditing::Exit => Span::styled("Discard changes", style),
                }
            } else {
                Span::styled("Not Editing Anything", Style::default().fg(Color::DarkGray))
            }
        },
    ];

    let header = Paragraph::new(Line::from(current_navigation_text))
        .block(Block::default().borders(Borders::ALL));

    frame.render_widget(header, area);
}

fn render_schedule(frame: &mut Frame, app: &mut App, area: Rect) {
    // Iterate through all elements in the `mode_list` and stylize them.
    if let Some(x) = app.radar_schedules.state.selected() {
        let schedule_block = Block::bordered()
            .border_set(symbols::border::PROPORTIONAL_TALL)
            .padding(Padding::horizontal(1))
            .border_style(tailwind::EMERALD.c700)
            .bg(NORMAL_ROW_COLOR);

        let [tabs_area, schedule_area] = Layout::vertical([Constraint::Length(1), Constraint::Min(0)])
            .areas(area);

        let [tabs_area] = Layout::horizontal([Constraint::Min(1)])
            .flex(Flex::Center)
            .areas(tabs_area);

        render_tabs(frame, app, tabs_area);

        let scd = &mut app.radar_schedules.items[x];
        let items: Vec<ListItem> = scd.lines
            .items
            .iter()
            .enumerate()
            .map(|(i, schedule_item)| schedule_item.to_list_item(i))
            .collect();

        // Create a List from all list items and highlight the currently selected one
        let items = List::new(items)
            .block(schedule_block)
            .highlight_style(
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .add_modifier(Modifier::REVERSED)
                    .fg(SELECTION_STYLE_FG),
            )
            .highlight_symbol(">")
            .highlight_spacing(HighlightSpacing::Always);

        // We can now render the item list
        frame.render_stateful_widget(items, schedule_area, &mut scd.lines.state);
    }
}

fn render_footer(frame: &mut Frame, app: &mut App, area: Rect) {
    let current_keys_hint: Line = {
        match app.current_screen {
            CurrentScreen::Main => vec![
                Span::styled("(q)", Style::default().fg(KEY_COLOR)),
                Span::styled(" to quit / ", Style::default().fg(HINT_COLOR)),
                Span::styled("(a)", Style::default().fg(KEY_COLOR)),
                Span::styled(
                    " to add a schedule line / ",
                    Style::default().fg(HINT_COLOR),
                ),
                Span::styled("(r)", Style::default().fg(KEY_COLOR)),
                Span::styled(
                    " to remove a schedule line / ",
                    Style::default().fg(HINT_COLOR),
                ),
                Span::styled("(Tab) / → ←", Style::default().fg(KEY_COLOR)),
                Span::styled(
                    " to switch to another schedule",
                    Style::default().fg(HINT_COLOR),
                ),
            ]
            .into(),
            CurrentScreen::Exiting => vec![
                Span::styled("(y)", Style::default().fg(KEY_COLOR)),
                Span::styled(" save schedules / ", Style::default().fg(HINT_COLOR)),
                Span::styled("Q", Style::default().fg(KEY_COLOR)),
                Span::styled(
                    "uit without saving / ",
                    Style::default().fg(HINT_COLOR),
                ),
                Span::styled("B", Style::default().fg(KEY_COLOR)),
                Span::styled(
                    "ack to editor / ",
                    Style::default().fg(HINT_COLOR),
                ),
                Span::styled("(Tab)", Style::default().fg(KEY_COLOR)),
                Span::styled(" or ", Style::default().fg(HINT_COLOR)),
                Span::styled("→ ←", Style::default().fg(KEY_COLOR)),
                Span::styled(
                    " switch to another schedule",
                    Style::default().fg(HINT_COLOR),
                ),
            ]
                .into(),
            CurrentScreen::Adding => vec![
                Span::styled("(ESC)", Style::default().fg(KEY_COLOR)),
                Span::styled(" to cancel / ", Style::default().fg(HINT_COLOR)),
                Span::styled("(Tab)", Style::default().fg(KEY_COLOR)),
                Span::styled(" or ", Style::default().fg(HINT_COLOR)),
                Span::styled("↑↓", Style::default().fg(KEY_COLOR)),
                Span::styled(" or ", Style::default().fg(HINT_COLOR)),
                Span::styled("End", Style::default().fg(KEY_COLOR)),
                Span::styled(" to switch field / ", Style::default().fg(HINT_COLOR)),
                Span::styled("→ ←", Style::default().fg(KEY_COLOR)),
                Span::styled(
                    " to start/finish selecting / ",
                    Style::default().fg(HINT_COLOR),
                ),
                Span::styled("Enter", Style::default().fg(KEY_COLOR)),
                Span::styled(" to complete", Style::default().fg(HINT_COLOR)),
            ]
            .into(),
            CurrentScreen::Removing => vec![
                Span::styled("(ESC)", Style::default().fg(KEY_COLOR)),
                Span::styled(" to cancel / ", Style::default().fg(HINT_COLOR)),
                Span::styled("(Tab) / → ←", Style::default().fg(KEY_COLOR)),
                Span::styled(
                    " to switch to another schedule / ",
                    Style::default().fg(HINT_COLOR),
                ),
                Span::styled("↑↓", Style::default().fg(KEY_COLOR)),
                Span::styled(" or ", Style::default().fg(HINT_COLOR)),
                Span::styled("PgUp", Style::default().fg(KEY_COLOR)),
                Span::styled("/", Style::default().fg(HINT_COLOR)),
                Span::styled("PgDn", Style::default().fg(KEY_COLOR)),
                Span::styled(" or ", Style::default().fg(HINT_COLOR)),
                Span::styled("g", Style::default().fg(KEY_COLOR)),
                Span::styled("/", Style::default().fg(HINT_COLOR)),
                Span::styled("G", Style::default().fg(KEY_COLOR)),
                Span::styled(" to switch field / ", Style::default().fg(HINT_COLOR)),
                Span::styled("Enter", Style::default().fg(KEY_COLOR)),
                Span::styled(" to select", Style::default().fg(HINT_COLOR)),
            ]
            .into(),
            CurrentScreen::Selecting => vec![
                Span::styled("(ESC)", Style::default().fg(KEY_COLOR)),
                Span::styled(" to cancel / ", Style::default().fg(HINT_COLOR)),
                Span::styled("↑↓", Style::default().fg(KEY_COLOR)),
                Span::styled(" or ", Style::default().fg(HINT_COLOR)),
                Span::styled("g", Style::default().fg(KEY_COLOR)),
                Span::styled("/", Style::default().fg(HINT_COLOR)),
                Span::styled("G", Style::default().fg(KEY_COLOR)),
                Span::styled(" to switch field / ", Style::default().fg(HINT_COLOR)),
                Span::styled("Enter", Style::default().fg(KEY_COLOR)),
                Span::styled(" to select", Style::default().fg(HINT_COLOR)),
            ]
            .into(),
        }
    };

    let key_notes_footer =
        Paragraph::new(current_keys_hint).block(Block::default().borders(Borders::ALL));

    frame.render_widget(key_notes_footer, area);
}

fn render_editor(frame: &mut Frame, app: &mut App) {
    let popup_block = Block::default()
        .title("Create a schedule entry")
        .borders(Borders::ALL)
        .border_style(Style::default().bg(tailwind::INDIGO.c700))
        .style(Style::default().bg(tailwind::INDIGO.c700));

    let area = centered_rect(60, 30, frame.area());
    frame.render_widget(Clear, area);

    let popup_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .margin(1)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
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
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(popup_chunks[0]);
    
    let [add_chunk, add_all_chunk, exit_chunk] = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![Constraint::Percentage(25), Constraint::Percentage(50), Constraint::Percentage(25)])
        .areas(line_chunks[10]);

    let mut year_block = Paragraph::new(format!("Year: {}", app.year_input.clone()));
    let mut month_block = Paragraph::new(format!("Month: {}", app.month_input.clone()));
    let mut day_block = Paragraph::new(format!("Day: {}", app.day_input.clone()));
    let mut hour_block = Paragraph::new(format!("Hour: {}", app.hour_input.clone()));
    let mut minute_block = Paragraph::new(format!("Minute: {}", app.minute_input.clone()));
    let mut duration_block = Paragraph::new(format!("Duration: {}", app.duration_input.clone()));
    let mut priority_block = Paragraph::new(format!("Priority: {}", app.priority_input.clone()));
    let mut experiment_block = if let Some(i) = app.experiment_list.state.selected() {
        Paragraph::new(
            Line::from(
            vec![
                Span::styled("Experiment: ", Style::default()),
                Span::styled(app.experiment_list.items[i].clone(), Style::default().bold()),
                ]
            )
        )
    } else {
        Paragraph::new("Experiment: ")
    };
    let mut mode_block = if let Some(i) = app.mode_list.state.selected() {
        Paragraph::new(
            Line::from(
                vec![
                    Span::styled("Scheduling Mode: ", Style::default()),
                    Span::styled(format!("{}", app.mode_list.items[i]), Style::default().bold()),
                ]
            )
        )
    } else {
        Paragraph::new("Scheduling Mode: ")
    };

    let mut kwargs_block = Paragraph::new(format!("Kwargs: {}", app.kwarg_input.clone())).wrap(Wrap { trim: true });
    let mut add_block = Paragraph::new("[Add]");
    let mut add_all_block = Paragraph::new("[Add To All]");
    let mut exit_block = Paragraph::new("[Exit]");

    let active_style = Style::default().bg(tailwind::SLATE.c200).fg(tailwind::INDIGO.c700);

    match app
        .currently_editing
        .unwrap_or_else(|| CurrentlyEditing::Year)
    {
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
        CurrentlyEditing::Add => add_block = add_block.style(active_style),
        CurrentlyEditing::AddAll => add_all_block = add_all_block.style(active_style),
        CurrentlyEditing::Exit => exit_block = exit_block.style(active_style),
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
    frame.render_widget(add_block, add_chunk);
    frame.render_widget(add_all_block, add_all_chunk);
    frame.render_widget(exit_block, exit_chunk);

    let title = match app.currently_editing {
        Some(CurrentlyEditing::Experiment) => "Possible Experiments",
        Some(CurrentlyEditing::SchedulingMode) => "Scheduling Modes",
        _ => {
            if app.last_err.is_some() {
                "Error"
            } else {
                "Restrictions"
            }
        }
    };

    // We create two blocks, one is for the header (outer) and the other is for list (inner).
    let outer_block = Block::default()
        .borders(Borders::ALL)
        .fg(tailwind::SLATE.c200)
        .bg(tailwind::INDIGO.c700)
        .title(title)
        .title_alignment(Alignment::Center);
    let inner_block = Block::default()
        .borders(Borders::NONE)
        .fg(tailwind::SLATE.c200)
        .bg(tailwind::INDIGO.c700);

    // We get the inner area from outer_block. We'll use this area later to render the table.
    let outer_area = popup_chunks[1];
    let inner_area = outer_block.inner(outer_area);

    // We can render the header in outer_area.
    frame.render_widget(outer_block, outer_area);

    let paragraph: Paragraph;
    if app.last_err.is_some() {
        paragraph = Paragraph::new(format!("{:?}", app.last_err.clone().unwrap()))
            .style(Style::default().bg(Color::Indexed(196)).fg(Color::White))
            .block(inner_block)
            .wrap(Wrap { trim: true });
        frame.render_widget(paragraph, inner_area);
    } else if let Some(editing) = app.currently_editing {
        match editing {
            CurrentlyEditing::Experiment => {
                // Iterate through all elements in the `mode_list` and stylize them.
                let items: Vec<ListItem> = app
                    .experiment_list
                    .items
                    .iter()
                    .map(|item| ListItem::new(Line::styled(item, tailwind::SLATE.c200)).bg(tailwind::INDIGO.c700))
                    .collect();
                // Create a List from all list items and highlight the currently selected one
                let items = List::new(items)
                    .block(inner_block)
                    .highlight_style(
                        Style::default()
                            .add_modifier(Modifier::BOLD)
                            .add_modifier(Modifier::REVERSED)
                    )
                    .highlight_symbol(">")
                    .highlight_spacing(HighlightSpacing::Always);
                frame.render_stateful_widget(items, inner_area, &mut app.experiment_list.state);
            }
            CurrentlyEditing::SchedulingMode => {
                let items: Vec<ListItem> = app
                    .mode_list
                    .items
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
                    )
                    .highlight_symbol(">")
                    .highlight_spacing(HighlightSpacing::Always);
                frame.render_stateful_widget(items, inner_area, &mut app.mode_list.state);
            }
            _ => {
                let text = match editing {
                    CurrentlyEditing::Year => "2000 <= year <= 2050",
                    CurrentlyEditing::Month => "1 <= month <= 12",
                    CurrentlyEditing::Day => "1 <= day <= 31",
                    CurrentlyEditing::Hour => "0 <= hour <= 23",
                    CurrentlyEditing::Minute => "0 <= minute <= 59",
                    CurrentlyEditing::Duration => {
                        "in minutes, > 0\n1 day = 1440 min\n3 days = 4320 min\n5 days = 7200 min"
                    }
                    CurrentlyEditing::Priority => {
                        "0 <= priority <= 20\n0 is lowest priority\n20 is highest priority\n"
                    }
                    _ => "",
                };
                paragraph = Paragraph::new(text)
                    .style(Style::default().fg(tailwind::SLATE.c200).bg(tailwind::INDIGO.c700))
                    .block(inner_block)
                    .wrap(Wrap { trim: true })
                    .alignment(Alignment::Center);
                frame.render_widget(paragraph, inner_area);
            }
        }
    }
}

fn render_tabs(frame: &mut Frame, app: &mut App, area: Rect) {
    if let Some(x) = app.radar_schedules.state.selected() {
        let tabs = Tabs::new(
            app.radar_schedules
                .items
                .iter()
                .map(|scd| {
                    format!("  {}  ", scd.path
                        .file_name()
                        .expect("Schedule path lacks file name")
                        .to_str()
                        .expect("schedule filename is not unicode")
                    )
                        .fg(tailwind::SLATE.c200)
                        .bg(tailwind::INDIGO.c900)
                })
        )
            .style(Style::default())
            .highlight_style((Color::default(), tailwind::EMERALD.c700))
            .select(x)
            .divider(" ")
            .padding("", "");

        // We can render the header in outer_block.
        frame.render_widget(tabs, area);
    }
}
fn render_exit_screen(frame: &mut Frame, app: &mut App, area: Rect) {
    // Iterate through all elements in the `mode_list` and stylize them.
    if app.radar_schedules.state.selected().is_some() {
        let schedule_block = Block::bordered()
            .border_set(symbols::border::PROPORTIONAL_TALL)
            .padding(Padding::horizontal(1))
            .border_style(tailwind::EMERALD.c700)
            .bg(NORMAL_ROW_COLOR);

        let [tabs_area, schedule_area] = Layout::vertical([Constraint::Length(1), Constraint::Min(0)])
            .areas(area);

        let [tabs_area] = Layout::horizontal([Constraint::Min(1)])
            .flex(Flex::Center)
            .areas(tabs_area);

        render_tabs(frame, app, tabs_area);

        let schedule_area_inner = schedule_block.inner(schedule_area);
        frame.render_widget(schedule_block, schedule_area);
        let diff_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .margin(1)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(schedule_area_inner);
        let add_block = Block::default()
            .title("Additions")
            .borders(Borders::ALL)
            .style(Style::default().bg(BG_COLOR).fg(Color::LightGreen));
        let delete_block = Block::default()
            .title("Deletions")
            .borders(Borders::ALL)
            .style(Style::default().bg(BG_COLOR).fg(Color::LightRed));

        let mut add_text = String::new();
        let mut del_text = String::new();
        let scd = match app.radar_schedules.state.selected() {
            Some(x) => &mut app.radar_schedules.items[x],
            _ => panic!("Could not render exit screen without schedule selected")
        };
        scd.additions.sort();
        scd.deletions.sort();
        for line in scd.additions.iter().rev() {
            add_text.extend(line.format().chars());
            add_text.push('\n');
        }
        for line in scd.deletions.iter().rev() {
            del_text.extend(line.format().chars());
            del_text.push('\n');
        }
        let add_widget = Paragraph::new(add_text).block(add_block);
        let del_widget = Paragraph::new(del_text).block(delete_block);
        frame.render_widget(add_widget, diff_chunks[0]);
        frame.render_widget(del_widget, diff_chunks[1]);
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
