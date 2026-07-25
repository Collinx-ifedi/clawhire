# 🦞 ClawHire

> **A Self-Hosted Solana AI Freelancer powered by ZeroClaw**

ClawHire is a production-grade Rust application built on top of the ZeroClaw AI Runtime. It allows users to hire an autonomous AI blockchain consultant, securely pay using Solana, and automatically receive professional blockchain analysis reports.

Unlike traditional AI chatbots, ClawHire is designed around **machine commerce**.

The AI offers professional blockchain services, generates an invoice, waits for blockchain payment, verifies the transaction, performs the requested work, and delivers a professional report—all without human intervention after payment.

---

# Features

## Smart Contract Security Review

Upload a Solana smart contract for a professional security review.

Supported inputs:

- GitHub Repository
- ZIP Archive
- Rust Source Files
- Pasted Source Code

Generated report includes:

- Security Score
- Critical Vulnerabilities
- High Risk Findings
- Medium Risk Findings
- Low Risk Findings
- Best Practices
- Recommendations
- Professional PDF Report

---

## On-chain Intelligence Report

Investigate any Solana wallet or transaction.

Supported inputs:

- Wallet Address
- Transaction Signature

Generated report includes:

- SOL Balance
- Token Holdings
- NFT Holdings
- Recent Activity
- Program Interactions
- AI-generated Executive Summary
- Risk Indicators
- Professional PDF Report

---

# Machine Commerce Workflow

```
Choose Service

↓

Submit Input

↓

Invoice Generated

↓

User Sends SOL

↓

Payment Verification

↓

Service Execution

↓

Report Generation

↓

Report Delivery
```

---

# Architecture

```
Browser

↓

Streamlit Terminal

↓

Rust Backend

↓

ZeroClaw Runtime

↓

Solana RPC
```

---

# Technology Stack

Backend

- Rust

Frontend

- Streamlit

Runtime

- ZeroClaw

Blockchain

- Solana

Reports

- Markdown
- PDF

Deployment

- Render

---

# Project Structure

```
clawhire/
│
├── Cargo.toml
├── Cargo.lock
├── .env
├── .gitignore
├── README.md
│
├── assets/
│   ├── logo/
│   ├── reports/
│   └── uploads/
│
├── configs/
│   ├── app.toml
│   ├── services.toml
│   └── pricing.toml
│
├── prompts/
│   ├── smart_contract_review.md
│   └── onchain_intelligence.md
│
├── src/
│   ├── main.rs
│   ├── core.rs
│   ├── services.rs
│   ├── payments.rs
│   └── reports.rs
│
├── ui/
│   ├── app.py
│   ├── terminal.py
│   ├── api.py
│   ├── styles.py
│   └── components.py
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
    ├── architecture.md
    ├── api.md
    └── workflow.md
```

---

# Rust Backend

The backend intentionally contains only **five modules**.

| Module | Responsibility |
|---------|----------------|
| main.rs | Application startup |
| core.rs | Runtime, state management, events |
| services.rs | Blockchain services |
| payments.rs | Payment engine |
| reports.rs | Report generation |

---

# Service Lifecycle

Every request follows the same lifecycle.

```
Created

↓

Quoted

↓

Awaiting Payment

↓

Payment Confirmed

↓

Executing

↓

Generating Report

↓

Completed

↓

Archived
```

---

# Configuration

Application configuration is stored inside the `configs` directory.

```
configs/

app.toml

services.toml

pricing.toml
```

This allows pricing, services, and runtime behavior to be modified without recompiling the application.

---

# AI Prompts

Prompt engineering is separated from Rust source code.

```
prompts/

smart_contract_review.md

onchain_intelligence.md
```

Updating prompts never requires rebuilding the binary.

---

# Streamlit Terminal

ClawHire intentionally avoids a traditional dashboard.

Instead, users interact through a terminal-style interface that simulates a professional command-line environment.

Typical workflow:

```
> services

↓

Choose Service

↓

Submit Input

↓

Invoice Generated

↓

Awaiting Payment

↓

Payment Confirmed

↓

Executing Service

↓

Generating Report

↓

Done
```

---

# Reports

Generated reports are stored in

```
storage/reports/
```

Each report includes

- Metadata
- Executive Summary
- Findings
- Risk Assessment
- Recommendations
- Generation Timestamp

Reports can be exported as Markdown or PDF.

---

# Logging

Application logs

```
logs/clawhire.log
```

Payment logs

```
logs/payments.log
```

---

# Configuration via Environment Variables

Environment variables are stored in

```
.env
```

Configuration includes

- Wallet Address
- RPC Endpoint
- AI Provider
- API Keys
- Upload Paths
- Report Paths
- Logging
- Feature Flags

---

# Deployment

Recommended platform

- Render

Other supported environments

- Linux
- Docker
- VPS
- Bare Metal

---

# Development Roadmap

Current Version

- Smart Contract Security Review
- On-chain Intelligence Report
- Solana Payments
- Streamlit Terminal
- ZeroClaw Runtime Integration

Future

- USDC Payments
- Solana Pay Support
- Multi-agent Collaboration
- Additional Blockchain Services
- Customer Authentication
- Job History Dashboard
- Plugin Marketplace

---

# Design Principles

ClawHire follows a few core principles.

- Minimal architecture
- Production-quality Rust
- Self-hosted by default
- Solana-native payments
- AI-first workflows
- Separation of concerns
- Clean module boundaries
- Reproducible deployments

---

# License

MIT License

---

# Acknowledgements

Built using

- Rust
- ZeroClaw
- Solana
- Streamlit

---

## ClawHire

**A Self-Hosted Solana AI Freelancer.**
```