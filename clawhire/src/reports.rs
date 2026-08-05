//! Reports module for ClawHire.
//!
//! This module is responsible for generating, formatting, exporting, storing,
//! and retrieving reports produced by ClawHire services. It transforms structured
//! service responses into professional Markdown and PDF reports.

use crate::core::{App, EventType, ServiceType};
use crate::services::ServiceResponse;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use config::{Config, File as ConfigFile};
use log::info;
use printpdf::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use thiserror::Error;
use tokio::fs;
use tokio::sync::RwLock;
use uuid::Uuid;

// ============================================================================
// Errors
// ============================================================================

/// Errors that can occur during report generation and storage.
#[derive(Error, Debug)]
pub enum ReportError {
    #[error("Report generation error: {0}")]
    ReportGenerationError(String),
    #[error("Markdown rendering error: {0}")]
    MarkdownError(String),
    #[error("PDF rendering error: {0}")]
    PDFError(String),
    #[error("Storage error: {0}")]
    StorageError(String),
    #[error("Configuration error: {0}")]
    ConfigurationError(String),
    #[error("Template error: {0}")]
    TemplateError(String),
    #[error("Serialization error: {0}")]
    SerializationError(String),
    #[error("Filesystem error: {0}")]
    FileSystemError(String),
}

// ============================================================================
// Enums
// ============================================================================

/// Supported types of reports.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ReportType {
    SmartContractReview,
    OnChainIntelligence,
}

impl ToString for ReportType {
    fn to_string(&self) -> String {
        match self {
            ReportType::SmartContractReview => "SmartContractReview".to_string(),
            ReportType::OnChainIntelligence => "OnChainIntelligence".to_string(),
        }
    }
}

/// Lifecycle status of a generated report.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ReportStatus {
    Pending,
    Generating,
    Completed,
    Failed,
    Archived,
}

impl ToString for ReportStatus {
    fn to_string(&self) -> String {
        match self {
            ReportStatus::Pending => "Pending".to_string(),
            ReportStatus::Generating => "Generating".to_string(),
            ReportStatus::Completed => "Completed".to_string(),
            ReportStatus::Failed => "Failed".to_string(),
            ReportStatus::Archived => "Archived".to_string(),
        }
    }
}

// ============================================================================
// Models
// ============================================================================

/// Represents a generated report and its associated metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Report {
    pub report_id: String,
    pub job_id: String,
    pub report_type: ReportType,
    pub service: ServiceType,
    pub status: String,
    pub filename: String,
    pub markdown_path: String,
    pub pdf_path: String,
    pub created_at: DateTime<Utc>,
    pub generated_at: Option<DateTime<Utc>>,
    pub summary: String,
}

/// Represents a distinct section within a report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportSection {
    pub title: String,
    pub content: String,
    pub order: usize,
}

/// Statistics collected during report generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportStatistics {
    pub generation_time_ms: u128,
    pub section_count: usize,
    pub finding_count: usize,
    pub word_count: usize,
    pub page_count: usize,
}

// ============================================================================
// Configuration
// ============================================================================

#[derive(Debug, Clone, Deserialize)]
pub struct ReportConfig {
    pub output_dir: String,
    pub template_dir: String,
    pub generate_pdf: bool,
    pub generate_markdown: bool,
    pub compression: bool,
    pub history_retention: usize,
}

/// Loads report configuration from configs/app.toml.
pub fn load_report_configuration() -> Result<ReportConfig, ReportError> {
    let config = Config::builder()
        .add_source(ConfigFile::with_name("configs/app.toml").required(false))
        .build()
        .map_err(|e| ReportError::ConfigurationError(e.to_string()))?;

    let output_dir = config.get_string("reports.output_dir").unwrap_or_else(|_| "storage/reports".to_string());
    let template_dir = config.get_string("reports.template_dir").unwrap_or_else(|_| "templates".to_string());
    let generate_pdf = config.get_bool("reports.generate_pdf").unwrap_or(true);
    let generate_markdown = config.get_bool("reports.generate_markdown").unwrap_or(true);
    let compression = config.get_bool("reports.compression").unwrap_or(false);
    let history_retention = config.get_int("reports.history_retention").unwrap_or(100) as usize;

    Ok(ReportConfig {
        output_dir,
        template_dir,
        generate_pdf,
        generate_markdown,
        compression,
        history_retention,
    })
}

// ============================================================================
// Filename Generator
// ============================================================================

pub struct FilenameGenerator;

