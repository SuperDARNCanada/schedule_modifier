use crate::schedule::{parse_duration, ScheduleError, ScheduleLine, SchedulingMode};
use chrono::{DateTime, Duration, NaiveDate, Utc};
use ratatui::widgets::ListState;
use std::error::Error;
use std::fs::File;
use std::io;
use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};

pub enum CurrentScreen {
    Main,
    Adding,
    Removing,
    Selecting,
    Exiting,
}

#[derive(Debug, Copy, Clone)]
pub enum CurrentlyEditing {
    Year,
    Month,
    Day,
    Hour,
    Minute,
    Duration,
    Priority,
    Experiment,
    SchedulingMode,
    Kwargs,
    Done,
}

pub struct ExperimentList {
    pub(crate) items: Vec<String>,
    pub state: ListState,
}

impl ExperimentList {
    pub fn next(&mut self) {
        if self.items.len() == 0 {
            self.unselect()
        } else {
            let i = match self.state.selected() {
                Some(i) => {
                    if i >= self.items.len() - 1 {
                        0
                    } else {
                        i + 1
                    }
                }
                None => 0,
            };
            self.state.select(Some(i));
        }
    }

    pub fn previous(&mut self) {
        if self.items.len() == 0 {
            self.unselect()
        } else {
            let i = match self.state.selected() {
                Some(i) => {
                    if i == 0 {
                        self.items.len() - 1
                    } else {
                        i - 1
                    }
                }
                None => 0,
            };
            self.state.select(Some(i));
        }
    }

    pub fn first(&mut self) {
        if self.items.len() == 0 {
            self.unselect()
        } else {
            self.state.select(Some(0));
        }
    }

    pub fn last(&mut self) {
        if self.items.len() == 0 {
            self.unselect()
        } else {
            self.state.select(Some(self.items.len() - 1));
        }
    }

    pub fn unselect(&mut self) {
        let offset = self.state.offset();
        self.state.select(None);
        *self.state.offset_mut() = offset;
    }
}

pub struct ModeList {
    pub(crate) modes: Vec<SchedulingMode>,
    pub state: ListState,
}

impl ModeList {
    pub fn next(&mut self) {
        if self.modes.len() > 0 {
            let i = match self.state.selected() {
                Some(i) => {
                    if i >= self.modes.len() - 1 {
                        0
                    } else {
                        i + 1
                    }
                }
                None => 0,
            };
            self.state.select(Some(i));
        }
    }

    pub fn previous(&mut self) {
        if self.modes.len() > 0 {
            let i = match self.state.selected() {
                Some(i) => {
                    if i == 0 {
                        self.modes.len() - 1
                    } else {
                        i - 1
                    }
                }
                None => 0,
            };
            self.state.select(Some(i));
        }
    }

    pub fn first(&mut self) {
        if self.modes.len() > 0 {
            self.state.select(Some(0));
        }
    }

    pub fn last(&mut self) {
        if self.modes.len() > 0 {
            self.state.select(Some(self.modes.len() - 1));
        }
    }
}

pub struct ScheduleList {
    pub(crate) lines: Vec<ScheduleLine>,
    pub state: ListState,
}

impl ScheduleList {
    pub fn next(&mut self) {
        if self.lines.len() == 0 {
            self.unselect()
        } else {
            let i = match self.state.selected() {
                Some(i) => {
                    if i >= self.lines.len() - 1 {
                        0
                    } else {
                        i + 1
                    }
                }
                None => 0,
            };
            self.state.select(Some(i));
        }
    }

    pub fn previous(&mut self) {
        if self.lines.len() == 0 {
            self.unselect()
        } else {
            let i = match self.state.selected() {
                Some(i) => {
                    if i == 0 {
                        self.lines.len() - 1
                    } else {
                        i - 1
                    }
                }
                None => 0,
            };
            self.state.select(Some(i));
        }
    }

    pub fn first(&mut self) {
        if self.lines.len() == 0 {
            self.unselect()
        } else {
            self.state.select(Some(0));
        }
    }

    pub fn last(&mut self) {
        if self.lines.len() == 0 {
            self.unselect()
        } else {
            self.state.select(Some(self.lines.len() - 1));
        }
    }

    pub fn unselect(&mut self) {
        let offset = self.state.offset();
        self.state.select(None);
        *self.state.offset_mut() = offset;
    }
}

pub struct App {
    pub year_input: String,
    pub month_input: String,
    pub day_input: String,
    pub hour_input: String,
    pub minute_input: String,
    pub duration_input: String,
    pub priority_input: String,
    pub experiment_list: ExperimentList,
    pub mode_list: ModeList,
    pub kwarg_input: String,
    pub schedule_list: ScheduleList,
    pub current_screen: CurrentScreen,
    pub currently_editing: Option<CurrentlyEditing>,
    pub last_err: Option<ScheduleError>,
    pub scd_path: PathBuf,
    pub additions: Vec<ScheduleLine>,
    pub deletions: Vec<ScheduleLine>,
}

