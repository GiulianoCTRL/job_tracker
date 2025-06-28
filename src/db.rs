use crate::model::{JobApplication, SalaryRange, Status};
use sqlx::{
    Row,
    sqlite::{SqliteConnectOptions, SqlitePool},
};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use time::Date;

#[derive(Debug, thiserror::Error)]
pub enum DbError {
    #[error("Database connection error: {0}")]
    Connection(#[from] sqlx::Error),
    #[error("Invalid status format: {0}")]
    InvalidStatus(String),
    #[error("Job application not found with id: {0}")]
    NotFound(i64),
}

#[derive(Debug, Clone)]
pub struct Database {
    pool: SqlitePool,
}

impl Database {
    /// Creates a new database connection and initializes the schema.
    ///
    /// This function will create the database file if it doesn't exist and
    /// set up the required schema for job applications.
    ///
    /// # Arguments
    ///
    /// * `database_url` - The `SQLite` database URL (e.g., `sqlite:jobs.db` or `sqlite::memory:`)
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The database connection cannot be established
    /// - The parent directory cannot be created
    /// - The database schema creation fails
    /// - The database URL is malformed
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use job_tracker::db::Database;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let db = Database::new("sqlite:jobs.db").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn new(database_url: &str) -> Result<Self, DbError> {
        Self::ensure_database_directory(database_url)?;

        let connection_options =
            SqliteConnectOptions::from_str(database_url)?.create_if_missing(true);

        let pool = SqlitePool::connect_with(connection_options).await?;
        let db = Self { pool };
        db.create_schema().await?;
        Ok(db)
    }

    fn ensure_database_directory(database_url: &str) -> Result<(), DbError> {
        if database_url == "sqlite::memory:" {
            return Ok(());
        }

        if let Some(file_path) = database_url.strip_prefix("sqlite:") {
            let path = Path::new(file_path);

            if let Some(parent) = path.parent()
                && !parent.as_os_str().is_empty()
                && !parent.exists()
            {
                std::fs::create_dir_all(parent)
                    .map_err(|e| DbError::Connection(sqlx::Error::Io(e)))?;
            }
        }

        Ok(())
    }

    /// Creates the database schema for job applications.
    ///
    /// This function creates the `job_applications` table with all required
    /// columns if it doesn't already exist.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The SQL execution fails
    /// - The database connection is lost
    async fn create_schema(&self) -> Result<(), DbError> {
        sqlx::query(
            r"
            CREATE TABLE IF NOT EXISTS job_applications (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                date TEXT,
                cv_path TEXT,
                company TEXT NOT NULL,
                position TEXT NOT NULL,
                status TEXT NOT NULL,
                location TEXT NOT NULL,
                salary_min INTEGER NOT NULL DEFAULT 0,
                salary_max INTEGER NOT NULL DEFAULT 0,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )
            ",
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Inserts a new job application into the database.
    ///
    /// # Arguments
    ///
    /// * `job` - The job application to insert
    ///
    /// # Returns
    ///
    /// The ID of the newly inserted job application.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The database connection is lost
    /// - The SQL execution fails
    /// - Required fields are missing or invalid
    /// - Database constraints are violated
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use job_tracker::db::Database;
    /// # use job_tracker::model::{JobApplication, SalaryRange, Status};
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let db = Database::new("sqlite::memory:").await?;
    /// let job = JobApplication::new()
    ///     .company("TechCorp")
    ///     .position("Developer")
    ///     .location("Remote");
    /// let id = db.insert_job(&job).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn insert_job(&self, job: &JobApplication) -> Result<i64, DbError> {
        let date_str = job.date.map(|d| d.to_string());
        let cv_path_str = job.cv.as_ref().map(|p| p.to_string_lossy().to_string());
        let status_str = job.status.to_db_string();

        let result = sqlx::query(
            r"
            INSERT INTO job_applications (date, cv_path, company, position, status, location, salary_min, salary_max)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            ",
        )
        .bind(date_str)
        .bind(cv_path_str)
        .bind(&job.company)
        .bind(&job.position)
        .bind(status_str)
        .bind(&job.location)
        .bind(i64::from(job.salary.min))
        .bind(i64::from(job.salary.max))
        .execute(&self.pool)
        .await?;

        Ok(result.last_insert_rowid())
    }

    /// Retrieves all job applications from the database.
    ///
    /// Returns job applications ordered by creation date (most recent first).
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The database connection is lost
    /// - The SQL query fails
    /// - A stored status string cannot be parsed
    /// - A stored date cannot be parsed
    /// - Database corruption prevents reading
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use job_tracker::db::Database;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let db = Database::new("sqlite::memory:").await?;
    /// let jobs = db.get_all_jobs().await?;
    /// println!("Found {} jobs", jobs.len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_all_jobs(&self) -> Result<Vec<JobApplication>, DbError> {
        let rows = sqlx::query("SELECT * FROM job_applications ORDER BY created_at DESC")
            .fetch_all(&self.pool)
            .await?;

        let mut jobs = Vec::new();
        for row in rows {
            let job = Self::row_to_job_application(&row)?;
            jobs.push(job);
        }

        Ok(jobs)
    }

