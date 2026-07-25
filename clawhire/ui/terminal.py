import asyncio
import html
import logging
import time
import uuid
from datetime import datetime
from enum import Enum, auto
from typing import List, Dict, Optional, Any, Callable
from dataclasses import dataclass, field

import streamlit as st

# Setup standard python logging for backend observability
logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s [%(levelname)s] %(name)s: %(message)s"
)
logger = logging.getLogger("ClawHire.Terminal")

# ============================================================================
# Enums & Data Models
# ============================================================================

class JobStatus(Enum):
    PENDING = auto()
    AWAITING_PAYMENT = auto()
    EXECUTING = auto()
    COMPLETED = auto()
    FAILED = auto()

class ServiceType(Enum):
    SMART_CONTRACT_REVIEW = "SmartContractReview"
    ON_CHAIN_INTEL = "OnChainIntelligence"

@dataclass
class TerminalEvent:
    id: str
    type: str
    timestamp: datetime
    message: str
    metadata: Dict[str, Any]

@dataclass
class TerminalEvents:
    """Thread-safe-ish event tracker for terminal actions."""
    events: List[TerminalEvent] = field(default_factory=list)

    def emit(self, event_type: str, message: str, metadata: Optional[Dict[str, Any]] = None) -> None:
        event = TerminalEvent(
            id=str(uuid.uuid4()),
            type=event_type,
            timestamp=datetime.now(),
            message=message,
            metadata=metadata or {}
        )
        self.events.append(event)
        logger.info(f"Event Emitted: {event_type} - {message}")

@dataclass
class TerminalHistory:
    """Manages command and output history."""
    commands: List[str] = field(default_factory=list)
    outputs: List[Dict[str, Any]] = field(default_factory=list)

    def append(self, command: str, output: str, is_html: bool = False) -> None:
        if command:
            self.commands.append(command)
        self.outputs.append({
            "command": command,
            "output": output,
            "is_html": is_html,
            "timestamp": datetime.now()
        })

    def clear(self) -> None:
        self.commands.clear()
        self.outputs.clear()

    def render(self) -> None:
        for entry in self.outputs:
            if entry["command"]:
                st.markdown(f"**user@clawhire:~$** {html.escape(entry['command'])}", unsafe_allow_html=True)
            if entry["is_html"]:
                st.markdown(entry["output"], unsafe_allow_html=True)
            else:
                st.markdown(f"```text\n{entry['output']}\n```")

@dataclass
class TerminalSession:
    """Core state model for the terminal session."""
    is_active: bool = False
    current_job_id: Optional[str] = None
    current_job_status: Optional[JobStatus] = None
    selected_service: Optional[ServiceType] = None
    invoice_ref: Optional[str] = None
    report_data: Optional[Dict[str, str]] = None
    wallet_address: str = "Unknown"
    
    history: TerminalHistory = field(default_factory=TerminalHistory)
    events: TerminalEvents = field(default_factory=TerminalEvents)

    def start(self) -> None:
        if not self.is_active:
            self.is_active = True
            self.events.emit("Startup", "Terminal session initialized.")

    def reset_job_state(self) -> None:
        self.current_job_id = None
        self.current_job_status = None
        self.selected_service = None
        self.invoice_ref = None
        self.report_data = None
        self.events.emit("StateReset", "Job state cleared.")

# ============================================================================
# State Management
# ============================================================================

class SessionManager:
    """Encapsulates Streamlit session state access."""
    SESSION_KEY = "clawhire_terminal_session"

    @classmethod
    def get(cls) -> TerminalSession:
        if cls.SESSION_KEY not in st.session_state:
            st.session_state[cls.SESSION_KEY] = TerminalSession()
        return st.session_state[cls.SESSION_KEY]

    @classmethod
    def save(cls, session: TerminalSession) -> None:
        st.session_state[cls.SESSION_KEY] = session

# ============================================================================
# UI Components & Rendering
# ============================================================================

class UILogger:
    """Formats logs specifically for the frontend terminal display."""
    @staticmethod
    def _format(level: str, color: str, message: str) -> str:
        timestamp = datetime.now().strftime("%H:%M:%S")
        safe_msg = html.escape(message)
        return f"<span style='color: #888;'>[{timestamp}]</span> <span style='color: {color}; font-weight: bold;'>[{level}]</span> {safe_msg}<br>"

    @classmethod
    def info(cls, msg: str) -> str: return cls._format("INFO", "#4A90E2", msg)
    
    @classmethod
    def success(cls, msg: str) -> str: return cls._format("SUCCESS", "#2ECC71", msg)
    
    @classmethod
    def warning(cls, msg: str) -> str: return cls._format("WARNING", "#F1C40F", msg)
    
    @classmethod
    def error(cls, msg: str) -> str: return cls._format("ERROR", "#E74C3C", msg)

