use std::io;
use std::error::Error;
use chrono::{DateTime, Duration, NaiveDate, NaiveTime, Utc};
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::BufRead;
use std::path::Path;
use ratatui::style::Stylize;
use ratatui::text::Line;
use ratatui::widgets::ListItem;
use thiserror::Error;
use crate::ui::{ALT_ROW_COLOR, BG_COLOR, NORMAL_ROW_COLOR, TEXT_COLOR};

#[derive(Error, Debug, Clone)]
pub enum ScheduleError {
    #[error("{0}")]
    InvalidDate(String),

    #[error("{0}")]
    InvalidTime(String),

    #[error("{0}")]
    InvalidDuration(String),

    #[error("{0}")]
    InvalidPriority(String),

    #[error("{0}")]
    InvalidExperiment(String),

    #[error("{0}")]
    InvalidMode(String),

    #[error("{0}")]
    InvalidKwargs(String),

    #[error("Missing fields")]
    MissingFields,
}

#[derive(Debug, Clone, Copy, Default)]
pub enum SchedulingMode {
    #[default]
    Common,
    Discretionary,
    Special,
}
impl Display for SchedulingMode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Common => Ok(f.write_str("common")?),
            Self::Discretionary => Ok(f.write_str("discretionary")?),
            Self::Special => Ok(f.write_str("special")?),
        }
    }
}

#[derive(Default, Debug, Clone)]
pub struct ScheduleLine {
    pub timestamp: DateTime<Utc>,
    pub duration: Duration,
    pub is_infinite: bool,
    pub priority: u8,
    pub experiment: String,
    pub scheduling_mode: SchedulingMode,
    pub kwargs: Vec<String>,
}

pub fn parse_date(date: &String) -> Result<NaiveDate, ScheduleError> {
    NaiveDate::parse_from_str(date.as_str(), "%Y%m%d")
        .map_err(|_| ScheduleError::InvalidDate(date.clone()))
}

pub fn parse_time(time: &String) -> Result<NaiveTime, ScheduleError> {
    NaiveTime::parse_from_str(time.as_str(), "%H:%M")
        .map_err(|_| ScheduleError::InvalidTime(time.clone()))
}

pub fn parse_duration(dur: &String) -> Result<Duration, ScheduleError> {
    Duration::try_minutes(
        dur.parse()
            .map_err(|_| ScheduleError::InvalidDuration(dur.clone()))?,
    )
    .ok_or_else(|| ScheduleError::InvalidDuration(dur.clone()))
}

impl TryFrom<&String> for ScheduleLine {
    type Error = ScheduleError;

    fn try_from(value: &String) -> Result<Self, Self::Error> {
        let fields: Vec<String> = value.split(' ').map(|s| s.to_string()).collect();
        if fields.len() < 6 {
            Err(ScheduleError::MissingFields)?;
        }
        let timestamp = parse_date(&fields[0])?
            .and_time(parse_time(&fields[1])?)
            .and_utc();
        let duration: Duration;
        let mut is_infinite: bool = false;
        if &fields[2] == &"-".to_string() {
            duration = Duration::default();
            is_infinite = true;
        } else {
            duration = parse_duration(&fields[2])?;
        }

        let priority: u8 = fields[3]
            .parse()
            .map_err(|_| ScheduleError::InvalidPriority(fields[3].clone()))?;
        if priority > 20 {
            return Err(ScheduleError::InvalidPriority(format!(
                "{} > 20",
                fields[3].clone()
            )));
        }

        let scheduling_mode = match fields[5].as_str() {
            "common" => SchedulingMode::Common,
            "discretionary" => SchedulingMode::Discretionary,
            "special" => SchedulingMode::Special,
            _ => return Err(ScheduleError::InvalidMode(fields[5].clone())),
        };

        Ok(ScheduleLine {
            timestamp,
            duration,
            is_infinite,
            priority,
            experiment: fields[4].clone(),
            scheduling_mode,
            kwargs: fields[6..].into(),
        })
    }
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
        format!(
            "{: <14} {} {} {} {}{}",
            self.timestamp.format("%Y%m%d %H:%M"),
            duration_string,
            self.priority,
            self.experiment,
            self.scheduling_mode,
            kwargs_string
        )
    }

    pub fn to_list_item(&self, index: usize) -> ListItem {
        let bg_color = match index % 2 {
            0 => NORMAL_ROW_COLOR,
            _ => ALT_ROW_COLOR,
        };
        ListItem::new(Line::styled(self.format(), TEXT_COLOR)).bg(bg_color)
    }

    pub fn load_schedule<P>(filename: P) -> Result<Vec<ScheduleLine>, Box<dyn Error>>
    where P: AsRef<Path> {
        let schedule_file = File::open(filename)?;

        let mut schedule_lines = vec![];
        for line in io::BufReader::new(schedule_file).lines().flatten() {
            schedule_lines.extend([ScheduleLine::try_from(&line)?]);
        }
        schedule_lines.reverse();
        Ok(schedule_lines)
    }
}

impl SchedulingMode {
    pub fn to_list_item(&self) -> ListItem {
        ListItem::new(Line::styled(format!("{self}"), TEXT_COLOR)).bg(BG_COLOR)
    }
}