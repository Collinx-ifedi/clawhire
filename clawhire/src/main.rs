//! ClawHire CLI entry point.
//!
//! This binary is invoked two different ways:
//!
//!   - With no arguments: runs as a long-lived background process. This is
//!     how `start.sh` launches it, and is kept for that reason - there is
//!     no other work for it to do in this mode, since every real operation
//!     now happens through a one-shot subcommand invocation below.
//!   - With a subcommand: `RustCLIAdapter` (the Python frontend's default
//!     backend adapter, see ui/api.py) spawns this binary as a one-shot
//!     subprocess per request. On success it expects a single JSON object
//!     (or, for report downloads, raw file bytes) on stdout and exit code
//!     0. On failure it expects a non-zero exit code and a message on
//!     stderr - nothing else should ever be written to stdout, since the
//!     Python side parses stdout directly as JSON.
//!
//! Job, invoice, and report state is persisted to disk (see `core.rs`,
//! `payments.rs`, `reports.rs`) specifically so that state survives across
//! these separate per-command process invocations.

mod core;
mod payments;
mod reports;
mod services;

use crate::core::{App, AIConfig, Invoice, Job, JobStatus, ServiceType};
use crate::payments::PaymentEngine;
use crate::reports::{Report, ReportGenerator, ReportType};
use crate::services::{
    AIProvider, AnthropicProvider, OnChainIntelligenceService, OpenAIProvider,
    OpenRouterProvider, ServiceManager, ServiceRegistry, ServiceRequest,
    SmartContractReviewService,
};

use anyhow::{anyhow, Context, Result};
use chrono::Utc;
use clap::{Parser, Subcommand};
use log::{error, info, warn};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::io::Write;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;

// ============================================================================
// CLI definition
// ============================================================================

#[derive(Parser)]
#[command(name = "clawhire", about = "ClawHire backend")]
struct Cli {
    /// Output format. "json" for structured responses, "raw" for file bytes
    /// (only meaningful for `reports download`).
    #[arg(long, default_value = "json")]
    #[allow(dead_code)]
    format: String,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Basic liveness check.
    Health,
    /// Build and version info.
    Version,
    /// Connection and job summary.
    Status,
    /// Merchant wallet operations.
    Wallet {
        #[command(subcommand)]
        action: WalletAction,
    },
    /// Available blockchain services.
    Services {
        #[command(subcommand)]
        action: ServicesAction,
    },
    /// Job lifecycle operations.
    Jobs {
        #[command(subcommand)]
        action: JobsAction,
    },
    /// Invoice lifecycle operations.
    Invoices {
        #[command(subcommand)]
        action: InvoicesAction,
    },
    /// Payment monitoring operations.
    Payments {
        #[command(subcommand)]
        action: PaymentsAction,
    },
    /// Generated report operations.
    Reports {
        #[command(subcommand)]
        action: ReportsAction,
    },
    /// Runtime configuration.
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
}

#[derive(Subcommand)]
enum WalletAction {
    Info,
}

#[derive(Subcommand)]
enum ServicesAction {
    List,
}

#[derive(Subcommand)]
enum JobsAction {
    Create {
        #[arg(long)]
        service: String,
        #[arg(long)]
        target: String,
    },
    Get {
        job_id: String,
    },
    List,
}

#[derive(Subcommand)]
enum InvoicesAction {
    Create {
        #[arg(long = "job-id")]
        job_id: String,
    },
    Get {
        invoice_id: String,
    },
}

#[derive(Subcommand)]
enum PaymentsAction {
    Wait {
        #[arg(long = "invoice-id")]
        invoice_id: String,
    },
}

#[derive(Subcommand)]
enum ReportsAction {
    List {
        #[arg(long = "job-id")]
        job_id: String,
    },
    Download {
        report_id: String,
        #[arg(long = "type")]
        format_type: String,
    },
}

#[derive(Subcommand)]
enum ConfigAction {
    Get,
}

// ============================================================================
// Output
// ============================================================================

enum Output {
    Json(Value),
    Raw(Vec<u8>),
}

// ============================================================================
// Entry point
// ============================================================================

