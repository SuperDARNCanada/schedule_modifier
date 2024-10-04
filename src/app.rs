use crate::schedule::*;
use chrono::{DateTime, Duration, NaiveDate, Utc};


pub enum CurrentScreen {
    Main,
    Adding,
    Removing,
    Exiting,
}

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


pub struct App {
    pub year_input: String,
    pub month_input: String,
    pub day_input: String,
    pub hour_input: String,
    pub minute_input: String,
    pub duration_input: String,
    pub priority_input: String,
    pub experiment_input: String,
    pub mode_input: String,
    pub kwarg_input: String,
    pub schedule_lines: Vec<ScheduleLine>,
    pub current_screen: CurrentScreen,
    pub currently_editing: Option<CurrentlyEditing>,
}

impl App {
    pub fn new() -> App {
        App {
            year_input: String::new(),
            month_input: String::new(),
            day_input: String::new(),
            hour_input: String::new(),
            minute_input: String::new(),
            duration_input: String::new(),
            priority_input: String::new(),
            experiment_input: String::new(),
            mode_input: String::new(),
            kwarg_input: String::new(),
            schedule_lines: vec![],
            current_screen: CurrentScreen::Main,
            currently_editing: None,
        }
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
            return Err(ScheduleError::InvalidDate(format!("Bad year: {year}")));
        }
        let month: u8 = self
            .month_input
            .parse()
            .map_err(|_| ScheduleError::InvalidDate(format!("Bad month: {}", self.month_input)))?;
        if month > 12 {
            return Err(ScheduleError::InvalidDate(format!("Bad month: {month}")));
        }
        let day: u8 = self
            .day_input
            .parse()
            .map_err(|_| ScheduleError::InvalidDate(format!("Bad day: {}", self.day_input)))?;
        if day > 31 {
            return Err(ScheduleError::InvalidDate(format!("Bad day: {day}")));
        }
        let hour: u8 = self
            .month_input
            .parse()
            .map_err(|_| ScheduleError::InvalidDate(format!("Bad hour: {}", self.hour_input)))?;
        if hour > 23 {
            return Err(ScheduleError::InvalidTime(format!("Bad hour: {hour}")));
        }
        let minute: u8 = self.month_input.parse().map_err(|_| {
            ScheduleError::InvalidDate(format!("Bad minute: {}", self.minute_input))
        })?;
        if minute > 59 {
            return Err(ScheduleError::InvalidTime(format!("Bad minute: {minute}")));
        }

        let timestamp: DateTime<Utc> =
            NaiveDate::from_ymd_opt(year as i32, month as u32, day as u32)
                .ok_or_else(|| ScheduleError::InvalidDate(format!("{year}{month}{day}")))?
                .and_hms_opt(hour as u32, minute as u32, 0)
                .ok_or_else(|| ScheduleError::InvalidTime(format!("{hour}:{minute}")))?
                .and_utc();

        let duration: Duration = parse_duration(&self.duration_input)?;

        Ok(ScheduleLine {
            timestamp,
            duration,
            ..Default::default()
        })
    }

    pub fn save_entry(&mut self) -> Result<(), ScheduleError> {
        let new_line = self.create_line_from_inputs()?;

        // only clear the inputs if the line was valid
        self.schedule_lines.push(new_line);
        self.year_input = String::new();
        self.month_input = String::new();
        self.day_input = String::new();
        self.hour_input = String::new();
        self.minute_input = String::new();
        self.duration_input = String::new();
        self.priority_input = String::new();
        self.experiment_input = String::new();
        self.mode_input = String::new();
        self.kwarg_input = String::new();

        Ok(())
    }
}
