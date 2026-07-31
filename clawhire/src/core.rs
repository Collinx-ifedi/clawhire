//! Core module for ClawHire[span_4](start_span)[span_4](end_span)
//!
//! This module contains the application runtime, shared state, global configuration,
//! job models, and event system.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use config::{Config, Environment, File};
use log::{error, info, LevelFilter};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::{Path, PathBuf}, sync::Arc};
use tokio::sync::RwLock;
use uuid::Uuid;
use thiserror::Error;

// ============================================================================
// Errors
// ============================================================================

/// Core errors for ClawHire application operations.
#[derive(Error, Debug)]
pub enum CoreError {
    #[error("Job not found: {0}")]
    JobNotFound(String),
    #[error("Invoice not found: {0}")]
    InvoiceNotFound(String),
    #[error("Report not found: {0}")]
    ReportNotFound(String),
    #[error("Configuration error: {0}")]
    ConfigError(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

// ============================================================================
// Enums[span_5](start_span)[span_5](end_span)
// ============================================================================

/// Represents the current state of a Job in the system.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum JobStatus {
    Created,
    Quoted,
    AwaitingPayment,
    PaymentDetected,
    Executing,
    GeneratingReport,
    Completed,
    Failed,
    Cancelled,
    Archived,
}

/// The available blockchain services.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ServiceType {
    SmartContractReview,
    OnChainIntelligence,
}

/// Represents the evaluated risk level of an asset or smart contract.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RiskLevel {
    VeryLow,
    Low,
    Medium,
    High,
    Critical,
}

/// Events that can be emitted within the application.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum EventType {
    JobCreated,
    InvoiceCreated,
    PaymentDetected,
    JobStarted,
    JobCompleted,
    ReportGenerated,
    ApplicationStarted,
    ApplicationStopped,
}

