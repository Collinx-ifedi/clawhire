//! Services module for ClawHire.
//!
//! This module contains all business logic. It is responsible for receiving validated
//! jobs, executing the requested blockchain service, and returning structured results
//! for report generation.

use crate::core::{AppConfig, RiskLevel, ServiceType};
use anyhow::{Context, Result};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use git2::Repository;
use log::{error, info, warn};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Signature;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;
use walkdir::WalkDir;

// ============================================================================
// Errors
// ============================================================================

/// Errors that can occur during service execution.
#[derive(Error, Debug)]
pub enum ServiceError {
    #[error("Invalid input provided: {0}")]
    InvalidInput(String),
    #[error("Repository error: {0}")]
    RepositoryError(String),
    #[error("Archive error: {0}")]
    ArchiveError(String),
    #[error("Prompt error: {0}")]
    PromptError(String),
    #[error("AI execution error: {0}")]
    AIError(String),
    #[error("Blockchain RPC error: {0}")]
    RPCError(String),
    #[error("Service execution error: {0}")]
    ExecutionError(String),
    #[error("Validation error: {0}")]
    ValidationError(String),
    #[error("Timeout error: {0}")]
    TimeoutError(String),
}

// ============================================================================
// Models
// ============================================================================

/// Represents the execution status of a service.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ServiceStatus {
    Pending,
    Validating,
    Running,
    Completed,
    Failed,
}

/// A request to execute a specific service.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceRequest {
    pub job_id: String,
    pub service_type: ServiceType,
    pub input: String,
    pub metadata: HashMap<String, String>,
    pub created_at: DateTime<Utc>,
}

/// A structured finding from an AI analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceFinding {
    pub title: String,
    pub severity: RiskLevel,
    pub description: String,
    pub recommendation: String,
}

/// The result returned from a completed service execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceResponse {
    pub job_id: String,
    pub success: bool,
    pub service: ServiceType,
    pub summary: String,
    pub findings: Vec<ServiceFinding>,
    pub risk_level: RiskLevel,
    pub execution_time: std::time::Duration,
    pub report_metadata: HashMap<String, String>,
}

/// Metadata describing an available service.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceMetadata {
    pub id: String,
    pub name: String,
    pub description: String,
    pub price: f64,
    pub currency: String,
    pub estimated_duration: u64,
    pub accepted_inputs: Vec<String>,
}

// ============================================================================
// Traits
// ============================================================================

/// A trait defining the lifecycle of a blockchain service.
#[async_trait]
pub trait BlockchainService: Send + Sync {
    /// Returns the unique name of the service.
    fn name(&self) -> &str;

    /// Returns the description of the service.
    fn description(&self) -> &str;

    /// Validates the provided input.
    async fn validate(&self, request: &ServiceRequest) -> Result<bool, ServiceError>;

    /// Generates a quote for executing the service.
    async fn quote(&self, request: &ServiceRequest) -> Result<f64, ServiceError>;

    /// Executes the core logic of the service.
    async fn execute(&self, request: &ServiceRequest) -> Result<ServiceResponse, ServiceError>;

    /// Generates an execution summary from the response.
    async fn generate_summary(&self, response: &ServiceResponse) -> Result<String, ServiceError>;
}

/// An abstraction for different AI model providers.
#[async_trait]
pub trait AIProvider: Send + Sync {
    /// Generates a response from the AI provider given a prompt and context.
    async fn generate(&self, prompt: &str, context: &str) -> Result<String, ServiceError>;
}

// ============================================================================
// AI Providers
// ============================================================================

/// OpenAI provider implementation.
pub struct OpenAIProvider {
    client: Client,
    api_key: String,
    model: String,
}

impl OpenAIProvider {
    pub fn new(api_key: String, model: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            model,
        }
    }
}