#[tokio::main]
async fn main() {
    // Every error path funnels through here so stdout is guaranteed to stay
    // clean (no partial JSON, no stray log lines) whenever something fails -
    // RustCLIAdapter treats a non-zero exit as failure and only looks at
    // stderr for the message.
    if let Err(e) = run().await {
        eprintln!("{:#}", e);
        std::process::exit(1);
    }
}

async fn run() -> Result<()> {
    let cli = Cli::parse();

    let app = Arc::new(App::new().await.context("Failed to initialize application")?);
    app.initialize()
        .await
        .context("Failed to initialize application storage")?;

    let command = match cli.command {
        None => {
            info!("ClawHire backend daemon started. Awaiting shutdown signal...");
            tokio::signal::ctrl_c()
                .await
                .context("Failed to listen for shutdown signal")?;
            app.shutdown().await?;
            return Ok(());
        }
        Some(command) => command,
    };

    match dispatch(app, command).await? {
        Output::Json(value) => {
            println!(
                "{}",
                serde_json::to_string(&value).context("Failed to serialize response")?
            );
        }
        Output::Raw(bytes) => {
            std::io::stdout()
                .write_all(&bytes)
                .context("Failed to write raw output")?;
        }
    }

    Ok(())
}

async fn dispatch(app: Arc<App>, command: Command) -> Result<Output> {
    match command {
        Command::Health => cmd_health().await,
        Command::Version => cmd_version(&app).await,
        Command::Status => cmd_status(&app).await,
        Command::Wallet { action: WalletAction::Info } => cmd_wallet_info(&app).await,
        Command::Services { action: ServicesAction::List } => cmd_services_list().await,
        Command::Jobs { action } => match action {
            JobsAction::Create { service, target } => cmd_jobs_create(&app, &service, &target).await,
            JobsAction::Get { job_id } => cmd_jobs_get(&app, &job_id).await,
            JobsAction::List => cmd_jobs_list(&app).await,
        },
        Command::Invoices { action } => match action {
            InvoicesAction::Create { job_id } => cmd_invoices_create(&app, &job_id).await,
            InvoicesAction::Get { invoice_id } => cmd_invoices_get(&app, &invoice_id).await,
        },
        Command::Payments { action } => match action {
            PaymentsAction::Wait { invoice_id } => cmd_payments_wait(&app, &invoice_id).await,
        },
        Command::Reports { action } => match action {
            ReportsAction::List { job_id } => cmd_reports_list(&app, &job_id).await,
            ReportsAction::Download { report_id, format_type } => {
                cmd_reports_download(&app, &report_id, &format_type).await
            }
        },
        Command::Config { action: ConfigAction::Get } => cmd_config_get(&app).await,
    }
}

// ============================================================================
// Commands
// ============================================================================

async fn cmd_health() -> Result<Output> {
    Ok(Output::Json(json!({
        "status": "healthy",
        "uptime": 0,
        "timestamp": Utc::now().to_rfc3339(),
    })))
}

async fn cmd_version(app: &Arc<App>) -> Result<Output> {
    Ok(Output::Json(json!({
        "version": app.state.config.application.version,
        "build_hash": option_env!("CLAWHIRE_BUILD_HASH").unwrap_or("unknown"),
        "environment": app.state.config.application.environment,
    })))
}

async fn cmd_status(app: &Arc<App>) -> Result<Output> {
    let jobs = app.list_jobs().await;
    let active_count = jobs
        .iter()
        .filter(|j| {
            !matches!(
                j.status,
                JobStatus::Completed | JobStatus::Archived | JobStatus::Failed | JobStatus::Cancelled
            )
        })
        .count();

    Ok(Output::Json(json!({
        "connection": "CONNECTED",
        "backend_mode": "CLI",
        "active_jobs": active_count,
    })))
}

/// Wallet balance lookups must never take down the whole command - this is
/// called during the frontend's boot sequence, so any failure here (an
/// unreachable RPC endpoint, a misconfigured wallet address) falls back to
/// a balance of 0 rather than propagating as a hard error.
async fn cmd_wallet_info(app: &Arc<App>) -> Result<Output> {
    let wallet_cfg = &app.state.config.wallet;
    let solana_cfg = &app.state.config.solana;

    let balance = match solana_sdk::pubkey::Pubkey::from_str(&wallet_cfg.merchant_wallet) {
        Ok(pubkey) => {
            let client = solana_client::nonblocking::rpc_client::RpcClient::new(solana_cfg.rpc_url.clone());
            match client.get_balance(&pubkey).await {
                Ok(lamports) => lamports as f64 / 1_000_000_000.0,
                Err(e) => {
                    warn!("Failed to fetch wallet balance: {}", e);
                    0.0
                }
            }
        }
        Err(_) => {
            warn!("Configured merchant wallet is not a valid Solana address; reporting balance as 0.");
            0.0
        }
    };

    Ok(Output::Json(json!({
        "address": wallet_cfg.merchant_wallet,
        "network": solana_cfg.network,
        "balance": balance,
    })))
}

