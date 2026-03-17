/// Template rendering module
pub mod categorize;
pub mod landing;
pub mod views;

pub use categorize::categorize_runs;
pub use landing::{landing_page, status_page};
pub use views::{StatusCounts, StatusFlags, StatusViewContext, render};
