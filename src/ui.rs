use std::fmt;

use crate::db::Database;
use crate::model::{JobApplication, SalaryRange, Status};
use iced::widget::{Space, button, column, container, row, scrollable, text, text_input};
use iced::{Element, Length, Task, Theme};
use std::path::PathBuf;
use time::Date;

/// Theme selection for the application.
///
/// Determines the visual appearance of the user interface,
/// supporting both light and dark modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppTheme {
    Light,
    Dark,
}

impl AppTheme {
    /// Converts the app theme to an Iced theme.
    ///
    /// # Examples
    ///
    /// ```
    /// # use job_tracker::ui::AppTheme;
    /// let theme = AppTheme::Light.to_iced_theme();
    /// ```
    #[must_use]
    pub const fn to_iced_theme(self) -> Theme {
        match self {
            Self::Light => Theme::Light,
            Self::Dark => Theme::Dark,
        }
    }
}

/// Messages that can be sent within the application.
///
/// These messages represent all possible user interactions and
/// system events that can occur in the job tracker application.
#[derive(Debug, Clone)]
pub enum Message {
    /// Database has been initialized with the given jobs.
    DatabaseInitialized(Database, Vec<JobApplication>),
    /// Job data has been loaded from the database.
    JobsLoaded(Result<Vec<JobApplication>, String>),
    /// User wants to add a new job application.
    AddNewJob,
    /// User wants to edit an existing job application.
    EditJob(i64),
    /// User wants to save the current job application.
    SaveJob(i64),
    /// User wants to cancel the current edit operation.
    CancelEdit,
    /// User wants to delete a job application.
    DeleteJob(i64),
    /// User wants to clear all job applications.
    ClearDatabase,
    /// Database has been cleared.
    DatabaseCleared(Result<(), String>),
    /// User wants to toggle the application theme.
    ToggleTheme,
    /// User has selected a job application.
    SelectJob(Option<i64>),

    /// Form field changes for editing job applications.
    CompanyChanged(String),
    PositionChanged(String),
    LocationChanged(String),
    DateChanged(String),
    SalaryMinChanged(String),
    SalaryMaxChanged(String),
    StatusChanged(StatusSelection),
    CvPathChanged(String),
    InterviewRoundChanged(String),
    OfferAmountChanged(String),
}

/// Status selection enum for the UI dropdown.
///
/// Simplified version of the Status enum used in form controls
/// to avoid dealing with associated data in the UI layer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StatusSelection {
    Applied,
    Interview,
    Offer,
    Rejected,
}

impl fmt::Display for StatusSelection {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Applied => write!(f, "Applied"),
            Self::Interview => write!(f, "Interview"),
            Self::Offer => write!(f, "Offer"),
            Self::Rejected => write!(f, "Rejected"),
        }
    }
}

impl StatusSelection {
    /// Creates a `StatusSelection` from a `Status` enum.
    ///
    /// This function maps the more complex `Status` enum (which may contain
    /// associated data) to the simpler `StatusSelection` enum used in the UI.
    ///
    /// # Examples
    ///
    /// ```
    /// # use job_tracker::ui::StatusSelection;
    /// # use job_tracker::model::Status;
    /// let status = Status::Interview(3);
    /// let selection = StatusSelection::from_status(&status);
    /// assert_eq!(selection, StatusSelection::Interview);
    /// ```
    #[must_use]
    pub const fn from_status(status: &Status) -> Self {
        match status {
            Status::Applied => Self::Applied,
            Status::Interview(_) => Self::Interview,
            Status::Offer(_) => Self::Offer,
            Status::Rejected => Self::Rejected,
        }
    }
}

/// Edit form state for job applications.
///
/// Holds the current state of the job application edit form,
/// including all user inputs as strings for easy UI binding.
#[derive(Debug, Clone)]
pub struct EditForm {
    pub company: String,
    pub position: String,
    pub location: String,
    pub date: String,
    pub salary_min: String,
    pub salary_max: String,
    pub status: StatusSelection,
    pub cv_path: String,
    pub interview_round: String,
    pub offer_amount: String,
}

impl Default for EditForm {
    fn default() -> Self {
        Self::new()
    }
}

