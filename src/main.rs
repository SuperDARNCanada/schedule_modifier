mod app;
mod schedule;
mod ui;

use crate::app::{App, CurrentScreen, CurrentlyEditing};
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
    /// Directory containing schedule files (overrides `LOCAL_SCHEDULE_DIR` from environment)
    #[arg()]
    schedule_dir: Option<PathBuf>,

    /// Path to borealis experiments directory (defaults to `$BOREALISPATH/src/borealis_experiments`)
    #[arg()]
    experiments_dir: Option<PathBuf>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = ModifierArgs::parse();
    let schedule_path = if let Some(x) = cli.schedule_dir {
        x
    } else {
        PathBuf::from(env::var("LOCAL_SCHEDULE_DIR").expect("LOCAL_SCHEDULE_DIR unset"))
    };
    let schedule_paths = schedule_path.read_dir()?
        .filter_map(|res| res.ok())
        .filter_map(|dir_entry| {
            if dir_entry.path().extension().map_or(false, |ext| ext == "scd") {
                Some(dir_entry.path())
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    
    let experiments_path = if let Some(x) = cli.experiments_dir {
        x
    } else {
        let mut temp = PathBuf::from(env::var("BOREALISPATH").expect("BOREALISPATH unset"));
        temp.push("src");
        temp.push("borealis_experiments");
        temp
    };

    // setup terminal
    init_panic_hook();
    let mut tui = init_tui()?;

    // create app and run it
    let mut app = App::new(schedule_paths, experiments_path);
    let res = run_app(&mut tui, &mut app);

    // restore terminal
    restore_tui()?;

    if let Err(err) = res {
        println!("{err:?}");
    }

    Ok(())
}

pub fn init_panic_hook() {
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        // intentionally ignore errors here since we're already in a panic
        let _ = restore_tui();
        original_hook(panic_info);
    }));
}

pub fn init_tui() -> io::Result<Terminal<impl Backend>> {
    enable_raw_mode()?;
    let mut stderr = io::stderr(); // This is a special case. Normally using stdout is fine.
    execute!(stderr, EnterAlternateScreen, EnableMouseCapture)?;
    Terminal::new(CrosstermBackend::new(stderr))
}

