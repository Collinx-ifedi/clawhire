#!/usr/bin/env bash
#
# ClawHire - Production Start Script
# Orchestrates the execution of the Rust backend and Streamlit frontend.
#
# Target: Linux, Render, macOS, WSL, Debian, Ubuntu
# Shell: bash (ShellCheck compliant)

set -Eeuo pipefail

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# CONSTANTS & CONFIGURATION
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

readonly PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
readonly RUST_BINARY="target/release/clawhire"
readonly PYTHON_ENTRY="ui/app.py"

BACKEND_PID=""
FRONTEND_PID=""

# ANSI Colors for logging
readonly COLOR_INFO='\033[0;34m'
readonly COLOR_SUCCESS='\033[0;32m'
readonly COLOR_WARNING='\033[1;33m'
readonly COLOR_ERROR='\033[0;31m'
readonly COLOR_DEBUG='\033[0;35m'
readonly COLOR_RESET='\033[0m'

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# LOGGING UTILITIES
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

log_timestamp() {
    date +"%Y-%m-%d %H:%M:%S"
}

log_info() {
    echo -e "${COLOR_INFO}[INFO] $(log_timestamp) - ${1}${COLOR_RESET}"
}

log_success() {
    echo -e "${COLOR_SUCCESS}[SUCCESS] $(log_timestamp) - ${1}${COLOR_RESET}"
}

log_warning() {
    echo -e "${COLOR_WARNING}[WARNING] $(log_timestamp) - ${1}${COLOR_RESET}"
}

log_error() {
    echo -e "${COLOR_ERROR}[ERROR] $(log_timestamp) - ${1}${COLOR_RESET}" >&2
}

log_debug() {
    if [[ "${DEBUG:-false}" == "true" ]]; then
        echo -e "${COLOR_DEBUG}[DEBUG] $(log_timestamp) - ${1}${COLOR_RESET}"
    fi
}

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# LIFECYCLE MANAGEMENT
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

cleanup() {
    local exit_code=$?
    log_info "Signal caught or exit triggered. Initiating graceful shutdown..."
    
    if [[ -n "${FRONTEND_PID}" ]] && kill -0 "${FRONTEND_PID}" 2>/dev/null; then
        log_info "Stopping Streamlit frontend (PID: ${FRONTEND_PID})..."
        kill -TERM "${FRONTEND_PID}" || true
    fi

    if [[ -n "${BACKEND_PID}" ]] && kill -0 "${BACKEND_PID}" 2>/dev/null; then
        log_info "Stopping Rust backend (PID: ${BACKEND_PID})..."
        kill -TERM "${BACKEND_PID}" || true
    fi

    # Wait for processes to exit
    wait "${FRONTEND_PID}" "${BACKEND_PID}" 2>/dev/null || true
    
    log_success "Shutdown complete."
    exit "${exit_code}"
}

trap cleanup SIGINT SIGTERM SIGHUP EXIT

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# SYSTEM CHECKS & INITIALIZATION
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

