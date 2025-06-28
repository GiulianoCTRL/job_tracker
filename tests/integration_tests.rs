use job_tracker::model::{JobApplication, SalaryRange, Status};

mod common;
use common::*;

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_database_file_creation() {
        let test_db = TestDb::new("integration_file_creation").await;

        assert_db_file_exists(&test_db.test_dir, "test.db");

        let jobs = test_db.db.get_all_jobs().await.unwrap();
        assert_eq!(jobs.len(), 0);

        test_db.cleanup().await;
    }

    #[tokio::test]
    async fn test_persistent_data_across_sessions() {
        let test_db = TestDb::new("integration_sessions").await;

        let test_job = JobApplication::new()
            .company("Session Test Corp")
            .position("Integration Engineer")
            .location("Remote")
            .salary(SalaryRange::new(80_000, 120_000))
            .status(Status::Applied)
            .date(2024, 3, 1)
            .cv("resumes/integration_test.pdf");

        let job_id = test_db.db.insert_job(&test_job).await.unwrap();

        // First reconnection - verify original job
        let db2 = test_db.reconnect().await;
        let jobs = db2.get_all_jobs().await.unwrap();
        assert_eq!(jobs.len(), 1);
        let job = &jobs[0];
        assert_eq!(job.id, Some(job_id));
        assert_job_equals_ignoring_id(job, &test_job);

        // Add second job
        let second_job = create_job_with_params(
            "Another Corp",
            "Senior Developer",
            "New York",
            100_000,
            140_000,
            Status::Interview(2),
        );

        db2.insert_job(&second_job).await.unwrap();

        // Second reconnection - verify both jobs
        let db3 = test_db.reconnect().await;
        let jobs = db3.get_all_jobs().await.unwrap();
        assert_eq!(jobs.len(), 2);

        let companies: Vec<&String> = jobs.iter().map(|j| &j.company).collect();
        assert!(companies.contains(&&"Session Test Corp".to_string()));
        assert!(companies.contains(&&"Another Corp".to_string()));

        test_db.cleanup().await;
    }

    #[tokio::test]
    async fn test_database_operations_persistence() {
        let test_db = TestDb::new("integration_operations").await;

        let job1 = create_job_with_params(
            "Update Test Corp",
            "Developer",
            "Austin",
            70_000,
            90_000,
            Status::Applied,
        );

        let job2 = create_job_with_params(
            "Delete Test Corp",
            "Manager",
            "Boston",
            90_000,
            110_000,
            Status::Interview(1),
        );

        let id1 = test_db.db.insert_job(&job1).await.unwrap();
        let id2 = test_db.db.insert_job(&job2).await.unwrap();

        let job_vec = test_db.db.get_all_jobs().await.unwrap();
        assert_eq!(job_vec.len(), 2);

        // Update first job
        let mut updated_job = job1.clone();
        updated_job.id = Some(id1);
        updated_job.status = Status::Offer(85_000);
        updated_job.company = "Updated Corp".to_string();

        test_db.db.update_job(&updated_job).await.unwrap();
        test_db.db.delete_job(id2).await.unwrap();

        // Reconnect and verify changes persisted
        let db2 = test_db.reconnect().await;
        let job_vec = db2.get_all_jobs().await.unwrap();
        assert_eq!(job_vec.len(), 1);

        let remaining_job = &job_vec[0];
        assert_eq!(remaining_job.id, Some(id1));
        assert_eq!(remaining_job.company, "Updated Corp");
        assert_eq!(remaining_job.status, Status::Offer(85_000));

        let result = db2.get_job_by_id(id2).await;
        assert!(result.is_err());

        test_db.cleanup().await;
    }

    #[tokio::test]
    async fn test_complex_status_persistence() {
        let test_db = TestDb::new("integration_status").await;

        let multiple_jobs = multiple_jobs();

        // Insert all jobs with different statuses
        let ids = insert_multiple_jobs(&test_db.db, &multiple_jobs).await;

        // Create list of (id, job) pairs for verification
        let jobs_with_ids: Vec<(i64, &JobApplication)> = ids
            .iter()
            .zip(multiple_jobs.iter())
            .map(|(&id, job)| (id, job))
            .collect();

        // Reconnect and verify all statuses persisted correctly
        let db2 = test_db.reconnect().await;
        verify_status_persistence(&db2, &jobs_with_ids).await;

        test_db.cleanup().await;
    }

    #[tokio::test]
    async fn test_empty_database_no_sample_data() {
        let test_db = TestDb::new("integration_empty").await;

        let jobs = test_db.db.get_all_jobs().await.unwrap();
        assert_eq!(jobs.len(), 0);

        // Reconnect and verify still empty
        let db2 = test_db.reconnect().await;
        let jobs = db2.get_all_jobs().await.unwrap();
        assert_eq!(jobs.len(), 0);

        test_db.cleanup().await;
    }

    #[tokio::test]
    async fn test_nested_directory_creation() {
        let test_db =
            create_nested_test_db("integration_nested", "deep/nested/path/nested_test.db").await;

        assert_db_file_exists(&test_db.test_dir, "deep/nested/path/nested_test.db");

        let test_job = create_job_with_params(
            "Nested Corp",
            "Path Engineer",
            "Deep Location",
            75_000,
            95_000,
            Status::Applied,
        );

        let job_id = test_db.db.insert_job(&test_job).await.unwrap();
        assert!(job_id > 0);

        let jobs = test_db.db.get_all_jobs().await.unwrap();
        assert_eq!(jobs.len(), 1);
        assert_job_in_list(&jobs, &test_job);

        test_db.cleanup().await;
    }

    #[tokio::test]
    async fn test_database_clear_and_repopulate() {
        let test_db = TestDb::new("integration_clear").await;

        let initial_job = create_job_with_params(
            "Initial Corp",
            "Developer",
            "Remote",
            60_000,
            80_000,
            Status::Applied,
        );

        test_db.db.insert_job(&initial_job).await.unwrap();

        let jobs = test_db.db.get_all_jobs().await.unwrap();
        assert_eq!(jobs.len(), 1);

        test_db.db.clear_all().await.unwrap();

        let jobs = test_db.db.get_all_jobs().await.unwrap();
        assert_eq!(jobs.len(), 0);

        // Reconnect and verify still empty
        let db2 = test_db.reconnect().await;
        let jobs = db2.get_all_jobs().await.unwrap();
        assert_eq!(jobs.len(), 0);

        // Add new job after clear
        let new_job = create_job_with_params(
            "New Corp",
            "New Position",
            "New Location",
            70_000,
            90_000,
            Status::Interview(1),
        );

        db2.insert_job(&new_job).await.unwrap();

        // Reconnect and verify new job persisted
        let db3 = test_db.reconnect().await;
        let jobs = db3.get_all_jobs().await.unwrap();
        assert_eq!(jobs.len(), 1);
        assert_job_in_list(&jobs, &new_job);

        test_db.cleanup().await;
    }
}