pub fn restore_tui() -> io::Result<()> {
    disable_raw_mode()?;
    execute!(
        io::stdout(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> io::Result<()> {
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
                    KeyCode::Tab | KeyCode::Right => {
                        app.radar_schedules.next();
                    }
                    KeyCode::Left => {
                        app.radar_schedules.previous();
                    }
                    KeyCode::Char('q') => {
                        app.start_exit_procedures();
                    }
                    _ => {}
                },
                CurrentScreen::Exiting => match key.code {
                    KeyCode::Char('Y') | KeyCode::Char('y') => {
                        for i in 0..app.radar_schedules.items.len() {
                            app.radar_schedules.state.select(Some(i));
                            app.save_schedule().unwrap();
                        }
                        return Ok(())
                    }
                    KeyCode::Char('q') | KeyCode::Char('Q') => return Ok(()),
                    KeyCode::Tab | KeyCode::Right => {
                        app.radar_schedules.next();
                    }
                    KeyCode::Left => {
                        app.radar_schedules.previous();
                    }
                    KeyCode::Char('B') | KeyCode::Char('b') => {
                        app.current_screen = CurrentScreen::Main;
                    }
                    _ => {}
                },
                CurrentScreen::Removing => match key.code {
                    KeyCode::Char('q') => {
                        app.start_exit_procedures();
                    }
                    KeyCode::Enter => {
                        app.current_screen = CurrentScreen::Main;
                        app.remove_schedule_line();
                    }
                    KeyCode::Down => {
                        if let Some(x) = app.radar_schedules.state.selected() {
                            let scd = &mut app.radar_schedules.items[x];
                            scd.lines.next();
                        }
                    }
                    KeyCode::Tab | KeyCode::Right => {
                        app.radar_schedules.next();
                    }
                    KeyCode::Left => app.radar_schedules.previous(),
                    KeyCode::Up => {
                        if let Some(x) = app.radar_schedules.state.selected() {
                            let scd = &mut app.radar_schedules.items[x];
                            scd.lines.previous();
                        }
                    }
                    KeyCode::Char('g') => {
                        if let Some(x) = app.radar_schedules.state.selected() {
                            let scd = &mut app.radar_schedules.items[x];
                            scd.lines.first();
                        }
                    }
                    KeyCode::Char('G') => {
                        if let Some(x) = app.radar_schedules.state.selected() {
                            let scd = &mut app.radar_schedules.items[x];
                            scd.lines.last();
                        }
                    }
                    KeyCode::Esc => {
                        app.current_screen = CurrentScreen::Main;
                        if let Some(x) = app.radar_schedules.state.selected() {
                            let scd = &mut app.radar_schedules.items[x];
                            scd.lines.unselect();
                        }
                    }
                    KeyCode::PageDown => {
                        if let Some(x) = app.radar_schedules.state.selected() {
                            let scd = &mut app.radar_schedules.items[x];
                            scd.lines.state.scroll_down_by(30);
                        }
                    }
                    KeyCode::PageUp => {
                        if let Some(x) = app.radar_schedules.state.selected() {
                            let scd = &mut app.radar_schedules.items[x];
                            scd.lines.state.scroll_up_by(30);
                        }
                    }
                    _ => {}
                },
                CurrentScreen::Selecting => match key.code {
                    KeyCode::Char('q') => {
                        app.start_exit_procedures();
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
                                CurrentlyEditing::Add => {
                                    app.parse_scd_input().inspect(|x| app.save_entry(&x));
                                },
                                CurrentlyEditing::AddAll => {
                                    if let Some(new_line) = app.parse_scd_input() {
                                        let idx = app.radar_schedules.state.selected();
                                        for i in 0..app.radar_schedules.items.len() {
                                            app.radar_schedules.state.select(Some(i));
                                            app.save_entry(&new_line);
                                        }
                                        app.radar_schedules.state.select(idx);
                                    }
                                }
                                CurrentlyEditing::Exit => {
                                    app.current_screen = CurrentScreen::Main;
                                    app.currently_editing = None;
                                },
                                _ => {
                                    app.forward_toggle();
                                }
                            }
                        }
                    }
                    KeyCode::End => {
                        if let Some(_) = &app.currently_editing {
                            app.currently_editing = Some(CurrentlyEditing::Add);
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
                    KeyCode::Tab => {
                        app.forward_toggle();
                    }
                    KeyCode::Down => {
                        match app.currently_editing {
                            Some(CurrentlyEditing::Add) | Some(CurrentlyEditing::AddAll) | Some(CurrentlyEditing::Exit) => {},
                            _ => app.forward_toggle(),
                        }
                    }
                    KeyCode::Up => {
                        match app.currently_editing {
                            Some(CurrentlyEditing::Add) | Some(CurrentlyEditing::AddAll) | Some(CurrentlyEditing::Exit) => {
                                app.currently_editing = Some(CurrentlyEditing::Kwargs);
                            },
                            _ => app.backward_toggle(),
                        }
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
                                CurrentlyEditing::Add | CurrentlyEditing::AddAll => {app.forward_toggle()}
                                _ => {}
                            }
                        }
                    }
                    KeyCode::Left => {
                        match &app.current_screen {
                            CurrentScreen::Selecting => {
                                app.current_screen = CurrentScreen::Adding;
                            },
                            CurrentScreen::Adding => {
                                if let Some(editing) = &app.currently_editing {
                                    match editing {
                                        CurrentlyEditing::AddAll | CurrentlyEditing::Exit => {app.backward_toggle()}
                                        _ => {}
                                    }
                                }
                            },
                            _ => {}
                        }
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    }
}
