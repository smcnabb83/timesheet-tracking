use crate::TimeSheet::{TimeSheetEntry, TimeSheetSummary};
use chrono::{Date, DateTime, Duration, NaiveDate, Utc};
use egui::Ui;
use egui_extras::DatePickerButton;
use std::collections::HashMap;

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct TemplateApp {
    project_types: Vec<String>,
    time_sheet_entries: Vec<TimeSheetEntry>,
    //This field only exists so we can serialize and deserialize time_sheet_summary_start_date
    time_sheet_start_date: String,
    #[serde(skip)]
    state: State,
}

// Use this to persist anything that we don't want to save between user sessions
struct State {
    selected_project_type: Option<String>,
    new_project_type: String,
    work_start_time: Option<DateTime<Utc>>,
    current_notes: String,
    time_sheet_summary: Option<TimeSheetSummary>,
    time_sheet_summary_start_date: Date<Utc>,
    manual_add_project: String,
    manual_add_date: Date<Utc>,
    manual_add_minutes: String,
    manual_add_notes: String,
    time_sheet_filters: TimeSheetEntryFilters,
}

struct TimeSheetEntryFilters {
    project_type: String,
    start_date: Date<Utc>,
    end_date: Date<Utc>,
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            // Example stuff:
            project_types: vec!["Lunch".to_string(), "Meetings".to_string()],
            time_sheet_entries: Vec::new(),
            time_sheet_start_date: String::new(),
            state: State {
                selected_project_type: None,
                new_project_type: String::new().to_owned(),
                work_start_time: None,
                current_notes: String::new().to_owned(),
                time_sheet_summary: None,
                time_sheet_summary_start_date: chrono::offset::Utc::today(),
                manual_add_date: chrono::offset::Utc::today(),
                manual_add_notes: String::new().to_owned(),
                manual_add_minutes: String::new().to_owned(),
                manual_add_project: String::new().to_owned(),
                time_sheet_filters: TimeSheetEntryFilters {
                    project_type: String::new(),
                    start_date: chrono::offset::Utc::today() - Duration::days(365),
                    end_date: chrono::offset::Utc::today() + Duration::days(365),
                },
            },
        }
    }
}

impl TemplateApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customized the look at feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            let mut stored_state: TemplateApp =
                eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
            let deserialized_end_date =
                match DateTime::parse_from_rfc3339(&stored_state.time_sheet_start_date) {
                    Ok(result) => result.with_timezone(&Utc).date(),
                    Err(..) => chrono::offset::Utc::today(),
                };

            stored_state.state.time_sheet_summary_start_date = deserialized_end_date;
            return stored_state;
        }

        Default::default()
    }
}

