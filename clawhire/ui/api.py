"""
ClawHire - API Communication Module
Version: 1.0.0

ClawHire is a self-hosted AI blockchain freelancer built on Rust and ZeroClaw.[span_0](start_span)[span_0](end_span)[span_1](start_span)[span_1](end_span)
The frontend is written in Streamlit.[span_2](start_span)[span_2](end_span)
The backend is written in Rust.[span_3](start_span)[span_3](end_span)

This module is the ONLY communication layer between the Streamlit frontend and the Rust backend.[span_4](start_span)[span_4](end_span)
It abstracts every backend interaction behind a clean Python API.[span_5](start_span)[span_5](end_span)
"""

import asyncio
import json
import os
import shlex
import subprocess
import tomllib
from abc import ABC, abstractmethod
from dataclasses import dataclass, field
from datetime import datetime
from enum import Enum
from pathlib import Path
from typing import Any, Dict, List, Optional, Union
import uuid

import httpx
from cachetools import TTLCache, cached
from loguru import logger
from pydantic import BaseModel, Field
from tenacity import (
    retry,
    stop_after_attempt,
    wait_exponential,
    retry_if_exception_type,
)

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# ENUMS
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

class BackendMode(str, Enum):
    """Execution mode for the backend.[span_6](start_span)[span_6](end_span)"""
    CLI = "CLI"
    HTTP = "HTTP"
    AUTO = "AUTO"

class ConnectionStatus(str, Enum):
    """Current status of the backend connection.[span_7](start_span)[span_7](end_span)"""
    CONNECTED = "CONNECTED"
    DISCONNECTED = "DISCONNECTED"
    CONNECTING = "CONNECTING"
    ERROR = "ERROR"

class ServiceType(str, Enum):
    """Available AI blockchain services.[span_8](start_span)[span_8](end_span)"""
    SmartContractReview = "SmartContractReview"
    OnChainIntelligence = "OnChainIntelligence"

class JobStatus(str, Enum):
    """Lifecycle status of a service job.[span_9](start_span)[span_9](end_span)"""
    Created = "Created"
    Quoted = "Quoted"
    AwaitingPayment = "AwaitingPayment"
    PaymentDetected = "PaymentDetected"
    Executing = "Executing"
    GeneratingReport = "GeneratingReport"
    Completed = "Completed"
    Failed = "Failed"
    Cancelled = "Cancelled"

class InvoiceStatus(str, Enum):
    """Lifecycle status of a payment invoice.[span_10](start_span)[span_10](end_span)"""
    Created = "Created"
    AwaitingPayment = "AwaitingPayment"
    PendingConfirmation = "PendingConfirmation"
    Confirmed = "Confirmed"
    Expired = "Expired"
    Failed = "Failed"


# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# PYDANTIC MODELS
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

class HealthResponse(BaseModel):
    status: str
    uptime: int
    timestamp: datetime = Field(default_factory=datetime.utcnow)

class VersionResponse(BaseModel):
    version: str
    build_hash: str
    environment: str

class JobResponse(BaseModel):
    job_id: uuid.UUID
    service_type: ServiceType
    status: JobStatus
    created_at: datetime
    updated_at: datetime

class InvoiceResponse(BaseModel):
    invoice_id: uuid.UUID
    job_id: uuid.UUID
    amount: float
    currency: str
    wallet_address: str
    status: InvoiceStatus
    expires_at: datetime

class ReportResponse(BaseModel):
    report_id: uuid.UUID
    job_id: uuid.UUID
    title: str
    summary: str
    created_at: datetime
    markdown_path: str
    pdf_path: str

class WalletResponse(BaseModel):
    address: str
    network: str
    balance: float

class ServiceResponse(BaseModel):
    name: str
    service_type: ServiceType
    description: str
    base_price: float
    currency: str

class ConfigurationResponse(BaseModel):
    environment: str
    rpc_endpoint: str
    features: Dict[str, bool]

class PaymentResponse(BaseModel):
    transaction_signature: str
    status: str
    amount: float
    timestamp: datetime

class SystemStatus(BaseModel):
    connection: ConnectionStatus
    backend_mode: BackendMode
    active_jobs: int


# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# CONFIGURATION LOADER
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

