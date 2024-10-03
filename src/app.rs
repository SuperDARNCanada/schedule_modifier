use chrono::Utc;
use ratatui::widgets::TableState;

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
}

#[derive(Default)]
pub struct ScheduleLine {
    pub timestamp: chrono::DateTime<Utc>,
    pub duration: chrono::Duration,
    pub is_infinite: bool,
    pub priority: u8,
    pub experiment: String,
    pub scheduling_mode: String,
    pub kwargs: Vec<String>,
}

impl ScheduleLine {
    pub fn format(&self) -> String {
        let mut kwargs_string = String::new();
        for kw in self.kwargs.iter() {
            kwargs_string.push(' ');
            kwargs_string.extend(kw.chars());
        }
        let mut duration_string = String::new();
        if self.is_infinite {
            duration_string.push('-');
        } else {
            duration_string.extend(format!("{}", self.duration.num_minutes()).chars())
        }
        format!("{: <14} {} {} {} {}{}", self.timestamp.format("%Y%m%d %H:%M"), duration_string, self.priority, self.experiment, self.scheduling_mode, kwargs_string)
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
    pub experiment_input: String,
    pub mode_input: String,
    pub kwarg_input: String,
    pub schedule_lines: Vec<ScheduleLine>,
    pub current_screen: CurrentScreen,
    pub currently_editing: Option<CurrentlyEditing>,
    pub edit_state: TableState,
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
            edit_state: TableState::default(),
        }
    }

    pub fn backward_toggle(&mut self) {
        if let Some(editing) = &self.currently_editing {
            match editing {
                CurrentlyEditing::Year => {
                    self.currently_editing = Some(CurrentlyEditing::Kwargs);
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
                    self.currently_editing = Some(CurrentlyEditing::Year);
                }
            }
        }
    }

    pub fn save_entry(&mut self) {
        self.schedule_lines.push(ScheduleLine::default());
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
    }
}