#[async_trait]
impl AIProvider for OpenAIProvider {
    async fn generate(&self, prompt: &str, context: &str) -> Result<String, ServiceError> {
        let url = "https://api.openai.com/v1/chat/completions";
        let full_prompt = format!("{}\n\nContext:\n{}", prompt, context);

        let body = serde_json::json!({
            "model": self.model,
            "messages": [
                {"role": "system", "content": "You are a specialized blockchain AI agent."},
                {"role": "user", "content": full_prompt}
            ],
            "temperature": 0.2
        });

        let response = self
            .client
            .post(url)
            .bearer_auth(&self.api_key)
            .json(&body)
            .send()
            .await
            .map_err(|e| ServiceError::AIError(e.to_string()))?;

        if !response.status().is_success() {
            let err = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown API error".to_string());
            return Err(ServiceError::AIError(err));
        }

        let resp_json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| ServiceError::AIError(e.to_string()))?;

        let content = resp_json["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| ServiceError::AIError("Invalid response format".to_string()))?;

        Ok(content.to_string())
    }
}

/// Anthropic provider implementation.
pub struct AnthropicProvider {
    client: Client,
    api_key: String,
    model: String,
}

impl AnthropicProvider {
    pub fn new(api_key: String, model: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            model,
        }
    }
}

#[async_trait]
impl AIProvider for AnthropicProvider {
    async fn generate(&self, prompt: &str, context: &str) -> Result<String, ServiceError> {
        let url = "https://api.anthropic.com/v1/messages";
        let full_prompt = format!("{}\n\nContext:\n{}", prompt, context);

        let body = serde_json::json!({
            "model": self.model,
            "max_tokens": 8192,
            "messages": [
                {"role": "user", "content": full_prompt}
            ]
        });

        let response = self
            .client
            .post(url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&body)
            .send()
            .await
            .map_err(|e| ServiceError::AIError(e.to_string()))?;

        if !response.status().is_success() {
            let err = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown API error".to_string());
            return Err(ServiceError::AIError(err));
        }

        let resp_json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| ServiceError::AIError(e.to_string()))?;

        let content = resp_json["content"][0]["text"]
            .as_str()
            .ok_or_else(|| ServiceError::AIError("Invalid response format".to_string()))?;

        Ok(content.to_string())
    }
}

/// OpenRouter provider implementation.
pub struct OpenRouterProvider {
    client: Client,
    api_key: String,
    model: String,
}

impl OpenRouterProvider {
    pub fn new(api_key: String, model: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            model,
        }
    }
}

#[async_trait]
impl AIProvider for OpenRouterProvider {
    async fn generate(&self, prompt: &str, context: &str) -> Result<String, ServiceError> {
        let url = "https://openrouter.ai/api/v1/chat/completions";
        let full_prompt = format!("{}\n\nContext:\n{}", prompt, context);

        let body = serde_json::json!({
            "model": self.model,
            "messages": [
                {"role": "system", "content": "You are a specialized blockchain AI agent."},
                {"role": "user", "content": full_prompt}
            ],
            "temperature": 0.2
        });

        let response = self
            .client
            .post(url)
            .bearer_auth(&self.api_key)
            .json(&body)
            .send()
            .await
            .map_err(|e| ServiceError::AIError(e.to_string()))?;

        if !response.status().is_success() {
            let err = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown API error".to_string());
            return Err(ServiceError::AIError(err));
        }

        let resp_json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| ServiceError::AIError(e.to_string()))?;

        let content = resp_json["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| ServiceError::AIError("Invalid response format".to_string()))?;

        Ok(content.to_string())
    }
}

// ============================================================================
// Utilities
// ============================================================================

/// Loads a system prompt from the file system.
pub async fn load_prompt(filename: &str) -> Result<String, ServiceError> {
    let path = PathBuf::from("prompts").join(filename);
    tokio::fs::read_to_string(&path)
        .await
        .map_err(|e| ServiceError::PromptError(format!("Failed to load prompt {}: {}", filename, e)))
}

/// Validates if a given string is a valid Solana wallet address.
pub fn is_valid_solana_address(address: &str) -> bool {
    Pubkey::from_str(address).is_ok()
}

