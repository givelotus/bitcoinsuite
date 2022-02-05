use bitcoinsuite_error::{ErrorMeta, ErrorSeverity};
use thiserror::Error;

#[test]
fn test_derive() {
    #[derive(Error, ErrorMeta, Debug)]
    enum TestError {
        #[not_found()]
        #[error("Not found")]
        NotFound,

        #[invalid_user_input()]
        #[error("Invalid user input case")]
        InvalidUserInput,

        #[invalid_client_input(tag1 = "value1")]
        #[error("Invalid client input case")]
        InvalidClientInput,

        #[warning(tag1 = "value1", tag2 = "value2")]
        #[error("Warning case")]
        Warning,

        #[bug(tag1 = "value1", tag2 = "value2")]
        #[error("Bug case")]
        Bug,

        #[critical()]
        #[error("Critical case")]
        Critical,
    }
    {
        let error = TestError::NotFound;
        assert_eq!(error.severity(), ErrorSeverity::NotFound);
        assert_eq!(error.error_code(), "not-found");
        assert_eq!(error.tags().as_ref(), &[]);
    }
    {
        let error = TestError::InvalidUserInput;
        assert_eq!(error.severity(), ErrorSeverity::InvalidUserInput);
        assert_eq!(error.error_code(), "invalid-user-input");
        assert_eq!(error.tags().as_ref(), &[]);
    }
    {
        let error = TestError::InvalidClientInput;
        assert_eq!(error.severity(), ErrorSeverity::InvalidClientInput);
        assert_eq!(error.error_code(), "invalid-client-input");
        assert_eq!(error.tags().as_ref(), &[("tag1".into(), "value1".into())]);
    }
    {
        let error = TestError::Warning;
        assert_eq!(error.severity(), ErrorSeverity::Warning);
        assert_eq!(error.error_code(), "warning");
        assert_eq!(
            error.tags().as_ref(),
            &[
                ("tag1".into(), "value1".into()),
                ("tag2".into(), "value2".into()),
            ]
        );
    }
    {
        let error = TestError::Bug;
        assert_eq!(error.severity(), ErrorSeverity::Bug);
        assert_eq!(error.error_code(), "bug");
        assert_eq!(
            error.tags().as_ref(),
            &[
                ("tag1".into(), "value1".into()),
                ("tag2".into(), "value2".into()),
            ]
        );
    }
    {
        let error = TestError::Critical;
        assert_eq!(error.severity(), ErrorSeverity::Critical);
        assert_eq!(error.error_code(), "critical");
        assert_eq!(error.tags().as_ref(), &[]);
    }
}
