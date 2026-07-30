"""
ClawHire - Application Entry Point (app.py)
Version: 1.0.0

ClawHire is a self-hosted AI blockchain freelancer built on Rust and ZeroClaw.[span_0](start_span)[span_0](end_span)
The backend is written in Rust.[span_1](start_span)[span_1](end_span)
The frontend is written in Streamlit.[span_2](start_span)[span_2](end_span)
This application acts as a modern Linux terminal running inside a browser.[span_3](start_span)[span_3](end_span)
"""

import asyncio
import os
import sys
from dataclasses import dataclass, field
from datetime import datetime
from enum import Enum
from pathlib import Path
from typing import Any, Dict, List, Optional
import uuid

import streamlit as st
from loguru import logger

# Import internal modules[span_4](start_span)[span_4](end_span)
import styles
import components
import terminal
import api

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# LOGGING CONFIGURATION
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

logger.remove()
logger.add(sys.stderr, level="INFO")
logger.add("logs/clawhire_ui.log", rotation="10 MB", level="DEBUG")

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# SESSION STATE MODELS
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

class AppState(str, Enum):
    BOOTING = "BOOTING"
    READY = "READY"
    SERVICE_SELECTION = "SERVICE_SELECTION"
    INPUT_REQUIRED = "INPUT_REQUIRED"
    AWAITING_PAYMENT = "AWAITING_PAYMENT"
    EXECUTING = "EXECUTING"
    COMPLETED = "COMPLETED"
    ERROR = "ERROR"

@dataclass
class SessionState:
    """Manages robust Streamlit session state.[span_5](start_span)[span_5](end_span)"""
    state: AppState = AppState.BOOTING
    current_job: Optional[api.JobResponse] = None
    selected_service: Optional[api.ServiceType] = None
    wallet: Optional[api.WalletResponse] = None
    invoice: Optional[api.InvoiceResponse] = None
    payment_status: Optional[api.PaymentResponse] = None
    execution_status: str = "Idle"
    generated_reports: List[api.ReportResponse] = field(default_factory=list)
    command_history: List[str] = field(default_factory=list)
    terminal_output: List[str] = field(default_factory=list)
    logs: List[str] = field(default_factory=list)
    downloads: Dict[str, bytes] = field(default_factory=dict)
    notifications: List[Dict[str, Any]] = field(default_factory=list)
    health_info: Optional[api.HealthResponse] = None
    backend_status: Optional[api.SystemStatus] = None
    theme: str = "terminal"
    initialized: bool = False

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# APPLICATION MANAGERS & CONTROLLERS
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

class ThemeManager:
    """Loads and injects CSS for the terminal theme.[span_6](start_span)[span_6](end_span)"""
    def __init__(self):
        self.theme = styles.Theme()

    def load(self) -> str:
        """Loads terminal CSS.[span_7](start_span)[span_7](end_span)"""
        return styles.get_css()

    def apply(self) -> None:
        """Injects CSS into Streamlit.[span_8](start_span)[span_8](end_span)"""
        css = self.load()
        st.markdown(css, unsafe_allow_html=True)


class NotificationCenter:
    """Manages application notifications.[span_9](start_span)[span_9](end_span)"""
    def push(self, message: str, level: str = "info") -> None:
        """Pushes a new notification to the queue.[span_10](start_span)[span_10](end_span)"""
        st.session_state.app_state.notifications.append({
            "message": message,
            "level": level,
            "timestamp": datetime.utcnow()
        })
        logger.info(f"Notification [{level}]: {message}")

    def remove(self, index: int) -> None:
        """Removes a notification by index.[span_11](start_span)[span_11](end_span)"""
        if 0 <= index < len(st.session_state.app_state.notifications):
            st.session_state.app_state.notifications.pop(index)

    def clear(self) -> None:
        """Clears all notifications.[span_12](start_span)[span_12](end_span)"""
        st.session_state.app_state.notifications.clear()

    def render(self) -> None:
        """Renders active notifications via toast or inline components.[span_13](start_span)[span_13](end_span)"""
        for notif in st.session_state.app_state.notifications:
            icon = "ℹ️"
            if notif["level"] == "success": icon = "✅"
            elif notif["level"] == "error": icon = "❌"
            elif notif["level"] == "warning": icon = "⚠️"
            st.toast(notif["message"], icon=icon)
        self.clear()