    /// Retrieves a specific job application by ID.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the job application to retrieve
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - No job application exists with the given ID (`DbError::NotFound`)
    /// - The database connection is lost
    /// - The SQL query fails
    /// - A stored status string cannot be parsed
    /// - A stored date cannot be parsed
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use job_tracker::db::Database;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let db = Database::new("sqlite::memory:").await?;
    /// match db.get_job_by_id(1).await {
    ///     Ok(job) => println!("Found job: {}", job.company),
    ///     Err(e) => println!("Job not found: {}", e),
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_job_by_id(&self, id: i64) -> Result<JobApplication, DbError> {
        let row = sqlx::query("SELECT * FROM job_applications WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;

        row.map_or_else(
            || Err(DbError::NotFound(id)),
            |r| Self::row_to_job_application(&r),
        )
    }

    /// Updates an existing job application in the database.
    ///
    /// # Arguments
    ///
    /// * `job` - The job application to update (must have a valid ID)
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The job application doesn't have an ID (`DbError::NotFound`)
    /// - No job application exists with the given ID (`DbError::NotFound`)
    /// - The database connection is lost
    /// - The SQL execution fails
    /// - Database constraints are violated
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use job_tracker::db::Database;
    /// # use job_tracker::model::Status;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let db = Database::new("sqlite::memory:").await?;
    /// let mut job = db.get_job_by_id(1).await?;
    /// job.status = Status::Interview(2);
    /// db.update_job(&job).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn update_job(&self, job: &JobApplication) -> Result<(), DbError> {
        let id = job.id.ok_or(DbError::NotFound(0))?;
        let date_str = job.date.map(|d| d.to_string());
        let cv_path_str = job.cv.as_ref().map(|p| p.to_string_lossy().to_string());
        let status_str = job.status.to_db_string();

        let result = sqlx::query(
            r"
            UPDATE job_applications
            SET date = ?, cv_path = ?, company = ?, position = ?, status = ?, location = ?, salary_min = ?, salary_max = ?
            WHERE id = ?
            ",
        )
        .bind(date_str)
        .bind(cv_path_str)
        .bind(&job.company)
        .bind(&job.position)
        .bind(status_str)
        .bind(&job.location)
        .bind(i64::from(job.salary.min))
        .bind(i64::from(job.salary.max))
        .bind(id)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(DbError::NotFound(id));
        }

        Ok(())
    }

    /// Deletes a job application from the database.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the job application to delete
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - No job application exists with the given ID (`DbError::NotFound`)
    /// - The database connection is lost
    /// - The SQL execution fails
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use job_tracker::db::Database;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let db = Database::new("sqlite::memory:").await?;
    /// db.delete_job(1).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn delete_job(&self, id: i64) -> Result<(), DbError> {
        let result = sqlx::query("DELETE FROM job_applications WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(DbError::NotFound(id));
        }

