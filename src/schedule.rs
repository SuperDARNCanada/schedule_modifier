use crate::ui::{ALT_ROW_COLOR, BG_COLOR, NORMAL_ROW_COLOR, TEXT_COLOR};
use chrono::{DateTime, Duration, NaiveDate, NaiveDateTime, NaiveTime, Utc};
use ratatui::style::Stylize;
use ratatui::text::Line;
use ratatui::widgets::ListItem;
use std::fmt::{Display, Formatter, Write};
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
impl SchedulingMode {
    pub fn to_list_item(&self) -> ListItem {
        ListItem::new(Line::styled(format!("{self}"), TEXT_COLOR)).bg(BG_COLOR)
    }
}

/// Duration of a schedule line.
#[derive(Default, Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum ScdDuration {
    #[default]
    Infinite,
    Finite(Duration),
}
impl Display for ScdDuration {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Infinite => Ok(f.write_char('-')?),
            Self::Finite(x) => Ok(f.write_fmt(format_args!("{}", x.num_minutes()))?),
        }
    }
}
impl TryFrom<&String> for ScdDuration {
    type Error = ScheduleError;

    fn try_from(value: &String) -> Result<Self, Self::Error> {
        if value == &"-".to_string() {
            Ok(ScdDuration::Infinite)
        } else {
            let dur = parse_duration(value)?;
            if dur.num_minutes() < 1 {
                return Err(ScheduleError::InvalidDuration(format!("Expected minutes > 0, got {}", value.clone())))
            }
            Ok(ScdDuration::Finite(dur))
        }
    }
}

#[derive(Default, Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct ScheduleLine {
    pub timestamp: DateTime<Utc>,
    pub duration: ScdDuration,
    pub priority: u8,
    pub experiment: String,
    pub scheduling_mode: SchedulingMode,
    pub kwargs: Vec<String>,
}
impl ScheduleLine {
    pub fn new(timestamp: DateTime<Utc>,
               duration: ScdDuration,
               priority: u8,
               experiment: &String,
               scheduling_mode: &SchedulingMode,
               kwargs: Vec<String>) -> Result<ScheduleLine, ScheduleError> {
        if (timestamp < NaiveDateTime::parse_from_str("20000101 00:00", "%Y%m%d %H:%M").unwrap().and_utc()) ||
            (timestamp > NaiveDateTime::parse_from_str("20510101 00:00", "%Y%m%d %H:%M").unwrap().and_utc()) {
            return Err(ScheduleError::InvalidDate(format!("Expected date between years 2000 and 2049, got {}", timestamp)))
        }
        if let ScdDuration::Infinite = duration {
            if priority > 0 {
                return Err(ScheduleError::InvalidPriority("Cannot have priority > 0 for infinite schedule line".to_string()))
            }
        } else if let ScdDuration::Finite(dur) = duration {
            if dur.num_minutes() < 1 {
                return Err(ScheduleError::InvalidDuration(format!("Expected positive duration, got {}", dur)))
            }
        }
        if priority > 20 {
            return Err(ScheduleError::InvalidPriority(format!("{priority} > 20")))
        }

        Ok(ScheduleLine {
            timestamp,
            duration,
            priority,
            experiment: experiment.clone(),
            scheduling_mode: scheduling_mode.clone(),
            kwargs: kwargs.clone(),
        })
    }
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