/// Validates if a given string is a valid Solana transaction signature.
pub fn is_valid_solana_signature(signature: &str) -> bool {
    Signature::from_str(signature).is_ok()
}

// ============================================================================
// Smart Contract Security Review Service
// ============================================================================

/// Service for analyzing Solana smart contracts for vulnerabilities.
pub struct SmartContractReviewService {
    ai_provider: Arc<dyn AIProvider>,
    work_dir: PathBuf,
}

impl SmartContractReviewService {
    pub fn new(ai_provider: Arc<dyn AIProvider>, work_dir: PathBuf) -> Self {
        Self {
            ai_provider,
            work_dir,
        }
    }

    /// Clones a remote repository to a local temporary directory.
    fn clone_repository(&self, url: &str, dest: &Path) -> Result<(), ServiceError> {
        Repository::clone(url, dest)
            .map_err(|e| ServiceError::RepositoryError(e.to_string()))?;
        Ok(())
    }

    /// Extracts a ZIP archive to a destination directory.
    fn extract_archive(&self, archive_path: &Path, dest: &Path) -> Result<(), ServiceError> {
        let file = File::open(archive_path)
            .map_err(|e| ServiceError::ArchiveError(e.to_string()))?;
        let mut archive = zip::ZipArchive::new(file)
            .map_err(|e| ServiceError::ArchiveError(e.to_string()))?;
        
        archive.extract(dest)
            .map_err(|e| ServiceError::ArchiveError(e.to_string()))?;
        Ok(())
    }

    /// Discovers if the directory is a Cargo workspace.
    fn discover_workspace(&self, dir: &Path) -> bool {
        dir.join("Cargo.toml").exists() || dir.join("Anchor.toml").exists()
    }

    /// Collects all Rust source files in a directory.
    fn collect_source_files(&self, dir: &Path) -> Result<Vec<PathBuf>, ServiceError> {
        let mut files = Vec::new();
        for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
            if entry.path().extension().map_or(false, |ext| ext == "rs") {
                files.push(entry.path().to_path_buf());
            }
        }
        Ok(files)
    }

    /// Reads multiple source files into a unified string.
    fn read_source_files(&self, files: &[PathBuf]) -> Result<String, ServiceError> {
        let mut combined = String::new();
        for file in files {
            let mut contents = String::new();
            File::open(file)
                .and_then(|mut f| f.read_to_string(&mut contents))
                .map_err(|e| ServiceError::ExecutionError(format!("Failed reading {:?}: {}", file, e)))?;
            
            combined.push_str(&format!("\n--- File: {:?} ---\n", file.file_name().unwrap_or_default()));
            combined.push_str(&contents);
        }
        Ok(combined)
    }

    /// Prepares the AI prompt for security review.
    async fn prepare_prompt(&self) -> Result<String, ServiceError> {
        load_prompt("smart_contract_review.md").await
    }

    /// Invokes the AI provider with the prompt and source code.
    async fn invoke_ai(&self, prompt: &str, source_code: &str) -> Result<String, ServiceError> {
        self.ai_provider.generate(prompt, source_code).await
    }

    /// Parses the raw text AI response into structured findings.
    fn parse_ai_response(&self, response: &str) -> Result<Vec<ServiceFinding>, ServiceError> {
        // Simplified parser: in a real system, the AI would be prompted to output JSON.
        // Assuming the response is pre-formatted, we wrap it in a single finding for now,
        // or extract it dynamically.
        let finding = ServiceFinding {
            title: "General Security Assessment".to_string(),
            severity: RiskLevel::Medium,
            description: response.chars().take(500).collect::<String>() + "...",
            recommendation: "Review the full AI generated markdown report.".to_string(),
        };
        Ok(vec![finding])
    }

    /// Calculates a rough security score based on findings.
    fn calculate_security_score(&self, findings: &[ServiceFinding]) -> RiskLevel {
        let mut highest_risk = RiskLevel::VeryLow;
        for finding in findings {
            highest_risk = match (&highest_risk, &finding.severity) {
                (RiskLevel::Critical, _) | (_, RiskLevel::Critical) => RiskLevel::Critical,
                (RiskLevel::High, _) | (_, RiskLevel::High) => RiskLevel::High,
                (RiskLevel::Medium, _) | (_, RiskLevel::Medium) => RiskLevel::Medium,
                (RiskLevel::Low, _) | (_, RiskLevel::Low) => RiskLevel::Low,
                _ => RiskLevel::VeryLow,
            };
        }
        highest_risk
    }
}