async fn cmd_services_list() -> Result<Output> {
    let pricing = payments::load_pricing_configuration()
        .map_err(|e| anyhow!("Failed to load pricing configuration: {}", e))?;

    let mut list = Vec::new();
    if pricing.smart_contract_review.enabled {
        list.push(json!({
            "name": "Smart Contract Security Review",
            "service_type": "SmartContractReview",
            "description": "Professional AI-powered security audit for Solana smart contracts.",
            "base_price": pricing.smart_contract_review.price,
            "currency": pricing.smart_contract_review.currency,
        }));
    }
    if pricing.onchain_intelligence.enabled {
        list.push(json!({
            "name": "On-chain Intelligence Report",
            "service_type": "OnChainIntelligence",
            "description": "Professional AI-powered wallet and transaction intelligence report.",
            "base_price": pricing.onchain_intelligence.price,
            "currency": pricing.onchain_intelligence.currency,
        }));
    }

    Ok(Output::Json(Value::Array(list)))
}

async fn cmd_jobs_create(app: &Arc<App>, service: &str, target: &str) -> Result<Output> {
    let service_type = parse_service_type(service)?;
    let mut metadata = HashMap::new();
    metadata.insert("target".to_string(), target.to_string());

    let job = app
        .create_job(service_type, target.to_string(), metadata)
        .await
        .map_err(|e| anyhow!("Failed to create job: {}", e))?;

    Ok(Output::Json(job_to_json(&job)))
}

async fn cmd_jobs_get(app: &Arc<App>, job_id: &str) -> Result<Output> {
    let job = app
        .find_job(job_id)
        .await
        .map_err(|e| anyhow!("Job not found: {}", e))?;
    Ok(Output::Json(job_to_json(&job)))
}

async fn cmd_jobs_list(app: &Arc<App>) -> Result<Output> {
    let jobs = app.list_jobs().await;
    let list: Vec<Value> = jobs.iter().map(job_to_json).collect();
    Ok(Output::Json(Value::Array(list)))
}

async fn cmd_invoices_create(app: &Arc<App>, job_id: &str) -> Result<Output> {
    let job = app
        .find_job(job_id)
        .await
        .map_err(|e| anyhow!("Job not found: {}", e))?;

    let payment_engine = build_payment_engine(app)?;
    let invoice = payment_engine
        .create_invoice(job_id, job.service.clone())
        .await
        .map_err(|e| anyhow!("Failed to create invoice: {}", e))?;

    Ok(Output::Json(invoice_to_json(&invoice, job_id)))
}

async fn cmd_invoices_get(app: &Arc<App>, invoice_id: &str) -> Result<Output> {
    let invoice = app
        .find_invoice(invoice_id)
        .await
        .map_err(|e| anyhow!("Invoice not found: {}", e))?;
    let job_id = app
        .find_job_by_invoice(invoice_id)
        .await
        .map(|j| j.id)
        .unwrap_or_default();
    Ok(Output::Json(invoice_to_json(&invoice, &job_id)))
}

