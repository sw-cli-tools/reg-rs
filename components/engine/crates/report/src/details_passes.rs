use serde::Serialize;
use tinytemplate::TinyTemplate;

use reg_rs_store::db;

use crate::details::DetailsReportContext;
use crate::format::{pass_symbol, warn};

/// Passes report template
const PASSES_REPORT_TEMPLATE: &str =
    "{ pass_symbol } { passed_test_name } - created: { time_created }, passed: { time_last_ran }";

/// Show test result passes
pub(crate) fn show_passes(
    details_report_context: &DetailsReportContext,
    verbosity_level: u8,
) -> reg_rs_types::error::Result<()> {
    log::info!("details/show_passes");
    let passed_test_names = details_report_context.passed_test_names();
    println!("Passes:");
    if passed_test_names.is_empty() {
        println!("  (none)");
    }
    for test in passed_test_names {
        let db_path = reg_rs_store_rgt::rgt::db_path(test);
        let original_result = db::read_original_results(&db_path)?;
        let latest_result = db::read_latest_results(&db_path)?;
        show_pass_entry(
            test,
            &original_result.time_created,
            &latest_result.time_created,
        )?;
    }
    if verbosity_level > 3 {
        println!(
            "{} verbosity level {} exceeds max",
            warn("*warning*"),
            verbosity_level
        );
    }
    Ok(())
}

/// Show a single pass entry
fn show_pass_entry(
    test: &str,
    time_created: &str,
    time_last_ran: &str,
) -> reg_rs_types::error::Result<()> {
    #[derive(Serialize)]
    struct Ctx {
        pass_symbol: String,
        passed_test_name: String,
        required_blank: String,
        time_created: String,
        time_last_ran: String,
    }
    let ctx = Ctx {
        pass_symbol: pass_symbol(),
        passed_test_name: test.to_string(),
        required_blank: " ".to_string(),
        time_created: time_created.to_string(),
        time_last_ran: time_last_ran.to_string(),
    };
    let mut tt = TinyTemplate::new();
    tt.add_template("p", PASSES_REPORT_TEMPLATE)
        .map_err(|e| reg_rs_types::error::RegError::Template(e.to_string()))?;
    let rendered = tt
        .render("p", &ctx)
        .map_err(|e| reg_rs_types::error::RegError::Template(e.to_string()))?;
    println!("{}", rendered);
    Ok(())
}
