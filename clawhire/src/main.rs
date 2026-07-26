//! Core module for ClawHire
//!
//! This module contains the application runtime, shared state, global configuration,
//! job models, and event system.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use config::{Config, Environment, File};
use log::{error, info, LevelFilter};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::Path, sync::Arc};
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
// Enums
// ============================================================================

/// Represents the current state of a Job in the system.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum JobStatus {
    Created,
    Quoted,
    AwaitingPayment,
    PaymentConfirmed,
    Executing,
    GeneratingReport,
    Completed,
    Archived,
}

/// The available blockchain services.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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
// Configuration Models
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

/// Global Application Configuration mapping to configs/app.toml.
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
// Core Models
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
// Application State
// ============================================================================

/// Thread-safe shared state for the application.
#[derive(Debug)]
pub struct AppState {
    pub config: AppConfig,
    pub active_jobs: Arc<RwLock<HashMap<String, Job>>>,
    pub completed_jobs: Arc<RwLock<HashMap<String, Job>>>,
    pub invoice_cache: Arc<RwLock<HashMap<String, Invoice>>>,
    pub report_cache: Arc<RwLock<HashMap<String, ReportMetadata>>>,
    pub event_history: Arc<RwLock<Vec<ApplicationEvent>>>,
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

        let state = AppState {
            config,
            active_jobs: Arc::new(RwLock::new(HashMap::new())),
            completed_jobs: Arc::new(RwLock::new(HashMap::new())),
            invoice_cache: Arc::new(RwLock::new(HashMap::new())),
            report_cache: Arc::new(RwLock::new(HashMap::new())),
            event_history: Arc::new(RwLock::new(Vec::new())),
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

        let mut active_jobs = self.state.active_jobs.write().await;
        active_jobs.insert(job.id.clone(), job.clone());
        
        self.emit_event(EventType::JobCreated, Some(job.id.clone()), format!("Job created for service {:?}", job.service)).await?;
        Ok(job)
    }

    /// Updates the status of an existing job.
    pub async fn update_job_status(&self, job_id: &str, new_status: JobStatus) -> Result<()> {
        let mut active_jobs = self.state.active_jobs.write().await;
        
        let job = active_jobs
            .get_mut(job_id)
            .ok_or_else(|| CoreError::JobNotFound(job_id.to_string()))?;
            
        job.status = new_status.clone();
        job.updated_at = utc_now();
        
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

    /// Caches a generated invoice.
    pub async fn store_invoice(&self, invoice: Invoice) -> Result<()> {
        let mut cache = self.state.invoice_cache.write().await;
        let invoice_id = invoice.invoice_id.clone();
        cache.insert(invoice.invoice_id.clone(), invoice);
        
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
        let mut active = self.state.active_jobs.write().await;
        let mut completed = self.state.completed_jobs.write().await;
        
        let mut job = active.remove(job_id).ok_or_else(|| CoreError::JobNotFound(job_id.to_string()))?;
        job.status = JobStatus::Archived;
        job.updated_at = utc_now();
        
        completed.insert(job.id.clone(), job);
        Ok(())
    }
}

// ============================================================================
// Utility Functions
// ============================================================================

/// Loads application configuration from TOML files and environment variables.
pub fn load_configuration() -> Result<AppConfig> {
    let config = Config::builder()
        .add_source(File::with_name("configs/app.toml").required(true))
        .add_source(Environment::with_prefix("APP").separator("_"))
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

/// Loads environment variables from a `.env` file into the process environment.
pub fn load_environment() -> Result<()> {
    match dotenvy::dotenv() {
        Ok(_) => Ok(()),
        Err(e) => {
            log::warn!("Could not load .env file: {}", e);
            Ok(())
        }
    }
}

// ============================================================================
// Main Entry Point
// ============================================================================

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize the application and capture any startup errors
    let app = match App::new().await {
        Ok(app) => app,
        Err(e) => {
            // Utilizing the error! macro here consumes the import, fixing the Clippy lint
            error!("CRITICAL: Failed to initialize App instances: {:?}", e);
            std::process::exit(1);
        }
    };

    if let Err(e) = app.initialize().await {
        error!("CRITICAL: Failed to initialize application directories/state: {:?}", e);
        std::process::exit(1);
    }

    info!("ClawHire application is running. Awaiting shutdown signal...");

    // Block main thread until a termination signal is received
    match tokio::signal::ctrl_c().await {
        Ok(()) => {
            info!("Shutdown signal received.");
        }
        Err(err) => {
            error!("Unable to listen for shutdown signal: {}", err);
        }
    }

    if let Err(e) = app.shutdown().await {
        error!("Error during graceful shutdown: {:?}", e);
    }

    info!("ClawHire shutdown complete.");
    Ok(())
}
