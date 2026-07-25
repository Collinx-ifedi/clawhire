# ClawHire Architecture

**Version:** 1.0.0  
**Document Date:** 2026-07-25  
**Authors:** ClawHire Architecture Team  
**Status:** Production  

## Revision History

| Version | Date       | Author                     | Changes                            |
|---------|------------|----------------------------|------------------------------------|
| 1.0.0   | 2026-07-25 | ClawHire Architecture Team | Initial production specification   |

---

## 1. Executive Summary

### Purpose

ClawHire is a production-grade, self-hosted AI blockchain freelancer. It enables users to purchase autonomous professional blockchain analysis services, pay for them on-chain using Solana, and receive structured, downloadable reports without human intervention after payment confirmation.

### Vision

Machine commerce for blockchain intelligence. An AI agent that can be hired, paid, and that autonomously delivers professional-grade work product on the Solana network.

### Goals

- Provide two high-value AI services for the Solana ecosystem.
- Accept payment exclusively via Solana (Devnet in the current release).
- Execute work only after on-chain payment confirmation.
- Deliver professional Markdown and PDF reports.
- Offer a terminal-style user experience that feels like a modern Linux CLI.
- Remain fully self-hostable with minimal operational overhead.
- Maintain clean separation of concerns across Rust backend and Streamlit frontend.

### Problem Statement

Traditional AI chat interfaces are not designed for commercial service delivery. They lack payment verification, job lifecycle management, professional report generation, and secure isolation of execution. ClawHire solves this by treating the AI as a hired contractor rather than a conversational partner.

### Target Users

- Solana smart-contract developers seeking rapid security feedback.
- Researchers and investigators analyzing wallet or transaction activity.
- Protocol teams requiring reproducible, timestamped analysis artifacts.
- Open-source contributors and Superteam bounty reviewers evaluating the system.

### Why ZeroClaw

ZeroClaw provides the underlying AI agent runtime. ClawHire treats ZeroClaw as the execution substrate for long-running analysis jobs while owning the commercial, payment, and reporting layers itself.

### Why Rust

Rust delivers memory safety, predictable performance, and excellent async support via Tokio. Blockchain RPC clients, cryptographic verification, and concurrent job management benefit directly from these properties. The backend is intentionally limited to five modules to keep the attack surface and cognitive load low.

### Why Streamlit

Streamlit enables rapid construction of a rich interactive frontend while still allowing complete visual override. The ClawHire UI intentionally abandons the default dashboard aesthetic in favor of a terminal metaphor that matches the “machine commerce” product identity.

### Why Solana

Solana offers fast finality, low transaction cost, and a mature ecosystem of wallets and RPC providers. Native SOL payments remove the need for intermediate payment processors and keep the entire commercial flow on-chain.

---

## 2. High-Level Architecture

ClawHire is composed of six major subsystems:

1. **Browser Terminal** – Streamlit application rendered as a Linux-style terminal.
2. **API Communication Layer** – Python adapters that talk to the Rust binary via CLI or HTTP.
3. **Rust Runtime** – Core application state, job management, and event emission.
4. **Service Engine** – Smart Contract Security Review and On-chain Intelligence services.
5. **Payment Engine** – Invoice generation, Solana RPC monitoring, and payment verification.
6. **Report Engine** – Markdown and PDF generation, storage, and retrieval.