        ScheduleLine::new(
            timestamp,
            ScdDuration::try_from(&fields[2])?,
            priority,
            &fields[4].clone(),
            &scheduling_mode,
            fields[6..].into(),
        )
    }
}
impl ScheduleLine {
    pub fn format(&self) -> String {
        let mut kwargs_string = String::new();
        for kw in self.kwargs.iter() {
            kwargs_string.push(' ');
            kwargs_string.extend(kw.chars());
        }

        format!(
            "{: <14} {} {} {} {}{}",
            self.timestamp.format("%Y%m%d %H:%M"),
            self.duration,
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

pub fn parse_date(date: &String) -> Result<NaiveDate, ScheduleError> {
    NaiveDate::parse_from_str(date.as_str(), "%Y%m%d")
        .map_err(|_| ScheduleError::InvalidDate(format!("Expected YYYYMMDD, got {}", date.clone())))
}

pub fn parse_time(time: &String) -> Result<NaiveTime, ScheduleError> {
    NaiveTime::parse_from_str(time.as_str(), "%H:%M")
        .map_err(|_| ScheduleError::InvalidTime(format!("Expected HH:MM, got {}", time.clone())))
}

pub fn parse_duration(dur: &String) -> Result<Duration, ScheduleError> {
    Duration::try_minutes(
        dur.parse()
            .map_err(|_| ScheduleError::InvalidDuration(format!("Expected minutes > 0, got {}", dur.clone())))?,
    )
        .ok_or_else(|| ScheduleError::InvalidDuration(format!("Expected minutes > 0, got {}", dur.clone())))
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
        assert_eq!(parse_date(&"20000000".to_string()), Err(ScheduleError::InvalidDate("Expected YYYYMMDD, got 20000000".to_string())))
    }

    #[test]
    fn test_parse_time() -> Result<(), Box<dyn Error>> {
        assert_eq!(parse_time(&"00:00".to_string())?, NaiveTime::parse_from_str("00:00", "%H:%M")?);
        assert_eq!(parse_time(&"24:00".to_string()), Err(ScheduleError::InvalidTime("Expected HH:MM, got 24:00".to_string())));
        Ok(())
    }

    #[test]
    fn test_parse_duration() -> Result<(), Box<dyn Error>> {
        assert_eq!(parse_duration(&"120".to_string())?, Duration::new(7200, 0).unwrap());
        assert_eq!(parse_duration(&"one hundred".to_string()), Err(ScheduleError::InvalidDuration("Expected minutes > 0, got one hundred".to_string())));
        Ok(())
    }

    #[test]
    fn scheduling_mode_formatting() {
        assert_eq!(format!("{}", SchedulingMode::Common), "common".to_string());
        assert_eq!(format!("{}", SchedulingMode::Discretionary), "discretionary".to_string());
        assert_eq!(format!("{}", SchedulingMode::Special), "special".to_string());
    }

    #[test]
    fn schedule_line_from_str() -> Result<(), Box<dyn Error>> {
        let mut line = ScheduleLine::new(
            NaiveDateTime::parse_from_str("20000101 00:00", "%Y%m%d %H:%M")?.and_utc(),
            ScdDuration::Finite(Duration::new(60, 0).unwrap()),
             0,
            &"normalscan".to_string(),
            &SchedulingMode::Common,
            vec![],
        )?;
        assert_eq!(line, ScheduleLine::try_from(&"20000101 00:00 1 0 normalscan common".to_string())?);
        line.priority = 20;
        assert_eq!(line, ScheduleLine::try_from(&"20000101 00:00 1 20 normalscan common".to_string())?);
        line.priority = 0;
        line.duration = ScdDuration::Infinite;
        assert_eq!(line, ScheduleLine::try_from(&"20000101 00:00 - 0 normalscan common".to_string())?);
        line.experiment = "test".to_string();
        assert_eq!(line, ScheduleLine::try_from(&"20000101 00:00 - 0 test common".to_string())?);
        line.scheduling_mode = SchedulingMode::Discretionary;
        assert_eq!(line, ScheduleLine::try_from(&"20000101 00:00 - 0 test discretionary".to_string())?);
        line.kwargs.push("--embargo".to_string());
        assert_eq!(line, ScheduleLine::try_from(&"20000101 00:00 - 0 test discretionary --embargo".to_string())?);
        Ok(())
    }

    #[test]
    fn schedule_line_formatting() -> Result<(), Box<dyn Error>> {
        let mut line = ScheduleLine::new(
            NaiveDateTime::parse_from_str("20000101 00:00", "%Y%m%d %H:%M")?.and_utc(),
            ScdDuration::Infinite,
            0,
            &"normalscan".to_string(),
            &SchedulingMode::Common,
            vec![],
        )?;
        assert_eq!(line.format(), "20000101 00:00 - 0 normalscan common");
        line.duration = ScdDuration::Finite(Duration::new(60, 0).unwrap());
        assert_eq!(line.format(), "20000101 00:00 1 0 normalscan common");
        line.kwargs.push("--embargo".to_string());
        assert_eq!(line.format(), "20000101 00:00 1 0 normalscan common --embargo");
        line.experiment = "full_fov".to_string();
        assert_eq!(line.format(), "20000101 00:00 1 0 full_fov common --embargo");
        line.scheduling_mode = SchedulingMode::Special;
        assert_eq!(line.format(), "20000101 00:00 1 0 full_fov special --embargo");
        line.timestamp = NaiveDateTime::parse_from_str("20241231 23:59", "%Y%m%d %H:%M")?.and_utc();
        assert_eq!(line.format(), "20241231 23:59 1 0 full_fov special --embargo");
        line.duration = ScdDuration::Finite(Duration::new(1440*60, 0).unwrap());
        assert_eq!(line.format(), "20241231 23:59 1440 0 full_fov special --embargo");
        line.priority = 20;
        assert_eq!(line.format(), "20241231 23:59 1440 20 full_fov special --embargo");
        Ok(())
    }

    #[test]
    fn make_schedule_line() {
        assert_eq!(ScheduleLine::try_from(&"20000101 00:00 120 25 normalscan common".to_string()), Err(ScheduleError::InvalidPriority("25 > 20".to_string())));
        assert_eq!(ScheduleLine::try_from(&"20000101 00:00 120 -1 normalscan common".to_string()), Err(ScheduleError::InvalidPriority("-1".to_string())));
        assert_eq!(ScheduleLine::try_from(&"20000101 00:00 -10 20 normalscan common".to_string()), Err(ScheduleError::InvalidDuration("Expected minutes > 0, got -10".to_string())));
        assert_eq!(ScheduleLine::try_from(&"20000101 24:00 120 20 normalscan common".to_string()), Err(ScheduleError::InvalidTime("Expected HH:MM, got 24:00".to_string())));
        assert_eq!(ScheduleLine::try_from(&"20000101 0000 120 20 normalscan common".to_string()), Err(ScheduleError::InvalidTime("Expected HH:MM, got 0000".to_string())));

    }
}