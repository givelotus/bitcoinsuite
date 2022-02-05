use bitcoinsuite_error::ErrorMetaFunc;

pub const ERROR_META_FUNCS: &[ErrorMetaFunc] = &[
    &bitcoinsuite_bchd_grpc::error::extract_error_meta,
    &bitcoinsuite_test_utils::error::extract_error_meta,
];