class ConfigurationLoader:
    """
    Loads application configuration from .env and TOML files.[span_11](start_span)[span_11](end_span)
    """
    def __init__(self):
        self.config_dir = Path("configs")
        self.env_file = Path(".env")
        self.settings: Dict[str, Any] = {}
        self.load()

    def load(self) -> None:
        if self.env_file.exists():
            # Basic .env parsing for non-secrets needed in Python layer
            with open(self.env_file, "r") as f:
                for line in f:
                    if line.strip() and not line.startswith("#"):
                        key, val = line.strip().split("=", 1)
                        os.environ[key] = val

        app_toml = self.config_dir / "app.toml"
        if app_toml.exists():
            with open(app_toml, "rb") as f:
                self.settings = tomllib.load(f)

    @property
    def backend_mode(self) -> BackendMode:
        mode = os.getenv("BACKEND_MODE", self.settings.get("backend_mode", "AUTO"))
        return BackendMode(mode.upper())

    @property
    def executable_path(self) -> Path:
        path = os.getenv("RUST_EXECUTABLE_PATH", "target/release/clawhire")
        return Path(path)

    @property
    def http_url(self) -> str:
        return os.getenv("BACKEND_HTTP_URL", "http://127.0.0.1:8080")

    @property
    def timeout(self) -> int:
        return int(os.getenv("API_TIMEOUT", 30))

    @property
    def retry_count(self) -> int:
        return int(os.getenv("API_RETRY_COUNT", 3))

    @property
    def download_directory(self) -> Path:
        path = Path(os.getenv("DOWNLOAD_DIR", "storage/downloads"))
        path.mkdir(parents=True, exist_ok=True)
        return path


# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# CACHE MANAGER
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

class CacheManager:
    """
    Manages TTL caches for static or semi-static responses.[span_12](start_span)[span_12](end_span)
    """
    def __init__(self):
        self.services_cache = TTLCache(maxsize=1, ttl=3600)
        self.wallet_cache = TTLCache(maxsize=1, ttl=300)
        self.version_cache = TTLCache(maxsize=1, ttl=86400)
        self.config_cache = TTLCache(maxsize=1, ttl=3600)
        self.health_cache = TTLCache(maxsize=1, ttl=10)


# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# ABSTRACT BASE CLASS: BackendAdapter
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

class BackendAdapter(ABC):
    """
    Abstract interface for Backend communication.[span_13](start_span)[span_13](end_span)
    """
    @abstractmethod
    async def health(self) -> HealthResponse: pass

    @abstractmethod
    async def version(self) -> VersionResponse: pass

    @abstractmethod
    async def services(self) -> List[ServiceResponse]: pass

    @abstractmethod
    async def create_job(self, service_type: ServiceType, inputs: Dict[str, Any]) -> JobResponse: pass

    @abstractmethod
    async def create_invoice(self, job_id: uuid.UUID) -> InvoiceResponse: pass

    @abstractmethod
    async def wait_payment(self, invoice_id: uuid.UUID) -> PaymentResponse: pass

    @abstractmethod
    async def job(self, job_id: uuid.UUID) -> JobResponse: pass

    @abstractmethod
    async def jobs(self) -> List[JobResponse]: pass

    @abstractmethod
    async def invoice(self, invoice_id: uuid.UUID) -> InvoiceResponse: pass

    @abstractmethod
    async def reports(self, job_id: uuid.UUID) -> List[ReportResponse]: pass

    @abstractmethod
    async def download(self, report_id: uuid.UUID, format_type: str) -> bytes: pass

    @abstractmethod
    async def wallet(self) -> WalletResponse: pass

    @abstractmethod
    async def status(self) -> SystemStatus: pass

    @abstractmethod
    async def configuration(self) -> ConfigurationResponse: pass

    @abstractmethod
    async def shutdown(self) -> None: pass


# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# RUST CLI ADAPTER
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

