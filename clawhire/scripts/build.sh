#!/usr/bin/env bash
#
# ClawHire - Production Build & Release Script
# Orchestrates validation, compilation, testing, and artifact packaging.
#
# Target: Linux, Render, macOS, WSL, Debian, Ubuntu
# Shell: bash (ShellCheck compliant)

set -Eeuo pipefail

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# CONSTANTS & CONFIGURATION
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

readonly PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
readonly BUILD_START_TIME=$(date +%s)
readonly LOG_FILE="${PROJECT_ROOT}/logs/build.log"

# ANSI Colors for logging
readonly COLOR_INFO='\033[0;34m'
readonly COLOR_SUCCESS='\033[0;32m'
readonly COLOR_WARNING='\033[1;33m'
readonly COLOR_ERROR='\033[0;31m'
readonly COLOR_DEBUG='\033[0;35m'
readonly COLOR_RESET='\033[0m'

# Ensure logs directory exists before opening log file
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

log_debug() {
    if [[ "${DEBUG:-false}" == "true" ]]; then
        local msg="[DEBUG] $(log_timestamp) - ${1}"
        echo -e "${COLOR_DEBUG}${msg}${COLOR_RESET}"
        echo "${msg}" >> "${LOG_FILE}"
    fi
}

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# SYSTEM CHECKS & BANNER
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
    log_info "Starting ClawHire Build & Release Pipeline"
    log_info "OS: $(uname -s) $(uname -m)"
}

check_dependencies() {
    log_info "Checking required toolchain and utilities..."
    local missing=0

    for cmd in cargo rustc python3 pip git; do
        if ! command -v "${cmd}" >/dev/null 2>&1; then
            log_error "Required command '${cmd}' is not installed or not in PATH."
            missing=$((missing + 1))
        fi
    done

    if [[ "${missing}" -gt 0 ]]; then
        log_error "Missing ${missing} required tool(s). Aborting build."
        exit 1
    fi

    log_success "All base system dependencies verified."
}

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# PROJECT STRUCTURE & FILE VALIDATION
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

validate_project_structure() {
    log_info "Validating required project structure and core files..."
    
    local required_files=(
        "Cargo.toml"
        ".env"
        "configs/app.toml"
        "configs/services.toml"
        "configs/pricing.toml"
        "src/main.rs"
        "src/core.rs"
        "src/services.rs"
        "src/payments.rs"
        "src/reports.rs"
        "ui/app.py"
        "ui/api.py"
        "ui/terminal.py"
        "ui/components.py"
        "ui/styles.py"
    )

    for file in "${required_files[@]}"; do
        if [[ ! -f "${PROJECT_ROOT}/${file}" ]]; then
            log_error "Missing required project file: ${file}"
            exit 1
        fi
        log_debug "Verified file exists: ${file}"
    done

    log_success "Project structure validation passed."
}

initialize_directories() {
    log_info "Initializing build and storage directories..."
    local dirs=(
        "build"
        "dist"
        "release"
        "logs"
        "storage/history"
        "storage/reports"
        "storage/uploads"
        "storage/invoices"
        "assets/reports"
        "assets/uploads"
    )

    for dir in "${dirs[@]}"; do
        mkdir -p "${PROJECT_ROOT}/${dir}"
        log_debug "Ensured directory exists: ${dir}"
    done
    log_success "Directories initialized."
}

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# RUST BUILD PIPELINE
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

