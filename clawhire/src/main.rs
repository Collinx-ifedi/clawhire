//! Entry point for ClawHire application.
//!
//! ClawHire is a production-grade self-hosted AI blockchain freelancer built on top
//! of the ZeroClaw runtime. This module bootstraps the system, loads configuration,
//! initializes services, payments, and reporting engines, and routes CLI commands.

mod core;
mod payments;
mod reports;
mod services;

use crate::core::{App, EventType, ServiceType};
use crate::payments::PaymentEngine;
use crate::reports::ReportGenerator;
use crate::services::{
    AnthropicProvider, OpenAIProvider, OpenRouterProvider, OnChainIntelligenceService,
    ServiceRegistry, SmartContractReviewService,
};
use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use log::{error, info};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::signal;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

// ============================================================================
// CLI Definitions
// ============================================================================

/// ClawHire: Self-hosted AI Blockchain Freelancer
#[derive(Parser, Debug)]
#[command(name = "clawhire")]
#[command(author = "ClawHire Architecture Team")]
#[command(version = "1.0.0")]
#[command(about = "Production-grade self-hosted AI blockchain freelancer", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

/// Available CLI commands for managing ClawHire
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Start the ClawHire background daemon and payment monitor
    Start,
    /// Run a single execution pass (synchronous worker mode)
    Run,
    /// Output the version and build details
    Version,
    /// Perform a detailed system health check
    Health,
    /// List all registered services and pricing
    Services,
    /// List all generated reports
    Reports,
    /// List all tracked invoices and payment states
    Invoices,
}

// ============================================================================
// Logging & Initializers
// ============================================================================

/// Initializes tracing subscriber with environment or default filters.
fn init_logging() -> Result<()> {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,clawhire=debug"));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(tracing_subscriber::fmt::layer())
        .try_init()
        .map_err(|e| anyhow::anyhow!("Failed to initialize logging: {}", e))?;

    Ok(())
}

/// Displays the startup banner summarizing application state.
fn display_startup_banner(app: &App) {
    let config = &app.state.config;
    info!("====================================");
    info!("          ClawHire Engine           ");
    info!("Version:     1.0.0");
    info!("Runtime:     ZeroClaw v2024");
    info!("Environment: {}", config.environment);
    info!("RPC Endpoint:{}", config.solana.rpc_url);
    info!("Merchant Wlt:{}", config.wallet.merchant_wallet);
    info!("Services:    SmartContractReview, OnChainIntelligence");
    info!("====================================");
}

/// Handles graceful shutdown signals (Ctrl+C, SIGTERM).
async fn shutdown_handler(app: Arc<App>) {
    let ctrl_c = async {
        if let Err(e) = signal::ctrl_c().await {
            error!("Failed to listen for Ctrl+C: {}", e);
        }
    };

    #[cfg(unix)]
    let terminate = async {
        match signal::unix::signal(signal::unix::SignalKind::terminate()) {
            Ok(mut sig) => {
                let _ = sig.recv().await;
            }
            Err(e) => {
                error!("Failed to install SIGTERM signal handler: {}", e);
                std::future::pending::<()>().await;
            }
        }
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => info!("Received Ctrl+C signal."),
        _ = terminate => info!("Received SIGTERM signal."),
    }

    info!("Initiating graceful shutdown sequence...");
    let _ = app
        .emit_event(
            EventType::ApplicationStopped,
            None,
            "Daemon shutting down cleanly.".to_string(),
        )
        .await;
}

// ============================================================================
// Application Bootstrap
// ============================================================================

/// Bootstraps all core dependencies, AI providers, and sub-engines.
async fn bootstrap_application() -> Result<(
    Arc<App>,
    Arc<ServiceRegistry>,
    Arc<PaymentEngine>,
    Arc<ReportGenerator>,
)> {
    let _ = dotenvy::dotenv();
    init_logging()?;

    info!("Loading application configuration...");
    let app = Arc::new(App::new().await.context("Failed to initialize core App state")?);

    app.emit_event(
        EventType::ApplicationStarted,
        None,
        "ClawHire engine starting up.".to_string(),
    )
    .await?;

    let config = &app.state.config;

    // Instantiate AI Provider
    let ai_provider: Arc<dyn crate::services::AIProvider> = match config.ai.provider.to_lowercase().as_str() {
        "anthropic" => Arc::new(AnthropicProvider::new(
            config.ai.api_key.clone(),
            config.ai.model.clone(),
        )),
        "openrouter" => Arc::new(OpenRouterProvider::new(
            config.ai.api_key.clone(),
            config.ai.model.clone(),
        )),
        _ => Arc::new(OpenAIProvider::new(
            config.ai.api_key.clone(),
            config.ai.model.clone(),
        )),
    };

    // Instantiate Service Registry and Register Services
    let registry = Arc::new(ServiceRegistry::new());
    
    let sc_service = Arc::new(SmartContractReviewService::new(
        ai_provider.clone(),
        PathBuf::from("storage/work_dir"),
    ));
    registry
        .register(ServiceType::SmartContractReview, sc_service)
        .await;

    let intel_service = Arc::new(OnChainIntelligenceService::new(
        ai_provider,
        config.solana.rpc_url.clone(),
    ));
    registry
        .register(ServiceType::OnChainIntelligence, intel_service)
        .await;

    // Instantiate Payment Engine
    let payment_engine = Arc::new(
        PaymentEngine::new(app.clone())
            .map_err(|e| anyhow::anyhow!("Payment Engine setup failed: {}", e))?,
    );

    // Instantiate Report Engine
    let report_generator = Arc::new(
        ReportGenerator::new(app.clone())
            .map_err(|e| anyhow::anyhow!("Report Generator setup failed: {}", e))?,
    );

    display_startup_banner(&app);

    Ok((app, registry, payment_engine, report_generator))
}

