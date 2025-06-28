use std::path::PathBuf;
use time::{Date, Month, UtcDateTime};

#[derive(Debug, Default, Clone, Eq, Ord, PartialEq, PartialOrd)]
pub enum Status {
    #[default]
    Applied,
    Interview(u8),
    Offer(i32),
    Rejected,
}

impl Status {
    /// Converts the status to a database-compatible string representation.
    ///
    /// # Examples
    ///
    /// ```
    /// # use job_tracker::model::Status;
    /// assert_eq!(Status::Applied.to_db_string(), "applied");
    /// assert_eq!(Status::Interview(2).to_db_string(), "interview:2");
    /// assert_eq!(Status::Offer(75_000).to_db_string(), "offer:75000");
    /// assert_eq!(Status::Rejected.to_db_string(), "rejected");
    /// ```
    #[must_use]
    pub fn to_db_string(&self) -> String {
        match self {
            Self::Applied => "applied".to_string(),
            Self::Interview(round) => format!("interview:{round}"),
            Self::Offer(amount) => format!("offer:{amount}"),
            Self::Rejected => "rejected".to_string(),
        }
    }

    /// Creates a Status from a database string representation.
    ///
    /// # Arguments
    ///
    /// * `s` - The database string to parse
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The status string is not recognized
    /// - The interview round cannot be parsed as a number
    /// - The offer amount cannot be parsed as a number
    /// - The status format is malformed
    ///
    /// # Panics
    ///
    /// This function will panic if:
    /// - The input string starts with "interview:" but `strip_prefix` fails
    /// - The input string starts with "offer:" but `strip_prefix` fails
    ///
    /// Note: These panics should not occur in practice as the checks are performed
    /// before calling `unwrap()`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use job_tracker::model::Status;
    /// assert_eq!(Status::from_db_string("applied").unwrap(), Status::Applied);
    /// assert_eq!(Status::from_db_string("interview:3").unwrap(), Status::Interview(3));
    /// assert_eq!(Status::from_db_string("offer:80000").unwrap(), Status::Offer(80_000));
    /// assert_eq!(Status::from_db_string("rejected").unwrap(), Status::Rejected);
    ///
    /// assert!(Status::from_db_string("unknown").is_err());
    /// assert!(Status::from_db_string("interview:abc").is_err());
    /// ```
    pub fn from_db_string(s: &str) -> Result<Self, String> {
        match s {
            "applied" => Ok(Self::Applied),
            "rejected" => Ok(Self::Rejected),
            s if s.starts_with("interview:") => {
                let round_str = s.strip_prefix("interview:").unwrap();
                let round = round_str
                    .parse::<u8>()
                    .map_err(|_| format!("Invalid interview round: {round_str}"))?;
                Ok(Self::Interview(round))
            }
            s if s.starts_with("offer:") => {
                let amount_str = s.strip_prefix("offer:").unwrap();
                let amount = amount_str
                    .parse::<i32>()
                    .map_err(|_| format!("Invalid offer amount: {amount_str}"))?;
                Ok(Self::Offer(amount))
            }
            _ => Err(format!("Unknown status: {s}")),
        }
    }
}

#[derive(Debug, Default, Clone, Eq, Ord, PartialEq, PartialOrd)]
pub struct SalaryRange {
    pub min: u32,
    pub max: u32,
}

impl SalaryRange {
    #[must_use]
    pub const fn new(min: u32, max: u32) -> Self {
        Self { min, max }
    }
}

impl std::fmt::Display for SalaryRange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} - {}", self.min, self.max)
    }
}

#[derive(Debug, Clone)]
pub struct JobApplication {
    pub id: Option<i64>,
    pub date: Option<Date>,
    pub cv: Option<PathBuf>,
    pub company: String,
    pub position: String,
    pub status: Status,
    pub location: String,
    pub salary: SalaryRange,
}

