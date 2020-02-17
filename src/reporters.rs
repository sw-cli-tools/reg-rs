use crate::config;

pub mod details;
pub mod differences;
pub mod summary;

pub fn generate_reports(config: &config::Config) -> Result<(), Box<dyn std::error::Error>> {
    summary::show_summary(&config)?;
    details::show_details();
    differences::show_differences();
    Ok(())
}