// ============================================================================
// Configuration Models[span_6](start_span)[span_6](end_span)[span_7](start_span)[span_7](end_span)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationConfig {
    pub name: String,
    pub version: String,
    pub environment: String,
    pub debug: bool,
    pub host: String,
    pub port: u16,
    pub timezone: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeConfig {
    pub shutdown_timeout_seconds: u64,
    pub worker_threads: usize,
    pub max_concurrent_jobs: usize,
    pub enable_graceful_shutdown: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub log_file: String,
    pub payment_log: String,
    pub enable_console: bool,
    pub enable_file: bool,
    pub rotation: String,
    pub max_log_files: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolanaConfig {
    pub network: String,
    pub rpc_url: String,
    pub commitment: String,
    pub request_timeout_seconds: u64,
    pub max_retries: usize,
    pub retry_delay_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletConfig {
    pub merchant_wallet: String,
    pub currency: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentConfig {
    pub enabled: bool,
    pub poll_interval_seconds: u64,
    pub confirmation_count: usize,
    pub payment_timeout_seconds: u64,
    pub invoice_expiry_minutes: u64,
    pub allow_partial_payment: bool,
    pub allow_overpayment: bool,
    pub auto_start_job: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIConfig {
    pub provider: String,
    pub model: String,
    #[serde(default)]
    pub api_key: String,
    pub temperature: f32,
    pub max_tokens: usize,
    pub request_timeout_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportConfig {
    pub enabled: bool,
    pub output_directory: String,
    pub template_directory: String,
    pub generate_markdown: bool,
    pub generate_pdf: bool,
    pub keep_history: bool,
    pub compress_reports: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadConfig {
    pub directory: String,
    pub max_file_size_mb: u64,
    pub allowed_extensions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub invoice_directory: String,
    pub history_directory: String,
    pub report_directory: String,
    pub upload_directory: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub enable_rate_limiting: bool,
    pub requests_per_minute: u32,
    pub enable_cors: bool,
    pub allowed_origins: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalConfig {
    pub show_timestamp: bool,
    pub show_banner: bool,
    pub animated_cursor: bool,
    pub typing_speed: u64,
    pub enable_colors: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureFlags {
    pub smart_contract_review: bool,
    pub onchain_intelligence: bool,
    pub pdf_reports: bool,
    pub markdown_reports: bool,
    pub payment_monitor: bool,
    pub job_history: bool,
}

/// Global Application Configuration mapping to configs/app.toml[span_8](start_span)[span_8](end_span).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub application: ApplicationConfig,
    pub runtime: RuntimeConfig,
    pub logging: LoggingConfig,
    pub solana: SolanaConfig,
    pub wallet: WalletConfig,
    pub payments: PaymentConfig,
    pub ai: AIConfig,
    pub reports: ReportConfig,
    pub uploads: UploadConfig,
    pub storage: StorageConfig,
    pub security: SecurityConfig,
    pub terminal: TerminalConfig,
    pub features: FeatureFlags,
}

// ============================================================================
// Core Models[span_9](start_span)[span_9](end_span)
// ============================================================================

/// Represents an automated AI job within the system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub id: String,
    pub status: JobStatus,
    pub service: ServiceType,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub input_source: String,
    pub invoice_id: Option<String>,
    pub report_id: Option<String>,
    pub metadata: HashMap<String, String>,
}

/// Represents a customer invoice waiting for blockchain payment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Invoice {
    pub invoice_id: String,
    pub wallet: String,
    pub amount: f64,
    pub currency: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub signature: Option<String>,
}

/// Holds metadata about a generated professional report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportMetadata {
    pub report_id: String,
    pub job_id: String,
    pub filename: String,
    pub generated_at: DateTime<Utc>,
    pub service: ServiceType,
}

/// An application event used for lifecycle tracking and observability.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationEvent {
    pub event_type: EventType,
    pub job_id: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub message: String,
}

// ============================================================================
// Persistence
// ============================================================================
//
// RustCLIAdapter (the Python frontend's default backend adapter) spawns a
// brand-new OS process for every single command. Without persistence, each
// invocation would start from empty in-memory maps and lose every job and
// invoice created by the previous invocation. These helpers back the job
// and invoice pools with JSON files on disk so state survives across
// process boundaries.

/// On-disk snapshot of the job pools.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct JobsSnapshot {
    #[serde(default)]
    active: HashMap<String, Job>,
    #[serde(default)]
    completed: HashMap<String, Job>,
}

/// Reads a JSON file into `T`, returning `T::default()` if the file is
/// missing or unreadable (e.g. first run, or a corrupted/partial write).
async fn read_json_or_default<T: for<'de> Deserialize<'de> + Default>(path: &Path) -> T {
    match tokio::fs::read(path).await {
        Ok(bytes) => serde_json::from_slice(&bytes).unwrap_or_default(),
        Err(_) => T::default(),
    }
}

/// Writes `value` to `path` as pretty JSON, atomically (write to a temp
/// file in the same directory, then rename) so a crash mid-write can never
/// leave a truncated/corrupt state file behind.
async fn write_json_atomic<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await.map_err(CoreError::IoError)?;
    }
    let bytes = serde_json::to_vec_pretty(value)
        .map_err(|e| CoreError::ConfigError(format!("Failed to serialize state: {}", e)))?;
    let tmp_path = path.with_extension("json.tmp");
    tokio::fs::write(&tmp_path, &bytes).await.map_err(CoreError::IoError)?;
    tokio::fs::rename(&tmp_path, path).await.map_err(CoreError::IoError)?;
    Ok(())
}

// ============================================================================
// Application State
// ============================================================================

/// Thread-safe shared state for the application[span_10](start_span)[span_10](end_span).
#[derive(Debug)]
pub struct AppState {
    pub config: AppConfig,
    pub active_jobs: Arc<RwLock<HashMap<String, Job>>>,
    pub completed_jobs: Arc<RwLock<HashMap<String, Job>>>,
    pub invoice_cache: Arc<RwLock<HashMap<String, Invoice>>>,
    pub report_cache: Arc<RwLock<HashMap<String, ReportMetadata>>>,
    pub event_history: Arc<RwLock<Vec<ApplicationEvent>>>,
    jobs_path: PathBuf,
    invoices_path: PathBuf,
}