impl FilenameGenerator {
    pub fn generate(extension: &str) -> String {
        let now = Utc::now();
        let timestamp = now.format("%Y%m%d-%H%M%S");
        let uuid_str = Uuid::new_v4().to_string();
        format!("CHR-REPORT-{}-{}.{}", timestamp, uuid_str, extension)
    }

    pub fn validate_filename(filename: &str) -> Result<(), ReportError> {
        if filename.contains("..") || filename.contains('/') || filename.contains('\\') {
            return Err(ReportError::StorageError("Invalid filename containing traversal sequences".into()));
        }
        Ok(())
    }
}

// ============================================================================
// Markdown & PDF Renderers
// ============================================================================

pub struct MarkdownRenderer;

impl MarkdownRenderer {
    pub fn render(title: &str, sections: &[ReportSection]) -> String {
        let mut md = format!("# {}\n\n", title);
        md.push_str(&format!("*Generated at: {}*\n\n---\n\n", Utc::now().to_rfc3339()));

        for section in sections {
            md.push_str(&format!("## {}\n\n{}\n\n", section.title, section.content));
        }

        md
    }
}

pub struct PDFRenderer;

impl PDFRenderer {
    pub fn render_to_file(markdown_content: &str, output_path: &Path) -> Result<(), ReportError> {
        let mut doc = PdfDocument::new("ClawHire Professional Report");

        let mut ops: Vec<Op> = vec![Op::StartTextSection];
        let mut y = 800.0;

        ops.push(Op::SetFont {
            font: PdfFontHandle::Builtin(BuiltinFont::Helvetica),
            size: Pt(24.0),
        });
        ops.push(Op::SetTextCursor { pos: Point { x: Pt(50.0), y: Pt(y) } });
        ops.push(Op::ShowText {
            items: vec![TextItem::Text("ClawHire Intelligence & Audit Report".to_string())],
        });
        y -= 40.0;

        // Write simple line-by-line rendering from markdown text to PDF surface
        for line in markdown_content.lines() {
            if y < 50.0 {
                break; // Basic single page overflow guard for minimal renderer
            }
            if !line.starts_with('#') && !line.is_empty() {
                ops.push(Op::SetFont {
                    font: PdfFontHandle::Builtin(BuiltinFont::Helvetica),
                    size: Pt(10.0),
                });
                ops.push(Op::SetTextCursor { pos: Point { x: Pt(50.0), y: Pt(y) } });
                ops.push(Op::ShowText {
                    items: vec![TextItem::Text(line.to_string())],
                });
                y -= 20.0;
            } else if line.starts_with('#') {
                y -= 10.0;
                ops.push(Op::SetFont {
                    font: PdfFontHandle::Builtin(BuiltinFont::Helvetica),
                    size: Pt(14.0),
                });
                ops.push(Op::SetTextCursor { pos: Point { x: Pt(50.0), y: Pt(y) } });
                ops.push(Op::ShowText {
                    items: vec![TextItem::Text(line.trim_start_matches('#').trim().to_string())],
                });
                y -= 25.0;
            }
        }

        ops.push(Op::EndTextSection);

        let page = PdfPage::new(Mm(210.0), Mm(297.0), ops);
        let mut warnings = Vec::new();
        let pdf_bytes: Vec<u8> = doc.with_pages(vec![page]).save(&PdfSaveOptions::default(), &mut warnings);

        std::fs::write(output_path, pdf_bytes)
            .map_err(|e| ReportError::PDFError(format!("Failed to save PDF: {}", e)))?;

        Ok(())
    }
}

// ============================================================================
// Report Index & Storage
// ============================================================================

#[derive(Clone)]
pub struct ReportIndex {
    index: Arc<RwLock<HashMap<String, Report>>>,
    index_path: PathBuf,
}

impl ReportIndex {
    /// Loads the report index from disk if present. Each CLI invocation is
    /// a fresh process with otherwise-empty in-memory state, so without
    /// this, reports generated by one invocation would be invisible to
    /// `reports list` / `reports download` in the next.
    pub fn load(index_path: PathBuf) -> Self {
        let existing: HashMap<String, Report> = std::fs::read(&index_path)
            .ok()
            .and_then(|bytes| serde_json::from_slice(&bytes).ok())
            .unwrap_or_default();

        Self {
            index: Arc::new(RwLock::new(existing)),
            index_path,
        }
    }