class TerminalUI:
    """Handles the rendering of the terminal interface."""
    @staticmethod
    def render_boot_sequence(container: st.delta_generator) -> None:
        logo = """
   ____ _               _   _ _          
  / ___| | __ ___      | | | (_)_ __ ___ 
 | |   | |/ _` \\ \\ /\\ / /| |_| | | '__/ _ \\
 | |___| | (_| |\\ V  V / |  _  | | | |  __/
  \\____|_|\\__,_| \\_/\\_/  |_| |_|_|_|  \\___|
        """
        output = f"<pre style='color: #2ECC71;'>{html.escape(logo)}</pre>"
        output += UILogger.info("Initializing ZeroClaw Rust runtime...")
        output += UILogger.info("Connecting to Solana RPC...")
        output += UILogger.success("System loaded. Services available.")
        container.markdown(output, unsafe_allow_html=True)

    @staticmethod
    def render_status_bar(session: TerminalSession) -> None:
        status_text = session.current_job_status.name if session.current_job_status else "IDLE"
        job_text = session.current_job_id if session.current_job_id else "None"
        html_content = f"""
        <div style='background-color: #2C3E50; color: #ECF0F1; padding: 5px 10px; font-family: monospace; font-size: 12px; display: flex; justify-content: space-between; border-top: 1px solid #34495E;'>
            <span><b>Network:</b> Solana Mainnet-Beta</span>
            <span><b>Wallet:</b> {session.wallet_address}</span>
            <span><b>Job:</b> {job_text} <b>Status:</b> {status_text}</span>
        </div>
        """
        st.markdown(html_content, unsafe_allow_html=True)

    @staticmethod
    def get_styles() -> str:
        return """
        <style>
        .stApp { background-color: #0E1117; }
        .blinking-cursor {
            font-weight: bold;
            color: #2ECC71;
            animation: 1s blink step-end infinite;
        }
        @keyframes blink {
            from, to { opacity: 1; }
            50% { opacity: 0; }
        }
        </style>
        """

# ============================================================================
# Command Execution
# ============================================================================