impl Default for JobApplication {
    fn default() -> Self {
        Self {
            id: None,
            date: Some(UtcDateTime::now().date()),
            cv: None,
            company: String::new(),
            position: String::new(),
            status: Status::default(),
            location: String::new(),
            salary: SalaryRange::default(),
        }
    }
}

impl JobApplication {
    /// Creates a new `JobApplication` with default values.
    ///
    /// The application will have:
    /// - No ID (new application)
    /// - Current date
    /// - Empty company, position, and location
    /// - Applied status
    /// - Zero salary range
    /// - No CV path
    ///
    /// # Examples
    ///
    /// ```
    /// # use job_tracker::model::JobApplication;
    /// let job = JobApplication::new();
    /// assert!(job.id.is_none());
    /// assert!(job.date.is_some());
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    /// Sets the application date.
    ///
    /// # Arguments
    ///
    /// * `year` - The year (e.g., 2024)
    /// * `month` - The month (1-12)
    /// * `day` - The day of month (1-31)
    ///
    /// # Panics
    ///
    /// This function will panic if:
    /// - The month is not in the range 1-12
    /// - The day is not valid for the given month and year
    /// - The date values form an invalid date
    ///
    /// # Examples
    ///
    /// ```
    /// # use job_tracker::model::JobApplication;
    /// let job = JobApplication::new().date(2024, 3, 15);
    /// ```
    pub fn date(mut self, year: i32, month: u8, day: u8) -> Self {
        self.date =
            Some(Date::from_calendar_date(year, Month::try_from(month).unwrap(), day).unwrap());
        self
    }

    #[must_use]
    /// Sets the company name.
    ///
    /// # Arguments
    ///
    /// * `company` - The name of the company
    ///
    /// # Examples
    ///
    /// ```
    /// # use job_tracker::model::JobApplication;
    /// let job = JobApplication::new().company("TechCorp Inc.");
    /// ```
    pub fn company(mut self, company: &str) -> Self {
        self.company = company.to_string();
        self
    }

    #[must_use]
    /// Sets the job position/title.
    ///
    /// # Arguments
    ///
    /// * `position` - The job title or position name
    ///
    /// # Examples
    ///
    /// ```
    /// # use job_tracker::model::JobApplication;
    /// let job = JobApplication::new().position("Senior Software Engineer");
    /// ```
    pub fn position(mut self, position: &str) -> Self {
        self.position = position.to_string();
        self
    }

    #[must_use]
    /// Sets the job location.
    ///
    /// # Arguments
    ///
    /// * `location` - The job location (e.g., "Remote", "San Francisco, CA")
    ///
    /// # Examples
    ///
    /// ```
    /// # use job_tracker::model::JobApplication;
    /// let job = JobApplication::new().location("Remote");
    /// ```
    pub fn location(mut self, location: &str) -> Self {
        self.location = location.to_string();
        self
    }

    #[must_use]
    /// Sets the salary range for the position.
    ///
    /// # Arguments
    ///
    /// * `salary` - The salary range for this position
    ///
    /// # Examples
    ///
    /// ```
    /// # use job_tracker::model::{JobApplication, SalaryRange};
    /// let job = JobApplication::new()
    ///     .salary(SalaryRange::new(80_000, 120_000));
    /// ```
    pub const fn salary(mut self, salary: SalaryRange) -> Self {
        self.salary = salary;
        self
    }

    #[must_use]
    /// Sets the path to the CV/resume file.
    ///
    /// # Arguments
    ///
    /// * `cv` - The file path to the CV/resume
    ///
    /// # Examples
    ///
    /// ```
    /// # use job_tracker::model::JobApplication;
    /// let job = JobApplication::new().cv("resumes/my_resume.pdf");
    /// ```
    pub fn cv(mut self, cv: &str) -> Self {
        self.cv = Some(PathBuf::from(cv));
        self
    }