impl EditForm {
    /// Creates a new empty edit form.
    ///
    /// All fields are initialized to empty strings except:
    /// - `status`: defaults to `StatusSelection::Applied`
    /// - `interview_round`: defaults to "1"
    ///
    /// # Examples
    ///
    /// ```
    /// # use job_tracker::ui::EditForm;
    /// let form = EditForm::new();
    /// assert_eq!(form.company, "");
    /// assert_eq!(form.interview_round, "1");
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self {
            company: String::new(),
            position: String::new(),
            location: String::new(),
            date: String::new(),
            salary_min: String::new(),
            salary_max: String::new(),
            status: StatusSelection::Applied,
            cv_path: String::new(),
            interview_round: "1".to_string(),
            offer_amount: String::new(),
        }
    }

    /// Creates an edit form from an existing job application.
    ///
    /// Populates all form fields with data from the provided job application.
    /// Handles the conversion of complex types to strings for UI display.
    ///
    /// # Arguments
    ///
    /// * `job` - The job application to populate the form with
    ///
    /// # Examples
    ///
    /// ```
    /// # use job_tracker::ui::EditForm;
    /// # use job_tracker::model::JobApplication;
    /// let job = JobApplication::new().company("TechCorp");
    /// let form = EditForm::from_job(&job);
    /// assert_eq!(form.company, "TechCorp");
    /// ```
    #[must_use]
    pub fn from_job(job: &JobApplication) -> Self {
        let (interview_round, offer_amount) = match &job.status {
            Status::Interview(round) => (round.to_string(), String::new()),
            Status::Offer(amount) => (String::new(), amount.to_string()),
            _ => (String::new(), String::new()),
        };

        Self {
            company: job.company.clone(),
            position: job.position.clone(),
            location: job.location.clone(),
            date: job.date.map(|d| d.to_string()).unwrap_or_default(),
            salary_min: job.salary.min.to_string(),
            salary_max: job.salary.max.to_string(),
            status: StatusSelection::from_status(&job.status),
            cv_path: job
                .cv
                .as_ref()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default(),
            interview_round,
            offer_amount,
        }
    }

    /// Converts the edit form to a `JobApplication`.
    ///
    /// Validates and parses all form fields, converting them from strings
    /// to the appropriate types for the `JobApplication` model.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID to assign to the job application (None for new jobs)
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The date string is not in YYYY-MM-DD format
    /// - The minimum or maximum salary cannot be parsed as a number
    /// - The interview round cannot be parsed as a number (for Interview status)
    /// - The offer amount cannot be parsed as a number (for Offer status)
    ///
    /// # Examples
    ///
    /// ```
    /// # use job_tracker::ui::EditForm;
    /// let mut form = EditForm::new();
    /// form.company = "TechCorp".to_string();
    /// form.salary_min = "50000".to_string();
    /// form.salary_max = "80000".to_string();
    /// let job = form.to_job(None).unwrap();
    /// assert_eq!(job.company, "TechCorp");
    /// ```
    pub fn to_job(&self, id: Option<i64>) -> Result<JobApplication, String> {
        let date = if self.date.is_empty() {
            None
        } else {
            Some(
                Date::parse(
                    &self.date,
                    &time::format_description::well_known::Iso8601::DATE,
                )
                .map_err(|_| "Invalid date format. Use YYYY-MM-DD".to_string())?,
            )
        };

        let salary_min = self
            .salary_min
            .parse::<u32>()
            .map_err(|_| "Invalid minimum salary".to_string())?;
        let salary_max = self
            .salary_max
            .parse::<u32>()
            .map_err(|_| "Invalid maximum salary".to_string())?;

        let status = match self.status {
            StatusSelection::Applied => Status::Applied,
            StatusSelection::Interview => {
                let round = self
                    .interview_round
                    .parse::<u8>()
                    .map_err(|_| "Invalid interview round".to_string())?;
                Status::Interview(round)
            }
            StatusSelection::Offer => {
                let amount = self
                    .offer_amount
                    .parse::<i32>()
                    .map_err(|_| "Invalid offer amount".to_string())?;
                Status::Offer(amount)
            }
            StatusSelection::Rejected => Status::Rejected,
        };

        let cv = if self.cv_path.is_empty() {
            None
        } else {
            Some(PathBuf::from(&self.cv_path))
        };

        Ok(JobApplication {
            id,
            date,
            cv,
            company: self.company.clone(),
            position: self.position.clone(),
            status,
            location: self.location.clone(),
            salary: SalaryRange::new(salary_min, salary_max),
        })
    }
}