impl eframe::App for TemplateApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        self.time_sheet_start_date = self
            .state
            .time_sheet_summary_start_date
            .and_hms(0, 0, 0)
            .to_rfc3339();
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn clear_color(&self, _visuals: &egui::Visuals) -> egui::Rgba {
        egui::Color32::from_rgba_unmultiplied(4, 4, 4, 180).into()
    }

    #[cfg(not(target_arch = "wasm32"))] // no File->Quit on web pages!
    fn on_close_event(&mut self) -> bool {
        // Don't close when we're actively recording time
        self.state.work_start_time.is_none()
    }

    #[cfg(not(target_arch = "wasm32"))] // no File->Quit on web pages!
    fn persist_native_window(&self) -> bool {
        //TODO: See if we can set this up to only persist if we know
        //this is inside a screen area on closing. Problem with this
        //being on is it's possible for a user to close it with the
        //app being offscreen, leading to some confustion the next time
        //the user starts the app

        false
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let Self {
            time_sheet_entries,
            project_types,
            state,
            time_sheet_start_date: _,
        } = self;
        let mut entries_to_delete = Vec::new();
        let mut projects_to_delete = Vec::new();

        ctx.request_repaint_after(std::time::Duration::from_secs_f32(1.0));

        #[cfg(not(target_arch = "wasm32"))] // no File->Quit on web pages!
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:

            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        _frame.close();
                    }
                });
            });
        });

        egui::SidePanel::left("side_panel").show(ctx, |ui| {
            ui.heading("Select a project");
            if state.work_start_time.is_none() {
                egui::ComboBox::from_label("Select Project")
                    .selected_text(match &state.selected_project_type {
                        Some(project_type) => project_type.to_string(),
                        None => "select a project".to_owned(),
                    })
                    .show_ui(ui, |ui| {
                        for project_type in project_types.as_slice() {
                            ui.selectable_value(
                                &mut state.selected_project_type,
                                Some(project_type.to_string()),
                                project_type,
                            );
                        }
                    });

                if state.selected_project_type.is_some()
                    && ui.button("start work on project").clicked()
                {
                    state.work_start_time = Some(chrono::offset::Utc::now());
                }
            } else {
                let duration = match state.work_start_time {
                    Some(dt) => chrono::offset::Utc::now() - dt,
                    None => Duration::seconds(0),
                };

                ui.label(format!("Time elapsed: {}", format_duration(&duration)));
                ui.text_edit_multiline(&mut state.current_notes);
                if ui.button("Finish project work").clicked() {
                    time_sheet_entries.push(TimeSheetEntry {
                        project_type: state.selected_project_type.as_ref().unwrap().to_string(),
                        work_start_datetime: state.work_start_time.unwrap(),
                        work_end_datetime: chrono::offset::Utc::now(),
                        notes: state.current_notes.to_string(),
                    });
                    state.work_start_time = None;
                    state.current_notes = String::new();
                }
            }

            if state.work_start_time.is_none() {
                ui.add_space(20.0);
                ui.separator();
                egui::containers::CollapsingHeader::new("Project Configuration").show(ui, |ui| {
                    egui::Grid::new("project_types_grid").show(ui, |grid_ui| {
                        grid_ui.label("project type");
                        grid_ui.end_row();

                        for (index, prj) in project_types.iter().enumerate() {
                            grid_ui.label(prj.to_string());
                            if grid_ui.button("delete project type").clicked() {
                                projects_to_delete.push(index);
                            }
                            grid_ui.end_row();
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("Project type to add: ");
                        ui.text_edit_singleline(&mut state.new_project_type);
                        if ui.button("Add Project Type").clicked() {
                            project_types.push(state.new_project_type.to_owned());
                            state.new_project_type = "".to_string();
                        }
                    });
                });

                egui::containers::CollapsingHeader::new("Manual Add").show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("project");
                        ui.text_edit_singleline(&mut state.manual_add_project);
                    });

                    ui.horizontal(|ui| {
                        ui.label("date");
                        ui.add(
                            egui_extras::DatePickerButton::new(&mut state.manual_add_date)
                                .id_source("manual_project_date"),
                        );
                        ui.label("minutes");
                        ui.text_edit_singleline(&mut state.manual_add_minutes);
                    });

                    ui.text_edit_multiline(&mut state.manual_add_notes);
                    let minutes = match state.manual_add_minutes.parse::<f32>() {
                        Ok(mins) => mins,
                        _error => 0.0,
                    };
                    if !state.manual_add_project.is_empty()
                        && minutes > 0.0
                        && minutes < (24.0 * 60.0)
                        && ui.button("Add").clicked()
                        && !state.manual_add_project.is_empty()
                        && !state.manual_add_minutes.is_empty()
                    {
                        time_sheet_entries.push(TimeSheetEntry::from_minutes(
                            &state.manual_add_project,
                            minutes,
                            &state.manual_add_notes,
                            &state.manual_add_date,
                        ));
                    }
                });
            }
        });

        if state.work_start_time.is_none() {
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.heading("Timesheet Entries");

                egui::CollapsingHeader::new("Time Sheet Entries").show(ui, |ui| {
                    let filters = &mut state.time_sheet_filters;
                    ui.label("Filters");
                    ui.horizontal(|ui| {
                        ui.label("Project Name");
                        ui.text_edit_singleline(&mut filters.project_type);
                        ui.label("Start Date");
                        ui.add(
                            DatePickerButton::new(&mut filters.start_date)
                                .id_source("filter_start_date"),
                        );
                        ui.label("End Date");
                        ui.add(
                            DatePickerButton::new(&mut filters.end_date)
                                .id_source("filter_end_date"),
                        );
                        if filters.start_date > filters.end_date {
                            std::mem::swap(&mut filters.start_date, &mut filters.end_date);
                        }
                    });
                    egui::ScrollArea::new([false, true]).show(ui, |ui| {
                        show_timesheet_entries_grid(
                            ui,
                            time_sheet_entries,
                            &mut entries_to_delete,
                            &state.time_sheet_filters,
                        );
                    });
                });

                egui::CollapsingHeader::new("Time Sheet Summary").show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.add(
                            // TODO: do we only want one of these to actually do anything, and the
                            // other calculate 2 weeks after the first one?
                            DatePickerButton::new(&mut state.time_sheet_summary_start_date)
                                .id_source("Start_Date"),
                        );
                        ui.label(format!(
                            "through {}",
                            (state.time_sheet_summary_start_date + Duration::days(14)).format("%F")
                        ));
                        if ui.button("Genereate Timesheet Summary").clicked() {
                            let start_date = state.time_sheet_summary_start_date.naive_utc();
                            let end_date = (state.time_sheet_summary_start_date
                                + Duration::days(14))
                            .naive_utc();
                            state.time_sheet_summary = Some(TimeSheetSummary::new(
                                time_sheet_entries,
                                &start_date,
                                &end_date,
                            ));
                        }
                    });
                    show_timesheet_summary_grid(ui, &state.time_sheet_summary);
                });

                egui::warn_if_debug_build(ui);
            });
        }

        //TODO: does this actually work if entries_to_delete has more than 1 item?
        //TODO: Does entries to delete need to be a vec, or can this just be a
        // usize?
        for idx in entries_to_delete.iter() {
            time_sheet_entries.remove(*idx);
        }

        for idx in projects_to_delete.iter() {
            project_types.remove(*idx);
        }
    }
}