class RustCLIAdapter(BackendAdapter):
    """
    Executes the Rust binary as a subprocess.[span_14](start_span)[span_14](end_span)
    """
    def __init__(self, executable_path: Path, timeout: int):
        self.executable_path = executable_path
        self.timeout = timeout
        if not self.executable_path.exists():
            logger.warning(f"Rust executable not found at {self.executable_path}")

    @retry(stop=stop_after_attempt(3), wait=wait_exponential(multiplier=1, min=2, max=10), retry=retry_if_exception_type(subprocess.TimeoutExpired))
    async def execute(self, args: List[str]) -> str:
        """Executes CLI commands securely using argument lists.[span_15](start_span)[span_15](end_span)"""
        cmd = [str(self.executable_path)] + args
        logger.debug(f"Executing CLI: {shlex.join(cmd)}")
        
        loop = asyncio.get_running_loop()
        try:
            process = await loop.run_in_executor(
                None,
                lambda: subprocess.run(
                    cmd,
                    capture_output=True,
                    text=True,
                    timeout=self.timeout,
                    check=True
                )
            )
            return process.stdout
        except subprocess.CalledProcessError as e:
            logger.error(f"CLI Error: {e.stderr}")
            raise RuntimeError(f"Backend execution failed: {e.stderr}")

    async def execute_json(self, args: List[str]) -> Any:
        """Parses standard output as JSON.[span_16](start_span)[span_16](end_span)"""
        stdout = await self.execute(args)
        try:
            return json.loads(stdout)
        except json.JSONDecodeError as e:
            logger.error(f"Failed to parse JSON output: {stdout}")
            raise ValueError("Invalid JSON response from backend")

    async def health(self) -> HealthResponse:
        res = await self.execute_json(["--format", "json", "health"])
        return HealthResponse(**res)

    async def version(self) -> VersionResponse:
        res = await self.execute_json(["--format", "json", "version"])
        return VersionResponse(**res)

    async def services(self) -> List[ServiceResponse]:
        res = await self.execute_json(["--format", "json", "services", "list"])
        return [ServiceResponse(**s) for s in res]

    async def create_job(self, service_type: ServiceType, inputs: Dict[str, Any]) -> JobResponse:
        args = ["--format", "json", "jobs", "create", "--service", service_type.value]
        for k, v in inputs.items():
            args.extend([f"--{k}", str(v)])
        res = await self.execute_json(args)
        return JobResponse(**res)

    async def create_invoice(self, job_id: uuid.UUID) -> InvoiceResponse:
        res = await self.execute_json(["--format", "json", "invoices", "create", "--job-id", str(job_id)])
        return InvoiceResponse(**res)

    async def wait_payment(self, invoice_id: uuid.UUID) -> PaymentResponse:
        res = await self.execute_json(["--format", "json", "payments", "wait", "--invoice-id", str(invoice_id)])
        return PaymentResponse(**res)

    async def job(self, job_id: uuid.UUID) -> JobResponse:
        res = await self.execute_json(["--format", "json", "jobs", "get", str(job_id)])
        return JobResponse(**res)

    async def jobs(self) -> List[JobResponse]:
        res = await self.execute_json(["--format", "json", "jobs", "list"])
        return [JobResponse(**j) for j in res]

    async def invoice(self, invoice_id: uuid.UUID) -> InvoiceResponse:
        res = await self.execute_json(["--format", "json", "invoices", "get", str(invoice_id)])
        return InvoiceResponse(**res)

    async def reports(self, job_id: uuid.UUID) -> List[ReportResponse]:
        res = await self.execute_json(["--format", "json", "reports", "list", "--job-id", str(job_id)])
        return [ReportResponse(**r) for r in res]

    async def download(self, report_id: uuid.UUID, format_type: str) -> bytes:
        args = ["--format", "raw", "reports", "download", str(report_id), "--type", format_type]
        cmd = [str(self.executable_path)] + args
        loop = asyncio.get_running_loop()
        process = await loop.run_in_executor(
            None,
            lambda: subprocess.run(cmd, capture_output=True, timeout=self.timeout, check=True)
        )
        return process.stdout

    async def wallet(self) -> WalletResponse:
        res = await self.execute_json(["--format", "json", "wallet", "info"])
        return WalletResponse(**res)

    async def status(self) -> SystemStatus:
        res = await self.execute_json(["--format", "json", "status"])
        return SystemStatus(**res)

    async def configuration(self) -> ConfigurationResponse:
        res = await self.execute_json(["--format", "json", "config", "get"])
        return ConfigurationResponse(**res)

    async def shutdown(self) -> None:
        pass


# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# HTTP ADAPTER
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