build_rust_backend() {
    log_info "Starting Rust backend verification and compilation..."
    cd "${PROJECT_ROOT}"

    log_info "Running cargo fmt check..."
    if ! cargo fmt --all --check; then
        log_error "Cargo formatting check failed. Run 'cargo fmt' to fix."
        exit 1
    fi

    log_info "Running cargo clippy lints..."
    if ! cargo clippy --all-targets --all-features -- -D warnings; then
        log_error "Clippy lint checks failed."
        exit 1
    fi

    log_info "Compiling Rust backend in release mode..."
    if ! cargo build --release; then
        log_error "Cargo build failed."
        exit 1
    fi

    local binary_path="${PROJECT_ROOT}/target/release/clawhire"
    if [[ ! -f "${binary_path}" ]]; then
        log_error "Expected compiled binary not found at ${binary_path}"
        exit 1
    fi

    local binary_size
    binary_size=$(du -h "${binary_path}" | cut -f1)
    local rust_version
    rust_version=$(rustc --version)

    log_success "Rust compilation successful. Binary size: ${binary_size} | Version: ${rust_version}"
}

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# PYTHON VALIDATION PIPELINE
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

validate_python_environment() {
    log_info "Validating Python environment and bytecode compilation..."
    cd "${PROJECT_ROOT}"

    local py_version
    py_version=$(python3 -c 'import sys; print(f"{sys.version_info.major}.{sys.version_info.minor}.{sys.version_info.micro}")')
    log_info "Python version detected: ${py_version}"

    if [[ ! -d "${PROJECT_ROOT}/.venv" ]]; then
        log_info "Virtual environment not found. Creating .venv..."
        python3 -m venv "${PROJECT_ROOT}/.venv"
    fi

    # shellcheck disable=SC1091
    source "${PROJECT_ROOT}/.venv/bin/activate"

    if [[ -f "${PROJECT_ROOT}/requirements.txt" ]]; then
        log_info "Installing / updating Python requirements..."
        pip install --quiet --upgrade pip
        pip install --quiet -r "${PROJECT_ROOT}/requirements.txt"
    else
        log_warning "requirements.txt not found. Skipping dependency installation."
    fi

    log_info "Verifying required Python module imports..."
    python3 -c "
import streamlit, httpx, pydantic, loguru, toml, rich, markdown, reportlab
print('All critical Python packages imported successfully.')
" || {
        log_error "One or more required Python packages failed to import."
        exit 1
    }

    log_info "Compiling UI modules to bytecode..."
    local ui_files=("ui/app.py" "ui/api.py" "ui/terminal.py" "ui/components.py" "ui/styles.py")
    for f in "${ui_files[@]}"; do
        if ! python3 -m py_compile "${PROJECT_ROOT}/${f}"; then
            log_error "Bytecode compilation failed for ${f}"
            exit 1
        fi
        log_debug "Successfully compiled bytecode for ${f}"
    done

    log_success "Python validation completed successfully."
}

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# CONFIGURATION VALIDATION
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