/// Checks a pending invoice for payment once. If payment has been detected
/// and the associated job hasn't already moved past that point, this also
/// kicks off the actual service execution and report generation - there is
/// no separate "start job" command in RustCLIAdapter's API surface, so this
/// is the only point in the pipeline where that can happen.
async fn cmd_payments_wait(app: &Arc<App>, invoice_id: &str) -> Result<Output> {
    let payment_engine = build_payment_engine(app)?;
    let invoice = payment_engine
        .check_payment(invoice_id)
        .await
        .map_err(|e| anyhow!("Failed to check payment: {}", e))?;

    let paid = matches!(invoice.status.as_str(), "PendingConfirmation" | "Confirmed");

    if paid {
        if let Some(job) = app.find_job_by_invoice(invoice_id).await {
            if matches!(job.status, JobStatus::Quoted | JobStatus::AwaitingPayment) {
                if let Err(e) = execute_job_pipeline(app, &job).await {
                    error!("Job execution pipeline failed for {}: {:#}", job.id, e);
                }
            }
        }
    }

    let status_str = match invoice.status.as_str() {
        "PendingConfirmation" => "detected",
        "Confirmed" => "confirmed",
        "Expired" => "expired",
        _ => "pending",
    };

    Ok(Output::Json(json!({
        "transaction_signature": invoice.signature.clone().unwrap_or_default(),
        "status": status_str,
        "amount": invoice.amount,
        "timestamp": Utc::now().to_rfc3339(),
    })))
}

async fn cmd_reports_list(app: &Arc<App>, job_id: &str) -> Result<Output> {
    let report_generator = build_report_generator(app)?;
    let reports = report_generator.list_reports().await;
    let filtered: Vec<Value> = reports
        .iter()
        .filter(|r| r.job_id == job_id)
        .map(report_to_json)
        .collect();
    Ok(Output::Json(Value::Array(filtered)))
}

async fn cmd_reports_download(app: &Arc<App>, report_id: &str, format_type: &str) -> Result<Output> {
    let report_generator = build_report_generator(app)?;
    let report = report_generator
        .find_report(report_id)
        .await
        .ok_or_else(|| anyhow!("Report not found: {}", report_id))?;

    let path = match format_type.to_lowercase().as_str() {
        "pdf" => &report.pdf_path,
        _ => &report.markdown_path,
    };

    let bytes = tokio::fs::read(path)
        .await
        .with_context(|| format!("Failed to read report file: {}", path))?;

    Ok(Output::Raw(bytes))
}

async fn cmd_config_get(app: &Arc<App>) -> Result<Output> {
    let features = &app.state.config.features;
    Ok(Output::Json(json!({
        "environment": app.state.config.application.environment,
        "rpc_endpoint": app.state.config.solana.rpc_url,
        "features": {
            "smart_contract_review": features.smart_contract_review,
            "onchain_intelligence": features.onchain_intelligence,
            "pdf_reports": features.pdf_reports,
            "markdown_reports": features.markdown_reports,
            "payment_monitor": features.payment_monitor,
            "job_history": features.job_history,
        },
    })))
}

// ============================================================================
// Job execution pipeline
// ============================================================================

/// Runs the full pipeline for a job once payment has been detected:
/// dispatch to the right blockchain service, generate the report, and mark
/// the job Completed (or Failed, if any step errors out).
async fn execute_job_pipeline(app: &Arc<App>, job: &Job) -> Result<()> {
    app.update_job_status(&job.id, JobStatus::PaymentDetected).await?;
    app.update_job_status(&job.id, JobStatus::Executing).await?;

    let registry = build_service_registry(app).await;
    let manager = ServiceManager::new(Arc::new(registry));

    let request = ServiceRequest {
        job_id: job.id.clone(),
        service_type: job.service.clone(),
        input: job.input_source.clone(),
        metadata: job.metadata.clone(),
        created_at: Utc::now(),
    };

    let response = match manager.dispatch_job(request).await {
        Ok(response) => response,
        Err(e) => {
            app.update_job_status(&job.id, JobStatus::Failed).await.ok();
            return Err(anyhow!("Service execution failed: {}", e));
        }
    };

    app.update_job_status(&job.id, JobStatus::GeneratingReport).await?;

    let report_generator = build_report_generator(app)?;
    let report = match report_generator.generate(&response).await {
        Ok(report) => report,
        Err(e) => {
            app.update_job_status(&job.id, JobStatus::Failed).await.ok();
            return Err(anyhow!("Report generation failed: {}", e));
        }
    };

    app.set_job_report(&job.id, &report.report_id).await?;
    app.update_job_status(&job.id, JobStatus::Completed).await?;

    Ok(())
}

// ============================================================================
// Builders
// ============================================================================

fn build_payment_engine(app: &Arc<App>) -> Result<PaymentEngine> {
    PaymentEngine::new(app.clone()).map_err(|e| anyhow!("Failed to initialize payment engine: {}", e))
}

