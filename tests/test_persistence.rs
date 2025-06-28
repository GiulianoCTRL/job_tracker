use job_tracker::model::{JobApplication, Status};

mod common;
use common::*;

#[cfg(test)]
mod persistence_tests {
    use super::*;

    fn create_sample_job(company: &str, position: &str) -> JobApplication {
        create_job_with_params(
            company,
            position,
            "Test Location",
            75_000,
            100_000,
            Status::Applied,
        )
        .cv("test_cv.pdf")
    }

    #[tokio::test]
    async fn test_database_file_creation() {
        let test_db = TestDb::new("persistence_file_creation").await;

        assert_db_file_exists(&test_db.test_dir, "test.db");

        let jobs = test_db.db.get_all_jobs().await.unwrap();
        assert_eq!(jobs.len(), 0);

        test_db.cleanup().await;
    }

    #[tokio::test]
    async fn test_data_persistence_across_connections() {
        let test_db = TestDb::new("persistence_connections").await;

        let test_job = create_sample_job("Persistence Corp", "Test Engineer");
        let job_id = test_db.db.insert_job(&test_job).await.unwrap();
        assert!(job_id > 0);

        let db2 = test_db.reconnect().await;
        let jobs = db2.get_all_jobs().await.unwrap();
        assert_eq!(jobs.len(), 1);

        let retrieved_job = &jobs[0];
        assert_eq!(retrieved_job.id, Some(job_id));
        assert_job_equals_ignoring_id(retrieved_job, &test_job);

        test_db.cleanup().await;
    }

    #[tokio::test]
    async fn test_multiple_jobs_persistence() {
        let test_db = TestDb::new("persistence_multiple").await;

        let job1 = create_sample_job("Company A", "Developer");
        let job2 = create_sample_job("Company B", "Designer");
        let job3 = create_job_with_params(
            "Company C",
            "Manager",
            "Remote",
            90_000,
            120_000,
            Status::Interview(2),
        );

        let jobs_to_insert = vec![job1, job2, job3];
        insert_multiple_jobs(&test_db.db, &jobs_to_insert).await;

        let db2 = test_db.reconnect().await;
        let job_vec = db2.get_all_jobs().await.unwrap();
        assert_eq!(job_vec.len(), 3);

        for expected_job in &jobs_to_insert {
            assert_job_in_list(&job_vec, expected_job);
        }

        let manager_job = job_vec.iter().find(|j| j.position == "Manager").unwrap();
        assert_eq!(manager_job.status, Status::Interview(2));
        assert_eq!(manager_job.salary.min, 90_000);

        test_db.cleanup().await;
    }

    #[tokio::test]
    async fn test_crud_operations_persistence() {
        let test_db = TestDb::new("persistence_crud").await;

        let original_job = create_sample_job("CRUD Corp", "Original Position");
        let job_id = test_db.db.insert_job(&original_job).await.unwrap();

        // Test Update
        let db2 = test_db.reconnect().await;
        let mut job = db2.get_job_by_id(job_id).await.unwrap();
        job.company = "Updated CRUD Corp".to_string();
        job.status = Status::Offer(95_000);
        db2.update_job(&job).await.unwrap();

        // Test Read after Update
        let db3 = test_db.reconnect().await;
        let updated_job = db3.get_job_by_id(job_id).await.unwrap();
        assert_eq!(updated_job.company, "Updated CRUD Corp");
        assert_eq!(updated_job.status, Status::Offer(95_000));

        // Test Delete
        db3.delete_job(job_id).await.unwrap();

        // Test Read after Delete
        let db4 = test_db.reconnect().await;
        let result = db4.get_job_by_id(job_id).await;
        assert!(result.is_err());

        let jobs = db4.get_all_jobs().await.unwrap();
        assert_eq!(jobs.len(), 0);

        test_db.cleanup().await;
    }

    #[tokio::test]
    async fn test_status_types_persistence() {
        let test_db = TestDb::new("persistence_status").await;

        // Use the multiple_jobs function which has different status types
        let multiple_jobs = multiple_jobs();
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
    async fn test_clear_database_persistence() {
        let test_db = TestDb::new("persistence_clear").await;

        let job = create_sample_job("Clear Test Corp", "Test Position");
        test_db.db.insert_job(&job).await.unwrap();

        let jobs = test_db.db.get_all_jobs().await.unwrap();
        assert_eq!(jobs.len(), 1);

        test_db.db.clear_all().await.unwrap();

        let jobs = test_db.db.get_all_jobs().await.unwrap();
        assert_eq!(jobs.len(), 0);

        // Verify persistence after reconnection
        let db2 = test_db.reconnect().await;
        let jobs = db2.get_all_jobs().await.unwrap();
        assert_eq!(jobs.len(), 0);

        test_db.cleanup().await;
    }

    #[tokio::test]
    async fn test_nested_directory_creation() {
        let test_db =
            create_nested_test_db("persistence_nested", "level1/level2/level3/nested.db").await;

        assert_db_file_exists(&test_db.test_dir, "level1/level2/level3/nested.db");

        let job = create_sample_job("Nested Corp", "Deep Engineer");
        let job_id = test_db.db.insert_job(&job).await.unwrap();
        assert!(job_id > 0);

        let db2 = test_db.reconnect().await;
        let jobs = db2.get_all_jobs().await.unwrap();
        assert_eq!(jobs.len(), 1);
        assert_job_in_list(&jobs, &job);

        test_db.cleanup().await;
    }

    #[tokio::test]
    async fn test_database_starts_empty() {
        let test_db = TestDb::new("persistence_empty_start").await;

        let jobs = test_db.db.get_all_jobs().await.unwrap();
        assert_eq!(jobs.len(), 0);

        let db2 = test_db.reconnect().await;
        let jobs = db2.get_all_jobs().await.unwrap();
        assert_eq!(jobs.len(), 0);

        test_db.cleanup().await;
    }
}

#[tokio::test]
async fn test_manual_persistence() {
    println!("Testing manual persistence...");

    let test_db = TestDb::new("manual_persistence").await;
    println!("✓ Database created at: {}/test.db", test_db.test_dir);

    let job = create_job_with_params(
        "Manual Test Corp",
        "Test Engineer",
        "Test City",
        80_000,
        100_000,
        Status::Applied,
    );

    let job_id = test_db.db.insert_job(&job).await.unwrap();
    println!("✓ Job inserted with ID: {job_id}");

    let jobs = test_db.db.get_all_jobs().await.unwrap();
    println!("✓ Retrieved {} jobs from database", jobs.len());

    if !jobs.is_empty() {
        println!("✓ First job: {} at {}", jobs[0].company, jobs[0].position);
    }

    let db2 = test_db.reconnect().await;
    let jobs_after_reconnect = db2.get_all_jobs().await.unwrap();
    println!(
        "✓ After reconnection, found {} jobs",
        jobs_after_reconnect.len()
    );

    assert_eq!(jobs.len(), jobs_after_reconnect.len());
    assert_job_equals_ignoring_id(&jobs[0], &jobs_after_reconnect[0]);

    println!("✓ All persistence tests passed!");

    test_db.cleanup().await;
}