validate_configurations() {
    log_info "Validating configuration files and environment syntax..."
    cd "${PROJECT_ROOT}"

    # Validate TOML configurations using Python
    python3 -c "
import toml, sys
for cfg in ['configs/app.toml', 'configs/services.toml', 'configs/pricing.toml']:
    try:
        toml.load(cfg)
        print(f'Validated TOML: {cfg}')
    except Exception as e:
        print(f'Error parsing {cfg}: {e}', file=sys.stderr)
        sys.exit(1)
" || {
        log_error "TOML configuration syntax validation failed."
        exit 1
    }

    # Validate .env syntax (no spaces around equals, basic check)
    if [[ -f "${PROJECT_ROOT}/.env" ]]; then
        while IFS= read -r line || [[ -n "${line}" ]]; do
            # Skip comments and empty lines
            [[ "${line}" =~ ^[[:space:]]*# ]] && continue
            [[ -z "${line// /}" ]] && continue
            if [[ ! "${line}" =~ ^[A-Za-z_][A-Za-z0-9_]*= ]]; then
                log_warning "Suspicious line in .env format: ${line}"
            fi
        done < "${PROJECT_ROOT}/.env"
    fi

    log_success "Configuration syntax validation passed."
}

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# OPTIONAL TESTS
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

run_optional_tests() {
    log_info "Checking and running test suites if available..."
    cd "${PROJECT_ROOT}"

    if cargo test --all; then
        log_success "Cargo backend tests passed."
    else
        log_error "Cargo backend tests failed."
        exit 1
    fi

    if command -v pytest >/dev/null 2>&1 && [[ -d "tests" ]]; then
        if pytest; then
            log_success "Pytest suite passed."
        else
            log_error "Pytest suite failed."
            exit 1
        fi
    else
        log_info "No pytest directory or executable found. Skipping Python tests."
    fi
}

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# RELEASE ARTIFACTS & METADATA
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

generate_build_metadata() {
    log_info "Generating build metadata and release manifest..."
    cd "${PROJECT_ROOT}"

    local commit_hash
    commit_hash=$(git rev-parse HEAD 2>/dev/null || echo "unknown")
    local branch_name
    branch_name=$(git rev-parse --symbolic-full-name --abbrev-ref HEAD 2>/dev/null || echo "unknown")
    local build_timestamp
    build_timestamp=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
    local rust_version
    rust_version=$(rustc --version)
    local cargo_version
    cargo_version=$(cargo --version)
    local py_version
    py_version=$(python3 --version)
    local os_info
    os_info=$(uname -s)
    local arch_info
    arch_info=$(uname -m)

    local metadata_file="${PROJECT_ROOT}/release/build_info.json"
    cat << EOF > "${metadata_file}"
{
  "project_name": "ClawHire",
  "version": "1.0.0",
  "git_commit": "${commit_hash}",
  "git_branch": "${branch_name}",
  "build_timestamp": "${build_timestamp}",
  "rust_version": "${rust_version}",
  "cargo_version": "${cargo_version}",
  "python_version": "${py_version}",
  "operating_system": "${os_info}",
  "architecture": "${arch_info}",
  "build_mode": "release"
}
EOF

    log_success "Build metadata generated at release/build_info.json."
}

package_release_artifacts() {
    log_info "Packaging release artifacts..."
    cd "${PROJECT_ROOT}"

    local rel_dir="${PROJECT_ROOT}/release"
    
    # Copy compiled binary
    cp "${PROJECT_ROOT}/target/release/clawhire" "${rel_dir}/clawhire"
    chmod +x "${rel_dir}/clawhire"

    # Copy docs, configs, and prompts if present
    [[ -d "configs" ]] && cp -r "configs" "${rel_dir}/"
    [[ -d "docs" ]] && cp -r "docs" "${rel_dir}/"
    [[ -d "prompts" ]] && cp -r "prompts" "${rel_dir}/"
    [[ -f "README.md" ]] && cp "README.md" "${rel_dir}/"
    [[ -f "LICENSE" ]] && cp "LICENSE" "${rel_dir}/" || true

    # Generate checksums (SHA-256)
    log_info "Generating SHA-256 checksums for release artifacts..."
    cd "${rel_dir}"
    if command -v sha256sum >/dev/null 2>&1; then
        sha256sum clawhire configs/*.toml > checksums.sha256
    elif command -v shasum >/dev/null 2>&1; then
        shasum -a 256 clawhire configs/*.toml > checksums.sha256
    fi
    cd "${PROJECT_ROOT}"

    log_success "Release artifacts packaged successfully in release/."
}

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# MAIN EXECUTION
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

main() {
    print_banner
    check_dependencies
    validate_project_structure
    initialize_directories
    
    validate_configurations
    build_rust_backend
    validate_python_environment
    run_optional_tests
    
    generate_build_metadata
    package_release_artifacts

    local build_end_time
    build_end_time=$(date +%s)
    local duration=$((build_end_time - BUILD_START_TIME))

    echo -e "\n${COLOR_SUCCESS}============================================================${COLOR_RESET}"
    log_success "ClawHire Build Completed Successfully in ${duration} seconds."
    echo -e "${COLOR_SUCCESS}============================================================${COLOR_RESET}"
    echo -e "Release Artifacts Location : ${PROJECT_ROOT}/release"
    echo -e "Build Log Location         : ${LOG_FILE}"
    echo -e "${COLOR_SUCCESS}============================================================${COLOR_RESET}\n"
}

main "$@"