    async fn persist(&self, snapshot: &HashMap<String, Report>) -> Result<(), ReportError> {
        if let Some(parent) = self.index_path.parent() {
            fs::create_dir_all(parent).await.map_err(|e| ReportError::StorageError(e.to_string()))?;
        }
        let bytes = serde_json::to_vec_pretty(snapshot)
            .map_err(|e| ReportError::SerializationError(e.to_string()))?;
        let tmp_path = self.index_path.with_extension("json.tmp");
        fs::write(&tmp_path, &bytes).await.map_err(|e| ReportError::StorageError(e.to_string()))?;
        fs::rename(&tmp_path, &self.index_path).await.map_err(|e| ReportError::StorageError(e.to_string()))?;
        Ok(())
    }

    pub async fn insert(&self, report: Report) -> Result<(), ReportError> {
        let snapshot = {
            let mut idx = self.index.write().await;
            idx.insert(report.report_id.clone(), report);
            idx.clone()
        };
        self.persist(&snapshot).await
    }

    pub async fn get(&self, report_id: &str) -> Option<Report> {
        let idx = self.index.read().await;
        idx.get(report_id).cloned()
    }

    pub async fn list(&self) -> Vec<Report> {
        let idx = self.index.read().await;
        idx.values().cloned().collect()
    }
}

pub struct ReportStorage {
    base_dir: PathBuf,
}

impl ReportStorage {
    pub fn new(base_dir: String) -> Self {
        Self {
            base_dir: PathBuf::from(base_dir),
        }
    }

    pub async fn ensure_dir(&self) -> Result<(), ReportError> {
        fs::create_dir_all(&self.base_dir)
            .await
            .map_err(|e| ReportError::StorageError(e.to_string()))
    }

    pub async fn save_file(&self, filename: &str, content: &[u8]) -> Result<String, ReportError> {
        FilenameGenerator::validate_filename(filename)?;
        self.ensure_dir().await?;
        
        let path = self.base_dir.join(filename);
        fs::write(&path, content)
            .await
            .map_err(|e| ReportError::StorageError(e.to_string()))?;

        Ok(path.to_string_lossy().into_owned())
    }

    pub async fn delete_file(&self, filename: &str) -> Result<(), ReportError> {
        FilenameGenerator::validate_filename(filename)?;
        let path = self.base_dir.join(filename);
        if path.exists() {
            fs::remove_file(&path)
                .await
                .map_err(|e| ReportError::StorageError(e.to_string()))?;
        }
        Ok(())
    }
}

// ============================================================================
// Builders & Trait
// ============================================================================

#[async_trait]
pub trait ReportBuilder: Send + Sync {
    async fn build(&self, response: &ServiceResponse) -> Result<(String, Vec<ReportSection>, ReportStatistics), ReportError>;
    fn build_summary(&self, response: &ServiceResponse) -> String;
    fn build_sections(&self, response: &ServiceResponse) -> Vec<ReportSection>;
    fn build_statistics(&self, response: &ServiceResponse, sections: &[ReportSection]) -> ReportStatistics;
}

pub struct SmartContractReportBuilder;

#[async_trait]
impl ReportBuilder for SmartContractReportBuilder {
    async fn build(&self, response: &ServiceResponse) -> Result<(String, Vec<ReportSection>, ReportStatistics), ReportError> {
        let title = "Smart Contract Security Audit Report".to_string();
        let sections = self.build_sections(response);
        let stats = self.build_statistics(response, &sections);
        Ok((title, sections, stats))
    }

    fn build_summary(&self, response: &ServiceResponse) -> String {
        format!("Audit completed with risk level {:?}. Summary: {}", response.risk_level, response.summary)
    }

    fn build_sections(&self, response: &ServiceResponse) -> Vec<ReportSection> {
        let mut sections = Vec::new();
        
        sections.push(ReportSection {
            title: "Executive Summary".to_string(),
            content: response.summary.clone(),
            order: 1,
        });

        let mut findings_content = String::new();
        for finding in &response.findings {
            findings_content.push_str(&format!("### {}\n- **Severity**: {:?}\n- **Description**: {}\n- **Recommendation**: {}\n\n", finding.title, finding.severity, finding.description, finding.recommendation));
        }

        sections.push(ReportSection {
            title: "Detailed Findings".to_string(),
            content: if findings_content.is_empty() { "No vulnerabilities discovered.".to_string() } else { findings_content },
            order: 2,
        });

        sections
    }

    fn build_statistics(&self, response: &ServiceResponse, sections: &[ReportSection]) -> ReportStatistics {
        let word_count = sections.iter().map(|s| s.content.split_whitespace().count()).sum();
        ReportStatistics {
            generation_time_ms: response.execution_time.as_millis(),
            section_count: sections.len(),
            finding_count: response.findings.len(),
            word_count,
            page_count: 1,
        }
    }
}

pub struct OnChainReportBuilder;