print_banner() {
    echo -e "${COLOR_SUCCESS}"
    cat << "EOF"
   ____  _                  _    _  _             
  / ___|| |  __ _ __      _| |  | |(_) _ __  ___  
 | |    | | / _` |\ \ /\ / / |_ | || || '__|/ _ \ 
 | |___ | || (_| | \ V  V /|  _|| || || |  |  __/ 
  \____||_| \__,_|  \_/\_/ |_|  |_||_||_|   \___| 
EOF
    echo -e "${COLOR_RESET}"
    log_info "Starting ClawHire Execution Lifecycle"
    log_info "OS: $(uname -s) $(uname -m)"
}

initialize_directories() {
    log_info "Initializing required directories..."
    local dirs=(
        "logs"
        "storage/history"
        "storage/reports"
        "storage/uploads"
        "storage/invoices"
        "assets/reports"
        "assets/uploads"
        "configs"
    )

    for dir in "${dirs[@]}"; do
        if [[ ! -d "${PROJECT_ROOT}/${dir}" ]]; then
            mkdir -p "${PROJECT_ROOT}/${dir}"
            log_debug "Created directory: ${dir}"
        fi
    done
    log_success "Directory structure verified."
}

validate_environment() {
    log_info "Validating environment configuration..."
    if [[ -f "${PROJECT_ROOT}/.env" ]]; then
        log_info "Found .env file. Loading configuration..."
        # Safely export vars from .env, ignoring comments and empty lines
        set -a
        # shellcheck disable=SC1091
        source <(grep -v '^#' "${PROJECT_ROOT}/.env" | grep -v '^[[:space:]]*$')
        set +a
    else
        log_warning "No .env file found in ${PROJECT_ROOT}. Relying on system environment variables."
    fi
}

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# RUST BACKEND PIPELINE
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

prepare_rust_backend() {
    log_info "Checking Rust backend binary..."
    
    if [[ ! -f "${PROJECT_ROOT}/${RUST_BINARY}" ]]; then
        log_warning "Compiled Rust binary not found at ${RUST_BINARY}."
        
        if ! command -v cargo >/dev/null 2>&1; then
            log_error "cargo is not installed or not in PATH. Cannot build backend."
            exit 1
        fi
        
        log_info "Building ClawHire backend (Release profile). This may take a few minutes..."
        cd "${PROJECT_ROOT}"
        if cargo build --release; then
            log_success "Rust backend successfully compiled."
        else
            log_error "Failed to compile Rust backend."
            exit 1
        fi
    else
        log_info "Existing Rust binary found. Reusing compiled artifact."
    fi
}

start_rust_backend() {
    log_info "Starting Rust backend..."
    
    cd "${PROJECT_ROOT}"
    
    # Execute backend and capture output to logs
    "${PROJECT_ROOT}/${RUST_BINARY}" > "${PROJECT_ROOT}/logs/clawhire.log" 2>&1 &
    BACKEND_PID=$!
    
    log_info "Backend process launched with PID: ${BACKEND_PID}"
    
    # Wait and verify health (checking if process stays alive)
    local retries=5
    local wait_time=1
    
    for ((i=1; i<=retries; i++)); do
        if ! kill -0 "${BACKEND_PID}" 2>/dev/null; then
            log_error "Backend process died unexpectedly during startup."
            log_error "Check logs/clawhire.log for details."
            exit 1
        fi
        sleep "${wait_time}"
    done
    
    log_success "Backend is healthy and running."
}

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# PYTHON FRONTEND PIPELINE
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

prepare_python_environment() {
    log_info "Validating Python environment..."
    
    if ! command -v python3 >/dev/null 2>&1; then
        log_error "python3 is required but not found in PATH."
        exit 1
    fi
    
    local py_version
    py_version=$(python3 -c 'import sys; print(f"{sys.version_info.major}.{sys.version_info.minor}")')
    log_info "Detected Python version: ${py_version}"
    
    # Ensure venv exists
    if [[ ! -d "${PROJECT_ROOT}/.venv" ]]; then
        log_info "Creating Python virtual environment in .venv..."
        python3 -m venv "${PROJECT_ROOT}/.venv"
    fi
    
    log_info "Activating virtual environment..."
    # shellcheck disable=SC1091
    source "${PROJECT_ROOT}/.venv/bin/activate"
    
    # Validate Streamlit presence; install requirements if missing
    if ! command -v streamlit >/dev/null 2>&1; then
        if [[ -f "${PROJECT_ROOT}/requirements.txt" ]]; then
            log_info "Streamlit not found. Installing dependencies from requirements.txt..."
            pip install --quiet --upgrade pip
            pip install --quiet -r "${PROJECT_ROOT}/requirements.txt"
        else
            log_error "Streamlit is not installed and requirements.txt is missing."
            exit 1
        fi
    fi
    
    log_success "Python environment is ready."
}

start_python_frontend() {
    log_info "Starting Streamlit frontend..."
    
    cd "${PROJECT_ROOT}"
    
    local host="${STREAMLIT_SERVER_ADDRESS:-0.0.0.0}"
    local port="${STREAMLIT_SERVER_PORT:-8501}"
    
    streamlit run "${PYTHON_ENTRY}" \
        --server.address="${host}" \
        --server.port="${port}" \
        --server.headless=true \
        --browser.gatherUsageStats=false \
        --logger.level=error \
        > "${PROJECT_ROOT}/logs/streamlit.log" 2>&1 &
        
    FRONTEND_PID=$!
    
    log_info "Frontend process launched with PID: ${FRONTEND_PID}"
    
    # Verify health
    sleep 3
    if ! kill -0 "${FRONTEND_PID}" 2>/dev/null; then
        log_error "Streamlit frontend died unexpectedly."
        log_error "Check logs/streamlit.log for details."
        exit 1
    fi
    
    log_success "Frontend is healthy and listening on ${host}:${port}."
}

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# MAIN EXECUTION
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

main() {
    print_banner
    initialize_directories
    validate_environment
    
    prepare_rust_backend
    prepare_python_environment
    
    start_rust_backend
    start_python_frontend
    
    log_success "ClawHire application is fully operational."
    log_info "Monitoring processes (Backend: ${BACKEND_PID}, Frontend: ${FRONTEND_PID})..."
    
    # Wait for any background process to exit
    wait -n "${BACKEND_PID}" "${FRONTEND_PID}"
    
    local exit_status=$?
    log_warning "A monitored process has terminated. Initiating teardown."
    exit "${exit_status}"
}

# Execute main
main "$@"