fn show_timesheet_summary_grid<'a>(
    ui: &'a mut Ui,
    time_sheet_summary: &Option<TimeSheetSummary>,
) -> &'a mut Ui {
    match time_sheet_summary {
        Some(s) => {
            if s.summary.keys().len() > 0 {
                egui::Grid::new("Time_sheet_summary_grid").show(ui, |ui| {
                    ui.label("project");
                    for date in s.dates.iter() {
                        ui.label(date.format("%m/%d").to_string());
                    }
                    ui.end_row();
                    let mut total_date_times: HashMap<&NaiveDate, Duration> = HashMap::new();
                    for project in s.projects.iter() {
                        ui.label(project);
                        for date in s.dates.iter() {
                            let (hours, notes) = match s.summary.get(date) {
                                Some(date_match) => match date_match.summary.get(project) {
                                    Some(project_match) => (
                                        project_match.hours_worked,
                                        project_match.notes.to_string(),
                                    ),
                                    None => (Duration::zero(), "".to_string()),
                                },
                                None => (Duration::zero(), "".to_string()),
                            };
                            let this_date_duration = match total_date_times.get(date) {
                                Some(date_time) => *date_time,
                                None => Duration::zero(),
                            };
                            let updated_time = this_date_duration + hours;
                            total_date_times.insert(date, updated_time);

                            if !notes.is_empty() {
                                if ui.link(format_duration_hours(&hours)).hovered() {
                                    egui::Window::new(format!("Notes for {}", date))
                                        .fixed_pos(ui.next_widget_position())
                                        .show(ui.ctx(), |ui| {
                                            ui.label(notes.to_owned());
                                        });
                                }
                            } else {
                                ui.label(format_duration_hours(&hours));
                            }
                        }
                        ui.end_row();
                    }
                    ui.separator();
                    for _ in s.dates.iter() {
                        ui.separator();
                    }
                    ui.end_row();
                    ui.label("total");
                    for date in s.dates.iter() {
                        let total_hours = total_date_times.get(&date).unwrap();
                        ui.label(format_duration_hours(total_hours));
                    }
                });
            }
        }
        None => {}
    }
    ui
}

fn show_timesheet_entries_grid<'a>(
    ui: &'a mut Ui,
    time_sheet_entries: &[TimeSheetEntry],
    entries_to_delete: &mut Vec<usize>,
    filters: &TimeSheetEntryFilters,
) -> &'a mut Ui {
    egui::Grid::new("timesheet_entries_grid").show(ui, |ui| {
        ui.label("project");
        ui.label("start date");
        ui.label("end date");
        ui.label("elapsed time");
        ui.label("notes");
        ui.end_row();
        for (index, entry) in time_sheet_entries.iter().enumerate() {
            if !filters.project_type.is_empty()
                && !entry
                    .project_type
                    .to_lowercase()
                    .contains(&filters.project_type.to_lowercase())
            {
                continue;
            }
            if (filters.start_date > entry.work_start_datetime.date())
                || (filters.end_date < entry.work_end_datetime.date())
            {
                continue;
            }

            ui.label(&entry.project_type);
            ui.label(entry.work_start_datetime.format("%F").to_string());
            ui.label(entry.work_end_datetime.format("%F").to_string());
            let diff = entry.work_end_datetime - entry.work_start_datetime;

            ui.label(format_duration(&diff));
            ui.label(&entry.notes);
            if ui.button("delete").clicked() {
                entries_to_delete.push(index);
            }
            ui.end_row();
        }
    });
    ui
}

fn format_duration(span: &chrono::Duration) -> String {
    if span.num_days() > 0 {
        return format!("{}d:{}h", span.num_days(), (span.num_hours() % 24));
    }
    if span.num_hours() > 0 {
        return format!("{}h:{}m", span.num_hours(), (span.num_minutes() % 60));
    }
    if span.num_minutes() > 0 {
        return format!("{}m:{}s", span.num_minutes(), (span.num_seconds() % 60));
    }
    format!("{}s", span.num_seconds())
}

fn format_duration_hours(span: &chrono::Duration) -> String {
    let mut total_hours: f64 = span.num_minutes() as f64;
    total_hours /= 60.0;
    format!("{0:.2}", total_hours)
}