#[async_trait]
impl ReportBuilder for OnChainReportBuilder {
    async fn build(&self, response: &ServiceResponse) -> Result<(String, Vec<ReportSection>, ReportStatistics), ReportError> {
        let title = "On-Chain Intelligence & Wallet Analysis Report".to_string();
        let sections = self.build_sections(response);
        let stats = self.build_statistics(response, &sections);
        Ok((title, sections, stats))
    }

    fn build_summary(&self, response: &ServiceResponse) -> String {
        format!("On-chain intelligence analysis generated. Risk level: {:?}", response.risk_level)
    }

    fn build_sections(&self, response: &ServiceResponse) -> Vec<ReportSection> {
        vec![
            ReportSection {
                title: "Executive Summary".to_string(),
                content: response.summary.clone(),
                order: 1,
            },
            ReportSection {
                title: "Activity Summary".to_string(),
                content: "Detailed wallet transaction and program interactions analyzed successfully.".to_string(),
                order: 2,
            },
        ]
    }

    fn build_statistics(&self, response: &ServiceResponse, sections: &[ReportSection]) -> ReportStatistics {
        let word_count = sections.iter().map(|s| s.content.split_whitespace().count()).sum();
        ReportStatistics {
            generation_time_ms: response.execution_time.as_millis(),
            section_count: sections.len(),
            finding_count: 0,
            word_count,
            page_count: 1,
        }
    }
}

// ============================================================================
// Report Generator
// ============================================================================

pub struct ReportGenerator {
    config: ReportConfig,
    storage: ReportStorage,
    index: ReportIndex,
    app: Arc<App>,
}

impl ReportGenerator {
    pub fn new(app: Arc<App>) -> Result<Self, ReportError> {
        let config = load_report_configuration()?;
        let storage = ReportStorage::new(config.output_dir.clone());
        let index_path = Path::new(&config.output_dir).join("index.json");
        let index = ReportIndex::load(index_path);

        Ok(Self {
            config,
            storage,
            index,
            app,
        })
    }

    pub async fn generate(&self, response: &ServiceResponse) -> Result<Report, ReportError> {
        info!("Starting report generation for job {}", response.job_id);

        let report_type = match response.service {
            ServiceType::SmartContractReview => ReportType::SmartContractReview,
            ServiceType::OnChainIntelligence => ReportType::OnChainIntelligence,
        };

        let builder: Arc<dyn ReportBuilder> = match report_type {
            ReportType::SmartContractReview => Arc::new(SmartContractReportBuilder),
            ReportType::OnChainIntelligence => Arc::new(OnChainReportBuilder),
        };

        let (title, sections, _stats) = builder.build(response).await?;
        let summary = builder.build_summary(response);

        let markdown_content = MarkdownRenderer::render(&title, &sections);
        let md_filename = FilenameGenerator::generate("md");
        let pdf_filename = FilenameGenerator::generate("pdf");

        let md_path = self.storage.save_file(&md_filename, markdown_content.as_bytes()).await?;
        
        let pdf_full_path = Path::new(&self.config.output_dir).join(&pdf_filename);
        PDFRenderer::render_to_file(&markdown_content, &pdf_full_path)?;
        let pdf_path = pdf_full_path.to_string_lossy().into_owned();

        let report_id = Uuid::new_v4().to_string();
        let report = Report {
            report_id: report_id.clone(),
            job_id: response.job_id.clone(),
            report_type,
            service: response.service.clone(),
            status: ReportStatus::Completed.to_string(),
            filename: md_filename,
            markdown_path: md_path,
            pdf_path,
            created_at: Utc::now(),
            generated_at: Some(Utc::now()),
            summary,
        };

        self.index.insert(report.clone()).await?;

        self.app.emit_event(
            EventType::ReportGenerated,
            Some(response.job_id.clone()),
            format!("Report {} generated successfully.", report_id),
        ).await.unwrap_or_default();

        Ok(report)
    }

    pub async fn find_report(&self, report_id: &str) -> Option<Report> {
        self.index.get(report_id).await
    }

    pub async fn list_reports(&self) -> Vec<Report> {
        self.index.list().await
    }

    pub async fn archive(&self, report_id: &str) -> Result<(), ReportError> {
        if let Some(mut rep) = self.index.get(report_id).await {
            rep.status = ReportStatus::Archived.to_string();
            self.index.insert(rep).await?;
        }
        Ok(())
    }

    pub async fn delete(&self, report_id: &str) -> Result<(), ReportError> {
        if let Some(rep) = self.index.get(report_id).await {
            self.storage.delete_file(&rep.filename).await?;
        }
        Ok(())
    }
}
