mod app;
mod ui;

use std::error::Error;
use std::io;
use ratatui::backend::{Backend, CrosstermBackend};
use ratatui::crossterm::event::{EnableMouseCapture, DisableMouseCapture, Event, KeyCode, KeyEventKind};
use ratatui::crossterm::{event, execute};
use ratatui::crossterm::terminal::{enable_raw_mode, disable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::Terminal;
use crate::app::{App, CurrentlyEditing, CurrentScreen};
use crate::ui::ui;

fn main() -> Result<(), Box<dyn Error>> {
    // setup terminal
    enable_raw_mode()?;
    let mut stderr = io::stderr();  // This is a special case. Normally using stdout is fine.
    execute!(stderr, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stderr);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let mut app = App::new();
    let res = run_app(&mut terminal, &mut app);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{err:?}");
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> io::Result<bool> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind == event::KeyEventKind::Release {
                // Skip events that are not KeyEventKind::Press
                continue;
            }
            match app.current_screen {
                CurrentScreen::Main => match key.code {
                    KeyCode::Char('a') | KeyCode::Char('r') => {
                        app.current_screen = CurrentScreen::Adding;
                        app.currently_editing = Some(CurrentlyEditing::Year);
                    }
                    KeyCode::Char('q') => {
                        app.current_screen = CurrentScreen::Exiting;
                    }
                    _ => {}
                },
                CurrentScreen::Exiting => match key.code {
                    KeyCode::Char('y') => {
                        return Ok(true);
                    }
                    KeyCode::Char('n') | KeyCode::Char('q') => {
                        return Ok(false);
                    }
                    _ => {}
                },
                CurrentScreen::Adding | CurrentScreen::Removing if key.kind == KeyEventKind::Press => {
                    match key.code {
                        KeyCode::Enter => {
                            if let Some(editing) = &app.currently_editing {
                                match editing {
                                    CurrentlyEditing::Year => {
                                        app.currently_editing = Some(CurrentlyEditing::Month);
                                    }
                                    CurrentlyEditing::Month => {
                                        app.currently_editing = Some(CurrentlyEditing::Day);
                                    }
                                    CurrentlyEditing::Day => {
                                        app.currently_editing = Some(CurrentlyEditing::Hour);
                                    }
                                    CurrentlyEditing::Hour => {
                                        app.currently_editing = Some(CurrentlyEditing::Minute);
                                    }
                                    CurrentlyEditing::Minute => {
                                        app.currently_editing = Some(CurrentlyEditing::Duration);
                                    }
                                    CurrentlyEditing::Duration => {
                                        app.currently_editing = Some(CurrentlyEditing::Priority);
                                    }
                                    CurrentlyEditing::Priority => {
                                        app.currently_editing = Some(CurrentlyEditing::Experiment);
                                    }
                                    CurrentlyEditing::Experiment => {
                                        app.currently_editing = Some(CurrentlyEditing::SchedulingMode);
                                    }
                                    CurrentlyEditing::SchedulingMode => {
                                        app.currently_editing = Some(CurrentlyEditing::Kwargs);
                                    }
                                    CurrentlyEditing::Kwargs => {
                                        app.save_entry();
                                        app.currently_editing = None;
                                        app.current_screen = CurrentScreen::Main;
                                    }
                                }
                            }
                        }
                        KeyCode::Backspace => {
                            if let Some(editing) = &app.currently_editing {
                                match editing {
                                    CurrentlyEditing::Year => {
                                        app.year_input.pop();
                                    }
                                    CurrentlyEditing::Month => {
                                        app.month_input.pop();
                                    }
                                    CurrentlyEditing::Day => {
                                        app.day_input.pop();
                                    }
                                    CurrentlyEditing::Hour => {
                                        app.hour_input.pop();
                                    }
                                    CurrentlyEditing::Minute => {
                                        app.minute_input.pop();
                                    }
                                    CurrentlyEditing::Duration => {
                                        app.duration_input.pop();
                                    }
                                    CurrentlyEditing::Priority => {
                                        app.priority_input.pop();
                                    }
                                    CurrentlyEditing::Experiment => {
                                        app.experiment_input.pop();
                                    }
                                    CurrentlyEditing::SchedulingMode => {
                                        app.mode_input.pop();
                                    }
                                    CurrentlyEditing::Kwargs => {
                                        app.kwarg_input.pop();
                                    }
                                }
                            }
                        }
                        KeyCode::Esc => {
                            app.current_screen = CurrentScreen::Main;
                            app.currently_editing = None;
                        }
                        KeyCode::Tab | KeyCode::Down => {
                            app.forward_toggle();
                        }
                        KeyCode::Up => {
                            app.backward_toggle();
                        }
                        KeyCode::Char(value) => {
                            if let Some(editing) = &app.currently_editing {
                                match editing {
                                    CurrentlyEditing::Year => {
                                        app.year_input.push(value);
                                    }
                                    CurrentlyEditing::Month => {
                                        app.month_input.push(value);
                                    }
                                    CurrentlyEditing::Day => {
                                        app.day_input.push(value);
                                    }
                                    CurrentlyEditing::Hour => {
                                        app.hour_input.push(value);
                                    }
                                    CurrentlyEditing::Minute => {
                                        app.minute_input.push(value);
                                    }
                                    CurrentlyEditing::Duration => {
                                        app.duration_input.push(value);
                                    }
                                    CurrentlyEditing::Priority => {
                                        app.priority_input.push(value);
                                    }
                                    CurrentlyEditing::Experiment => {
                                        app.experiment_input.push(value);
                                    }
                                    CurrentlyEditing::SchedulingMode => {
                                        app.mode_input.push(value);
                                    }
                                    CurrentlyEditing::Kwargs => {
                                        app.kwarg_input.push(value);
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    }

}