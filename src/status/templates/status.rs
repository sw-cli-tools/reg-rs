/// Status templates module
pub mod status;
pub mod views;

pub use views::status::{StatusCounts, StatusFlags, StatusViewContext};

/// Status view template constant
const STATUS_VIEW_TEMPLATE: &str = include_str!("status_view.html");

/// Render status view template
pub fn render(status_view_context: &StatusViewContext) -> crate::error::Result<String> {
    log::info!("status/render");
    let mut tt = tinytemplate::TinyTemplate::new();
    tt.add_template("status_view_template", STATUS_VIEW_TEMPLATE)?;
    let rendered = tt.render("status_view_template", status_view_context)?;
    Ok(rendered)
}
