//! Common test utilities and fixtures for the job tracker application tests.
//!
//! This module provides shared functionality to reduce code duplication across
//! unit tests, integration tests, and persistence tests.

use job_tracker::db::Database;
use job_tracker::model::{JobApplication, SalaryRange, Status};
use std::fs;
use std::thread;
use std::time::Duration;
use tokio::time::sleep;

/// Test database context that handles setup and cleanup
pub struct TestDb {
    pub db: Database,
    pub path: String,
    pub test_dir: String,
}

impl TestDb {
    /// Creates a new test database with a unique path
    pub async fn new(test_name: &str) -> Self {
        let test_dir = get_unique_test_dir(test_name);
        cleanup_test_files(&test_dir).await;

        let db_path = format!("sqlite:{test_dir}/test.db");
        let db = Database::new(&db_path).await.unwrap();

        Self {
            db,
            path: db_path,
            test_dir,
        }
    }

    /// Creates a new database connection to the same file (for persistence testing)
    pub async fn reconnect(&self) -> Database {
        sleep(Duration::from_millis(50)).await;
        Database::new(&self.path).await.unwrap()
    }

    /// Closes the database and cleans up test files
    pub async fn cleanup(self) {
        drop(self.db);
        cleanup_test_files(&self.test_dir).await;
    }
}

/// Generates a unique test directory name to avoid conflicts between concurrent tests
pub fn get_unique_test_dir(test_name: &str) -> String {
    format!("test_{}_{:?}", test_name, thread::current().id())
}

/// Cleans up test files and directories
pub async fn cleanup_test_files(test_dir: &str) {
    let _ = fs::remove_dir_all(test_dir);
    let _ = fs::remove_file("test_jobs.db");
    sleep(Duration::from_millis(10)).await;
}

/// Creates a basic job application for testing
pub fn create_basic_job() -> JobApplication {
    JobApplication::new()
        .company("Test Corp")
        .position("Software Engineer")
        .location("Remote")
        .salary(SalaryRange::new(80_000, 120_000))
        .status(Status::Applied)
        .date(2024, 1, 15)
}

/// Creates a job application with custom parameters
pub fn create_job_with_params(
    company: &str,
    position: &str,
    location: &str,
    salary_min: u32,
    salary_max: u32,
    status: Status,
) -> JobApplication {
    JobApplication::new()
        .company(company)
        .position(position)
        .location(location)
        .salary(SalaryRange::new(salary_min, salary_max))
        .status(status)
        .date(2024, 1, 15)
}

/// Creates multiple test jobs
pub fn multiple_jobs() -> Vec<JobApplication> {
    vec![
        create_job_with_params(
            "Company A",
            "Developer",
            "Remote",
            60_000,
            80_000,
            Status::Applied,
        ),
        create_job_with_params(
            "Company B",
            "Senior Developer",
            "New York",
            90_000,
            120_000,
            Status::Interview(2),
        ),
        create_job_with_params(
            "Company C",
            "Lead Developer",
            "San Francisco",
            120_000,
            150_000,
            Status::Offer(135_000),
        ),
        create_job_with_params(
            "Company D",
            "Junior Developer",
            "Austin",
            50_000,
            70_000,
            Status::Rejected,
        ),
    ]
}

/// Helper function to assert job equality (ignoring ID)
pub fn assert_job_equals_ignoring_id(actual: &JobApplication, expected: &JobApplication) {
    assert_eq!(actual.company, expected.company);
    assert_eq!(actual.position, expected.position);
    assert_eq!(actual.location, expected.location);
    assert_eq!(actual.salary, expected.salary);
    assert_eq!(actual.status, expected.status);
    assert_eq!(actual.date, expected.date);
    assert_eq!(actual.cv, expected.cv);
}

/// Helper function to verify a job exists in a list
pub fn assert_job_in_list(jobs: &[JobApplication], expected: &JobApplication) {
    let found = jobs.iter().any(|job| {
        job.company == expected.company
            && job.position == expected.position
            && job.location == expected.location
            && job.salary == expected.salary
            && job.status == expected.status
    });
    assert!(found, "Job not found in list: {expected:?}");
}

/// Helper function to create a test database at a specific nested path
pub async fn create_nested_test_db(test_name: &str, nested_path: &str) -> TestDb {
    let test_dir = get_unique_test_dir(test_name);
    cleanup_test_files(&test_dir).await;

    let db_path = format!("sqlite:{test_dir}/{nested_path}");
    let db = Database::new(&db_path).await.unwrap();

    TestDb {
        db,
        path: db_path,
        test_dir,
    }
}

/// Helper function to assert database file exists
pub fn assert_db_file_exists(test_dir: &str, file_path: &str) {
    let full_path = format!("{test_dir}/{file_path}");
    assert!(
        std::path::Path::new(&full_path).exists(),
        "Database file does not exist: {full_path}"
    );
}

/// Async helper to insert multiple jobs and return their IDs
pub async fn insert_multiple_jobs(db: &Database, jobs: &[JobApplication]) -> Vec<i64> {
    let mut ids = Vec::new();
    for job in jobs {
        let id = db.insert_job(job).await.unwrap();
        ids.push(id);
    }
    ids
}

/// Helper to verify all status types persist correctly
pub async fn verify_status_persistence(db: &Database, jobs_with_ids: &[(i64, &JobApplication)]) {
    for (id, expected_job) in jobs_with_ids {
        let retrieved_job = db.get_job_by_id(*id).await.unwrap();
        assert_eq!(retrieved_job.status, expected_job.status);
        assert_eq!(retrieved_job.company, expected_job.company);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unique_test_dir_generation() {
        let dir1 = get_unique_test_dir("test1");
        let dir2 = get_unique_test_dir("test2");

        assert!(dir1.contains("test1"));
        assert!(dir2.contains("test2"));

        assert!(dir1.contains("ThreadId"));
        assert!(dir2.contains("ThreadId"));
    }

    #[test]
    fn test_basic_job_creation() {
        let job = create_basic_job();
        assert_eq!(job.company, "Test Corp");
        assert_eq!(job.position, "Software Engineer");
        assert_eq!(job.location, "Remote");
        assert_eq!(job.salary.min, 80_000);
        assert_eq!(job.salary.max, 120_000);
        assert_eq!(job.status, Status::Applied);
    }

    #[test]
    fn test_job_with_params_creation() {
        let job = create_job_with_params(
            "Custom Corp",
            "Custom Position",
            "Custom Location",
            100_000,
            150_000,
            Status::Interview(3),
        );

        assert_eq!(job.company, "Custom Corp");
        assert_eq!(job.position, "Custom Position");
        assert_eq!(job.location, "Custom Location");
        assert_eq!(job.salary.min, 100_000);
        assert_eq!(job.salary.max, 150_000);
        assert_eq!(job.status, Status::Interview(3));
    }

    #[test]
    fn test_assert_job_equals_ignoring_id() {
        let job1 = create_basic_job();
        let mut job2 = create_basic_job();
        job2.id = Some(42); // Different ID should be ignored

        assert_job_equals_ignoring_id(&job1, &job2);
    }

    #[test]
    fn test_assert_job_in_list() {
        let jobs = vec![
            create_job_with_params("A", "Dev", "Remote", 60_000, 80_000, Status::Applied),
            create_job_with_params("B", "Senior", "NYC", 90_000, 120_000, Status::Interview(2)),
        ];

        let target_job =
            create_job_with_params("A", "Dev", "Remote", 60_000, 80_000, Status::Applied);
        assert_job_in_list(&jobs, &target_job);
    }
}
