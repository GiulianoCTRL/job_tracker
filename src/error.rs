use crate::db::DbError;
use std::fmt;

/// Comprehensive error types for the job tracker application.
///
/// This enum represents all possible errors that can occur within the
/// application, providing a unified error handling approach.
#[derive(Debug)]
pub enum AppError {
    /// Database-related errors from `SQLite` operations.
    Database(DbError),
    /// Input validation errors with descriptive messages.
    Validation(String),
    /// File system operation errors.
    FileSystem(std::io::Error),
    /// Configuration-related errors.
    Configuration(String),
    /// User interface errors.
    UserInterface(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Database(err) => write!(f, "Database error: {err}"),
            Self::Validation(msg) => write!(f, "Validation error: {msg}"),
            Self::FileSystem(err) => write!(f, "File system error: {err}"),
            Self::Configuration(msg) => write!(f, "Configuration error: {msg}"),
            Self::UserInterface(msg) => write!(f, "UI error: {msg}"),
        }
    }
}

impl std::error::Error for AppError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Database(err) => Some(err),
            Self::FileSystem(err) => Some(err),
            _ => None,
        }
    }
}

impl From<DbError> for AppError {
    /// Converts a database error into an application error.
    ///
    /// # Examples
    ///
    /// ```
    /// # use job_tracker::error::{AppError};
    /// # use job_tracker::db::DbError;
    /// let db_err = DbError::NotFound(1);
    /// let app_err: AppError = db_err.into();
    /// ```
    fn from(err: DbError) -> Self {
        Self::Database(err)
    }
}

impl From<std::io::Error> for AppError {
    /// Converts an I/O error into an application error.
    ///
    /// # Examples
    ///
    /// ```
    /// # use job_tracker::error::AppError;
    /// # use std::io;
    /// let io_err = io::Error::new(io::ErrorKind::NotFound, "File not found");
    /// let app_err: AppError = io_err.into();
    /// ```
    fn from(err: std::io::Error) -> Self {
        Self::FileSystem(err)
    }
}

impl From<sqlx::Error> for AppError {
    /// Converts a `SQLx` error into an application error.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use job_tracker::error::AppError;
    /// # use sqlx;
    /// // This would happen automatically in error propagation
    /// // let sqlx_err = sqlx::Error::...;
    /// // let app_err: AppError = sqlx_err.into();
    /// ```
    fn from(err: sqlx::Error) -> Self {
        Self::Database(DbError::Connection(err))
    }
}

/// Result type alias for the job tracker application.
///
/// This type alias simplifies function signatures by providing a
/// default `Err` type of `AppError`.
///
/// # Examples
///
/// ```
/// # use job_tracker::error::AppResult;
/// fn might_fail() -> AppResult<String> {
///     Ok("Success".to_string())
/// }
/// ```
pub type AppResult<T> = Result<T, AppError>;

/// Validation error builder for input validation.
///
/// Represents a validation error for a specific field with a descriptive message.
pub struct ValidationError {
    field: String,
    message: String,
}

impl ValidationError {
    /// Creates a new validation error for a specific field.
    ///
    /// # Arguments
    ///
    /// * `field` - The name of the field that failed validation
    /// * `message` - The validation error message
    ///
    /// # Examples
    ///
    /// ```
    /// # use job_tracker::error::ValidationError;
    /// let error = ValidationError::new("email", "Invalid email format");
    /// ```
    #[must_use]
    pub fn new(field: &str, message: &str) -> Self {
        Self {
            field: field.to_string(),
            message: message.to_string(),
        }
    }

    /// Converts the validation error to an `AppError`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use job_tracker::error::{ValidationError, AppError};
    /// let validation_error = ValidationError::new("email", "Invalid format");
    /// let app_error = validation_error.into_app_error();
    /// ```
    #[must_use]
    pub fn into_app_error(self) -> AppError {
        AppError::Validation(format!("{}: {}", self.field, self.message))
    }
}

/// Trait for validating input data.
///
/// Types implementing this trait can be validated, returning a list
/// of validation errors if any exist.
pub trait Validate {
    /// Validates the implementing type and returns validation errors if any.
    ///
    /// # Returns
    ///
    /// A vector of `ValidationError` instances. An empty vector indicates
    /// that validation passed successfully.
    ///
    /// # Examples
    ///
    /// ```
    /// # use job_tracker::error::{Validate, ValidationError};
    /// struct Email(String);
    ///
    /// impl Validate for Email {
    ///     fn validate(&self) -> Vec<ValidationError> {
    ///         let mut errors = Vec::new();
    ///         if !self.0.contains('@') {
    ///             errors.push(ValidationError::new("email", "Must contain @"));
    ///         }
    ///         errors
    ///     }
    /// }
    /// ```
    fn validate(&self) -> Vec<ValidationError>;

    /// Checks if the implementing type is valid.
    ///
    /// This is a convenience method that returns `true` if `validate()`
    /// returns an empty vector.
    ///
    /// # Examples
    ///
    /// ```
    /// # use job_tracker::error::{Validate, ValidationError};
    /// # struct Email(String);
    /// # impl Validate for Email {
    /// #     fn validate(&self) -> Vec<ValidationError> { Vec::new() }
    /// # }
    /// let email = Email("test@example.com".to_string());
    /// assert!(email.is_valid());
    /// ```
    fn is_valid(&self) -> bool {
        self.validate().is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn test_app_error_display() {
        let db_error = DbError::NotFound(1);
        let app_error = AppError::Database(db_error);
        assert!(app_error.to_string().contains("Database error"));

        let validation_error = AppError::Validation("Invalid input".to_string());
        assert!(validation_error.to_string().contains("Validation error"));

        let fs_error = AppError::FileSystem(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "File not found",
        ));
        assert!(fs_error.to_string().contains("File system error"));

        let config_error = AppError::Configuration("Missing config".to_string());
        assert!(config_error.to_string().contains("Configuration error"));

        let ui_error = AppError::UserInterface("UI component failed".to_string());
        assert!(ui_error.to_string().contains("UI error"));
    }

    #[test]
    fn test_error_conversion() {
        let db_error = DbError::NotFound(1);
        let app_error: AppError = db_error.into();
        assert!(matches!(app_error, AppError::Database(_)));

        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
        let app_error: AppError = io_error.into();
        assert!(matches!(app_error, AppError::FileSystem(_)));
    }

    #[test]
    fn test_validation_error() {
        let validation_error = ValidationError::new("email", "Invalid email format");
        assert_eq!(validation_error.field, "email");
        assert_eq!(validation_error.message, "Invalid email format");

        let app_error = validation_error.into_app_error();
        assert!(
            app_error
                .to_string()
                .contains("email: Invalid email format")
        );
    }

    #[test]
    fn test_error_source() {
        let db_error = DbError::NotFound(1);
        let app_error = AppError::Database(db_error);
        assert!(app_error.source().is_some());

        let validation_error = AppError::Validation("test".to_string());
        assert!(validation_error.source().is_none());
    }

    struct TestStruct {
        value: i32,
    }

    impl Validate for TestStruct {
        fn validate(&self) -> Vec<ValidationError> {
            let mut errors = Vec::new();
            if self.value < 0 {
                errors.push(ValidationError::new("value", "Must be non-negative"));
            }
            errors
        }
    }

    #[test]
    fn test_validate_trait() {
        let valid_struct = TestStruct { value: 5 };
        assert!(valid_struct.is_valid());
        assert!(valid_struct.validate().is_empty());

        let invalid_struct = TestStruct { value: -1 };
        assert!(!invalid_struct.is_valid());
        assert_eq!(invalid_struct.validate().len(), 1);
    }
}