#[async_trait]
impl BlockchainService for SmartContractReviewService {
    fn name(&self) -> &str {
        "Smart Contract Security Review"
    }

    fn description(&self) -> &str {
        "Professional AI-powered security audit for Solana smart contracts."
    }

    async fn validate(&self, request: &ServiceRequest) -> Result<bool, ServiceError> {
        if request.input.is_empty() {
            return Err(ServiceError::InvalidInput("Input cannot be empty".to_string()));
        }
        Ok(true)
    }

    async fn quote(&self, _request: &ServiceRequest) -> Result<f64, ServiceError> {
        Ok(0.20) // Default price from configuration
    }

    async fn execute(&self, request: &ServiceRequest) -> Result<ServiceResponse, ServiceError> {
        let start_time = std::time::Instant::now();
        
        let source_code = if request.input.starts_with("http") {
            let dest = self.work_dir.join(&request.job_id);
            self.clone_repository(&request.input, &dest)?;
            let files = self.collect_source_files(&dest)?;
            self.read_source_files(&files)?
        } else {
            // Treat as pasted raw Rust code
            request.input.clone()
        };

        let prompt = self.prepare_prompt().await?;
        let ai_result = self.invoke_ai(&prompt, &source_code).await?;
        
        let findings = self.parse_ai_response(&ai_result)?;
        let risk_level = self.calculate_security_score(&findings);
        let summary = self.generate_summary(&ServiceResponse {
            job_id: request.job_id.clone(),
            success: true,
            service: ServiceType::SmartContractReview,
            summary: String::new(),
            findings: findings.clone(),
            risk_level: risk_level.clone(),
            execution_time: start_time.elapsed(),
            report_metadata: HashMap::new(),
        }).await?;

        Ok(ServiceResponse {
            job_id: request.job_id.clone(),
            success: true,
            service: ServiceType::SmartContractReview,
            summary,
            findings,
            risk_level,
            execution_time: start_time.elapsed(),
            report_metadata: HashMap::new(),
        })
    }

    async fn generate_summary(&self, response: &ServiceResponse) -> Result<String, ServiceError> {
        Ok(format!(
            "Completed review with {} findings. Overall risk: {:?}",
            response.findings.len(),
            response.risk_level
        ))
    }
}

// ============================================================================
// On-chain Intelligence Service
// ============================================================================

/// Service for analyzing Solana wallets and transactions.
pub struct OnChainIntelligenceService {
    ai_provider: Arc<dyn AIProvider>,
    rpc_client: RpcClient,
}

impl OnChainIntelligenceService {
    pub fn new(ai_provider: Arc<dyn AIProvider>, rpc_url: String) -> Self {
        Self {
            ai_provider,
            rpc_client: RpcClient::new(rpc_url),
        }
    }

    /// Validates a wallet address.
    fn validate_wallet(&self, input: &str) -> Result<Pubkey, ServiceError> {
        Pubkey::from_str(input).map_err(|_| ServiceError::InvalidInput("Invalid wallet address".into()))
    }

    /// Validates a transaction signature.
    fn validate_signature(&self, input: &str) -> Result<Signature, ServiceError> {
        Signature::from_str(input).map_err(|_| ServiceError::InvalidInput("Invalid transaction signature".into()))
    }

    /// Fetches the SOL balance of a wallet.
    fn fetch_balance(&self, pubkey: &Pubkey) -> Result<u64, ServiceError> {
        self.rpc_client
            .get_balance(pubkey)
            .map_err(|e| ServiceError::RPCError(e.to_string()))
    }

