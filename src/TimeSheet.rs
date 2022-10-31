use chrono::{Date, DateTime, Duration, NaiveDate, Utc};
use std::collections::{HashMap, HashSet};

#[derive(serde::Deserialize, serde::Serialize)]
pub struct TimeSheetEntry {
    pub project_type: String,
    pub work_start_datetime: DateTime<Utc>,
    pub work_end_datetime: DateTime<Utc>,
    pub notes: String,
}

pub struct TimeSheetSummary {
    pub summary: HashMap<NaiveDate, TimesheetDaySummary>,
    pub projects: Vec<String>,
    pub dates: Vec<NaiveDate>,
}

pub struct TimesheetDaySummary {
    pub summary: HashMap<String, ProjectDaySummary>,
}

pub struct ProjectDaySummary {
    pub hours_worked: Duration,
    pub notes: String,
}

impl TimeSheetEntry {
    pub fn from_minutes(
        project_type: &String,
        minutes: f32,
        notes: &String,
        today_date: &Date<Utc>,
    ) -> TimeSheetEntry {
        let work_start_datetime = today_date.and_hms(0, 0, 0);
        let mut work_end_datetime = work_start_datetime;
        if minutes >= 0.0 {
            let minutes_int = minutes.floor() as u32;
            let seconds_int = ((minutes - minutes.floor()) * 60.0).round() as u32;
            work_end_datetime = today_date.and_hms(minutes_int / 60, minutes_int % 60, seconds_int);
        }

        TimeSheetEntry {
            project_type: project_type.to_owned(),
            work_start_datetime,
            work_end_datetime,
            notes: notes.to_owned(),
        }
    }
}

impl TimeSheetSummary {
    pub fn new(
        entries: &Vec<TimeSheetEntry>,
        start_date: &NaiveDate,
        end_date: &NaiveDate,
    ) -> TimeSheetSummary {
        let mut summary: HashMap<NaiveDate, TimesheetDaySummary> = HashMap::new();
        let mut dates = HashSet::new();
        let mut projects = HashSet::new();

        for entry in entries.iter() {
            let date_worked = entry.work_start_datetime.date_naive();
            let project_worked = entry.project_type.to_string();
            let duration_worked = entry.work_end_datetime - entry.work_start_datetime;
            let project_notes = entry.notes.to_string();
            if date_worked < *start_date || date_worked > *end_date {
                continue;
            }
            dates.insert(date_worked);
            projects.insert(project_worked.to_string());

            let timesheet_day_summary = match summary.get_mut(&date_worked) {
                Some(day_summary) => day_summary,
                None => {
                    let ts_day_summary = TimesheetDaySummary {
                        summary: HashMap::new(),
                    };
                    summary.insert(date_worked, ts_day_summary);
                    summary.get_mut(&date_worked).unwrap()
                }
            };

            let mut project_day_summary =
                match timesheet_day_summary.summary.get_mut(&project_worked) {
                    Some(project_summary) => project_summary,
                    None => {
                        let p_day_summary = ProjectDaySummary {
                            hours_worked: Duration::zero(),
                            notes: String::new(),
                        };
                        timesheet_day_summary
                            .summary
                            .insert(project_worked.to_string(), p_day_summary);
                        timesheet_day_summary
                            .summary
                            .get_mut(&project_worked)
                            .unwrap()
                    }
                };

            project_day_summary.hours_worked = project_day_summary.hours_worked + duration_worked;
            if project_notes.len() > 0 {
                project_day_summary.notes =
                    format!("{} \n {}", project_day_summary.notes, project_notes);
            }
        }
        let mut final_dates: Vec<NaiveDate> = dates.into_iter().collect();
        final_dates.sort();

        TimeSheetSummary {
            summary,
            dates: final_dates,
            projects: projects.into_iter().collect(),
        }
    }
}