#![warn(clippy::all, rust_2018_idioms)]

mod app;
pub use app::TemplateApp;

#[allow(non_snake_case)]
mod TimeSheet;
pub use TimeSheet::{TimeSheetEntry, TimeSheetSummary};
