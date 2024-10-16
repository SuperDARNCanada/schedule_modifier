mod app;
mod schedule;
mod ui;

use crate::app::{App, CurrentScreen, CurrentlyEditing};
use crate::schedule::ScheduleError;
use crate::ui::ui;
use clap::Parser;
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
use std::path::PathBuf;
use std::{env, io};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct ModifierArgs {
    /// Three-letter site ID of radar to schedule
    #[arg()]
    site_id: String,

    /// Directory containing schedule files (overrides `BOREALIS_SCHEDULES` from environment)
    #[arg()]
    schedule_dir: Option<PathBuf>,

    /// Path to borealis experiments directory (defaults to `$BOREALISPATH/src/borealis_experiments`)
    #[arg()]
    experiments_dir: Option<PathBuf>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = ModifierArgs::parse();
    let mut schedule_path = if let Some(x) = cli.schedule_dir {
        x
    } else {
        PathBuf::from(env::var("LOCAL_SCHEDULE_DIR").expect("LOCAL_SCHEDULE_DIR unset"))
    };
    schedule_path.push(cli.site_id);
    schedule_path.set_extension("scd");

    let experiments_path = if let Some(x) = cli.experiments_dir {
        x
    } else {
        let mut temp = PathBuf::from(env::var("BOREALISPATH").expect("BOREALISPATH unset"));
        temp.push("src");
        temp.push("borealis_experiments");
        temp
    };

    // setup terminal
    enable_raw_mode()?;
    let mut stderr = io::stderr(); // This is a special case. Normally using stdout is fine.
    execute!(stderr, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stderr);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let mut app = App::new(schedule_path, experiments_path);
    let res = run_app(&mut terminal, &mut app);

    if let Ok(true) = res {
        app.save_schedule().unwrap();
    }

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
                    KeyCode::Char('n') => {
                        return Ok(false);
                    }
                    KeyCode::Char('b') => {
                        app.current_screen = CurrentScreen::Main;
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
                        app.schedule_list.next();
                    }
                    KeyCode::Up => {
                        app.schedule_list.previous();
                    }
                    KeyCode::Char('g') => {
                        app.schedule_list.first();
                    }
                    KeyCode::Char('G') => {
                        app.schedule_list.last();
                    }
                    KeyCode::Esc => {
                        app.current_screen = CurrentScreen::Main;
                        app.schedule_list.unselect();
                    }
                    KeyCode::PageDown => {
                        app.schedule_list.state.scroll_down_by(30);
                    }
                    KeyCode::PageUp => {
                        app.schedule_list.state.scroll_up_by(30);
                    }
                    _ => {}
                },
                CurrentScreen::Selecting => match key.code {
                    KeyCode::Char('q') => {
                        app.current_screen = CurrentScreen::Exiting;
                    }
                    KeyCode::Enter | KeyCode::Left => {
                        app.current_screen = CurrentScreen::Adding;
                    }
                    KeyCode::Down | KeyCode::Tab => match app.currently_editing {
                        Some(CurrentlyEditing::Experiment) => app.experiment_list.next(),
                        Some(CurrentlyEditing::SchedulingMode) => app.mode_list.next(),
                        _ => {}
                    },
                    KeyCode::Up => match app.currently_editing {
                        Some(CurrentlyEditing::Experiment) => app.experiment_list.previous(),
                        Some(CurrentlyEditing::SchedulingMode) => app.mode_list.previous(),
                        _ => {}
                    },
                    KeyCode::Char('g') => match app.currently_editing {
                        Some(CurrentlyEditing::Experiment) => app.experiment_list.first(),
                        Some(CurrentlyEditing::SchedulingMode) => app.mode_list.first(),
                        _ => {}
                    },
                    KeyCode::Char('G') => match app.currently_editing {
                        Some(CurrentlyEditing::Experiment) => app.experiment_list.last(),
                        Some(CurrentlyEditing::SchedulingMode) => app.mode_list.last(),
                        _ => {}
                    },
                    KeyCode::Esc => {
                        app.current_screen = CurrentScreen::Main;
                        app.currently_editing = None;
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
                                        ScheduleError::InvalidMode(_) => {
                                            app.currently_editing =
                                                Some(CurrentlyEditing::SchedulingMode)
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
                                CurrentlyEditing::Kwargs => {
                                    app.kwarg_input.pop();
                                }
                                _ => {}
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
                                CurrentlyEditing::Kwargs => {
                                    app.kwarg_input.push(value);
                                }
                                _ => {}
                            }
                        }
                    }
                    KeyCode::Right => {
                        if let Some(editing) = &app.currently_editing {
                            match editing {
                                CurrentlyEditing::Experiment | CurrentlyEditing::SchedulingMode => {
                                    app.current_screen = CurrentScreen::Selecting;
                                    app.last_err = None;
                                }
                                _ => {}
                            }
                        }
                    }
                    KeyCode::Left => {
                        if let CurrentScreen::Selecting = &app.current_screen {
                            app.current_screen = CurrentScreen::Adding;
                        }
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    }
}