impl App {
    pub fn new(scd_path: PathBuf, exp_path: PathBuf) -> App {
        let current_schedule =
            Self::load_schedule(&scd_path).expect("Unable to open schedule file");
        let available_experiments =
            load_experiments(&exp_path).expect("Unable to find Borealis experiments");
        let mut app = App {
            year_input: String::new(),
            month_input: String::new(),
            day_input: String::new(),
            hour_input: String::new(),
            minute_input: String::new(),
            duration_input: String::new(),
            priority_input: String::new(),
            experiment_list: ExperimentList {
                items: available_experiments,
                state: ListState::default(),
            },
            mode_list: ModeList {
                modes: vec![
                    SchedulingMode::Common,
                    SchedulingMode::Discretionary,
                    SchedulingMode::Special,
                ],
                state: ListState::default(),
            },
            kwarg_input: String::new(),
            schedule_list: ScheduleList {
                lines: vec![],
                state: ListState::default(),
            },
            current_screen: CurrentScreen::Main,
            currently_editing: None,
            last_err: None,
            scd_path,
            additions: vec![],
            deletions: vec![],
        };
        app.mode_list.first();
        app.schedule_list.lines = current_schedule;
        app
    }

    pub fn backward_toggle(&mut self) {
        if let Some(editing) = &self.currently_editing {
            match editing {
                CurrentlyEditing::Year => {
                    self.currently_editing = Some(CurrentlyEditing::Done);
                }
                CurrentlyEditing::Month => {
                    self.currently_editing = Some(CurrentlyEditing::Year);
                }
                CurrentlyEditing::Day => {
                    self.currently_editing = Some(CurrentlyEditing::Month);
                }
                CurrentlyEditing::Hour => {
                    self.currently_editing = Some(CurrentlyEditing::Day);
                }
                CurrentlyEditing::Minute => {
                    self.currently_editing = Some(CurrentlyEditing::Hour);
                }
                CurrentlyEditing::Duration => {
                    self.currently_editing = Some(CurrentlyEditing::Minute);
                }
                CurrentlyEditing::Priority => {
                    self.currently_editing = Some(CurrentlyEditing::Duration);
                }
                CurrentlyEditing::Experiment => {
                    self.currently_editing = Some(CurrentlyEditing::Priority);
                }
                CurrentlyEditing::SchedulingMode => {
                    self.currently_editing = Some(CurrentlyEditing::Experiment);
                }
                CurrentlyEditing::Kwargs => {
                    self.currently_editing = Some(CurrentlyEditing::SchedulingMode);
                }
                CurrentlyEditing::Done => {
                    self.currently_editing = Some(CurrentlyEditing::Kwargs);
                }
            }
        }
    }
    pub fn forward_toggle(&mut self) {
        if let Some(editing) = &self.currently_editing {
            match editing {
                CurrentlyEditing::Year => {
                    self.currently_editing = Some(CurrentlyEditing::Month);
                }
                CurrentlyEditing::Month => {
                    self.currently_editing = Some(CurrentlyEditing::Day);
                }
                CurrentlyEditing::Day => {
                    self.currently_editing = Some(CurrentlyEditing::Hour);
                }
                CurrentlyEditing::Hour => {
                    self.currently_editing = Some(CurrentlyEditing::Minute);
                }
                CurrentlyEditing::Minute => {
                    self.currently_editing = Some(CurrentlyEditing::Duration);
                }
                CurrentlyEditing::Duration => {
                    self.currently_editing = Some(CurrentlyEditing::Priority);
                }
                CurrentlyEditing::Priority => {
                    self.currently_editing = Some(CurrentlyEditing::Experiment);
                }
                CurrentlyEditing::Experiment => {
                    self.currently_editing = Some(CurrentlyEditing::SchedulingMode);
                }
                CurrentlyEditing::SchedulingMode => {
                    self.currently_editing = Some(CurrentlyEditing::Kwargs);
                }
                CurrentlyEditing::Kwargs => {
                    self.currently_editing = Some(CurrentlyEditing::Done);
                }
                CurrentlyEditing::Done => {
                    self.currently_editing = Some(CurrentlyEditing::Year);
                }
            }
        }
    }

