use crate::ui::{ALT_ROW_COLOR, BG_COLOR, NORMAL_ROW_COLOR, TEXT_COLOR};
use chrono::{DateTime, Duration, NaiveDate, NaiveTime, Utc};
use ratatui::style::Stylize;
use ratatui::text::Line;
use ratatui::widgets::ListItem;
use std::fmt::{Display, Formatter};
use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq)]
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
    InvalidMode(String),

    #[error("Missing fields")]
    MissingFields,
}

#[derive(Debug, Clone, Copy, Default, Ord, PartialOrd, Eq, PartialEq)]
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

#[derive(Default, Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
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
        .map_err(|_| ScheduleError::InvalidDate(format!("Expecting YYYYMMDD, got {}", date.clone())))
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
}

impl SchedulingMode {
    pub fn to_list_item(&self) -> ListItem {
        ListItem::new(Line::styled(format!("{self}"), TEXT_COLOR)).bg(BG_COLOR)
    }
}


#[cfg(test)]
mod tests {
    use std::error::Error;
    use super::*;
    use chrono::{NaiveDate, NaiveDateTime};

    #[test]
    fn test_parse_date() -> Result<(), Box<dyn Error>> {
        assert_eq!(parse_date(&"20000101".to_string())?, NaiveDate::parse_from_str("20000101", "%Y%m%d")?);
        Ok(())
    }
    #[test]
    fn test_parse_bad_date() {
        assert_eq!(parse_date(&"20000000".to_string()), Err(ScheduleError::InvalidDate("Expecting YYYYMMDD, got 20000000".to_string())))
    }

    #[test]
    fn test_parse_time() -> Result<(), Box<dyn Error>> {
        assert_eq!(parse_time(&"00:00".to_string())?, NaiveTime::parse_from_str("00:00", "%H:%M")?);
        Ok(())
    }
    #[test]
    fn test_parse_bad_time() {
        assert_eq!(parse_time(&"24:00".to_string()), Err(ScheduleError::InvalidTime("24:00".to_string())))
    }

    #[test]
    fn test_parse_duration() -> Result<(), Box<dyn Error>> {
        assert_eq!(parse_duration(&"120".to_string())?, Duration::new(7200, 0).unwrap());
        Ok(())
    }
    #[test]
    fn test_parse_bad_duration() {
        assert_eq!(parse_duration(&"one hundred".to_string()), Err(ScheduleError::InvalidDuration("one hundred".to_string())))
    }

    #[test]
    fn scheduling_mode_formatting() {
        assert_eq!(format!("{}", SchedulingMode::Common), "common".to_string());
        assert_eq!(format!("{}", SchedulingMode::Discretionary), "discretionary".to_string());
        assert_eq!(format!("{}", SchedulingMode::Special), "special".to_string());
    }

    #[test]
    fn schedule_line_from_str() -> Result<(), Box<dyn Error>> {
        assert_eq!(ScheduleLine {
            timestamp: NaiveDateTime::parse_from_str("20000101 00:00", "%Y%m%d %H:%M")?.and_utc(),
            duration: Duration::new(60, 0).unwrap(),
            is_infinite: false,
            priority: 0,
            experiment: "normalscan".to_string(),
            scheduling_mode: SchedulingMode::Common,
            kwargs: vec![],
        }, ScheduleLine::try_from(&"20000101 00:00 1 0 normalscan common".to_string())?);
        Ok(())
    }
}