        Ok(())
    }

    /// Clears all job applications from the database.
    ///
    /// This operation is irreversible and will remove all stored job applications.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The database connection is lost
    /// - The SQL execution fails
    /// - Database constraints prevent deletion
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use job_tracker::db::Database;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let db = Database::new("sqlite::memory:").await?;
    /// db.clear_all().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn clear_all(&self) -> Result<(), DbError> {
        sqlx::query("DELETE FROM job_applications")
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Closes the database connection pool.
    ///
    /// This gracefully closes all database connections in the pool.
    /// After calling this method, no further database operations should
    /// be performed on this instance.
    ///
    /// # Errors
    ///
    /// This function will return an error if there are issues closing
    /// the connection pool, though in practice this is rare.
    pub async fn close(&self) -> Result<(), DbError> {
        self.pool.close().await;
        Ok(())
    }

    /// Converts a database row to a `JobApplication` struct.
    ///
    /// # Arguments
    ///
    /// * `row` - The `SQLite` row to convert
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - A required column is missing from the row
    /// - A date string cannot be parsed into a valid date
    /// - A status string cannot be parsed into a valid status
    /// - Column data types are unexpected
    ///
    /// # Panics
    ///
    /// This function may panic if:
    /// - Required columns are missing (this indicates a schema mismatch)
    /// - Column types don't match expected types
    fn row_to_job_application(row: &sqlx::sqlite::SqliteRow) -> Result<JobApplication, DbError> {
        let id: i64 = row.get("id");
        let date_str: Option<String> = row.get("date");
        let cv_path_str: Option<String> = row.get("cv_path");
        let company: String = row.get("company");
        let position: String = row.get("position");
        let status_str: String = row.get("status");
        let location: String = row.get("location");
        let salary_min: i64 = row.get("salary_min");
        let salary_max: i64 = row.get("salary_max");

        let date = if let Some(date_str) = date_str {
            Some(
                Date::parse(
                    &date_str,
                    &time::format_description::well_known::Iso8601::DATE,
                )
                .map_err(|_| DbError::InvalidStatus(format!("Invalid date format: {date_str}")))?,
            )
        } else {
            None
        };

        let cv = cv_path_str.map(PathBuf::from);
        let status = Status::from_db_string(&status_str).map_err(DbError::InvalidStatus)?;
        let salary = SalaryRange::new(
            u32::try_from(salary_min).unwrap_or(0),
            u32::try_from(salary_max).unwrap_or(0),
        );

        Ok(JobApplication {
            id: Some(id),
            date,
            cv,
            company,
            position,
            status,
            location,
            salary,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;
    use std::fs;
    use std::thread;

    async fn create_test_db() -> Database {
        Database::new("sqlite::memory:").await.unwrap()
    }

    fn create_test_job() -> JobApplication {
        JobApplication::new()
            .company("Test Corp")
            .position("Software Engineer")
            .location("Remote")
            .salary(SalaryRange::new(80_000, 120_000))
            .status(Status::Applied)
            .date(2024, 1, 15)
    }

    fn create_job_with_params(
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

    async fn cleanup_test_files(test_dir: &str) {
        let _ = fs::remove_dir_all(test_dir);
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }

    fn get_unique_test_dir(test_name: &str) -> String {
        format!("test_{}_{:?}", test_name, thread::current().id())
    }

    async fn create_test_db_at_path(test_name: &str) -> (Database, String) {
        let test_dir = get_unique_test_dir(test_name);
        cleanup_test_files(&test_dir).await;
        let db_path = format!("sqlite:{test_dir}/test.db");
        let db = Database::new(&db_path).await.unwrap();
        (db, test_dir)
    }

    fn assert_job_equals_ignoring_id(actual: &JobApplication, expected: &JobApplication) {
        assert_eq!(actual.company, expected.company);
        assert_eq!(actual.position, expected.position);
        assert_eq!(actual.location, expected.location);
        assert_eq!(actual.salary, expected.salary);
        assert_eq!(actual.status, expected.status);
        assert_eq!(actual.date, expected.date);
        assert_eq!(actual.cv, expected.cv);
    }

    #[tokio::test]
    async fn test_database_creation() {
        let _db = create_test_db().await;
    }

    #[tokio::test]
    async fn test_directory_creation_debug() {
        let _ = fs::remove_dir_all("debug_test");

        let db_path = "sqlite:debug_test/test.db";
        println!("Creating database at: {db_path}");

        // Test directory creation manually first
        let path = std::path::Path::new("debug_test/test.db");
        if let Some(parent) = path.parent() {
            println!("Parent directory: {parent:?}");
            println!("Parent exists: {}", parent.exists());
            std::fs::create_dir_all(parent).unwrap();
            println!("Directory created successfully");
        }

        let _db = Database::new(db_path).await.unwrap();

        assert!(std::path::Path::new("debug_test/test.db").exists());

        let _ = fs::remove_dir_all("debug_test");
    }

    #[tokio::test]
    async fn test_persistent_database_creation() {
        let (db, test_dir) = create_test_db_at_path("db_creation").await;

        // Verify the database file was created
        assert!(std::path::Path::new(&format!("{test_dir}/test.db")).exists());

        // Database should start empty
        let jobs = db.get_all_jobs().await.unwrap();
        assert_eq!(jobs.len(), 0);

        cleanup_test_files(&test_dir).await;
    }

    #[tokio::test]
    async fn test_persistent_database_empty_start() {
        let (db, test_dir) = create_test_db_at_path("db_empty_start").await;

        let jobs = db.get_all_jobs().await.unwrap();
        assert_eq!(jobs.len(), 0);

        cleanup_test_files(&test_dir).await;
    }

    #[tokio::test]
    async fn test_persistent_database_data_survives_reconnect() {
        let test_dir = get_unique_test_dir("db_data_survives");
        cleanup_test_files(&test_dir).await;
        let db_path = format!("sqlite:{test_dir}/test.db");
        let test_job = create_test_job();

        let _job_id = {
            let db = Database::new(&db_path).await.unwrap();
            let id = db.insert_job(&test_job).await.unwrap();
            assert!(id > 0);
            id
        };

        // Reconnect to the same database
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        let db2 = Database::new(&db_path).await.unwrap();
        let jobs = db2.get_all_jobs().await.unwrap();
        assert_eq!(jobs.len(), 1);
        assert_job_equals_ignoring_id(&jobs[0], &test_job);

        cleanup_test_files(&test_dir).await;
    }

    #[tokio::test]
    async fn test_persistent_database_multiple_operations() {
        let (db, test_dir) = create_test_db_at_path("db_multiple_ops").await;

        let job1 = create_test_job();
        let job2 = create_job_with_params(
            "Another Corp",
            "Data Scientist",
            "New York",
            90_000,
            130_000,
            Status::Interview(2),
        );

        let id1 = db.insert_job(&job1).await.unwrap();
        let id2 = db.insert_job(&job2).await.unwrap();

        let all_jobs = db.get_all_jobs().await.unwrap();
        assert_eq!(all_jobs.len(), 2);

        let mut updated_job = job1.clone();
        updated_job.id = Some(id1);
        updated_job.status = Status::Offer(95_000);
        db.update_job(&updated_job).await.unwrap();

        let retrieved_job = db.get_job_by_id(id1).await.unwrap();
        assert_eq!(retrieved_job.status, Status::Offer(95_000));

        db.delete_job(id2).await.unwrap();

        let jobs_after_delete = db.get_all_jobs().await.unwrap();
        assert_eq!(jobs_after_delete.len(), 1);
        assert_eq!(jobs_after_delete[0].id, Some(id1));

        cleanup_test_files(&test_dir).await;
    }

    #[tokio::test]
    async fn test_directory_creation() {
        let test_dir = get_unique_test_dir("dir_creation");
        cleanup_test_files(&test_dir).await;
        let db_path = format!("sqlite:{test_dir}/deeply/nested/test_data/test_dir.db");
        let _db = Database::new(&db_path).await.unwrap();

        assert!(
            std::path::Path::new(&format!("{test_dir}/deeply/nested/test_data/test_dir.db"))
                .exists()
        );

        cleanup_test_files(&test_dir).await;
    }

    #[tokio::test]
    async fn test_database_schema_persistence() {
        let test_dir = get_unique_test_dir("db_schema_persist");
        cleanup_test_files(&test_dir).await;
        let db_path = format!("sqlite:{test_dir}/test.db");

        let complex_job = JobApplication::new()
            .company("Complex Corp")
            .position("Full Stack Engineer")
            .location("San Francisco, CA")
            .salary(SalaryRange::new(100_000, 150_000))
            .status(Status::Interview(3))
            .date(2024, 2, 15)
            .cv("path/to/complex_cv.pdf");

        {
            let db = Database::new(&db_path).await.unwrap();
            db.insert_job(&complex_job).await.unwrap();
        }

        // Reconnect and verify data persistence
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        let db2 = Database::new(&db_path).await.unwrap();
        let jobs = db2.get_all_jobs().await.unwrap();
        assert_eq!(jobs.len(), 1);

        let job = &jobs[0];
        assert_eq!(job.company, "Complex Corp");
        assert_eq!(job.position, "Full Stack Engineer");
        assert_eq!(job.location, "San Francisco, CA");
        assert_eq!(job.salary.min, 100_000);
        assert_eq!(job.salary.max, 150_000);
        assert_eq!(job.status, Status::Interview(3));
        assert_eq!(job.date.unwrap().to_string(), "2024-02-15");
        assert_eq!(job.cv, Some(PathBuf::from("path/to/complex_cv.pdf")));

        cleanup_test_files(&test_dir).await;
    }

    #[tokio::test]
    async fn test_insert_and_get_job() {
        let db = create_test_db().await;
        let job = create_test_job();

        let id = db.insert_job(&job).await.unwrap();
        assert!(id > 0);

        let retrieved_job = db.get_job_by_id(id).await.unwrap();
        assert_eq!(retrieved_job.company, job.company);
        assert_eq!(retrieved_job.position, job.position);
        assert_eq!(retrieved_job.location, job.location);
        assert_eq!(retrieved_job.salary, job.salary);
        assert_eq!(retrieved_job.status, job.status);
        assert_eq!(retrieved_job.date, job.date);
    }

    #[tokio::test]
    async fn test_get_all_jobs() {
        let db = create_test_db().await;

        let job1 = create_test_job();
        let job2 = JobApplication::new()
            .company("Another Corp")
            .position("Data Scientist")
            .location("New York")
            .salary(SalaryRange::new(90_000, 130_000))
            .status(Status::Interview(2));

        db.insert_job(&job1).await.unwrap();
        db.insert_job(&job2).await.unwrap();

        let all_jobs = db.get_all_jobs().await.unwrap();
        assert_eq!(all_jobs.len(), 2);
    }

    #[tokio::test]
    async fn test_update_job() {
        let db = create_test_db().await;
        let mut job = create_test_job();

        let id = db.insert_job(&job).await.unwrap();
        job.id = Some(id);
        job.company = "Updated Corp".to_string();
        job.status = Status::Interview(1);

        db.update_job(&job).await.unwrap();

        let updated_job = db.get_job_by_id(id).await.unwrap();
        assert_eq!(updated_job.company, "Updated Corp");
        assert_eq!(updated_job.status, Status::Interview(1));
    }

    #[tokio::test]
    async fn test_delete_job() {
        let db = create_test_db().await;
        let job = create_test_job();

        let id = db.insert_job(&job).await.unwrap();
        db.delete_job(id).await.unwrap();

        let result = db.get_job_by_id(id).await;
        assert!(matches!(result, Err(DbError::NotFound(_))));
    }

    #[tokio::test]
    async fn test_clear_all() {
        let db = create_test_db().await;

        db.insert_job(&create_test_job()).await.unwrap();
        db.insert_job(&create_test_job()).await.unwrap();

        let jobs_before = db.get_all_jobs().await.unwrap();
        assert_eq!(jobs_before.len(), 2);

        db.clear_all().await.unwrap();

        let jobs_after = db.get_all_jobs().await.unwrap();
        assert_eq!(jobs_after.len(), 0);
    }

    #[tokio::test]
    async fn test_job_with_cv_path() {
        let db = create_test_db().await;
        let job = create_test_job().cv("path/to/resume.pdf");

        let id = db.insert_job(&job).await.unwrap();
        let retrieved_job = db.get_job_by_id(id).await.unwrap();

        assert_eq!(retrieved_job.cv, Some(PathBuf::from("path/to/resume.pdf")));
    }

    #[rstest]
    #[case(Status::Applied)]
    #[case(Status::Interview(1))]
    #[case(Status::Interview(3))]
    #[case(Status::Offer(95_000))]
    #[case(Status::Rejected)]
    #[tokio::test]
    async fn test_job_status_persistence(#[case] status: Status) {
        let db = create_test_db().await;
        let job = create_test_job().status(status.clone());

        let id = db.insert_job(&job).await.unwrap();
        let retrieved_job = db.get_job_by_id(id).await.unwrap();

        assert_eq!(retrieved_job.status, status);
    }

    #[tokio::test]
    async fn test_update_nonexistent_job() {
        let db = create_test_db().await;
        let mut job = create_test_job();
        job.id = Some(999);

        let result = db.update_job(&job).await;
        assert!(matches!(result, Err(DbError::NotFound(999))));
    }

    #[tokio::test]
    async fn test_delete_nonexistent_job() {
        let db = create_test_db().await;
        let result = db.delete_job(999).await;
        assert!(matches!(result, Err(DbError::NotFound(999))));
    }

    #[tokio::test]
    async fn test_get_nonexistent_job() {
        let db = create_test_db().await;
        let result = db.get_job_by_id(999).await;
        assert!(matches!(result, Err(DbError::NotFound(999))));
    }
}
