use bitcoinsuite_error::{ErrorDetails, ErrorFmt, ErrorMeta, ErrorSeverity, Report};

pub fn extract_error_meta(report: &Report) -> Option<&dyn ErrorMeta> {
    if let Some(err) = bitcoinsuite_bchd_grpc::error::extract_error_meta(report) {
        Some(err)
    } else if let Some(err) = bitcoinsuite_test_utils::error::extract_error_meta(report) {
        Some(err)
    } else {
        None
    }
}

pub fn report_to_details(report: &Report) -> ErrorDetails {
    let short_msg = report.to_string();
    let msg = report.fmt_err();
    let full_debug_report = format!("{:?}", report);
    match extract_error_meta(report) {
        Some(meta) => ErrorDetails {
            severity: meta.severity(),
            error_code: meta.error_code(),
            tags: meta.tags(),
            short_msg,
            msg,
            full_debug_report,
        },
        None => ErrorDetails {
            severity: ErrorSeverity::Unknown,
            error_code: "unknown".into(),
            tags: [].as_ref().into(),
            short_msg,
            msg,
            full_debug_report,
        },
    }
}