    fn create_line_from_inputs(&mut self) -> Result<ScheduleLine, ScheduleError> {
        let year: u16 = self
            .year_input
            .parse()
            .map_err(|_| ScheduleError::InvalidDate(format!("Bad year: {}", self.year_input)))?;
        if year < 2000 || year > 2050 {
            return Err(ScheduleError::InvalidDate(format!(
                "Bad year: {year} not in range [2000, 2050]"
            )));
        }
        let month: u8 = self
            .month_input
            .parse()
            .map_err(|_| ScheduleError::InvalidDate(format!("Bad month: {}", self.month_input)))?;
        if month == 0 || month > 12 {
            return Err(ScheduleError::InvalidDate(format!(
                "Bad month: {month} not in range [1, 12]"
            )));
        }
        let day: u8 = self
            .day_input
            .parse()
            .map_err(|_| ScheduleError::InvalidDate(format!("Bad day: {}", self.day_input)))?;
        if day == 0 || day > 31 {
            return Err(ScheduleError::InvalidDate(format!(
                "Bad day: {day} not in range [1, 31]"
            )));
        }
        let hour: u8 = self
            .hour_input
            .parse()
            .map_err(|_| ScheduleError::InvalidTime(format!("Bad hour: {}", self.hour_input)))?;
        if hour > 23 {
            return Err(ScheduleError::InvalidTime(format!(
                "Bad hour: {hour} not in range [0, 23]"
            )));
        }
        let minute: u8 = self.minute_input.parse().map_err(|_| {
            ScheduleError::InvalidTime(format!("Bad minute: {}", self.minute_input))
        })?;
        if minute > 59 {
            return Err(ScheduleError::InvalidTime(format!(
                "Bad minute: {minute} not in range [0, 59]"
            )));
        }

        let timestamp: DateTime<Utc> =
            NaiveDate::from_ymd_opt(year as i32, month as u32, day as u32)
                .ok_or_else(|| ScheduleError::InvalidDate(format!("{year}{month}{day}")))?
                .and_hms_opt(hour as u32, minute as u32, 0)
                .ok_or_else(|| ScheduleError::InvalidTime(format!("{hour}:{minute}")))?
                .and_utc();

        let duration: Duration;
        let mut is_infinite = false;
        if &self.duration_input == &"-".to_string() {
            duration = Duration::default();
            is_infinite = true;
        } else {
            duration = parse_duration(&self.duration_input)?;
        }
        let priority: u8 = self
            .priority_input
            .parse()
            .map_err(|_| ScheduleError::InvalidPriority(self.priority_input.clone()))?;
        if priority > 20 {
            return Err(ScheduleError::InvalidPriority(format!("{priority} > 20")));
        }

        let experiment = if let Some(i) = self.experiment_list.state.selected() {
            self.experiment_list.items[i].clone()
        } else {
            "normalscan".to_string()
        };

        let scheduling_mode = if let Some(i) = self.mode_list.state.selected() {
            self.mode_list.modes[i]
        } else {
            SchedulingMode::default()
        };

        Ok(ScheduleLine {
            timestamp,
            duration,
            is_infinite,
            priority,
            experiment,
            scheduling_mode,
            kwargs: self.kwarg_input.split(' ').map(|s| s.to_string()).collect(),
        })
    }

    pub fn save_entry(&mut self) -> Result<(), ScheduleError> {
        let res = self.create_line_from_inputs();
        match res {
            Err(e) => {
                self.last_err = Some(e.clone());
                return Err(e);
            }
            Ok(new_line) => {
                self.additions.push(new_line.clone());
                self.last_err = None;
                self.schedule_list.lines.push(new_line);
                self.schedule_list.lines.sort();
                self.schedule_list.lines.reverse();
                self.year_input = String::new();
                self.month_input = String::new();
                self.day_input = String::new();
                self.hour_input = String::new();
                self.minute_input = String::new();
                self.duration_input = String::new();
                self.priority_input = String::new();
                self.kwarg_input = String::new();
                return Ok(());
            }
        }
    }

    pub fn remove_schedule_line(&mut self) {
        if let Some(x) = self.schedule_list.state.selected() {
            self.deletions.push(self.schedule_list.lines.remove(x));
        }
        self.schedule_list.unselect();
    }

    pub fn load_schedule<P>(filename: P) -> Result<Vec<ScheduleLine>, Box<dyn Error>>
    where
        P: AsRef<Path>,
    {
        let schedule_file = File::open(filename)?;

        let mut schedule_lines = vec![];
        for line in io::BufReader::new(schedule_file).lines().flatten() {
            schedule_lines.extend([ScheduleLine::try_from(&line)?]);
        }
        schedule_lines.reverse();
        Ok(schedule_lines)
    }

    pub fn save_schedule(&self) -> Result<(), Box<dyn Error>> {
        let mut backup_file = self.scd_path.clone();
        backup_file.set_extension("scd.bak");
        std::fs::copy(&self.scd_path, backup_file)?;

        let mut schedule_file = File::create(&self.scd_path)?;
        for line in self.schedule_list.lines.iter().rev() {
            let mut chars = line.format().into_bytes();
            chars.push(b'\n');
            schedule_file.write_all(&*chars)?;
        }
        Ok(())
    }
}

/// Loads in the names of all experiments (files) in `dir`, ignoring `superdarn_common_fields.py`
fn load_experiments<P>(dir: P) -> Result<Vec<String>, Box<dyn Error>>
where
    P: AsRef<Path>,
{
    let mut files: Vec<String> = vec![];
    let ignored_files = [
        ".git",
        ".gitignore",
        "__init__",
        "superdarn_common_fields",
        "LICENSE",
        "README",
    ];
    for entry in std::fs::read_dir(dir)? {
        if let Ok(x) = entry {
            if !x.metadata().unwrap().is_file() {
                continue;
            }
            let stripped_path = x.path().with_extension("");
            let filename = stripped_path.file_name().unwrap();
            if ignored_files.contains(&filename.to_str().unwrap()) {
                continue;
            }
            files.push(filename.to_str().unwrap().to_string());
        }
    }
    files.sort();

    Ok(files)
}