fn build_report_generator(app: &Arc<App>) -> Result<ReportGenerator> {
    ReportGenerator::new(app.clone()).map_err(|e| anyhow!("Failed to initialize report generator: {}", e))
}

async fn build_service_registry(app: &Arc<App>) -> ServiceRegistry {
    let ai_provider = build_ai_provider(&app.state.config.ai);
    let registry = ServiceRegistry::new();

    let work_dir = PathBuf::from(&app.state.config.storage.upload_directory);
    registry
        .register(
            ServiceType::SmartContractReview,
            Arc::new(SmartContractReviewService::new(ai_provider.clone(), work_dir)),
        )
        .await;

    registry
        .register(
            ServiceType::OnChainIntelligence,
            Arc::new(OnChainIntelligenceService::new(
                ai_provider.clone(),
                app.state.config.solana.rpc_url.clone(),
            )),
        )
        .await;

    registry
}

fn build_ai_provider(config: &AIConfig) -> Arc<dyn AIProvider> {
    match config.provider.to_lowercase().as_str() {
        "openai" => Arc::new(OpenAIProvider::new(config.api_key.clone(), config.model.clone())),
        "anthropic" => Arc::new(AnthropicProvider::new(config.api_key.clone(), config.model.clone())),
        _ => Arc::new(OpenRouterProvider::new(config.api_key.clone(), config.model.clone())),
    }
}

// ============================================================================
// JSON mapping
// ============================================================================
//
// Field names and enum string values here must match api.py's Pydantic
// models exactly (JobResponse, InvoiceResponse, ReportResponse, ...) -
// Pydantic validates strictly, so a mismatch here fails the same way the
// original "ai.api_key" and "styles.get_terminal_css" bugs did: silently,
// until something downstream breaks.

fn parse_service_type(value: &str) -> Result<ServiceType> {
    match value {
        "SmartContractReview" => Ok(ServiceType::SmartContractReview),
        "OnChainIntelligence" => Ok(ServiceType::OnChainIntelligence),
        other => Err(anyhow!("Unknown service type: {}", other)),
    }
}

fn service_type_str(t: &ServiceType) -> &'static str {
    match t {
        ServiceType::SmartContractReview => "SmartContractReview",
        ServiceType::OnChainIntelligence => "OnChainIntelligence",
    }
}

fn job_status_str(s: &JobStatus) -> &'static str {
    match s {
        JobStatus::Created => "Created",
        JobStatus::Quoted => "Quoted",
        JobStatus::AwaitingPayment => "AwaitingPayment",
        JobStatus::PaymentDetected => "PaymentDetected",
        JobStatus::Executing => "Executing",
        JobStatus::GeneratingReport => "GeneratingReport",
        JobStatus::Completed => "Completed",
        JobStatus::Failed => "Failed",
        JobStatus::Cancelled => "Cancelled",
        // The Python client has no "Archived" state - an archived job is,
        // from its point of view, simply a finished one.
        JobStatus::Archived => "Completed",
    }
}

fn job_to_json(job: &Job) -> Value {
    json!({
        "job_id": job.id,
        "service_type": service_type_str(&job.service),
        "status": job_status_str(&job.status),
        "created_at": job.created_at.to_rfc3339(),
        "updated_at": job.updated_at.to_rfc3339(),
    })
}

fn invoice_to_json(invoice: &Invoice, job_id: &str) -> Value {
    json!({
        "invoice_id": invoice.invoice_id,
        "job_id": job_id,
        "amount": invoice.amount,
        "currency": invoice.currency,
        "wallet_address": invoice.wallet,
        "status": invoice.status,
        "expires_at": invoice.expires_at.to_rfc3339(),
    })
}

fn report_to_json(report: &Report) -> Value {
    let title = match report.report_type {
        ReportType::SmartContractReview => "Smart Contract Security Audit Report",
        ReportType::OnChainIntelligence => "On-Chain Intelligence & Wallet Analysis Report",
    };
    json!({
        "report_id": report.report_id,
        "job_id": report.job_id,
        "title": title,
        "summary": report.summary,
        "created_at": report.created_at.to_rfc3339(),
        "markdown_path": report.markdown_path,
        "pdf_path": report.pdf_path,
    })
}