class HTTPAdapter(BackendAdapter):
    """
    Communicates with a remote Rust HTTP server.[span_17](start_span)[span_17](end_span)
    """
    def __init__(self, base_url: str, timeout: int):
        self.client = httpx.AsyncClient(base_url=base_url, timeout=timeout)

    @retry(stop=stop_after_attempt(3), wait=wait_exponential(multiplier=1, min=2, max=10), retry=retry_if_exception_type((httpx.RequestError, httpx.TimeoutException)))
    async def _request(self, method: str, endpoint: str, **kwargs) -> Any:
        logger.debug(f"HTTP {method} {endpoint}")
        response = await self.client.request(method, endpoint, **kwargs)
        response.raise_for_status()
        if response.headers.get("Content-Type") == "application/json":
            return response.json()
        return response.content

    async def health(self) -> HealthResponse:
        res = await self._request("GET", "/health")
        return HealthResponse(**res)

    async def version(self) -> VersionResponse:
        res = await self._request("GET", "/version")
        return VersionResponse(**res)

    async def services(self) -> List[ServiceResponse]:
        res = await self._request("GET", "/services")
        return [ServiceResponse(**s) for s in res]

    async def create_job(self, service_type: ServiceType, inputs: Dict[str, Any]) -> JobResponse:
        payload = {"service_type": service_type.value, "inputs": inputs}
        res = await self._request("POST", "/jobs", json=payload)
        return JobResponse(**res)

    async def create_invoice(self, job_id: uuid.UUID) -> InvoiceResponse:
        res = await self._request("POST", f"/jobs/{job_id}/invoice")
        return InvoiceResponse(**res)

    async def wait_payment(self, invoice_id: uuid.UUID) -> PaymentResponse:
        res = await self._request("GET", f"/invoices/{invoice_id}/wait_payment", timeout=120)
        return PaymentResponse(**res)

    async def job(self, job_id: uuid.UUID) -> JobResponse:
        res = await self._request("GET", f"/jobs/{job_id}")
        return JobResponse(**res)

    async def jobs(self) -> List[JobResponse]:
        res = await self._request("GET", "/jobs")
        return [JobResponse(**j) for j in res]

    async def invoice(self, invoice_id: uuid.UUID) -> InvoiceResponse:
        res = await self._request("GET", f"/invoices/{invoice_id}")
        return InvoiceResponse(**res)

    async def reports(self, job_id: uuid.UUID) -> List[ReportResponse]:
        res = await self._request("GET", f"/jobs/{job_id}/reports")
        return [ReportResponse(**r) for r in res]

    async def download(self, report_id: uuid.UUID, format_type: str) -> bytes:
        return await self._request("GET", f"/reports/{report_id}/download?format={format_type}")

    async def wallet(self) -> WalletResponse:
        res = await self._request("GET", "/wallet")
        return WalletResponse(**res)

    async def status(self) -> SystemStatus:
        res = await self._request("GET", "/status")
        return SystemStatus(**res)

    async def configuration(self) -> ConfigurationResponse:
        res = await self._request("GET", "/configuration")
        return ConfigurationResponse(**res)

    async def shutdown(self) -> None:
        await self.client.aclose()


# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# MANAGERS
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

class DownloadManager:
    """
    Handles report downloads securely.[span_18](start_span)[span_18](end_span)
    """
    def __init__(self, download_dir: Path):
        self.download_dir = download_dir

    def verify(self, file_path: Path) -> bool:
        """Validates the downloaded file exists and is not empty.[span_19](start_span)[span_19](end_span)"""
        return file_path.exists() and file_path.stat().st_size > 0

    async def download_markdown(self, adapter: BackendAdapter, report_id: uuid.UUID) -> Path:
        content = await adapter.download(report_id, "markdown")
        path = self.download_dir / f"{report_id}.md"
        with open(path, "wb") as f:
            f.write(content)
        if not self.verify(path):
            raise IOError("Markdown download verification failed.")
        return path

    async def download_pdf(self, adapter: BackendAdapter, report_id: uuid.UUID) -> Path:
        content = await adapter.download(report_id, "pdf")
        path = self.download_dir / f"{report_id}.pdf"
        with open(path, "wb") as f:
            f.write(content)
        if not self.verify(path):
            raise IOError("PDF download verification failed.")
        return path


