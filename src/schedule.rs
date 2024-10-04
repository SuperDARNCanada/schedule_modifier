use chrono::{DateTime, Duration, NaiveDate, NaiveTime, Utc};
use thiserror::Error;

#[derive(Error, Debug)]
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
    MissingFields
}


#[derive(Default)]
pub struct ScheduleLine {
    pub timestamp: DateTime<Utc>,
    pub duration: Duration,
    pub is_infinite: bool,
    pub priority: u8,
    pub experiment: String,
    pub scheduling_mode: String,
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

        let priority: u8 = fields[3].parse().map_err(|_| ScheduleError::InvalidPriority(fields[3].clone()))?;
        if priority > 20 {
            return Err(ScheduleError::InvalidPriority(format!("{} > 20", fields[3].clone())));
        }

        Ok(ScheduleLine {
            timestamp,
            duration,
            is_infinite,
            priority,
            experiment: fields[4].clone(),
            scheduling_mode: fields[5].clone(),
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
}