impl AppState {
    /// Persists the current job pools to disk.
    async fn persist_jobs(&self) -> Result<()> {
        let snapshot = JobsSnapshot {
            active: self.active_jobs.read().await.clone(),
            completed: self.completed_jobs.read().await.clone(),
        };
        write_json_atomic(&self.jobs_path, &snapshot).await
    }

    /// Persists the current invoice cache to disk.
    async fn persist_invoices(&self) -> Result<()> {
        let cache = self.invoice_cache.read().await.clone();
        write_json_atomic(&self.invoices_path, &cache).await
    }
}

// ============================================================================
// Application Core
// ============================================================================

/// The main application wrapper containing state and lifecycle methods.
#[derive(Debug, Clone)]
pub struct App {
    pub state: Arc<AppState>,
}

impl App {
    /// Initializes a new instance of the App by loading configurations and state.
    pub async fn new() -> Result<Self> {
        load_environment()?;
        let config = load_configuration().context("Failed to load application configuration")?;
        
        initialize_logger(&config.logging)?;

        let jobs_path = Path::new(&config.storage.history_directory).join("jobs.json");
        let invoices_path = Path::new(&config.storage.invoice_directory).join("invoices.json");

        let jobs_snapshot: JobsSnapshot = read_json_or_default(&jobs_path).await;
        let invoice_cache: HashMap<String, Invoice> = read_json_or_default(&invoices_path).await;

        let state = AppState {
            config,
            active_jobs: Arc::new(RwLock::new(jobs_snapshot.active)),
            completed_jobs: Arc::new(RwLock::new(jobs_snapshot.completed)),
            invoice_cache: Arc::new(RwLock::new(invoice_cache)),
            report_cache: Arc::new(RwLock::new(HashMap::new())),
            event_history: Arc::new(RwLock::new(Vec::new())),
            jobs_path,
            invoices_path,
        };

        let app = Self {
            state: Arc::new(state),
        };

        app.emit_event(
            EventType::ApplicationStarted, 
            None, 
            "ClawHire Core Runtime Initialized".to_string()
        ).await?;

        Ok(app)
    }

    /// Prepares required directories and performs system readiness checks.
    pub async fn initialize(&self) -> Result<()> {
        let storage = &self.state.config.storage;
        ensure_directory(&storage.invoice_directory).await?;
        ensure_directory(&storage.history_directory).await?;
        ensure_directory(&storage.report_directory).await?;
        ensure_directory(&storage.upload_directory).await?;
        
        info!("System storage directories initialized.");
        Ok(())
    }

    /// Handles graceful shutdown of the application.
    pub async fn shutdown(&self) -> Result<()> {
        info!("Initiating ClawHire shutdown sequence...");
        self.emit_event(
            EventType::ApplicationStopped, 
            None, 
            "Application shutting down".to_string()
        ).await?;
        Ok(())
    }

    /// Emits a system event and appends it to the internal event history.
    pub async fn emit_event(&self, event_type: EventType, job_id: Option<String>, message: String) -> Result<()> {
        let event = ApplicationEvent {
            event_type,
            job_id,
            timestamp: utc_now(),
            message,
        };
        
        info!("[EVENT] {:?} - {}", event.event_type, event.message);
        let mut history = self.state.event_history.write().await;
        history.push(event);
        Ok(())
    }

    /// Creates a new job and stores it in the active jobs pool.
    pub async fn create_job(&self, service: ServiceType, input_source: String, metadata: HashMap<String, String>) -> Result<Job> {
        let job = Job {
            id: generate_uuid(),
            status: JobStatus::Created,
            service,
            created_at: utc_now(),
            updated_at: utc_now(),
            input_source,
            invoice_id: None,
            report_id: None,
            metadata,
        };

        {
            let mut active_jobs = self.state.active_jobs.write().await;
            active_jobs.insert(job.id.clone(), job.clone());
        }
        self.state.persist_jobs().await?;

        self.emit_event(EventType::JobCreated, Some(job.id.clone()), format!("Job created for service {:?}", job.service)).await?;
        Ok(job)
    }