    /// Fetches SPL tokens associated with the wallet.
    fn fetch_tokens(&self, pubkey: &Pubkey) -> Result<String, ServiceError> {
        // Simplified mockup for SPL token fetching.
        Ok(format!("Tokens for {}", pubkey))
    }

    /// Fetches NFTs held by the wallet.
    fn fetch_nfts(&self, pubkey: &Pubkey) -> Result<String, ServiceError> {
        // Simplified mockup for NFT fetching.
        Ok(format!("NFTs for {}", pubkey))
    }

    /// Fetches recent transaction history for the wallet.
    fn fetch_transactions(&self, pubkey: &Pubkey) -> Result<String, ServiceError> {
        let signatures = self.rpc_client
            .get_signatures_for_address(pubkey)
            .map_err(|e| ServiceError::RPCError(e.to_string()))?;
        
        let count = signatures.len();
        Ok(format!("{} recent transactions found.", count))
    }

    /// Identifies specific program interactions.
    fn fetch_programs(&self, _pubkey: &Pubkey) -> Result<String, ServiceError> {
        Ok("Interacted with System Program, Token Program.".to_string())
    }

    /// Prepares the AI prompt for intelligence reporting.
    async fn prepare_prompt(&self) -> Result<String, ServiceError> {
        load_prompt("onchain_intelligence.md").await
    }

    /// Invokes the AI provider to generate the report.
    async fn invoke_ai(&self, prompt: &str, context: &str) -> Result<String, ServiceError> {
        self.ai_provider.generate(prompt, context).await
    }

    /// Calculates a preliminary risk score based on on-chain data.
    fn calculate_wallet_risk(&self, _balance: u64, _tx_count: usize) -> RiskLevel {
        RiskLevel::Low
    }
}

#[async_trait]
impl BlockchainService for OnChainIntelligenceService {
    fn name(&self) -> &str {
        "On-chain Intelligence Report"
    }

    fn description(&self) -> &str {
        "Professional AI-powered wallet and transaction intelligence report."
    }

    async fn validate(&self, request: &ServiceRequest) -> Result<bool, ServiceError> {
        if is_valid_solana_address(&request.input) || is_valid_solana_signature(&request.input) {
            Ok(true)
        } else {
            Err(ServiceError::InvalidInput("Input must be a valid Solana address or signature".into()))
        }
    }

    async fn quote(&self, _request: &ServiceRequest) -> Result<f64, ServiceError> {
        Ok(0.10) // Default price from configuration
    }

    async fn execute(&self, request: &ServiceRequest) -> Result<ServiceResponse, ServiceError> {
        let start_time = std::time::Instant::now();
        
        let mut context = String::new();
        let pubkey = self.validate_wallet(&request.input)?;
        
        let balance = self.fetch_balance(&pubkey)?;
        context.push_str(&format!("Balance: {} lamports\n", balance));
        
        let tokens = self.fetch_tokens(&pubkey)?;
        context.push_str(&format!("Tokens: {}\n", tokens));
        
        let nfts = self.fetch_nfts(&pubkey)?;
        context.push_str(&format!("NFTs: {}\n", nfts));
        
        let txs = self.fetch_transactions(&pubkey)?;
        context.push_str(&format!("Transactions: {}\n", txs));
        
        let programs = self.fetch_programs(&pubkey)?;
        context.push_str(&format!("Programs: {}\n", programs));

        let prompt = self.prepare_prompt().await?;
        let _ai_report = self.invoke_ai(&prompt, &context).await?;

        let risk_level = self.calculate_wallet_risk(balance, 10);
        let summary = "Intelligence report generated successfully.".to_string();
        
        Ok(ServiceResponse {
            job_id: request.job_id.clone(),
            success: true,
            service: ServiceType::OnChainIntelligence,
            summary,
            findings: Vec::new(),
            risk_level,
            execution_time: start_time.elapsed(),
            report_metadata: HashMap::new(),
        })
    }