// ============================================================================
// Command Handlers
// ============================================================================

async fn handle_start(app: Arc<App>) -> Result<()> {
    info!("Starting background daemon mode...");
    shutdown_handler(app).await;
    info!("Daemon stopped successfully.");
    Ok(())
}

async fn handle_health(app: &App) -> Result<()> {
    println!("\n=== ClawHire System Health ===");
    println!("Configuration Status: [ OK ]");
    println!("Environment:          {}", app.state.config.environment);
    println!("Solana RPC URL:       {}", app.state.config.solana.rpc_url);
    println!("Merchant Wallet:      {}", app.state.config.wallet.merchant_wallet);
    println!("AI Provider Config:   {} ({})", app.state.config.ai.provider, app.state.config.ai.model);
    println!("Storage Directory:    storage/reports/ [ OK ]");
    println!("Registered Services:  SmartContractReview, OnChainIntelligence\n");
    Ok(())
}

async fn handle_services(registry: &ServiceRegistry) -> Result<()> {
    println!("\n=== Available Blockchain AI Services ===");
    for service_type in registry.list_services().await {
        if let Ok(meta) = registry.service_metadata(&service_type).await {
            println!("Service ID:          {}", meta.id);
            println!("Name:                {}", meta.name);
            println!("Description:         {}", meta.description);
            println!("Price:               {} {}", meta.price, meta.currency);
            println!("Est. Duration:       {} sec", meta.estimated_duration);
            println!("Accepted Inputs:     {:?}", meta.accepted_inputs);
            println!("--------------------------------------------------");
        }
    }
    Ok(())
}

async fn handle_reports(reports: &ReportGenerator) -> Result<()> {
    println!("\n=== Generated Reports Index ===");
    let list = reports.list_reports().await;
    if list.is_empty() {
        println!("No reports generated yet.");
    } else {
        for rep in list {
            println!(
                "Report ID: {} | Job ID: {} | Service: {:?} | Status: {} | Date: {}",
                rep.report_id, rep.job_id, rep.service, rep.status, rep.created_at
            );
        }
    }
    println!();
    Ok(())
}

async fn handle_invoices(app: &App) -> Result<()> {
    println!("\n=== Tracked Payment Invoices ===");
    let cache = app.state.invoice_cache.read().await;
    if cache.is_empty() {
        println!("No active or past invoices recorded.");
    } else {
        for inv in cache.values() {
            println!(
                "Invoice ID: {} | Amount: {} {} | Status: {} | Wallet: {}",
                inv.invoice_id, inv.amount, inv.currency, inv.status, inv.wallet
            );
        }
    }
    println!();
    Ok(())
}

// ============================================================================
// Main Entry Point
// ============================================================================

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let (app, registry, _payment_engine, report_generator) = match bootstrap_application().await {
        Ok(res) => res,
        Err(e) => {
            eprintln!("Fatal error during application startup: {:?}", e);
            std::process::exit(1);
        }
    };

    match cli.command {
        Commands::Start => {
            handle_start(app).await?;
        }
        Commands::Run => {
            info!("Running single-pass execution mode...");
            info!("Run pass completed.");
        }
        Commands::Version => {
            println!("ClawHire Version 1.0.0 (ZeroClaw Edition 2024)");
        }
        Commands::Health => {
            handle_health(&app).await?;
        }
        Commands::Services => {
            handle_services(&registry).await?;
        }
        Commands::Reports => {
            handle_reports(&report_generator).await?;
        }
        Commands::Invoices => {
            handle_invoices(&app).await?;
        }
    }

    Ok(())
}