class HealthDashboard:
    """Displays system health in the sidebar.[span_14](start_span)[span_14](end_span)"""
    def __init__(self, client: api.BackendClient):
        self.client = client

    def render(self) -> None:
        """Renders health information.[span_15](start_span)[span_15](end_span)"""
        st.sidebar.markdown("### 🏥 System Health")
        state = st.session_state.app_state
        if state.health_info:
            st.sidebar.success(f"Backend: {state.health_info.status}")
        else:
            st.sidebar.error("Backend: Disconnected")
            
        if state.backend_status:
            st.sidebar.info(f"Mode: {state.backend_status.backend_mode.value}")
            st.sidebar.info(f"Active Jobs: {state.backend_status.active_jobs}")


class SidebarController:
    """Developer mode sidebar for monitoring.[span_16](start_span)[span_16](end_span)"""
    def __init__(self, context: 'ApplicationContext'):
        self.context = context

    def render(self) -> None:
        """Renders sidebar components.[span_17](start_span)[span_17](end_span)"""
        with st.sidebar:
            st.markdown("## Developer Mode")
            self.context.health_dashboard.render()
            
            st.markdown("### Settings")
            if st.button("Clear Cache"):
                st.cache_data.clear()
                self.context.notifications.push("Cache cleared", "success")
                st.rerun()
            if st.button("Reset Session"):
                st.session_state.app_state = SessionState()
                st.rerun()


class BackgroundTasks:
    """Handles polling and async synchronization for Streamlit.[span_18](start_span)[span_18](end_span)"""
    def __init__(self, context: 'ApplicationContext'):
        self.context = context

    def start(self) -> None:
        """Starts background tasks (conceptual in Streamlit).[span_19](start_span)[span_19](end_span)"""
        pass

    def stop(self) -> None:
        """Stops background tasks.[span_20](start_span)[span_20](end_span)"""
        pass

    async def run(self) -> None:
        """Executes necessary async polls for the current tick.[span_21](start_span)[span_21](end_span)"""
        state = st.session_state.app_state
        if state.state == AppState.BOOTING:
            return

        try:
            # Poll Health
            state.health_info = await self.context.client.health()
            state.backend_status = await self.context.client.status()

            # Poll Payment
            if state.state == AppState.AWAITING_PAYMENT and state.invoice:
                payment = await self.context.client.wait_payment(state.invoice.invoice_id)
                if payment.status.lower() in ["confirmed", "detected", "success"]:
                    state.payment_status = payment
                    state.state = AppState.EXECUTING
                    self.context.notifications.push("Payment confirmed. Executing job...", "success")
                    st.rerun()
                elif payment.status.lower() == "expired":
                    state.state = AppState.ERROR
                    self.context.notifications.push("Invoice expired.", "error")
                    st.rerun()

            # Poll Job Execution
            if state.state == AppState.EXECUTING and state.current_job:
                job = await self.context.client.job(state.current_job.job_id)
                state.current_job = job
                if job.status == api.JobStatus.Completed:
                    state.state = AppState.COMPLETED
                    # Fetch reports
                    reports = await self.context.client.reports(job.job_id)
                    state.generated_reports = reports
                    self.context.notifications.push("Report generated successfully.", "success")
                    st.rerun()
                elif job.status == api.JobStatus.Failed:
                    state.state = AppState.ERROR
                    self.context.notifications.push("Job execution failed.", "error")
                    st.rerun()

        except Exception as e:
            logger.error(f"Background task error: {e}")


class EventLoop:
    """Coordinates events between UI and Backend for the current render cycle.[span_22](start_span)[span_22](end_span)"""
    def __init__(self, context: 'ApplicationContext'):
        self.context = context

    def start(self) -> None:
        """Starts event coordination.[span_23](start_span)[span_23](end_span)"""
        pass

    def tick(self) -> None:
        """Ticks the event loop synchronously for Streamlit.[span_24](start_span)[span_24](end_span)"""
        asyncio.run(self.context.background_tasks.run())

    def stop(self) -> None:
        """Stops event coordination.[span_25](start_span)[span_25](end_span)"""
        pass


