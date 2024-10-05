mod app;
mod schedule;
mod ui;

use crate::app::{App, CurrentScreen, CurrentlyEditing};
use crate::schedule::ScheduleError;
use crate::ui::ui;
use ratatui::backend::{Backend, CrosstermBackend};
use ratatui::crossterm::event::{
    DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind,
};
use ratatui::crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::crossterm::{event, execute};
use ratatui::Terminal;
use std::error::Error;
use std::io;

fn main() -> Result<(), Box<dyn Error>> {
    // setup terminal
    enable_raw_mode()?;
    let mut stderr = io::stderr(); // This is a special case. Normally using stdout is fine.
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
            if key.kind == KeyEventKind::Release {
                // Skip events that are not KeyEventKind::Press
                continue;
            }
            match app.current_screen {
                CurrentScreen::Main => match key.code {
                    KeyCode::Char('a') => {
                        app.current_screen = CurrentScreen::Adding;
                        app.currently_editing = Some(CurrentlyEditing::Year);
                    }
                    KeyCode::Char('r') => {
                        app.current_screen = CurrentScreen::Removing;
                        app.currently_editing = None;
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
                CurrentScreen::Removing => match key.code {
                    KeyCode::Char('q') => {
                        app.current_screen = CurrentScreen::Exiting;
                    }
                    KeyCode::Enter => {
                        app.current_screen = CurrentScreen::Main;
                        app.remove_schedule_line();
                    }
                    KeyCode::Down | KeyCode::Tab => {
                        app.schedule_list.state.select_next();
                    }
                    KeyCode::Up => {
                        app.schedule_list.state.select_previous();
                    }
                    KeyCode::Char('g') => {
                        app.schedule_list.state.select_first();
                    }
                    KeyCode::Char('G') => {
                        app.schedule_list.state.select_last();
                    }
                    _ => {}
                },
                CurrentScreen::Adding if key.kind == KeyEventKind::Press => match key.code {
                    KeyCode::Enter => {
                        if let Some(editing) = &app.currently_editing {
                            match editing {
                                CurrentlyEditing::Done => match app.save_entry() {
                                    Ok(_) => {
                                        app.currently_editing = None;
                                        app.current_screen = CurrentScreen::Main;
                                    }
                                    Err(e) => match e {
                                        ScheduleError::InvalidDate(s) if s.contains("day") => {
                                            app.currently_editing = Some(CurrentlyEditing::Day)
                                        }
                                        ScheduleError::InvalidDate(s) if s.contains("month") => {
                                            app.currently_editing = Some(CurrentlyEditing::Month)
                                        }
                                        ScheduleError::InvalidDate(_) => {
                                            app.currently_editing = Some(CurrentlyEditing::Year)
                                        }
                                        ScheduleError::InvalidTime(s) if s.contains("minute") => {
                                            app.currently_editing = Some(CurrentlyEditing::Minute)
                                        }
                                        ScheduleError::InvalidTime(_) => {
                                            app.currently_editing = Some(CurrentlyEditing::Hour)
                                        }
                                        ScheduleError::InvalidDuration(_) => {
                                            app.currently_editing = Some(CurrentlyEditing::Duration)
                                        }
                                        ScheduleError::InvalidPriority(_) => {
                                            app.currently_editing = Some(CurrentlyEditing::Priority)
                                        }
                                        ScheduleError::InvalidExperiment(_) => {
                                            app.currently_editing =
                                                Some(CurrentlyEditing::Experiment)
                                        }
                                        ScheduleError::InvalidMode(_) => {
                                            app.currently_editing =
                                                Some(CurrentlyEditing::SchedulingMode)
                                        }
                                        ScheduleError::InvalidKwargs(_) => {
                                            app.currently_editing = Some(CurrentlyEditing::Kwargs)
                                        }
                                        _ => {}
                                    },
                                },
                                _ => {
                                    app.forward_toggle();
                                }
                            }
                        }
                    }
                    KeyCode::End => {
                        if let Some(_) = &app.currently_editing {
                            app.currently_editing = Some(CurrentlyEditing::Done);
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
                                CurrentlyEditing::SchedulingMode => {}
                                CurrentlyEditing::Kwargs => {
                                    app.kwarg_input.pop();
                                }
                                CurrentlyEditing::Done => {}
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
                                CurrentlyEditing::SchedulingMode => {}
                                CurrentlyEditing::Kwargs => {
                                    app.kwarg_input.push(value);
                                }
                                CurrentlyEditing::Done => {}
                            }
                        }
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    }
}