    /// Updates the status of an existing job.
    pub async fn update_job_status(&self, job_id: &str, new_status: JobStatus) -> Result<()> {
        {
            let mut active_jobs = self.state.active_jobs.write().await;

            let job = active_jobs
                .get_mut(job_id)
                .ok_or_else(|| CoreError::JobNotFound(job_id.to_string()))?;

            job.status = new_status.clone();
            job.updated_at = utc_now();
        }
        self.state.persist_jobs().await?;

        // Handle specialized status events
        match new_status {
            JobStatus::Executing => {
                self.emit_event(EventType::JobStarted, Some(job_id.to_string()), "Job execution started".to_string()).await?;
            },
            JobStatus::Completed => {
                self.emit_event(EventType::JobCompleted, Some(job_id.to_string()), "Job execution completed".to_string()).await?;
            },
            _ => {}
        }
        
        Ok(())
    }

    /// Links a generated invoice to the job that requested it, so payment
    /// confirmation can look the job back up from the invoice alone.
    pub async fn set_job_invoice(&self, job_id: &str, invoice_id: &str) -> Result<()> {
        {
            let mut active_jobs = self.state.active_jobs.write().await;
            let job = active_jobs
                .get_mut(job_id)
                .ok_or_else(|| CoreError::JobNotFound(job_id.to_string()))?;
            job.invoice_id = Some(invoice_id.to_string());
            job.updated_at = utc_now();
        }
        self.state.persist_jobs().await
    }

    /// Attaches a generated report's ID to the job it was produced for.
    pub async fn set_job_report(&self, job_id: &str, report_id: &str) -> Result<()> {
        {
            let mut active_jobs = self.state.active_jobs.write().await;
            let job = active_jobs
                .get_mut(job_id)
                .ok_or_else(|| CoreError::JobNotFound(job_id.to_string()))?;
            job.report_id = Some(report_id.to_string());
            job.updated_at = utc_now();
        }
        self.state.persist_jobs().await
    }

    /// Finds the job that a given invoice was issued for, if any.
    pub async fn find_job_by_invoice(&self, invoice_id: &str) -> Option<Job> {
        let active = self.state.active_jobs.read().await;
        if let Some(job) = active.values().find(|j| j.invoice_id.as_deref() == Some(invoice_id)) {
            return Some(job.clone());
        }
        let completed = self.state.completed_jobs.read().await;
        completed.values().find(|j| j.invoice_id.as_deref() == Some(invoice_id)).cloned()
    }

    /// Lists every job, active and completed.
    pub async fn list_jobs(&self) -> Vec<Job> {
        let mut jobs: Vec<Job> = self.state.active_jobs.read().await.values().cloned().collect();
        jobs.extend(self.state.completed_jobs.read().await.values().cloned());
        jobs
    }

    /// Caches a generated invoice.
    pub async fn store_invoice(&self, invoice: Invoice) -> Result<()> {
        let invoice_id = invoice.invoice_id.clone();
        {
            let mut cache = self.state.invoice_cache.write().await;
            cache.insert(invoice.invoice_id.clone(), invoice);
        }
        self.state.persist_invoices().await?;

        self.emit_event(EventType::InvoiceCreated, None, format!("Invoice {} stored", invoice_id)).await?;
        Ok(())
    }

    /// Caches the metadata for a generated report.
    pub async fn store_report(&self, report: ReportMetadata) -> Result<()> {
        let mut cache = self.state.report_cache.write().await;
        let job_id = report.job_id.clone();
        cache.insert(report.report_id.clone(), report);
        
        self.emit_event(EventType::ReportGenerated, Some(job_id), "Report successfully stored".to_string()).await?;
        Ok(())
    }