class CommandExecutor:
    """Executes validated terminal commands."""
    def __init__(self, session: TerminalSession):
        self.session = session

    async def execute(self, raw_input: str) -> str:
        if not raw_input or not raw_input.strip():
            return ""

        parts = raw_input.strip().split()
        cmd, args = parts[0].lower(), parts[1:]

        try:
            # Command Router
            if cmd in ["clear", "cls"]:
                self.session.history.clear()
                return ""
            elif cmd == "help":
                return self._cmd_help()
            elif cmd == "services":
                return self._cmd_services()
            elif cmd == "service":
                return self._cmd_service(args)
            elif cmd == "invoice":
                return await self._cmd_invoice()
            elif cmd == "pay":
                return await self._cmd_pay(args)
            elif cmd == "status":
                return self._cmd_status()
            elif cmd == "reset":
                self.session.reset_job_state()
                return UILogger.success("Session state reset.")
            else:
                return UILogger.error(f"Command not found: {cmd}. Type 'help' for available commands.")
                
        except Exception as e:
            logger.error(f"Command execution failed: {str(e)}", exc_info=True)
            return UILogger.error("An internal error occurred during execution.")

    def _cmd_help(self) -> str:
        return f"""<div style='font-family: monospace;'>
        <b>Available Commands:</b><br>
        <span style='color:#3498DB'>help</span>     - Show this help message<br>
        <span style='color:#3498DB'>clear</span>    - Clear terminal screen<br>
        <span style='color:#3498DB'>services</span> - List available AI blockchain services<br>
        <span style='color:#3498DB'>service</span>  - Select a service (e.g., 'service 1')<br>
        <span style='color:#3498DB'>invoice</span>  - Generate invoice for selected service<br>
        <span style='color:#3498DB'>pay</span>      - Verify payment (e.g., 'pay [tx_signature]')<br>
        <span style='color:#3498DB'>status</span>   - Show current job status<br>
        <span style='color:#3498DB'>reset</span>    - Clear current job state<br>
        </div>"""

    def _cmd_services(self) -> str:
        return f"""<div style='font-family: monospace;'>
        <b>Available Services:</b><br>
        [1] {ServiceType.SMART_CONTRACT_REVIEW.value} (0.20 SOL)<br>
        [2] {ServiceType.ON_CHAIN_INTEL.value} (0.10 SOL)<br>
        <br>Type 'service [number]' to select.
        </div>"""

    def _cmd_service(self, args: List[str]) -> str:
        if not args:
            return UILogger.error("Usage: service [number]")
        
        mapping = {"1": ServiceType.SMART_CONTRACT_REVIEW, "2": ServiceType.ON_CHAIN_INTEL}
        selection = mapping.get(args[0])
        
        if not selection:
            return UILogger.error("Invalid selection.")

        self.session.selected_service = selection
        self.session.current_job_id = str(uuid.uuid4())[:8]
        self.session.current_job_status = JobStatus.PENDING
        return UILogger.success(f"Selected: {selection.value}. Type 'invoice' to proceed.")

    async def _cmd_invoice(self) -> str:
        if not self.session.selected_service:
            return UILogger.warning("Select a service first.")
            
        if self.session.current_job_status != JobStatus.PENDING:
            return UILogger.warning("Invoice already generated or job in progress.")

        # Simulate API network call to pricing/invoice backend
        await asyncio.sleep(0.5) 
        
        self.session.invoice_ref = f"INV-{int(time.time())}"
        self.session.current_job_status = JobStatus.AWAITING_PAYMENT
        amount = "0.20" if self.session.selected_service == ServiceType.SMART_CONTRACT_REVIEW else "0.10"
        
        return f"""<div style='border: 1px dashed #F1C40F; padding: 10px; margin-top: 10px; font-family: monospace;'>
            <h4 style='color: #F1C40F; margin:0;'>INVOICE GENERATED</h4>
            <b>Reference:</b> {self.session.invoice_ref}<br>
            <b>Amount Due:</b> {amount} SOL<br>
            <br><i>Type 'pay [tx_signature]' to verify payment.</i>
        </div>"""

    async def _cmd_pay(self, args: List[str]) -> str:
        if self.session.current_job_status != JobStatus.AWAITING_PAYMENT:
            return UILogger.error("No pending invoice.")
        if not args:
            return UILogger.error("Usage: pay [tx_signature]")

        tx_sig = args[0]
        out = UILogger.info(f"Verifying transaction: {tx_sig}...")
        
        # In production, replace this sleep with actual Solana RPC call via a backend API
        await asyncio.sleep(1.5)
        
        out += UILogger.success("Payment Confirmed! Initializing AI Execution...")
        self.session.current_job_status = JobStatus.EXECUTING
        
        # Simulate long-running agent execution
        await asyncio.sleep(2)
        self.session.current_job_status = JobStatus.COMPLETED
        self.session.report_data = {"url": f"https://api.clawhire.com/reports/{self.session.current_job_id}.pdf"}
        
        out += UILogger.success("Execution finished.")
        out += f"<br><a href='{self.session.report_data['url']}' target='_blank' style='color:#3498DB'>📄 Download Report</a>"
        return out

    def _cmd_status(self) -> str:
        if not self.session.current_job_id:
            return UILogger.info("System is idle.")
        return UILogger.info(f"Job: {self.session.current_job_id} | Status: {self.session.current_job_status.name}")


# ============================================================================
# Main Application Entry
# ============================================================================

def main():
    st.set_page_config(page_title="ClawHire Terminal", layout="wide")
    st.markdown(TerminalUI.get_styles(), unsafe_allow_html=True)
    
    session = SessionManager.get()
    
    # Handle initial boot up sequence
    if not session.is_active:
        boot_container = st.empty()
        TerminalUI.render_boot_sequence(boot_container)
        time.sleep(1.2) # Initial boot delay for UX
        boot_container.empty()
        session.start()
        SessionManager.save(session)

    # Render top bar + prompt line
    st.markdown("<span class='blinking-cursor'>█</span>", unsafe_allow_html=True)
    st.markdown(
        f"<span style='color:#2ECC71;font-weight:bold;'>user@clawhire</span>:<span style='color:#3498DB;font-weight:bold;'>~</span>$ ", 
        unsafe_allow_html=True
    )
    
    # Render historical output
    history_container = st.container()
    with history_container:
        session.history.render()

    # Input capture
    command_input = st.chat_input("Enter command...")

    # Process command execution loop
    if command_input:
        executor = CommandExecutor(session)
        
        # Execute asynchronously and await completion
        try:
            output = asyncio.run(executor.execute(command_input))
        except Exception as e:
            logger.critical("Event loop failed", exc_info=True)
            output = UILogger.error("Fatal exception in async execution loop.")
        
        # Store in history if not a clear command
        if command_input.lower().strip() not in ["clear", "cls"]:
            session.history.append(command_input, output, is_html=True)
            
        SessionManager.save(session)
        st.rerun()

    # Render fixed bottom status bar
    TerminalUI.render_status_bar(session)

if __name__ == "__main__":
    main()