    #[must_use]
    /// Sets the application status.
    ///
    /// # Arguments
    ///
    /// * `status` - The current status of the application
    ///
    /// # Examples
    ///
    /// ```
    /// # use job_tracker::model::{JobApplication, Status};
    /// let job = JobApplication::new().status(Status::Interview(2));
    /// ```
    pub const fn status(mut self, status: Status) -> Self {
        self.status = status;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_job_application() {
        let job = JobApplication::new();
        assert_eq!(job.id, None);
        assert!(job.date.is_some());
        assert!(job.cv.is_none());
        assert_eq!(job.company, "");
        assert_eq!(job.position, "");
        assert_eq!(job.status, Status::default());
        assert_eq!(job.location, "");
        assert_eq!(job.salary, SalaryRange::default());
    }

    #[test]
    fn test_date() {
        let job = JobApplication::new().date(2023, 1, 1);
        assert_eq!(
            job.date.unwrap(),
            Date::from_calendar_date(2023, Month::January, 1).unwrap()
        );
    }

    #[test]
    fn test_position() {
        let job = JobApplication::new().position("Software Engineer");
        assert_eq!(job.position, "Software Engineer");
    }

    #[test]
    fn test_location() {
        let job = JobApplication::new().location("New York");
        assert_eq!(job.location, "New York");
    }

    #[test]
    fn test_status() {
        let job = JobApplication::new().status(Status::Applied);
        assert_eq!(job.status, Status::Applied);
    }

    #[test]
    fn test_salary() {
        let job = JobApplication::new().salary(SalaryRange::new(50_000, 100_000));
        assert_eq!(job.salary, SalaryRange::new(50_000, 100_000));
    }

    #[test]
    fn test_company() {
        let job = JobApplication::new().company("ABC Corp");
        assert_eq!(job.company, "ABC Corp");
    }

    #[test]
    fn test_cv() {
        let path_str = "path/to/cv.pdf";
        let job = JobApplication::new().cv(path_str);
        assert_eq!(job.cv, Some(PathBuf::from(path_str)));
    }

    #[test]
    fn test_complete_builder_chain() {
        let year = 2024;
        let month = 3;
        let day = 15;
        let company = "Tech Solutions Inc";
        let position = "Senior Software Engineer";
        let location = "San Francisco, CA";
        let min_salary = 120_000;
        let max_salary = 150_000;
        let cv_path = "resumes/senior_dev_cv.pdf";
        let interview_round = 2;

        let job = JobApplication::new()
            .date(year, month, day)
            .company(company)
            .position(position)
            .location(location)
            .salary(SalaryRange::new(min_salary, max_salary))
            .cv(cv_path)
            .status(Status::Interview(interview_round));

        assert_eq!(
            job.date.unwrap(),
            Date::from_calendar_date(year, Month::March, day).unwrap()
        );
        assert_eq!(job.company, company);
        assert_eq!(job.position, position);
        assert_eq!(job.location, location);
        assert_eq!(job.salary.min, min_salary);
        assert_eq!(job.salary.max, max_salary);
        assert_eq!(job.cv, Some(PathBuf::from(cv_path)));
        assert_eq!(job.status, Status::Interview(interview_round));
    }

    #[test]
    fn test_status_db_conversion() {
        let applied = Status::Applied;
        assert_eq!(applied.to_db_string(), "applied");
        assert_eq!(Status::from_db_string("applied").unwrap(), applied);

        let interview = Status::Interview(3);
        assert_eq!(interview.to_db_string(), "interview:3");
        assert_eq!(Status::from_db_string("interview:3").unwrap(), interview);

        let offer = Status::Offer(75000);
        assert_eq!(offer.to_db_string(), "offer:75000");
        assert_eq!(Status::from_db_string("offer:75000").unwrap(), offer);

        let rejected = Status::Rejected;
        assert_eq!(rejected.to_db_string(), "rejected");
        assert_eq!(Status::from_db_string("rejected").unwrap(), rejected);
    }

    #[test]
    fn test_status_db_conversion_errors() {
        assert!(Status::from_db_string("unknown").is_err());
        assert!(Status::from_db_string("interview:abc").is_err());
        assert!(Status::from_db_string("offer:xyz").is_err());
    }
}
