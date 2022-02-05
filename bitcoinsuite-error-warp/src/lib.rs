use std::borrow::Cow;

use bitcoinsuite_error::{report_to_details, ErrorMetaFunc, ErrorSeverity, Report};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use warp::{hyper::StatusCode, Rejection};

#[derive(Debug, Error)]
#[error(transparent)]
pub struct WarpError(pub Report);

impl warp::reject::Reject for WarpError {}

pub fn err(err: Report) -> Rejection {
    warp::reject::custom(WarpError(err))
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HttpError {
    error_code: Cow<'static, str>,
    msg: String,
    is_user_error: bool,
}

pub fn rejection_to_reply(err: Rejection, detail_funcs: &[ErrorMetaFunc]) -> impl warp::Reply {
    let (status_code, http_error) = if let Some(WarpError(report)) = err.find::<WarpError>() {
        let details = report_to_details(report, detail_funcs);
        match details.severity {
            ErrorSeverity::NotFound => (
                StatusCode::NOT_FOUND,
                HttpError {
                    error_code: details.error_code,
                    msg: details.msg,
                    is_user_error: true,
                },
            ),
            ErrorSeverity::InvalidUserInput => (
                StatusCode::BAD_REQUEST,
                HttpError {
                    error_code: details.error_code,
                    msg: details.msg,
                    is_user_error: true,
                },
            ),
            ErrorSeverity::InvalidClientInput => {
                println!("Invalid client input: {}", details.msg);
                (
                    StatusCode::BAD_REQUEST,
                    HttpError {
                        error_code: details.error_code,
                        msg: details.msg,
                        is_user_error: false,
                    },
                )
            }
            ErrorSeverity::Critical
            | ErrorSeverity::Unknown
            | ErrorSeverity::Bug
            | ErrorSeverity::Warning => {
                println!("Unhandled error ({:?}):", details.severity);
                println!("{}", details.full_debug_report);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    HttpError {
                        error_code: "internal-server-error".into(),
                        msg: "Internal server error".to_string(),
                        is_user_error: false,
                    },
                )
            }
        }
    } else if let Some(err) = err.find::<warp::reject::InvalidHeader>() {
        (
            StatusCode::BAD_REQUEST,
            HttpError {
                error_code: "http-invalid-header".into(),
                is_user_error: false,
                msg: err.to_string(),
            },
        )
    } else if let Some(err) = err.find::<warp::reject::InvalidQuery>() {
        (
            StatusCode::BAD_REQUEST,
            HttpError {
                error_code: "http-invalid-query".into(),
                is_user_error: false,
                msg: err.to_string(),
            },
        )
    } else if let Some(err) = err.find::<warp::reject::LengthRequired>() {
        (
            StatusCode::LENGTH_REQUIRED,
            HttpError {
                error_code: "http-length-required".into(),
                is_user_error: false,
                msg: err.to_string(),
            },
        )
    } else if let Some(err) = err.find::<warp::reject::MethodNotAllowed>() {
        (
            StatusCode::METHOD_NOT_ALLOWED,
            HttpError {
                error_code: "http-method-not-allowed".into(),
                is_user_error: false,
                msg: err.to_string(),
            },
        )
    } else if let Some(err) = err.find::<warp::reject::MissingCookie>() {
        (
            StatusCode::BAD_REQUEST,
            HttpError {
                error_code: "http-missing-cookie".into(),
                is_user_error: false,
                msg: err.to_string(),
            },
        )
    } else if let Some(err) = err.find::<warp::reject::MissingHeader>() {
        (
            StatusCode::BAD_REQUEST,
            HttpError {
                error_code: "http-missing-header".into(),
                is_user_error: false,
                msg: err.to_string(),
            },
        )
    } else if let Some(err) = err.find::<warp::reject::PayloadTooLarge>() {
        (
            StatusCode::PAYLOAD_TOO_LARGE,
            HttpError {
                error_code: "http-payload-too-large".into(),
                is_user_error: false,
                msg: err.to_string(),
            },
        )
    } else if let Some(err) = err.find::<warp::reject::UnsupportedMediaType>() {
        (
            StatusCode::PAYLOAD_TOO_LARGE,
            HttpError {
                error_code: "http-unsupported-media-type".into(),
                is_user_error: false,
                msg: err.to_string(),
            },
        )
    } else {
        println!("Unknown error: {:?}", err);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            HttpError {
                error_code: "internal-server-error".into(),
                msg: "Unknown error".to_string(),
                is_user_error: false,
            },
        )
    };
    warp::reply::with_status(warp::reply::json(&http_error), status_code)
}
