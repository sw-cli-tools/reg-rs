use serde::Serialize;
use tinytemplate::TinyTemplate;

use reg_rs_store::db;
use reg_rs_store::db_ops;
use reg_rs_types::types::RegressionType;

use crate::details::DetailsReportContext;
use crate::format::fail_symbol;

/// Failures report template
const FAILURES_REPORT_TEMPLATE: &str =
    "{ fail_symbol } { failed_test_name } - created: { time_created }, failed: { time_last_ran }, differences count: { differences_count }
{{- for difference in difference_types }}
{{- if @first }}, difference types:{{ endif -}}{ required_blank }{ difference }
{{- if not @last }},{{ endif -}}{{ endfor -}}
";

/// Differences report template
const DIFFERENCES_REPORT_TEMPLATE: &str = "
 ** Differences ** (-vvv)
{{ for difference in differences }}  { difference.type_name } - { difference.chunk }
{{ endfor }}";

/// Describe a difference for display
#[derive(Debug, Serialize)]
struct DisplayDifference {
    /// Difference type
    type_name: String,
    /// Difference data
    chunk: String,
}

/// Data for a differences report template
#[derive(Debug, Serialize)]
struct DifferencesReportContext {
    /// List of differences
    differences: Vec<DisplayDifference>,
    /// Failed test name
    failed_test_name: String,
}

/// Show test result failures
pub(crate) fn show_failures(
    details_report_context: &DetailsReportContext,
    verbosity_level: u8,
) -> reg_rs_types::error::Result<()> {
    log::info!("details/show_failures");
    let failed_test_names = details_report_context.failed_test_names();
    println!("Failures: (-vv)");
    if failed_test_names.is_empty() {
        println!("  (none)");
    }
    for test in failed_test_names {
        let db_path = reg_rs_store_rgt::rgt::db_path(test);
        let original_result = db::read_original_results(&db_path)?;
        let latest_result = db::read_latest_results(&db_path)?;
        let diffs = db_ops::read_differences(&db_path)?;
        let difference_types = if verbosity_level > 2 {
            collect_difference_types(&db_path)?
        } else {
            vec![]
        };
        let same_count =
            db_ops::difference_count_by_type(&db_path, RegressionType::StderrSame as u8)?
                + db_ops::difference_count_by_type(&db_path, RegressionType::StdoutSame as u8)?;
        let differences_count = diffs.len() as u32 - same_count;
        show_failure_entry(
            &difference_types,
            differences_count,
            test,
            &original_result.time_created,
            &latest_result.time_created,
        )?;
        show_doc_metadata(&db_path)?;
        if verbosity_level > 2 {
            show_verbose_differences(test, &diffs)?;
        }
    }
    Ok(())
}

/// Show a single failure entry
fn show_failure_entry(
    difference_types: &[String],
    differences_count: u32,
    test: &str,
    time_created: &str,
    time_last_ran: &str,
) -> reg_rs_types::error::Result<()> {
    #[derive(Serialize)]
    struct Ctx {
        difference_types: Vec<String>,
        differences_count: u32,
        fail_symbol: String,
        failed_test_name: String,
        required_blank: String,
        time_created: String,
        time_last_ran: String,
    }
    let ctx = Ctx {
        difference_types: difference_types.to_vec(),
        differences_count,
        fail_symbol: fail_symbol(),
        failed_test_name: test.to_string(),
        required_blank: " ".to_string(),
        time_created: time_created.to_string(),
        time_last_ran: time_last_ran.to_string(),
    };
    let mut tt = TinyTemplate::new();
    tt.add_template("f", FAILURES_REPORT_TEMPLATE)
        .map_err(|e| reg_rs_types::error::RegError::Template(e.to_string()))?;
    let rendered = tt
        .render("f", &ctx)
        .map_err(|e| reg_rs_types::error::RegError::Template(e.to_string()))?;
    println!("{rendered}");
    Ok(())
}

/// Collect difference type labels for a test database
fn collect_difference_types(db_path: &str) -> reg_rs_types::error::Result<Vec<String>> {
    let mut types = vec![];
    if 0 < db_ops::difference_count_by_type(db_path, RegressionType::ActualCode as u8)? {
        types.push("exit_code".to_string());
    }
    if 0 < db_ops::difference_count_by_type(db_path, RegressionType::StderrAdd as u8)?
        || 0 < db_ops::difference_count_by_type(db_path, RegressionType::StderrRemove as u8)?
    {
        types.push("stderr".to_string());
    }
    if 0 < db_ops::difference_count_by_type(db_path, RegressionType::StdoutAdd as u8)?
        || 0 < db_ops::difference_count_by_type(db_path, RegressionType::StdoutRemove as u8)?
    {
        types.push("stdout".to_string());
    }
    Ok(types)
}

/// Show verbose difference details for a single failed test
fn show_verbose_differences(
    test: &str,
    diffs: &[(String, String)],
) -> reg_rs_types::error::Result<()> {
    let display_differences: Vec<_> = diffs
        .iter()
        .filter_map(|difference| {
            RegressionType::display_label(&difference.0).map(|label| DisplayDifference {
                type_name: format!("{label:022}"),
                chunk: difference.1.to_string(),
            })
        })
        .collect();
    let ctx = DifferencesReportContext {
        differences: display_differences,
        failed_test_name: test.to_string(),
    };
    log::info!("differences/show_differences");
    let mut tt = TinyTemplate::new();
    tt.add_template("differences_report_template", DIFFERENCES_REPORT_TEMPLATE)
        .map_err(|e| reg_rs_types::error::RegError::Template(e.to_string()))?;
    let rendered = tt
        .render("differences_report_template", &ctx)
        .map_err(|e| reg_rs_types::error::RegError::Template(e.to_string()))?;
    println!("{rendered}");
    Ok(())
}

/// Show documentation metadata for a test
fn show_doc_metadata(test: &str) -> reg_rs_types::error::Result<()> {
    let desc = db_ops::read_metadata(test, "desc")?;
    let expects = db_ops::read_metadata(test, "expects")?;
    let flaky_note = db_ops::read_metadata(test, "flaky_note")?;
    if desc.is_some() || expects.is_some() || flaky_note.is_some() {
        if let Some(d) = desc {
            println!("    desc:       {d}");
        }
        if let Some(e) = expects {
            println!("    expects:    {e}");
        }
        if let Some(f) = flaky_note {
            println!("    flaky_note: {f}");
        }
    }
    Ok(())
}