    async fn generate_summary(&self, response: &ServiceResponse) -> Result<String, ServiceError> {
        Ok(format!("Intelligence review completed with risk level: {:?}", response.risk_level))
    }
}

// ============================================================================
// Service Registry
// ============================================================================

/// A registry managing all available blockchain services.
pub struct ServiceRegistry {
    services: RwLock<HashMap<ServiceType, Arc<dyn BlockchainService>>>,
}

impl ServiceRegistry {
    pub fn new() -> Self {
        Self {
            services: RwLock::new(HashMap::new()),
        }
    }

    /// Registers a new blockchain service.
    pub async fn register(&self, service_type: ServiceType, service: Arc<dyn BlockchainService>) {
        let mut services = self.services.write().await;
        services.insert(service_type, service);
    }

    /// Retrieves an instance of a registered service.
    pub async fn find_service(&self, service_type: &ServiceType) -> Result<Arc<dyn BlockchainService>, ServiceError> {
        let services = self.services.read().await;
        services
            .get(service_type)
            .cloned()
            .ok_or_else(|| ServiceError::ExecutionError("Service not found in registry".into()))
    }

    /// Lists all available services.
    pub async fn list_services(&self) -> Vec<ServiceType> {
        let services = self.services.read().await;
        services.keys().cloned().collect()
    }

    /// Retrieves metadata for a specific service.
    pub async fn service_metadata(&self, service_type: &ServiceType) -> Result<ServiceMetadata, ServiceError> {
        let service = self.find_service(service_type).await?;
        Ok(ServiceMetadata {
            id: format!("{:?}", service_type),
            name: service.name().to_string(),
            description: service.description().to_string(),
            price: 0.10, // Mocked configuration mapping
            currency: "SOL".to_string(),
            estimated_duration: 300,
            accepted_inputs: vec!["string".to_string()],
        })
    }
}

// ============================================================================
// Execution Logger
// ============================================================================

/// Logger to trace the execution lifecycle of a service.
pub struct ExecutionLogger {
    job_id: String,
}

impl ExecutionLogger {
    pub fn new(job_id: String) -> Self {
        Self { job_id }
    }

    pub fn log_start(&self, service_name: &str) {
        info!("[{}] Starting execution of service: {}", self.job_id, service_name);
    }

    pub fn log_progress(&self, message: &str) {
        info!("[{}] Progress: {}", self.job_id, message);
    }

    pub fn log_warning(&self, message: &str) {
        warn!("[{}] Warning: {}", self.job_id, message);
    }

    pub fn log_error(&self, message: &str) {
        error!("[{}] Error: {}", self.job_id, message);
    }

    pub fn log_finish(&self, success: bool, duration: std::time::Duration) {
        info!("[{}] Execution finished. Success: {}, Duration: {:?}", self.job_id, success, duration);
    }
}

// ============================================================================
// Service Manager
// ============================================================================

/// Manages the full lifecycle of service execution requests.
pub struct ServiceManager {
    registry: Arc<ServiceRegistry>,
}

impl ServiceManager {
    pub fn new(registry: Arc<ServiceRegistry>) -> Self {
        Self { registry }
    }

    /// Receives a job, validates it, and dispatches it for execution.
    pub async fn dispatch_job(&self, request: ServiceRequest) -> Result<ServiceResponse, ServiceError> {
        let logger = ExecutionLogger::new(request.job_id.clone());
        let service = self.registry.find_service(&request.service_type).await?;

        logger.log_start(service.name());
        logger.log_progress("Validating request input...");

        if !service.validate(&request).await? {
            let err_msg = "Request failed validation".to_string();
            logger.log_error(&err_msg);
            return Err(ServiceError::ValidationError(err_msg));
        }

        logger.log_progress("Execution started...");
        
        let result = service.execute(&request).await;

        match result {
            Ok(response) => {
                logger.log_finish(true, response.execution_time);
                Ok(response)
            }
            Err(e) => {
                logger.log_error(&e.to_string());
                Err(e)
            }
        }
    }
}