/// Main application state.
///
/// Contains all the state needed to run the job tracker application,
/// including database connection, job data, UI state, and user preferences.
pub struct JobTrackerApp {
    database: Option<Database>,
    jobs: Vec<JobApplication>,
    selected_job_id: Option<i64>,
    editing_job_id: Option<i64>,
    edit_form: EditForm,
    theme: AppTheme,
    error_message: Option<String>,
}

impl Default for JobTrackerApp {
    fn default() -> Self {
        Self::new()
    }
}

impl JobTrackerApp {
    /// Creates a new job tracker application instance.
    ///
    /// Initializes the application with default state:
    /// - No database connection
    /// - Empty job list
    /// - Light theme
    /// - No selections or errors
    ///
    /// # Examples
    ///
    /// ```
    /// # use job_tracker::ui::JobTrackerApp;
    /// let app = JobTrackerApp::new();
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self {
            database: None,
            jobs: Vec::new(),
            selected_job_id: None,
            editing_job_id: None,
            edit_form: EditForm::new(),
            theme: AppTheme::Light,
            error_message: None,
        }
    }

    /// Loads all jobs from the database.
    ///
    /// Creates an asynchronous task to fetch all job applications from
    /// the database. If no database is connected, returns an empty task.
    ///
    /// # Returns
    ///
    /// A Task that will send a `JobsLoaded` message when complete.
    fn load_jobs(&self) -> Task<Message> {
        self.database.as_ref().map_or_else(Task::none, |db| {
            let db = db.clone();
            Task::perform(
                async move { db.get_all_jobs().await.map_err(|e| e.to_string()) },
                Message::JobsLoaded,
            )
        })
    }

    fn view_table(&self) -> Element<'_, Message> {
        let header = row![
            container(text("Company")).width(Length::FillPortion(2)),
            container(text("Position")).width(Length::FillPortion(2)),
            container(text("Location")).width(Length::FillPortion(2)),
            container(text("Status")).width(Length::FillPortion(2)),
            container(text("Salary")).width(Length::FillPortion(2)),
            container(text("Date")).width(Length::FillPortion(1)),
            container(text("Actions")).width(Length::FillPortion(1)),
        ]
        .spacing(10);

        let mut content = column![header].spacing(5);

        if self.editing_job_id == Some(0) {
            let edit_row = self.view_edit_row();
            content = content.push(edit_row);
        }

        for job in &self.jobs {
            let is_selected = self.selected_job_id == job.id;
            let is_editing = self.editing_job_id == job.id;

            if is_editing {
                let edit_row = self.view_edit_row();
                content = content.push(edit_row);
            } else {
                let status_text = match &job.status {
                    Status::Applied => "Applied".to_string(),
                    Status::Interview(round) => format!("Interview ({round})"),
                    Status::Offer(amount) => format!("Offer ({amount})"),
                    Status::Rejected => "Rejected".to_string(),
                };

                let job_row = row![
                    container(button(text(&job.company)).on_press(Message::SelectJob(job.id)))
                        .width(Length::FillPortion(2)),
                    container(text(&job.position)).width(Length::FillPortion(2)),
                    container(text(&job.location)).width(Length::FillPortion(2)),
                    container(text(status_text)).width(Length::FillPortion(2)),
                    container(text(job.salary.to_string())).width(Length::FillPortion(2)),
                    container(text(job.date.map(|d| d.to_string()).unwrap_or_default()))
                        .width(Length::FillPortion(1)),
                    container(
                        row![
                            button(text("Edit")).on_press(Message::EditJob(job.id.unwrap_or(0))),
                            button(text("Delete"))
                                .on_press(Message::DeleteJob(job.id.unwrap_or(0))),
                        ]
                        .spacing(5)
                    )
                    .width(Length::FillPortion(1)),
                ]
                .spacing(10);

                let styled_row = if is_selected {
                    container(job_row).style(|_theme| container::Style {
                        background: Some(iced::Background::Color(iced::Color::from_rgb(
                            0.9, 0.9, 1.0,
                        ))),
                        ..Default::default()
                    })
                } else {
                    container(job_row)
                };

                content = content.push(styled_row);
            }
        }

        scrollable(content).into()
    }

    #[allow(clippy::too_many_lines)]
    fn view_edit_row(&self) -> Element<'_, Message> {
        let theme = self.theme;

        let status_controls = match self.edit_form.status {
            StatusSelection::Interview => row![
                text("Interview Round:").style(move |_| {
                    match theme {
                        AppTheme::Light => iced::widget::text::Style {
                            color: Some(iced::Color::from_rgb(0.0, 0.0, 0.0)),
                        },
                        AppTheme::Dark => iced::widget::text::Style {
                            color: Some(iced::Color::from_rgb(0.9, 0.9, 0.9)),
                        },
                    }
                }),
                text_input("Round", &self.edit_form.interview_round)
                    .on_input(Message::InterviewRoundChanged)
                    .width(Length::Fixed(80.0))
            ]
            .spacing(5),
            StatusSelection::Offer => row![
                text("Offer Amount:").style(move |_| {
                    match theme {
                        AppTheme::Light => iced::widget::text::Style {
                            color: Some(iced::Color::from_rgb(0.0, 0.0, 0.0)),
                        },
                        AppTheme::Dark => iced::widget::text::Style {
                            color: Some(iced::Color::from_rgb(0.9, 0.9, 0.9)),
                        },
                    }
                }),
                text_input("Amount", &self.edit_form.offer_amount)
                    .on_input(Message::OfferAmountChanged)
                    .width(Length::Fixed(120.0))
            ]
            .spacing(5),
            _ => row![].spacing(5),
        };

        let edit_form = column![
            row![
                column![
                    text("Company:").style(move |_| {
                        match theme {
                            AppTheme::Light => iced::widget::text::Style {
                                color: Some(iced::Color::from_rgb(0.0, 0.0, 0.0)),
                            },
                            AppTheme::Dark => iced::widget::text::Style {
                                color: Some(iced::Color::from_rgb(0.9, 0.9, 0.9)),
                            },
                        }
                    }),
                    text_input("Company", &self.edit_form.company)
                        .on_input(Message::CompanyChanged)
                        .width(Length::Fixed(200.0))
                ]
                .spacing(2),
                column![
                    text("Position:").style(move |_| {
                        match theme {
                            AppTheme::Light => iced::widget::text::Style {
                                color: Some(iced::Color::from_rgb(0.0, 0.0, 0.0)),
                            },
                            AppTheme::Dark => iced::widget::text::Style {
                                color: Some(iced::Color::from_rgb(0.9, 0.9, 0.9)),
                            },
                        }
                    }),
                    text_input("Position", &self.edit_form.position)
                        .on_input(Message::PositionChanged)
                        .width(Length::Fixed(200.0))
                ]
                .spacing(2),
                column![
                    text("Location:").style(move |_| {
                        match theme {
                            AppTheme::Light => iced::widget::text::Style {
                                color: Some(iced::Color::from_rgb(0.0, 0.0, 0.0)),
                            },
                            AppTheme::Dark => iced::widget::text::Style {
                                color: Some(iced::Color::from_rgb(0.9, 0.9, 0.9)),
                            },
                        }
                    }),
                    text_input("Location", &self.edit_form.location)
                        .on_input(Message::LocationChanged)
                        .width(Length::Fixed(150.0))
                ]
                .spacing(2)
            ]
            .spacing(10),
            row![
                column![
                    text("Date (YYYY-MM-DD):").style(move |_| {
                        match theme {
                            AppTheme::Light => iced::widget::text::Style {
                                color: Some(iced::Color::from_rgb(0.0, 0.0, 0.0)),
                            },
                            AppTheme::Dark => iced::widget::text::Style {
                                color: Some(iced::Color::from_rgb(0.9, 0.9, 0.9)),
                            },
                        }
                    }),
                    text_input("Date", &self.edit_form.date)
                        .on_input(Message::DateChanged)
                        .width(Length::Fixed(150.0))
                ]
                .spacing(2),
                column![
                    text("Min Salary:").style(move |_| {
                        match theme {
                            AppTheme::Light => iced::widget::text::Style {
                                color: Some(iced::Color::from_rgb(0.0, 0.0, 0.0)),
                            },
                            AppTheme::Dark => iced::widget::text::Style {
                                color: Some(iced::Color::from_rgb(0.9, 0.9, 0.9)),
                            },
                        }
                    }),
                    text_input("Min", &self.edit_form.salary_min)
                        .on_input(Message::SalaryMinChanged)
                        .width(Length::Fixed(100.0))
                ]
                .spacing(2),
                column![
                    text("Max Salary:").style(move |_| {
                        match theme {
                            AppTheme::Light => iced::widget::text::Style {
                                color: Some(iced::Color::from_rgb(0.0, 0.0, 0.0)),
                            },
                            AppTheme::Dark => iced::widget::text::Style {
                                color: Some(iced::Color::from_rgb(0.9, 0.9, 0.9)),
                            },
                        }
                    }),
                    text_input("Max", &self.edit_form.salary_max)
                        .on_input(Message::SalaryMaxChanged)
                        .width(Length::Fixed(100.0))
                ]
                .spacing(2)
            ]
            .spacing(10),
            row![
                column![
                    text("Status:").style(move |_| {
                        match theme {
                            AppTheme::Light => iced::widget::text::Style {
                                color: Some(iced::Color::from_rgb(0.0, 0.0, 0.0)),
                            },
                            AppTheme::Dark => iced::widget::text::Style {
                                color: Some(iced::Color::from_rgb(0.9, 0.9, 0.9)),
                            },
                        }
                    }),
                    row![
                        button(text("Applied"))
                            .on_press(Message::StatusChanged(StatusSelection::Applied)),
                        button(text("Interview"))
                            .on_press(Message::StatusChanged(StatusSelection::Interview)),
                        button(text("Offer"))
                            .on_press(Message::StatusChanged(StatusSelection::Offer)),
                        button(text("Rejected"))
                            .on_press(Message::StatusChanged(StatusSelection::Rejected)),
                    ]
                    .spacing(5)
                ]
                .spacing(2),
                status_controls
            ]
            .spacing(10),
            row![
                column![
                    text("CV Path:").style(move |_| {
                        match theme {
                            AppTheme::Light => iced::widget::text::Style {
                                color: Some(iced::Color::from_rgb(0.0, 0.0, 0.0)),
                            },
                            AppTheme::Dark => iced::widget::text::Style {
                                color: Some(iced::Color::from_rgb(0.9, 0.9, 0.9)),
                            },
                        }
                    }),
                    text_input("CV Path", &self.edit_form.cv_path)
                        .on_input(Message::CvPathChanged)
                        .width(Length::Fixed(300.0))
                ]
                .spacing(2)
            ],
            row![
                button(text("Save")).on_press(Message::SaveJob(self.editing_job_id.unwrap_or(0))),
                button(text("Cancel")).on_press(Message::CancelEdit)
            ]
            .spacing(10)
        ]
        .spacing(10);

        container(edit_form)
            .style(|_theme| container::Style {
                background: Some(iced::Background::Color(iced::Color::from_rgb(
                    0.95, 0.95, 0.95,
                ))),
                border: iced::Border {
                    radius: 5.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            })
            .padding(10)
            .into()
    }

    fn view_cv_panel(&self) -> Element<'_, Message> {
        let content = self.selected_job_id.map_or_else(
            || column![text("Select a job to view CV information")],
            |selected_id| {
                self.jobs
                    .iter()
                    .find(|j| j.id == Some(selected_id))
                    .map_or_else(
                        || column![text("Job not found")],
                        |job| {
                            column![
                                text("Selected Job").size(20),
                                text(format!("Company: {}", job.company)),
                                text(format!("Position: {}", job.position)),
                                Space::with_height(Length::Fixed(20.0)),
                                text("CV Information").size(16),
                                job.cv.as_ref().map_or_else(
                                    || text("No CV"),
                                    |cv_path| text(format!("CV Path: {}", cv_path.display()))
                                )
                            ]
                            .spacing(5)
                        },
                    )
            },
        );

        container(content)
            .padding(20)
            .width(Length::Fixed(300.0))
            .height(Length::Fill)
            .style(move |_theme| container::Style {
                background: Some(iced::Background::Color(match self.theme {
                    AppTheme::Light => iced::Color::from_rgb(0.98, 0.98, 0.98),
                    AppTheme::Dark => iced::Color::from_rgb(0.15, 0.15, 0.15),
                })),
                border: iced::Border {
                    width: 1.0,
                    color: iced::Color::from_rgb(0.8, 0.8, 0.8),
                    ..Default::default()
                },
                ..Default::default()
            })
            .into()
    }

    async fn initialize_database() -> Result<(Database, Vec<JobApplication>), String> {
        if let Err(e) = std::fs::create_dir_all("data") {
            eprintln!("Warning: Could not create data directory: {e}");
        }

        let db = Database::new("sqlite:data/jobs.db")
            .await
            .map_err(|e| e.to_string())?;

        let jobs = db.get_all_jobs().await.unwrap_or_else(|_| Vec::new());

        Ok((db, jobs))
    }
}