    /// Retrieves a job by ID from the active or completed job pools.
    pub async fn find_job(&self, job_id: &str) -> Result<Job> {
        let active = self.state.active_jobs.read().await;
        if let Some(job) = active.get(job_id) {
            return Ok(job.clone());
        }
        
        let completed = self.state.completed_jobs.read().await;
        if let Some(job) = completed.get(job_id) {
            return Ok(job.clone());
        }

        Err(CoreError::JobNotFound(job_id.to_string()).into())
    }

    /// Retrieves an invoice by ID.
    pub async fn find_invoice(&self, invoice_id: &str) -> Result<Invoice> {
        let cache = self.state.invoice_cache.read().await;
        cache.get(invoice_id)
            .cloned()
            .ok_or_else(|| CoreError::InvoiceNotFound(invoice_id.to_string()).into())
    }

    /// Retrieves report metadata by ID.
    pub async fn find_report(&self, report_id: &str) -> Result<ReportMetadata> {
        let cache = self.state.report_cache.read().await;
        cache.get(report_id)
            .cloned()
            .ok_or_else(|| CoreError::ReportNotFound(report_id.to_string()).into())
    }

    /// Moves a job from the active pool to the completed/archived pool.
    pub async fn archive_job(&self, job_id: &str) -> Result<()> {
        {
            let mut active = self.state.active_jobs.write().await;
            let mut completed = self.state.completed_jobs.write().await;

            let mut job = active.remove(job_id).ok_or_else(|| CoreError::JobNotFound(job_id.to_string()))?;
            job.status = JobStatus::Archived;
            job.updated_at = utc_now();

            completed.insert(job.id.clone(), job);
        }
        self.state.persist_jobs().await
    }
}

// ============================================================================
// Utility Functions
// ============================================================================

/// Loads application configuration from TOML files and environment variables.
pub fn load_configuration() -> Result<AppConfig> {
    let config = Config::builder()
        .add_source(File::with_name("configs/app.toml").required(true))
        .add_source(Environment::with_prefix("APP").separator("__"))
        .build()
        .map_err(|e| CoreError::ConfigError(e.to_string()))?;

    let app_config: AppConfig = config.try_deserialize()
        .map_err(|e| CoreError::ConfigError(e.to_string()))?;

    Ok(app_config)
}

/// Initializes the global application logger based on configuration settings.
pub fn initialize_logger(config: &LoggingConfig) -> Result<()> {
    let mut builder = env_logger::Builder::new();
    
    let level = match config.level.to_lowercase().as_str() {
        "debug" => LevelFilter::Debug,
        "warn" => LevelFilter::Warn,
        "error" => LevelFilter::Error,
        "trace" => LevelFilter::Trace,
        _ => LevelFilter::Info,
    };
    
    builder.filter_level(level);
    
    if config.enable_console {
        builder.init();
    }
    
    Ok(())
}

/// Generates a universally unique identifier (UUID v4) as a String.
pub fn generate_uuid() -> String {
    Uuid::new_v4().to_string()
}

/// Returns the current UTC date and time.
pub fn utc_now() -> DateTime<Utc> {
    Utc::now()
}

/// Ensures a directory exists, creating it if necessary.
pub async fn ensure_directory<P: AsRef<Path>>(path: P) -> Result<()> {
    if !path.as_ref().exists() {
        tokio::fs::create_dir_all(&path).await
            .map_err(CoreError::IoError)?;
    }
    Ok(())
}

/// Loads environment variables from a `.env` file into the process environment[span_11](start_span)[span_11](end_span)[span_12](start_span)[span_12](end_span).
pub fn load_environment() -> Result<()> {
    match dotenvy::dotenv() {
        Ok(_) => Ok(()),
        Err(e) => {
            // It's acceptable if the .env file is missing in production environments
            // where variables are injected directly.
            log::warn!("Could not load .env file: {}", e);
            Ok(())
        }
    }
}