\`\`\`mermaid
flowchart TB
    subgraph Client["Browser"]
        ST[Streamlit Terminal UI]
    end

    subgraph Frontend["Python Frontend Layer"]
        APP[app.py]
        TERM[terminal.py]
        COMP[components.py]
        STY[styles.py]
        API[api.py]
    end

    subgraph Backend["Rust Backend"]
        MAIN[main.rs]
        CORE[core.rs]
        SVC[services.rs]
        PAY[payments.rs]
        REP[reports.rs]
    end

    subgraph External["External Systems"]
        SOL[Solana RPC]
        AI[AI Provider<br/>OpenAI / Anthropic / OpenRouter]
        FS[Local Filesystem<br/>storage/ + logs/]
    end

    ST --> APP
    APP --> TERM
    APP --> COMP
    APP --> STY
    APP --> API
    API -->|CLI or HTTP| MAIN
    MAIN --> CORE
    CORE --> SVC
    CORE --> PAY
    CORE --> REP
    SVC --> AI
    PAY --> SOL
    REP --> FS
    CORE --> FS
\`\`\`

### Subsystem Responsibilities

| Subsystem              | Language | Primary Responsibility                                      |
|------------------------|----------|-------------------------------------------------------------|
| Streamlit Terminal     | Python   | User interaction, state machine, visual terminal experience |
| API Layer              | Python   | Backend discovery, retries, caching, download management    |
| Core Runtime           | Rust     | Shared state, job lifecycle, event bus                      |
| Services               | Rust     | AI prompt execution, source analysis, on-chain queries      |
| Payments               | Rust     | Invoice creation, RPC polling, signature verification       |
| Reports                | Rust     | Markdown + PDF generation, storage, indexing                |
| Configuration          | TOML     | Runtime, pricing, service, and feature flags                |
| Storage                | Filesystem | Reports, invoices, uploads, history, logs                 |

---

## 3. Project Structure

\`\`\`
clawhire/
├── Cargo.toml
├── Cargo.lock
├── .env
├── .gitignore
├── README.md
│
├── assets/
│   ├── logo/
│   ├── reports/          # Report templates
│   └── uploads/          # Transient upload staging
│
├── configs/
│   ├── app.toml          # Application, runtime, Solana, AI, security
│   ├── services.toml     # Service definitions and capabilities
│   └── pricing.toml      # Prices, invoice rules, refund policy
│
├── prompts/
│   ├── smart_contract_review.md
│   └── onchain_intelligence.md
│
├── src/
│   ├── main.rs           # CLI entry point and bootstrap
│   ├── core.rs           # App state, Job, Invoice, Event models
│   ├── services.rs       # BlockchainService trait + implementations
│   ├── payments.rs       # PaymentEngine, SolanaPaymentProvider
│   └── reports.rs        # ReportGenerator, Markdown/PDF renderers
│
├── ui/
│   ├── app.py            # Application orchestrator and state machine
│   ├── terminal.py       # Command parser and session management
│   ├── api.py            # BackendClient, adapters, managers
│   ├── styles.py         # Theme, Typography, CSSBuilder
│   └── components.py     # All reusable terminal UI components
│
├── storage/
│   ├── invoices/
│   ├── reports/
│   ├── history/
│   └── uploads/
│
├── logs/
│   ├── clawhire.log
│   └── payments.log
│
├── scripts/
│   ├── start.sh
│   ├── build.sh
│   └── deploy.sh
│
└── docs/
    ├── architecture.md   # This document
    ├── api.md
    └── workflow.md
\`\`\`

### Folder Responsibilities

| Path              | Responsibility |
|-------------------|----------------|
| \`configs/\`        | All runtime configuration. Never hard-coded prices or service flags. |
| \`prompts/\`        | System prompts loaded at execution time. Changing a prompt never requires a rebuild. |
| \`src/\`            | Complete Rust backend. Five modules only. |
| \`ui/\`             | Complete Streamlit frontend. Zero business logic in components. |
| \`storage/\`        | Persistent artifacts. Reports and invoices survive restarts. |
| \`logs/\`           | Rotated application and payment logs. |
| \`scripts/\`        | Build, start, and deployment helpers. |

---

## 4. Rust Backend Architecture

The backend is deliberately constrained to five modules. Each module owns a clear responsibility and communicates only through well-defined types and the shared \`App\` state.

\`\`\`mermaid
graph LR
    MAIN[main.rs] --> CORE[core.rs]
    MAIN --> SVC[services.rs]
    MAIN --> PAY[payments.rs]
    MAIN --> REP[reports.rs]
    SVC --> CORE
    PAY --> CORE
    REP --> CORE
\`\`\`

### 4.1 main.rs

- Parses CLI with \`clap\`.
- Bootstraps configuration, logging, and the \`App\` instance.
- Instantiates AI providers, service registry, payment engine, and report generator.
- Routes subcommands: \`Start\`, \`Run\`, \`Version\`, \`Health\`, \`Services\`, \`Reports\`, \`Invoices\`.
- Installs graceful shutdown handlers for SIGINT / SIGTERM.

### 4.2 core.rs

Owns the global application state:

- \`AppConfig\` – deserialized from \`configs/app.toml\`.
- \`AppState\` – thread-safe maps of active jobs, completed jobs, invoices, reports, and event history protected by \`tokio::sync::RwLock\`.
- \`Job\`, \`Invoice\`, \`ReportMetadata\`, \`ApplicationEvent\` models.
- Lifecycle methods: \`create_job\`, \`update_job_status\`, \`store_invoice\`, \`store_report\`, \`archive_job\`, \`emit_event\`.

All other modules receive an \`Arc<App>\` and never own configuration or job state themselves.

### 4.3 services.rs

Defines the \`BlockchainService\` trait and two concrete implementations:

- \`SmartContractReviewService\` – clones repositories or accepts pasted Rust, collects \`.rs\` files, loads the security-review prompt, invokes the AI provider, parses findings, and calculates a risk level.
- \`OnChainIntelligenceService\` – validates Solana addresses/signatures, fetches balance, tokens, NFTs, recent transactions and program interactions via RPC, then asks the AI to synthesize an intelligence report.

AI providers (\`OpenAIProvider\`, \`AnthropicProvider\`, \`OpenRouterProvider\`) implement a common \`AIProvider\` trait. The concrete provider is selected at bootstrap from configuration.

### 4.4 payments.rs

- Loads pricing exclusively from \`configs/pricing.toml\`.
- \`InvoiceGenerator\` produces human-readable invoice numbers (\`CHR-YYYYMMDD-XXXXXX\`) and UUID references.
- \`SolanaPaymentProvider\` implements the \`PaymentProvider\` trait: quote creation, signature scanning, and transaction verification against merchant wallet and exact amount.
- \`PaymentMonitor\` runs a background polling loop (default 5 s) over pending invoices.
- \`PaymentEngine\` orchestrates invoice creation, verification, and job-state transitions.

### 4.5 reports.rs

- \`ReportGenerator\` selects a builder (\`SmartContractReportBuilder\` or \`OnChainReportBuilder\`) according to service type.
- Markdown is rendered by a pure-text renderer; PDF is produced with \`printpdf\`.
- Files are written under \`storage/reports/\` with deterministic names of the form \`CHR-REPORT-YYYYMMDD-HHMMSS-<uuid>.{md,pdf}\`.
- An in-memory \`ReportIndex\` provides fast lookup; the filesystem is the source of truth.

### Ownership and Communication

- Configuration is owned by \`core::App\` and never mutated after bootstrap.
- Jobs, invoices and reports are stored in \`AppState\` maps.
- Modules communicate exclusively through public methods on \`App\` or through the trait objects registered in \`ServiceRegistry\`.
- No circular dependencies exist among the five modules.

---

## 5. Frontend Architecture

The frontend transforms Streamlit into a terminal experience. All visual styling is performed by injecting a comprehensive CSS stylesheet that hides Streamlit chrome and applies a dark terminal theme.

\`\`\`mermaid
flowchart TD
    A[app.py Application] --> B[ThemeManager]
    A --> C[Router / State Machine]
    A --> D[BackgroundTasks]
    A --> E[SidebarController]
    C --> F[components.*]
    C --> G[terminal.py]
    A --> H[api.BackendClient]
    H --> I[RustCLIAdapter or HTTPAdapter]
\`\`\`

### 5.1 app.py

- Owns the top-level \`Application\` and \`ApplicationContext\`.
- Manages a finite-state machine (\`AppState\` enum) that drives the entire user journey.
- Coordinates background polling for payment confirmation and job completion.
- Renders the outer \`TerminalWindow\` and dispatches to the appropriate handler for the current state.

### 5.2 terminal.py

- Provides a command-line interface alternative (or complement) to the guided UI flow.
- Maintains \`TerminalSession\`, command history, and event log.
- Implements a simple command router (\`help\`, \`services\`, \`service\`, \`invoice\`, \`pay\`, \`status\`, \`reset\`, \`clear\`).

### 5.3 components.py

Contains every reusable visual element:

- \`TerminalWindow\`, \`TerminalBanner\`, \`PromptLine\`, \`OutputBlock\`
- \`ServiceCard\`, \`ServiceSelector\`, \`InvoiceCard\`, \`PaymentStatusCard\`, \`JobStatusCard\`
- \`ProgressCard\`, \`ReportCard\`, \`ErrorPanel\`, \`SuccessPanel\`
- \`FileUploaderCard\`, \`DownloadButtons\`, \`CopyToClipboardButton\`

No component performs network calls or mutates global state; they are pure renderers.

### 5.4 styles.py

- Defines \`Theme\`, \`Typography\`, and \`Spacing\` dataclasses.
- \`CSSBuilder\` assembles a single stylesheet that overrides Streamlit defaults.
- Provides helper functions for terminal-colored text, status badges, and severity coloring.
- CSS is cached with \`@st.cache_data\` and injected once per session.

### 5.5 api.py

- Abstract \`BackendAdapter\` with two concrete implementations: \`RustCLIAdapter\` (subprocess) and \`HTTPAdapter\` (httpx).
- Auto-detection of mode (\`CLI\` / \`HTTP\` / \`AUTO\`).
- Retries with exponential backoff via \`tenacity\`.
- TTL caches for services, wallet, version, and health.
- \`DownloadManager\` and \`WalletManager\` encapsulate file and address handling.

### Rendering Flow

1. \`Application.run()\` calls \`initialize()\` → injects CSS → starts event loop.
2. \`EventLoop.tick()\` runs background polls.
3. \`Router.route()\` selects the handler for the current \`AppState\`.
4. Handler renders the appropriate components inside a \`TerminalWindow\`.
5. User actions update session state and trigger \`st.rerun()\`.

---

## 6. Configuration Architecture

Configuration is layered and never compiled into the binary.

### Loading Order

1. \`.env\` (optional) – secrets and environment-specific overrides.
2. \`configs/app.toml\` – primary application configuration.
3. \`configs/services.toml\` – service capabilities and flags.
4. \`configs/pricing.toml\` – all monetary values.
5. Environment variables with prefix \`APP_\` override TOML values (Rust side).

### Key Files

**configs/app.toml**  
Application identity, runtime limits, Solana RPC, merchant wallet placeholder, AI provider, report paths, security (rate limiting, CORS), Streamlit and terminal settings, feature flags, job lifecycle, GitHub clone options, health-check interval.

**configs/services.toml**  
Service IDs, display names, accepted inputs, validation limits (repository size, file count), AI prompt paths, report section flags, job states, payment behaviour, terminal display options.

**configs/pricing.toml**  
Fixed prices (0.20 SOL for Smart Contract Review, 0.10 SOL for On-chain Intelligence), invoice prefix, expiry, payment confirmation requirements, refund policy (disabled by default).

### Validation and Fallback

- Missing required keys cause a hard startup failure.
- Optional keys receive sensible defaults inside the Rust deserializers.
- Pricing is never hard-coded in service or payment logic; every amount is read from the pricing configuration at quote time.

---

## 7. Job Lifecycle

\`\`\`mermaid
sequenceDiagram
    participant U as User
    participant T as Terminal UI
    participant A as API Layer
    participant R as Rust Core
    participant P as Payment Engine
    participant S as Service Engine
    participant G as Report Engine

    U->>T: Open terminal / Start New Job
    T->>A: list services
    A->>R: services
    R-->>A: metadata + prices
    A-->>T: display ServiceCards
    U->>T: select service + submit input
    T->>A: create_job
    A->>R: create Job (status=Created)
    R-->>A: Job
    T->>A: create_invoice
    A->>P: create_invoice
    P->>R: store Invoice, set Job=Quoted → AwaitingPayment
    P-->>A: Invoice
    A-->>T: display InvoiceCard + wallet
    U->>Solana: transfer SOL
    loop every 5 s
        P->>Solana: get_signatures_for_address
        P->>P: verify amount + receiver
    end
    P->>R: update Job=PaymentConfirmed
    R->>S: execute service
    S->>AI: generate analysis
    AI-->>S: raw response
    S->>R: ServiceResponse
    R->>G: generate report
    G->>R: store ReportMetadata, set Job=Completed
    T->>A: poll job / reports
    A-->>T: ReportCard + download buttons
\`\`\`

### State Transitions

| State              | Trigger                          | Next State          |
|--------------------|----------------------------------|---------------------|
| Created            | Job created                     | Quoted              |
| Quoted             | Invoice generated                | AwaitingPayment     |
| AwaitingPayment    | Payment verified                 | PaymentConfirmed    |
| PaymentConfirmed   | Execution starts                 | Executing           |
| Executing          | Analysis finished                | GeneratingReport    |
| GeneratingReport   | Report written                   | Completed           |
| Completed          | Archive policy / manual          | Archived            |

Timeouts:

- Invoice expires after 15 minutes (configurable).
- Job timeout is 30 minutes (configurable).

---

## 8. Payment Architecture

\`\`\`mermaid
stateDiagram-v2
    [*] --> Created
    Created --> AwaitingPayment: invoice stored
    AwaitingPayment --> PendingConfirmation: signature detected
    PendingConfirmation --> Confirmed: verification success
    AwaitingPayment --> Expired: timeout
    PendingConfirmation --> Failed: verification failure
    Confirmed --> [*]
    Expired --> [*]
    Failed --> [*]
    AwaitingPayment --> Cancelled: manual cancel
\`\`\`

### Invoice Creation

1. Service type is mapped to a fixed price from \`pricing.toml\`.
2. A UUID reference is generated.
3. Merchant wallet address is taken from configuration.
4. Expiry is set to \`now + invoice_expiry_minutes\`.
5. Invoice is stored in \`AppState.invoice_cache\` and the associated job is moved to \`Quoted\` / \`AwaitingPayment\`.

### Verification

- Background monitor polls \`get_signatures_for_address\` on the merchant wallet.
- On candidate detection the full transaction is fetched with \`JsonParsed\` encoding.
- Amount is compared with a small floating-point tolerance (1e-7 SOL).
- Receiver must match the configured merchant wallet.
- Successful verification writes the signature onto the invoice, emits a \`PaymentDetected\` event, and advances the job to \`PaymentConfirmed\`.

### Failure Modes

- Amount mismatch → \`AmountMismatch\` error, invoice marked Failed.
- Receiver mismatch → \`ReceiverMismatch\` error.
- Transaction failure on-chain → verification rejected.
- Timeout → invoice marked Expired; job remains in \`AwaitingPayment\` until manually cancelled or archived.

No private keys are ever stored or used by ClawHire. The merchant wallet is a receive-only address.

---

## 9. Service Execution Architecture

### Smart Contract Security Review Pipeline

1. **Input normalisation** – GitHub URL → shallow clone; ZIP → extract; raw Rust → treat as single file.
2. **Workspace discovery** – look for \`Cargo.toml\` / \`Anchor.toml\`.
3. **Source collection** – walk directory for all \`.rs\` files (respecting max-file limits).
4. **Prompt loading** – read \`prompts/smart_contract_review.md\`.
5. **AI invocation** – system role + full source context, temperature 0.2, max 8192 tokens.
6. **Response parsing** – currently simplified (single finding); production path expects structured JSON or markdown sections.
7. **Risk scoring** – highest severity among findings becomes overall \`RiskLevel\`.
8. **Result packaging** – \`ServiceResponse\` with findings, summary, execution duration.

### On-chain Intelligence Pipeline

1. **Input validation** – must be valid Solana pubkey or signature.
2. **Data gathering** (via RPC):
   - SOL balance
   - SPL / Token-2022 holdings
   - NFT holdings
   - Recent signatures (capped at 100)
   - Program interaction summary
3. **Context assembly** – structured text block.
4. **Prompt loading** – \`prompts/onchain_intelligence.md\`.
5. **AI synthesis** – produces executive summary and risk assessment.
6. **Result packaging** – \`ServiceResponse\` (findings list empty; risk level derived from observable metrics).

Both pipelines are executed only after payment confirmation and never run in parallel for the same job.

---

## 10. Report Generation Architecture

1. Service response is handed to \`ReportGenerator\`.
2. Correct builder is selected by \`ServiceType\`.
3. Builder produces title, ordered sections, and statistics.
4. Markdown is rendered by concatenating sections with standard headings.
5. PDF is generated by a minimal single-page renderer using Helvetica (printpdf).
6. Files are written atomically under \`storage/reports/\`.
7. Metadata is inserted into the in-memory index and an event is emitted.
8. Frontend polls the report list and offers download buttons for both formats.

Naming convention:

\`\`\`
CHR-REPORT-YYYYMMDD-HHMMSS-<uuid8>.md
CHR-REPORT-YYYYMMDD-HHMMSS-<uuid8>.pdf
\`\`\`

History retention and optional compression are controlled by configuration flags.

---

## 11. Data Flow

\`\`\`mermaid
flowchart LR
    Browser -->|HTTP| Streamlit
    Streamlit -->|CLI / HTTP| API
    API -->|subprocess or REST| RustMain
    RustMain --> Core
    Core --> Services
    Core --> Payments
    Core --> Reports
    Services -->|HTTPS| AIProvider
    Payments -->|RPC| Solana
    Reports -->|fs| Storage
    Core -->|fs| Storage
    Storage -->|download| Streamlit
    Streamlit -->|browser download| User
\`\`\`

Every boundary is typed. Python models mirror the Rust structs via JSON. No unstructured string passing occurs between frontend and backend for critical data (job IDs, amounts, signatures).

---

## 12. Storage Architecture

\`\`\`
storage/
├── invoices/     # Serialized invoice records (future persistence)
├── reports/      # Final Markdown + PDF artifacts
├── history/      # Archived job snapshots
└── uploads/      # Temporary user uploads (ZIP, .rs files)
\`\`\`

- All paths are configurable.
- Uploads are size-limited (100 MB) and extension-restricted.
- Reports are never overwritten; each generation receives a unique filename.
- Directory creation is idempotent and performed at application start.

---

## 13. Logging Architecture

| Log File            | Content                                      | Rotation   |
|---------------------|----------------------------------------------|------------|
| \`logs/clawhire.log\` | Application events, job lifecycle, errors    | Daily, 30 files |
| \`logs/payments.log\` | Invoice creation, detection, verification    | Daily, 30 files |

- Logging level is controlled by \`logging.level\` (info by default).
- Console and file outputs can be independently enabled.
- Structured events are also kept in-memory in \`AppState.event_history\` for the current process lifetime.
- Frontend logs to \`logs/clawhire_ui.log\` via loguru.

---

## 14. Error Handling Strategy

### Recoverable Errors

- Transient RPC failures → retried with exponential backoff (max 5 attempts, 2 s base delay).
- AI provider timeouts → surfaced as job failure with clear message.
- Network errors in the Python adapter → retried by tenacity.

### Fatal Errors

- Missing required configuration → process exits at bootstrap.
- Invalid merchant wallet address → payment engine refuses to start.
- Unrecoverable filesystem errors during report write → job marked Failed.

### Graceful Degradation

- If PDF generation fails, Markdown is still delivered.
- If the background payment monitor crashes, the process logs the error and continues; a restart recovers pending invoices from the in-memory cache (or future persistent store).

All public methods return \`Result\` / \`anyhow::Result\` or domain-specific error enums (\`CoreError\`, \`PaymentError\`, \`ServiceError\`, \`ReportError\`).

---

## 15. Security Architecture

### Principles

- **No private keys** – the system never holds signing keys.
- **Receive-only wallet** – merchant address is public and configured explicitly.
- **Input validation** – Solana addresses and signatures are validated with the official crates; repository size and file counts are capped.
- **Prompt isolation** – system prompts live outside the binary and are never concatenated with untrusted user instructions beyond the designated context window.
- **Path traversal protection** – report and upload filenames are sanitized; \`..\`, \`/\`, and \`\\\` are rejected.
- **Shell safety** – the CLI adapter always passes argument vectors; never a shell string.
- **Rate limiting** – configurable requests-per-minute on the HTTP surface (when enabled).
- **CORS** – controllable allowed origins (default \`*\` for development).
- **Secrets** – API keys and any future credentials live only in \`.env\` or environment variables; never in TOML that is committed.

### Threat Model (High Level)

| Threat                        | Mitigation                                      |
|-------------------------------|-------------------------------------------------|
| Payment amount underpayment   | Exact amount check with tolerance               |
| Payment to wrong address      | Receiver validation                             |
| Malicious repository          | Size + file count limits, shallow clone         |
| Prompt injection              | Fixed system prompts, low temperature           |
| Path traversal in downloads   | Filename validation                             |
| Resource exhaustion           | Job concurrency limit, timeouts                 |

---

## 16. Performance Architecture

- **Async runtime** – Tokio with configurable worker threads (default 4).
- **Concurrency limits** – \`max_concurrent_jobs = 5\`, \`max_running_jobs = 5\`.
- **RPC polling** – 5-second interval; only pending invoices are examined.
- **Caching** – Python side caches services, wallet, version, and health responses with TTLs.
- **Streamlit** – CSS and static data are cached; background polls are lightweight.
- **Report generation** – single-threaded per job; PDF rendering is intentionally simple to keep latency low.
- **Clone depth** – GitHub clones use depth 1 to minimise bandwidth and time.

---

## 17. Deployment Architecture

Recommended platform: **Render**.

\`\`\`mermaid
flowchart TB
    subgraph Render
        WEB[Web Service<br/>Streamlit UI]
        WORKER[Background Worker<br/>Rust binary + payment monitor]
        DISK[Persistent Disk<br/>storage/ + logs/]
    end

    USER[User Browser] --> WEB
    WEB --> WORKER
    WORKER --> DISK
    WORKER --> SOL[Solana Devnet RPC]
    WORKER --> AI[AI Provider APIs]
\`\`\`

### Required Environment Variables

- \`MERCHANT_WALLET\`
- \`SOLANA_RPC_URL\`
- \`AI_PROVIDER\` / \`AI_API_KEY\` / \`AI_MODEL\`
- \`BACKEND_MODE\` (\`CLI\` or \`HTTP\`)
- \`RUST_EXECUTABLE_PATH\` (when using CLI adapter)

### Health Checks

- Rust: \`clawhire health\`
- Streamlit: HTTP endpoint returning 200 when the session is alive.

### Scaling

- Horizontal scaling of the Streamlit tier is safe (stateless).
- The Rust payment monitor should remain a single instance to avoid duplicate detection; future versions may move invoice state to a shared store.

---

## 18. Future Roadmap

- USDC and Solana Pay support.
- Mainnet configuration profile.
- Persistent invoice and job store (SQLite or Postgres).
- HTTP server mode for the Rust backend (eliminate CLI subprocess).
- Multi-agent collaboration via ZeroClaw plugins.
- WhatsApp / Telegram / Discord notification channels.
- Customer authentication and job history dashboard.
- Additional services (program IDL analysis, tokenomics review, etc.).
- Dynamic pricing and volume discounts.
- Hardware-wallet-friendly payment flows.

---

## 19. Architecture Decisions

| Decision                  | Rationale |
|---------------------------|-----------|
| Rust                      | Memory safety, performance, excellent async ecosystem, strong Solana crate support. |
| Tokio                     | Industry-standard async runtime; natural fit for concurrent job and payment monitoring. |
| Streamlit                 | Fast UI iteration while still permitting complete visual override into a terminal aesthetic. |
| Terminal UX               | Matches the “machine commerce” product identity; reduces dashboard complexity. |
| Solana Devnet first       | Safe testing environment; identical APIs to mainnet. |
| Fixed pricing in TOML     | Zero hard-coded money values; non-engineers can change prices without a rebuild. |
| Markdown + PDF            | Markdown is human-readable and version-controllable; PDF is the professional deliverable. |
| Five-module backend       | Forces clear boundaries and keeps the cognitive load low for new contributors. |
| Configuration externalised| Prompts, prices, and feature flags can be changed without recompilation. |
| No private keys           | Dramatically reduces the security surface and operational risk. |

---

## 20. Conclusion

ClawHire is designed as a small, focused, production-ready system rather than a sprawling platform. Its architecture emphasises:

- **Scalability** – clear concurrency limits, async I/O, and a path to shared storage.
- **Maintainability** – five backend modules, pure UI components, externalised configuration and prompts.
- **Security** – no private keys, strict input validation, receive-only payments, path sanitisation.
- **Reliability** – explicit job and invoice state machines, retries, timeouts, and graceful degradation.
- **Developer experience** – comprehensive configuration, structured logging, and a terminal UX that is both functional and distinctive.
- **Open-source friendliness** – MIT license, clear module boundaries, and documentation written for external contributors and reviewers.

The system is ready for self-hosted deployment, Superteam evaluation, and iterative extension while remaining faithful to its core principle: an AI that can be hired, paid on-chain, and trusted to deliver professional work product autonomously.

---

*End of Architecture Specification*  
*ClawHire v1.0.0 – 2026-07-25*
