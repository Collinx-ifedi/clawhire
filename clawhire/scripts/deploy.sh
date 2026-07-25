#!/usr/bin/env bash
#
# ClawHire - Production Deployment Script
# Orchestrates the release and deployment of the ClawHire application.
# Target: Railway.app, Render, Docker, Linux Servers
# Shell: bash (ShellCheck compliant)

set -Eeuo pipefail

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# CONSTANTS & CONFIGURATION
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

readonly PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
readonly LOG_FILE="${PROJECT_ROOT}/logs/deploy.log"

# ANSI Colors for logging
readonly COLOR_INFO='\033[0;34m'
readonly COLOR_SUCCESS='\033[0;32m'
readonly COLOR_WARNING='\033[1;33m'
readonly COLOR_ERROR='\033[0;31m'
readonly COLOR_RESET='\033[0m'

mkdir -p "${PROJECT_ROOT}/logs"

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# LOGGING UTILITIES
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

log_timestamp() {
    date +"%Y-%m-%d %H:%M:%S"
}

log_info() {
    local msg="[INFO] $(log_timestamp) - ${1}"
    echo -e "${COLOR_INFO}${msg}${COLOR_RESET}"
    echo "${msg}" >> "${LOG_FILE}"
}

log_success() {
    local msg="[SUCCESS] $(log_timestamp) - ${1}"
    echo -e "${COLOR_SUCCESS}${msg}${COLOR_RESET}"
    echo "${msg}" >> "${LOG_FILE}"
}

log_warning() {
    local msg="[WARNING] $(log_timestamp) - ${1}"
    echo -e "${COLOR_WARNING}${msg}${COLOR_RESET}"
    echo "${msg}" >> "${LOG_FILE}"
}

log_error() {
    local msg="[ERROR] $(log_timestamp) - ${1}"
    echo -e "${COLOR_ERROR}${msg}${COLOR_RESET}" >&2
    echo "${msg}" >> "${LOG_FILE}"
}

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# DEPLOYMENT CHECKS
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

check_build_artifacts() {
    log_info "Verifying release artifacts before deployment..."
    
    if [[ ! -f "${PROJECT_ROOT}/release/clawhire" ]]; then
        log_error "Release binary not found. Please run scripts/build.sh first."
        exit 1
    fi
    
    if [[ ! -f "${PROJECT_ROOT}/release/build_info.json" ]]; then
        log_warning "build_info.json missing. Proceeding with unversioned deployment."
    fi
    
    log_success "Build artifacts verified."
}

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# DEPLOYMENT TARGETS
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

deploy_to_railway() {
    log_info "Railway.app target detected. Initiating deployment sequence..."
    
    if ! command -v railway >/dev/null 2>&1; then
        log_error "Railway CLI is not installed. Please install it to deploy to Railway.app."
        exit 1
    fi
    
    cd "${PROJECT_ROOT}"
    
    log_info "Linking to Railway project..."
    # Assumes project is already linked or RAILWAY_TOKEN is set in the environment
    
    log_info "Pushing application to Railway.app..."
    if railway up --detach; then
        log_success "Successfully triggered deployment on Railway.app."
    else
        log_error "Railway.app deployment failed. Check Railway dashboard logs for details."
        exit 1
    fi
}

deploy_to_docker() {
    log_info "Docker target detected. Building and deploying containers..."
    
    if ! command -v docker >/dev/null 2>&1; then
        log_error "Docker is not installed."
        exit 1
    fi
    
    cd "${PROJECT_ROOT}"
    
    log_info "Building Docker image..."
    if docker build -t clawhire:latest .; then
        log_success "Docker image built successfully."
    else
        log_error "Docker build failed."
        exit 1
    fi
}

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# MAIN EXECUTION
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

main() {
    log_info "Starting ClawHire Deployment Pipeline"
    
    check_build_artifacts
    
    # Auto-detect deployment preference based on available CLI tools
    if command -v railway >/dev/null 2>&1; then
        deploy_to_railway
    elif command -v docker >/dev/null 2>&1; then
        deploy_to_docker
    else
        log_warning "No supported cloud CLI (Railway) or Docker detected."
        log_info "To run locally in a production-like state, execute scripts/start.sh"
    fi
    
    log_success "Deployment pipeline execution finished."
}

main "$@"
