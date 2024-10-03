

pub enum CurrentScreen {
    Main,
    Editing,
    Exiting,
}

pub enum CurrentlyEditing {
    Date,
    Time,
    Duration,
    Priority,
    Experiment,
    SchedulingMode,
    Kwargs,
}

pub struct ScheduleLine {
    date: u32,
    time: String,
    duration: String,
    priority: u8,
    experiment: String,
    scheduling_mode: String,
    kwargs: Vec<String>,
}

pub struct App {
    pub key_input: String,
    pub value_input: String,
    pub schedule_lines: Vec<ScheduleLine>,
    pub current_screen: CurrentScreen,
    pub currently_editing: Option<CurrentlyEditing>,
}

impl App {
    pub fn new() -> App {
        App {
            key_input: String::new(),
            value_input: String::new(),
            schedule_lines: vec![],
            current_screen: CurrentScreen::Main,
            currently_editing: None,
        }
    }
}