class WalletManager:
    """
    Validates and fetches wallet information.[span_20](start_span)[span_20](end_span)
    """
    def __init__(self, adapter: BackendAdapter):
        self.adapter = adapter

    async def wallet(self) -> WalletResponse:
        return await self.adapter.wallet()

    def validate(self, address: str) -> bool:
        """Basic validation for Solana wallet length.[span_21](start_span)[span_21](end_span)"""
        return len(address) >= 32 and len(address) <= 44


class HealthMonitor:
    """
    Periodically checks backend health.[span_22](start_span)[span_22](end_span)
    """
    def __init__(self, adapter: BackendAdapter, interval: int = 30):
        self.adapter = adapter
        self.interval = interval
        self._running = False
        self._task: Optional[asyncio.Task] = None

    async def _loop(self):
        while self._running:
            await self.check()
            await asyncio.sleep(self.interval)

    async def check(self) -> None:
        try:
            status = await self.adapter.health()
            logger.debug(f"Health check OK: {status.status}")
        except Exception as e:
            logger.error(f"Health check failed: {e}")

    def start(self) -> None:
        if not self._running:
            self._running = True
            self._task = asyncio.create_task(self._loop())

    def stop(self) -> None:
        self._running = False
        if self._task:
            self._task.cancel()


# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# BACKEND CLIENT
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

class BackendClient:
    """
    Main communication client interfacing between Streamlit and Rust.[span_23](start_span)[span_23](end_span)
    Handles auto-detection, adapters, caching, and retries.[span_24](start_span)[span_24](end_span)
    """
    def __init__(self):
        self.config = ConfigurationLoader()
        self.cache = CacheManager()
        self.adapter: BackendAdapter = self._detect_adapter()
        self.downloads = DownloadManager(self.config.download_directory)
        self.wallets = WalletManager(self.adapter)
        self.monitor = HealthMonitor(self.adapter)
        logger.info(f"Initialized BackendClient with mode: {type(self.adapter).__name__}")

    def _detect_adapter(self) -> BackendAdapter:
        mode = self.config.backend_mode
        if mode == BackendMode.AUTO:
            if self.config.executable_path.exists():
                return RustCLIAdapter(self.config.executable_path, self.config.timeout)
            return HTTPAdapter(self.config.http_url, self.config.timeout)
        elif mode == BackendMode.CLI:
            return RustCLIAdapter(self.config.executable_path, self.config.timeout)
        elif mode == BackendMode.HTTP:
            return HTTPAdapter(self.config.http_url, self.config.timeout)
        raise ValueError(f"Unsupported backend mode: {mode}")

    async def connect(self) -> None:
        self.monitor.start()

    async def disconnect(self) -> None:
        self.monitor.stop()
        await self.adapter.shutdown()

    async def health(self) -> HealthResponse:
        return await self.adapter.health()

    async def status(self) -> SystemStatus:
        return await self.adapter.status()

    async def wallet(self) -> WalletResponse:
        return await self.wallets.wallet()

    async def services(self) -> List[ServiceResponse]:
        return await self.adapter.services()

    async def create_job(self, service_type: ServiceType, inputs: Dict[str, Any]) -> JobResponse:
        return await self.adapter.create_job(service_type, inputs)

    async def create_invoice(self, job_id: uuid.UUID) -> InvoiceResponse:
        return await self.adapter.create_invoice(job_id)

    async def wait_payment(self, invoice_id: uuid.UUID) -> PaymentResponse:
        return await self.adapter.wait_payment(invoice_id)

    async def job(self, job_id: uuid.UUID) -> JobResponse:
        return await self.adapter.job(job_id)

    async def jobs(self) -> List[JobResponse]:
        return await self.adapter.jobs()

    async def invoice(self, invoice_id: uuid.UUID) -> InvoiceResponse:
        return await self.adapter.invoice(invoice_id)

    async def reports(self, job_id: uuid.UUID) -> List[ReportResponse]:
        return await self.adapter.reports(job_id)

    async def download(self, report_id: uuid.UUID, format_type: str) -> bytes:
        return await self.adapter.download(report_id, format_type)

    async def configuration(self) -> ConfigurationResponse:
        return await self.adapter.configuration()

    async def version(self) -> VersionResponse:
        return await self.adapter.version()