class Router:
    """Dispatches application events and commands.[span_26](start_span)[span_26](end_span)"""
    def __init__(self, context: 'ApplicationContext'):
        self.context = context

    def route(self) -> None:
        """Routes execution based on current app state.[span_27](start_span)[span_27](end_span)"""
        state = st.session_state.app_state
        if state.state == AppState.BOOTING:
            self._handle_boot()
        elif state.state == AppState.READY:
            self._handle_ready()
        elif state.state == AppState.SERVICE_SELECTION:
            self._handle_service_selection()
        elif state.state == AppState.INPUT_REQUIRED:
            self._handle_input()
        elif state.state == AppState.AWAITING_PAYMENT:
            self._handle_payment()
        elif state.state == AppState.EXECUTING:
            self._handle_execution()
        elif state.state == AppState.COMPLETED:
            self._handle_completed()
        elif state.state == AppState.ERROR:
            self._handle_error()

    def dispatch(self, command: str) -> None:
        """Dispatches a terminal command.[span_28](start_span)[span_28](end_span)"""
        logger.info(f"Command dispatched: {command}")
        # Command handling logic would integrate with terminal.py

    def _handle_boot(self) -> None:
        async def boot_sequence():
            try:
                wallet = await self.context.client.wallet()
                st.session_state.app_state.wallet = wallet
                st.session_state.app_state.state = AppState.READY
                self.context.notifications.push("System Booted Successfully", "success")
            except Exception as e:
                logger.error(f"Boot error: {e}")
                self.context.notifications.push("Failed to connect to backend.", "error")
                st.session_state.app_state.state = AppState.ERROR
        asyncio.run(boot_sequence())
        if st.session_state.app_state.state == AppState.READY:
            st.rerun()

    def _handle_ready(self) -> None:
        components.TerminalBanner("1.0.0", "Production", "Mainnet", st.session_state.app_state.wallet.address if st.session_state.app_state.wallet else "Unknown").render()
        components.OutputBlock().render()
        if st.button("Start New Job", type="primary"):
            st.session_state.app_state.state = AppState.SERVICE_SELECTION
            st.rerun()

    def _handle_service_selection(self) -> None:
        st.markdown("### 📋 Available Services")
        components.ServiceCard("Smart Contract Security Review", "Professional security review for Solana programs.", 2.5, "SOL", "5-10 minutes", ["GitHub URL", "ZIP Archive", "Rust Source"]).render()
        components.ServiceCard("On-chain Intelligence Report", "Investigate a Solana wallet or transaction.", 1.0, "SOL", "2-5 minutes", ["Wallet Address", "Transaction Signature"]).render()
        
        service_map = {
            "Smart Contract Security Review": api.ServiceType.SmartContractReview,
            "On-chain Intelligence Report": api.ServiceType.OnChainIntelligence
        }
        
        selection = components.ServiceSelector().render()
        if st.button("Proceed"):
            st.session_state.app_state.selected_service = service_map.get(selection)
            st.session_state.app_state.state = AppState.INPUT_REQUIRED
            st.rerun()

    def _handle_input(self) -> None:
        st.markdown(f"### Provide Input for {st.session_state.app_state.selected_service.value}")
        input_type = "GitHub URL" if st.session_state.app_state.selected_service == api.ServiceType.SmartContractReview else "Wallet Address"
        target_input = components.FileUploaderCard(input_type, "primary_input").render()
        
        if st.button("Submit & Generate Invoice"):
            if target_input:
                async def create_job():
                    inputs = {"target": target_input}
                    job = await self.context.client.create_job(st.session_state.app_state.selected_service, inputs)
                    invoice = await self.context.client.create_invoice(job.job_id)
                    st.session_state.app_state.current_job = job
                    st.session_state.app_state.invoice = invoice
                    st.session_state.app_state.state = AppState.AWAITING_PAYMENT
                asyncio.run(create_job())
                st.rerun()
            else:
                self.context.notifications.push("Input required.", "warning")

    def _handle_payment(self) -> None:
        st.markdown("### 💳 Awaiting Payment")
        invoice = st.session_state.app_state.invoice
        if invoice:
            components.InvoiceCard(str(invoice.invoice_id), invoice.amount, invoice.wallet_address, invoice.status.value, 3600).render()
            components.PaymentStatusCard("Awaiting Payment").render()
            components.LoadingSpinner("Listening for transaction on Solana...").render()

    def _handle_execution(self) -> None:
        st.markdown("### ⚙️ Executing Job")
        components.JobStatusCard(str(st.session_state.app_state.current_job.job_id), st.session_state.app_state.current_job.status.value).render()
        components.ProgressCard("Analyzing Data").update(50, "00:02:15", "00:01:00")
        components.TypingIndicator("Generating professional report...").render()

    def _handle_completed(self) -> None:
        components.SuccessPanel("Job completed successfully.").render()
        reports = st.session_state.app_state.generated_reports
        if reports:
            for report in reports:
                # Fetch bytes mock or use context to download
                async def fetch_bytes():
                    return await self.context.client.download(report.report_id, "pdf"), await self.context.client.download(report.report_id, "markdown")
                try:
                    pdf_bytes, md_bytes = asyncio.run(fetch_bytes())
                    components.ReportCard(report.title, str(report.job_id), report.created_at.isoformat(), md_bytes.decode('utf-8', errors='ignore'), pdf_bytes).render()
                except Exception as e:
                    logger.error(f"Failed to fetch report bytes: {e}")
                    components.ReportCard(report.title, str(report.job_id), report.created_at.isoformat(), "# Report Data Unavailable", b"").render()

        if st.button("Start New Session"):
            st.session_state.app_state = SessionState()
            st.rerun()

    def _handle_error(self) -> None:
        components.ErrorPanel("System Error", "An unexpected error occurred during execution.", "ERR_SYS_01", "Please reset the session and try again.").render()
        if st.button("Reset Session"):
            st.session_state.app_state = SessionState()
            st.rerun()

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# APPLICATION CONTEXT
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