impl JobTrackerApp {
    fn init() -> (Self, Task<Message>) {
        let app = Self::new();
        let task = Task::perform(Self::initialize_database(), |result| match result {
            Ok((db, _jobs)) => Message::DatabaseInitialized(db, Vec::new()),
            Err(e) => Message::JobsLoaded(Err(e)),
        });
        (app, task)
    }

    #[allow(clippy::too_many_lines)]
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::DatabaseInitialized(db, _jobs) => {
                self.database = Some(db);
                self.error_message = None;
                self.load_jobs()
            }
            Message::JobsLoaded(result) => {
                match result {
                    Ok(jobs) => {
                        self.jobs = jobs;
                        self.error_message = None;
                    }
                    Err(e) => {
                        self.error_message = Some(e);
                    }
                }
                Task::none()
            }
            Message::AddNewJob => {
                if self.editing_job_id.is_none() {
                    self.editing_job_id = Some(0); // Use 0 for new jobs
                    self.edit_form = EditForm::new();

                    self.edit_form.date = "2024-01-01".to_string();
                }
                Task::none()
            }
            Message::EditJob(id) => {
                if let Some(job) = self.jobs.iter().find(|j| j.id == Some(id)) {
                    self.editing_job_id = Some(id);
                    self.edit_form = EditForm::from_job(job);
                }
                Task::none()
            }
            Message::SaveJob(id) => {
                match self.edit_form.to_job(if id == 0 { None } else { Some(id) }) {
                    Ok(job) => {
                        self.editing_job_id = None;
                        self.error_message = None;
                        if let Some(db) = &self.database {
                            let db = db.clone();
                            let is_new_job = id == 0;
                            return Task::perform(
                                async move {
                                    let result = if is_new_job {
                                        db.insert_job(&job).await.map(|_| ())
                                    } else {
                                        db.update_job(&job).await
                                    };
                                    match result {
                                        Ok(()) => {
                                            db.get_all_jobs().await.map_err(|e| e.to_string())
                                        }
                                        Err(e) => Err(e.to_string()),
                                    }
                                },
                                Message::JobsLoaded,
                            );
                        }
                    }
                    Err(e) => {
                        self.error_message = Some(e);
                    }
                }
                Task::none()
            }
            Message::CancelEdit => {
                self.editing_job_id = None;
                Task::none()
            }
            Message::DeleteJob(id) => {
                self.selected_job_id = None;
                if let Some(db) = &self.database {
                    let db = db.clone();
                    return Task::perform(
                        async move {
                            match db.delete_job(id).await {
                                Ok(()) => db.get_all_jobs().await.map_err(|e| e.to_string()),
                                Err(e) => Err(e.to_string()),
                            }
                        },
                        Message::JobsLoaded,
                    );
                }
                Task::none()
            }
            Message::ClearDatabase => {
                if let Some(db) = &self.database {
                    let db = db.clone();
                    return Task::perform(
                        async move {
                            match db.clear_all().await {
                                Ok(()) => Ok(Vec::new()),
                                Err(e) => Err(e.to_string()),
                            }
                        },
                        Message::JobsLoaded,
                    );
                }
                Task::none()
            }
            Message::DatabaseCleared(result) => {
                match result {
                    Ok(()) => {
                        self.jobs.clear();
                        self.selected_job_id = None;
                        self.error_message = None;
                    }
                    Err(e) => {
                        self.error_message = Some(e);
                    }
                }
                Task::none()
            }
            Message::ToggleTheme => {
                self.theme = match self.theme {
                    AppTheme::Light => AppTheme::Dark,
                    AppTheme::Dark => AppTheme::Light,
                };
                Task::none()
            }
            Message::SelectJob(id) => {
                self.selected_job_id = id;
                Task::none()
            }
            Message::CompanyChanged(value) => {
                self.edit_form.company = value;
                Task::none()
            }
            Message::PositionChanged(value) => {
                self.edit_form.position = value;
                Task::none()
            }
            Message::LocationChanged(value) => {
                self.edit_form.location = value;
                Task::none()
            }
            Message::DateChanged(value) => {
                self.edit_form.date = value;
                Task::none()
            }
            Message::SalaryMinChanged(value) => {
                self.edit_form.salary_min = value;
                Task::none()
            }
            Message::SalaryMaxChanged(value) => {
                self.edit_form.salary_max = value;
                Task::none()
            }
            Message::StatusChanged(status) => {
                self.edit_form.status = status;
                Task::none()
            }
            Message::CvPathChanged(value) => {
                self.edit_form.cv_path = value;
                Task::none()
            }
            Message::InterviewRoundChanged(value) => {
                self.edit_form.interview_round = value;
                Task::none()
            }
            Message::OfferAmountChanged(value) => {
                self.edit_form.offer_amount = value;
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let toolbar = row![
            button(text("Add Job")).on_press(Message::AddNewJob),
            button(text("Clear Database")).on_press(Message::ClearDatabase),
            button(text(match self.theme {
                AppTheme::Light => "Dark Mode",
                AppTheme::Dark => "Light Mode",
            }))
            .on_press(Message::ToggleTheme),
        ]
        .spacing(10);

        let main_content = row![
            container(self.view_table()).width(Length::Fill),
            self.view_cv_panel()
        ];

        let mut content = column![toolbar, main_content].spacing(20);

        if let Some(error) = &self.error_message {
            let theme = self.theme;
            content = content.push(
                container(
                    text(format!("Error: {error}")).style(move |_theme_ref| match theme {
                        AppTheme::Light => iced::widget::text::Style {
                            color: Some(iced::Color::from_rgb(0.8, 0.0, 0.0)),
                        },
                        AppTheme::Dark => iced::widget::text::Style {
                            color: Some(iced::Color::from_rgb(1.0, 0.4, 0.4)),
                        },
                    }),
                )
                .style(move |_theme| container::Style {
                    background: Some(iced::Background::Color(match theme {
                        AppTheme::Light => iced::Color::from_rgb(1.0, 0.9, 0.9),
                        AppTheme::Dark => iced::Color::from_rgb(0.3, 0.1, 0.1),
                    })),
                    border: iced::Border {
                        width: 1.0,
                        color: match theme {
                            AppTheme::Light => iced::Color::from_rgb(1.0, 0.5, 0.5),
                            AppTheme::Dark => iced::Color::from_rgb(0.8, 0.3, 0.3),
                        },
                        radius: 5.0.into(),
                    },
                    ..Default::default()
                })
                .padding(10),
            );
        }

        container(content)
            .padding(20)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    const fn theme(&self) -> Theme {
        self.theme.to_iced_theme()
    }
}

/// Runs the job tracker application.
///
/// Initializes and starts the Iced application with the job tracker UI.
/// This function blocks until the application is closed by the user.
///
/// # Errors
///
/// This function will return an error if:
/// - The windowing system cannot be initialized
/// - The graphics context cannot be created
/// - The application framework encounters a fatal error
///
/// # Examples
///
/// ```no_run
/// # use job_tracker::ui;
/// ui::run().unwrap();
/// ```
pub fn run() -> iced::Result {
    iced::application("Job Tracker", JobTrackerApp::update, JobTrackerApp::view)
        .theme(JobTrackerApp::theme)
        .run_with(JobTrackerApp::init)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_theme_conversion() {
        assert_eq!(AppTheme::Light.to_iced_theme(), Theme::Light);
        assert_eq!(AppTheme::Dark.to_iced_theme(), Theme::Dark);
    }

    #[test]
    fn test_status_selection_conversion() {
        assert_eq!(StatusSelection::Applied.to_string(), "Applied");
        assert_eq!(StatusSelection::Interview.to_string(), "Interview");
        assert_eq!(StatusSelection::Offer.to_string(), "Offer");
        assert_eq!(StatusSelection::Rejected.to_string(), "Rejected");
    }

    #[test]
    fn test_status_selection_from_status() {
        assert_eq!(
            StatusSelection::from_status(&Status::Applied),
            StatusSelection::Applied
        );
        assert_eq!(
            StatusSelection::from_status(&Status::Interview(1)),
            StatusSelection::Interview
        );
        assert_eq!(
            StatusSelection::from_status(&Status::Offer(50_000)),
            StatusSelection::Offer
        );
        assert_eq!(
            StatusSelection::from_status(&Status::Rejected),
            StatusSelection::Rejected
        );
    }

    #[test]
    fn test_edit_form_new() {
        let form = EditForm::new();
        assert_eq!(form.company, "");
        assert_eq!(form.position, "");
        assert_eq!(form.location, "");
        assert_eq!(form.date, "");
        assert_eq!(form.salary_min, "");
        assert_eq!(form.salary_max, "");
        assert_eq!(form.status, StatusSelection::Applied);
        assert_eq!(form.cv_path, "");
        assert_eq!(form.interview_round, "1");
        assert_eq!(form.offer_amount, "");
    }

    #[test]
    fn test_edit_form_from_job() {
        let job = JobApplication::new()
            .company("Test Corp")
            .position("Developer")
            .location("Remote")
            .salary(SalaryRange::new(50_000, 80_000))
            .status(Status::Interview(2))
            .date(2024, 1, 15)
            .cv("path/to/cv.pdf");

        let form = EditForm::from_job(&job);
        assert_eq!(form.company, "Test Corp");
        assert_eq!(form.position, "Developer");
        assert_eq!(form.location, "Remote");
        assert_eq!(form.salary_min, "50000");
        assert_eq!(form.salary_max, "80000");
        assert_eq!(form.status, StatusSelection::Interview);
        assert_eq!(form.interview_round, "2");
        assert_eq!(form.cv_path, "path/to/cv.pdf");
    }

    #[test]
    fn test_edit_form_to_job_applied() {
        let mut form = EditForm::new();
        form.company = "Test Corp".to_string();
        form.position = "Developer".to_string();
        form.location = "Remote".to_string();
        form.salary_min = "50000".to_string();
        form.salary_max = "80000".to_string();
        form.date = "2024-01-15".to_string();

        let job = form.to_job(Some(1)).unwrap();
        assert_eq!(job.id, Some(1));
        assert_eq!(job.company, "Test Corp");
        assert_eq!(job.position, "Developer");
        assert_eq!(job.location, "Remote");
        assert_eq!(job.salary.min, 50_000);
        assert_eq!(job.salary.max, 80_000);
        assert_eq!(job.status, Status::Applied);
        assert_eq!(job.date.unwrap().to_string(), "2024-01-15");
    }

    #[test]
    fn test_edit_form_to_job_interview() {
        let mut form = EditForm::new();
        form.company = "Test Corp".to_string();
        form.position = "Developer".to_string();
        form.location = "Remote".to_string();
        form.salary_min = "50000".to_string();
        form.salary_max = "80000".to_string();
        form.status = StatusSelection::Interview;
        form.interview_round = "3".to_string();

        let job = form.to_job(None).unwrap();
        assert_eq!(job.status, Status::Interview(3));
    }

    #[test]
    fn test_edit_form_to_job_offer() {
        let mut form = EditForm::new();
        form.company = "Test Corp".to_string();
        form.position = "Developer".to_string();
        form.location = "Remote".to_string();
        form.salary_min = "50000".to_string();
        form.salary_max = "80000".to_string();
        form.status = StatusSelection::Offer;
        form.offer_amount = "75000".to_string();

        let job = form.to_job(None).unwrap();
        assert_eq!(job.status, Status::Offer(75000));
    }

    #[test]
    fn test_edit_form_to_job_invalid_salary() {
        let mut form = EditForm::new();
        form.company = "Test Corp".to_string();
        form.position = "Developer".to_string();
        form.salary_min = "invalid".to_string();
        form.salary_max = "80000".to_string();

        let result = form.to_job(None);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid minimum salary"));
    }

    #[test]
    fn test_edit_form_to_job_invalid_date() {
        let mut form = EditForm::new();
        form.company = "Test Corp".to_string();
        form.position = "Developer".to_string();
        form.salary_min = "50000".to_string();
        form.salary_max = "80000".to_string();
        form.date = "invalid-date".to_string();

        let result = form.to_job(None);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid date format"));
    }

    #[test]
    fn test_edit_form_to_job_invalid_interview_round() {
        let mut form = EditForm::new();
        form.company = "Test Corp".to_string();
        form.position = "Developer".to_string();
        form.salary_min = "50000".to_string();
        form.salary_max = "80000".to_string();
        form.status = StatusSelection::Interview;
        form.interview_round = "invalid".to_string();

        let result = form.to_job(None);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid interview round"));
    }

    #[test]
    fn test_edit_form_to_job_invalid_offer_amount() {
        let mut form = EditForm::new();
        form.company = "Test Corp".to_string();
        form.position = "Developer".to_string();
        form.salary_min = "50000".to_string();
        form.salary_max = "80000".to_string();
        form.status = StatusSelection::Offer;
        form.offer_amount = "invalid".to_string();

        let result = form.to_job(None);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid offer amount"));
    }

    #[test]
    fn test_app_creation() {
        let app = JobTrackerApp::new();
        assert!(app.database.is_none());
        assert!(app.jobs.is_empty());
        assert_eq!(app.selected_job_id, None);
        assert_eq!(app.editing_job_id, None);
        assert_eq!(app.theme, AppTheme::Light);
        assert_eq!(app.error_message, None);
    }
}