class ApplicationContext:
    """Holds instances of managers and dependencies.[span_29](start_span)[span_29](end_span)"""
    def __init__(self):
        self.client = api.BackendClient()
        self.theme_manager = ThemeManager()
        self.notifications = NotificationCenter()
        self.health_dashboard = HealthDashboard(self.client)
        self.sidebar = SidebarController(self)
        self.background_tasks = BackgroundTasks(self)
        self.event_loop = EventLoop(self)
        self.router = Router(self)
        # Terminal session mock, would interface directly with terminal.py
        self.terminal_session = None 

    def load(self) -> None:
        """Loads configuration and state.[span_30](start_span)[span_30](end_span)"""
        if "app_state" not in st.session_state:
            st.session_state.app_state = SessionState()
        self.theme_manager.apply()

    def save(self) -> None:
        """Persists state if needed.[span_31](start_span)[span_31](end_span)"""
        pass

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# MAIN APPLICATION
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

class Application:
    """Main application orchestrator.[span_32](start_span)[span_32](end_span)"""
    def __init__(self):
        self.context = ApplicationContext()

    def initialize(self) -> None:
        """Configures Streamlit page and initializes context.[span_33](start_span)[span_33](end_span)"""
        st.set_page_config(
            page_title="ClawHire",
            page_icon="🦞",
            layout="wide",
            initial_sidebar_state="collapsed"
        )
        self.context.load()

    def run(self) -> None:
        """Main execution loop.[span_34](start_span)[span_34](end_span)"""
        self.initialize()
        self.context.sidebar.render()
        self.context.event_loop.tick()
        self.render()
        self.context.notifications.render()

    def render(self) -> None:
        """Renders the primary UI.[span_35](start_span)[span_35](end_span)"""
        def window_content():
            self.context.router.route()
            
        term = components.TerminalWindow("clawhire@terminal: ~")
        term.render(window_content)
        
        # If input is required at a base prompt level
        if st.session_state.app_state.state in [AppState.READY, AppState.COMPLETED, AppState.ERROR]:
            prompt = components.PromptLine("~")
            prompt.render()

    def shutdown(self) -> None:
        """Cleans up resources.[span_36](start_span)[span_36](end_span)"""
        asyncio.run(self.context.client.disconnect())

    def refresh(self) -> None:
        """Forces UI refresh.[span_37](start_span)[span_37](end_span)"""
        st.rerun()


# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# ENTRY POINT
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

if __name__ == "__main__":
    app = Application()
    try:
        app.run()
    except Exception as ex:
        logger.exception("Application crashed")
        st.error(f"Critical System Failure: {str(ex)}")
