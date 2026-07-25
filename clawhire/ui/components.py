"""
ClawHire - UI Components Module
Version: 1.0.0

ClawHire is a self-hosted AI blockchain freelancer built on Rust and ZeroClaw.[span_0](start_span)[span_0](end_span)
The frontend is written in Streamlit but must visually behave like a modern Linux terminal.[span_1](start_span)[span_1](end_span)

This module contains EVERY reusable UI component used throughout the application.[span_2](start_span)[span_2](end_span)
NO application logic, payment logic, API logic, or terminal command parser is included here.[span_3](start_span)[span_3](end_span)
"""

import streamlit as st
import time
from dataclasses import dataclass, field
from typing import List, Dict, Any, Optional, Callable
from pathlib import Path
import styles


class ComponentTheme:
    """
    Loads colors from styles.py and provides helper methods for semantic coloring.[span_4](start_span)[span_4](end_span)
    """
    def __init__(self):
        self.theme = styles.Theme()

    def success(self) -> str:
        """Returns the success color (green).[span_5](start_span)[span_5](end_span)"""
        return self.theme.green

    def error(self) -> str:
        """Returns the error color (red).[span_6](start_span)[span_6](end_span)"""
        return self.theme.red

    def warning(self) -> str:
        """Returns the warning color (yellow/orange).[span_7](start_span)[span_7](end_span)"""
        return self.theme.orange

    def info(self) -> str:
        """Returns the info color (cyan/blue).[span_8](start_span)[span_8](end_span)"""
        return self.theme.cyan

    def primary(self) -> str:
        """Returns the primary text color.[span_9](start_span)[span_9](end_span)"""
        return self.theme.text_primary

    def secondary(self) -> str:
        """Returns the secondary/muted text color.[span_10](start_span)[span_10](end_span)"""
        return self.theme.text_muted


class TerminalWindow:
    """
    Responsibilities: Render terminal frame, title bar, traffic-light buttons, and body.[span_11](start_span)[span_11](end_span)
    Support: Header, Footer, Status, Content, Scrollable output.[span_12](start_span)[span_12](end_span)
    """
    def __init__(self, title: str = "clawhire@terminal: ~"):
        self.title = title

    def header(self) -> None:
        """Renders the top title bar and traffic lights.[span_13](start_span)[span_13](end_span)"""
        header_html = f"""
        <div class="terminal-titlebar">
            <div class="terminal-traffic-lights">
                <div class="traffic-light tl-red"></div>
                <div class="traffic-light tl-yellow"></div>
                <div class="traffic-light tl-green"></div>
            </div>
            <div class="terminal-title">{self.title}</div>
        </div>
        """
        st.markdown(header_html, unsafe_allow_html=True)

    def body(self, content_func: Callable) -> None:
        """Renders the terminal body content.[span_14](start_span)[span_14](end_span)"""
        st.markdown('<div class="terminal-window">', unsafe_allow_html=True)
        self.header()
        content_func()
        self.footer()
        st.markdown('</div>', unsafe_allow_html=True)

    def footer(self) -> None:
        """Renders the terminal footer/status area.[span_15](start_span)[span_15](end_span)"""
        st.markdown('<div class="scanline-overlay"></div>', unsafe_allow_html=True)

    def render(self, content_func: Callable) -> None:
        """Main render method.[span_16](start_span)[span_16](end_span)"""
        self.body(content_func)


class TerminalBanner:
    """
    Displays ASCII logo, Application name, Version, Environment, RPC, and Wallet.[span_17](start_span)[span_17](end_span)
    """
    def __init__(self, version: str, environment: str, rpc: str, wallet: str):
        self.version = version
        self.environment = environment
        self.rpc = rpc
        self.wallet = wallet

    def render(self) -> None:
        """Renders the banner to the screen.[span_18](start_span)[span_18](end_span)"""
        ascii_art = r"""
<pre style="color: #00FF88; background: transparent; border: none; padding: 0;">
   ____  _                  _    _  _             
  / ___|| |  __ _ __      _| |  | |(_) _ __  ___  
 | |    | | / _` |\ \ /\ / / |_ | || || '__|/ _ \ 
 | |___ | || (_| | \ V  V /|  _|| || || |  |  __/ 
  \____||_| \__,_|  \_/\_/ |_|  |_||_||_|   \___| 
</pre>
        """
        banner_info = f"""
        <div style="color: #9A9A9A; font-size: 0.85rem; margin-bottom: 1rem;">
            <div><span style="color: #4FC3F7;">v{self.version}</span> | {self.environment}</div>
            <div>RPC: <span style="color: #00FF88;">{self.rpc}</span></div>
            <div>Wallet: <span style="color: #B388FF;">{self.wallet}</span></div>
        </div>
        """
        st.markdown(ascii_art, unsafe_allow_html=True)
        st.markdown(banner_info, unsafe_allow_html=True)


class PromptLine:
    """
    Displays User prompt, Typing cursor, Current directory.[span_19](start_span)[span_19](end_span)
    """
    def __init__(self, directory: str = "~"):
        self.directory = directory
        self.prompt = ""

    def set_prompt(self, text: str) -> None:
        """Sets the prompt text.[span_20](start_span)[span_20](end_span)"""
        self.prompt = text

    def render(self) -> None:
        """Renders the prompt line.[span_21](start_span)[span_21](end_span)"""
        html = f"""
        <div style="display: flex; font-family: monospace; font-size: 0.9rem; margin: 0.5rem 0;">
            <span style="color: #00FF88;">clawhire</span>
            <span style="color: #F2F2F2;">:</span>
            <span style="color: #4FC3F7;">{self.directory}</span>
            <span style="color: #F2F2F2; margin-left: 0.5rem;">$ {self.prompt}</span>
            <span style="display: inline-block; width: 8px; height: 1.2em; background-color: #00FF88; margin-left: 4px; animation: blink 1s step-end infinite;"></span>
        </div>
        """
        st.markdown(html, unsafe_allow_html=True)


class OutputBlock:
    """
    Render Normal output, Colored output, Markdown, Logs, Streaming text.[span_22](start_span)[span_22](end_span)
    """
    def __init__(self):
        self.content: List[str] = []

    def append(self, text: str, color_key: str = "white") -> None:
        """Appends output with specified styling.[span_23](start_span)[span_23](end_span)"""
        formatted = styles.terminal_style(text, color=color_key)
        self.content.append(formatted)

    def clear(self) -> None:
        """Clears the output block.[span_24](start_span)[span_24](end_span)"""
        self.content.clear()

    def render(self) -> None:
        """Renders the accumulated output block.[span_25](start_span)[span_25](end_span)"""
        if not self.content:
            return
        combined = "<br>".join(self.content)
        st.markdown(f"<div style='margin: 0.5rem 0; font-family: monospace;'>{combined}</div>", unsafe_allow_html=True)


@dataclass
class CommandHistory:
    """
    Maintain command history.[span_26](start_span)[span_26](end_span)
    """
    history: List[str] = field(default_factory=list)
    current_index: int = -1

    def add(self, command: str) -> None:
        """Adds a command to the history.[span_27](start_span)[span_27](end_span)"""
        if not command.strip():
            return
        self.history.append(command)
        self.current_index = len(self.history)

    def clear(self) -> None:
        """Clears the command history.[span_28](start_span)[span_28](end_span)"""
        self.history.clear()
        self.current_index = -1

    def previous(self) -> Optional[str]:
        """Retrieves the previous command.[span_29](start_span)[span_29](end_span)"""
        if self.current_index > 0:
            self.current_index -= 1
            return self.history[self.current_index]
        return None

    def next(self) -> Optional[str]:
        """Retrieves the next command.[span_30](start_span)[span_30](end_span)"""
        if self.current_index < len(self.history) - 1:
            self.current_index += 1
            return self.history[self.current_index]
        self.current_index = len(self.history)
        return ""

    def render(self) -> None:
        """Renders the history if needed.[span_31](start_span)[span_31](end_span)"""
        pass  # History is usually structural, rendered via input fields


class ServiceCard:
    """
    Display Service Name, Description, Price, Currency, Estimated Duration, Supported Inputs.[span_32](start_span)[span_32](end_span)
    """
    def __init__(self, name: str, description: str, price: float, currency: str, duration: str, inputs: List[str]):
        self.name = name
        self.description = description
        self.price = price
        self.currency = currency
        self.duration = duration
        self.inputs = inputs

    def render(self) -> None:
        """Renders the service card.[span_33](start_span)[span_33](end_span)"""
        inputs_html = ", ".join([f"<span class='badge badge-info'>{i}</span>" for i in self.inputs])
        html = f"""
        <div class="terminal-card">
            <h3 style="color: #00FF88; margin-top: 0;">{self.name}</h3>
            <p style="color: #9A9A9A;">{self.description}</p>
            <div style="margin: 1rem 0;">
                <strong>Price:</strong> <span style="color: #FFD54F;">{self.price} {self.currency}</span><br>
                <strong>Duration:</strong> {self.duration}
            </div>
            <div><strong>Inputs:</strong> {inputs_html}</div>
        </div>
        """
        st.markdown(html, unsafe_allow_html=True)


class ServiceSelector:
    """
    Display exactly TWO services. Support Selection, Highlighting, Keyboard navigation.[span_34](start_span)[span_34](end_span)
    1. Smart Contract Security Review
    2. On-chain Intelligence Report
    """
    def __init__(self):
        self.services = [
            "Smart Contract Security Review",
            "On-chain Intelligence Report"
        ]

    def render(self) -> str:
        """Renders the selector and returns the selection.[span_35](start_span)[span_35](end_span)"""
        st.markdown("<div style='margin: 1rem 0;'><strong>Select a Service:</strong></div>", unsafe_allow_html=True)
        selection = st.radio("Services", self.services, label_visibility="collapsed")
        return selection

    def selected(self, current_selection: str) -> str:
        """Returns the selected service.[span_36](start_span)[span_36](end_span)"""
        return current_selection


class WalletCard:
    """
    Display Wallet Address, Copy Button, Network.[span_37](start_span)[span_37](end_span)
    """
    def __init__(self, address: str, network: str):
        self.address = address
        self.network = network

    def render(self) -> None:
        """Renders the wallet card.[span_38](start_span)[span_38](end_span)"""
        html = f"""
        <div class="terminal-card">
            <div style="color: #B388FF; font-weight: bold;">Wallet Information</div>
            <div><strong>Network:</strong> {self.network}</div>
            <div style="margin-top: 0.5rem; word-break: break-all;">
                <code>{self.address}</code>
            </div>
        </div>
        """
        st.markdown(html, unsafe_allow_html=True)
        CopyToClipboardButton(self.address, "Copy Address").render()


class InvoiceCard:
    """
    Display Invoice Number, Amount, Wallet, Status, Countdown.[span_39](start_span)[span_39](end_span)
    """
    def __init__(self, invoice_id: str, amount: float, wallet: str, status: str, countdown: int):
        self.invoice_id = invoice_id
        self.amount = amount
        self.wallet = wallet
        self.status = status
        self.countdown = countdown

    def update_status(self, new_status: str) -> None:
        """Updates the invoice status.[span_40](start_span)[span_40](end_span)"""
        self.status = new_status

    def render(self) -> None:
        """Renders the invoice card.[span_41](start_span)[span_41](end_span)"""
        status_color = styles.status_color(self.status)
        badge = styles.badge_style(self.status, badge_type="warning" if self.status.lower() == "pending" else "info")
        html = f"""
        <div class="invoice-card">
            <div style="font-size: 0.85rem; color: #9A9A9A;">Invoice #{self.invoice_id}</div>
            <div class="invoice-amount">{self.amount} SOL</div>
            <div style="margin-bottom: 1rem;"><code>{self.wallet}</code></div>
            <div>Status: {badge}</div>
            <div style="margin-top: 1rem; color: #FF5252; font-size: 0.85rem;">Expires in {self.countdown}s</div>
        </div>
        """
        st.markdown(html, unsafe_allow_html=True)


class PaymentStatusCard:
    """
    Statuses: Awaiting Payment, Payment Detected, Confirmed, Expired, Failed. Uses colored badges.[span_42](start_span)[span_42](end_span)
    """
    def __init__(self, status: str):
        self.status = status

    def render(self) -> None:
        """Renders the payment status card.[span_43](start_span)[span_43](end_span)"""
        color_map = {
            "Awaiting Payment": "warning",
            "Payment Detected": "info",
            "Confirmed": "success",
            "Expired": "danger",
            "Failed": "danger"
        }
        b_type = color_map.get(self.status, "info")
        badge = styles.badge_style(self.status, badge_type=b_type)
        html = f"""
        <div class="terminal-card" style="display: flex; align-items: center; justify-content: space-between;">
            <strong>Payment Status:</strong>
            {badge}
        </div>
        """
        st.markdown(html, unsafe_allow_html=True)


class JobStatusCard:
    """
    Statuses: Created, Quoted, Awaiting Payment, Executing, Generating Report, Completed, Archived.[span_44](start_span)[span_44](end_span)
    """
    def __init__(self, job_id: str, status: str):
        self.job_id = job_id
        self.status = status

    def render(self) -> None:
        """Renders the job status card.[span_45](start_span)[span_45](end_span)"""
        badge = styles.badge_style(self.status, badge_type="info")
        html = f"""
        <div class="terminal-card">
            <div><strong>Job ID:</strong> <code>{self.job_id}</code></div>
            <div style="margin-top: 0.5rem;"><strong>Status:</strong> {badge}</div>
        </div>
        """
        st.markdown(html, unsafe_allow_html=True)


class ProgressCard:
    """
    Animated progress bar. Percentage, Elapsed Time, ETA.[span_46](start_span)[span_46](end_span)
    """
    def __init__(self, task_name: str):
        self.task_name = task_name
        self.percentage = 0
        self.elapsed = "0s"
        self.eta = "Calculating..."
        self.progress_bar = st.empty()

    def update(self, percentage: int, elapsed: str, eta: str) -> None:
        """Updates progress metrics.[span_47](start_span)[span_47](end_span)"""
        self.percentage = percentage
        self.elapsed = elapsed
        self.eta = eta
        self.render()

    def render(self) -> None:
        """Renders the progress card.[span_48](start_span)[span_48](end_span)"""
        html = f"""
        <div class="terminal-card">
            <div style="display: flex; justify-content: space-between; margin-bottom: 0.5rem;">
                <strong>{self.task_name}</strong>
                <span style="color: #00FF88;">{self.percentage}%</span>
            </div>
            <div style="width: 100%; background-color: #2B2B2B; border-radius: 4px; height: 8px; margin-bottom: 0.5rem;">
                <div style="width: {self.percentage}%; background: linear-gradient(90deg, #4FC3F7, #00FF88); height: 100%; border-radius: 4px; transition: width 0.3s ease;"></div>
            </div>
            <div style="display: flex; justify-content: space-between; font-size: 0.75rem; color: #9A9A9A;">
                <span>Elapsed: {self.elapsed}</span>
                <span>ETA: {self.eta}</span>
            </div>
        </div>
        """
        self.progress_bar.markdown(html, unsafe_allow_html=True)


class ReportCard:
    """
    Display Report Name, Job ID, Created, Download Buttons (Markdown, PDF).[span_49](start_span)[span_49](end_span)
    """
    def __init__(self, name: str, job_id: str, created_at: str, md_content: str, pdf_bytes: bytes):
        self.name = name
        self.job_id = job_id
        self.created_at = created_at
        self.md_content = md_content
        self.pdf_bytes = pdf_bytes

    def render(self) -> None:
        """Renders the report card with downloads.[span_50](start_span)[span_50](end_span)"""
        html = f"""
        <div class="terminal-card report-card">
            <h3 style="color: #00FF88; margin-top: 0;">{self.name}</h3>
            <div style="color: #9A9A9A; font-size: 0.85rem; margin-bottom: 1rem;">
                Job ID: <code>{self.job_id}</code> | Created: {self.created_at}
            </div>
        </div>
        """
        st.markdown(html, unsafe_allow_html=True)
        DownloadButtons(self.md_content, self.pdf_bytes, f"{self.job_id}_report").render()


class ReportViewer:
    """
    Display Markdown, Syntax highlighting, Code blocks, Tables, Images.[span_51](start_span)[span_51](end_span)
    """
    def __init__(self, markdown_content: str):
        self.markdown_content = markdown_content

    def render(self) -> None:
        """Renders the markdown content natively via Streamlit.[span_52](start_span)[span_52](end_span)"""
        st.markdown('<div class="terminal-card">', unsafe_allow_html=True)
        st.markdown(self.markdown_content)
        st.markdown('</div>', unsafe_allow_html=True)


class Notification:
    """
    Support Success, Error, Warning, Info.[span_53](start_span)[span_53](end_span)
    """
    def __init__(self):
        self.container = st.empty()

    def show(self, message: str, level: str = "info") -> None:
        """Shows a notification block.[span_54](start_span)[span_54](end_span)"""
        color_map = {
            "success": ("#00FF88", "rgba(0, 255, 136, 0.1)"),
            "error": ("#FF5252", "rgba(255, 82, 82, 0.1)"),
            "warning": ("#FF9800", "rgba(255, 152, 0, 0.1)"),
            "info": ("#4FC3F7", "rgba(79, 195, 247, 0.1)")
        }
        color, bg = color_map.get(level, color_map["info"])
        html = f"""
        <div style="border-left: 4px solid {color}; background-color: {bg}; padding: 1rem; border-radius: 4px; margin-bottom: 1rem;">
            <span style="color: {color};">{message}</span>
        </div>
        """
        self.container.markdown(html, unsafe_allow_html=True)

    def hide(self) -> None:
        """Hides the notification.[span_55](start_span)[span_55](end_span)"""
        self.container.empty()


class Toast:
    """
    Animated popup. Auto dismiss.[span_56](start_span)[span_56](end_span)
    """
    def __init__(self):
        pass

    def show(self, message: str, icon: str = "🤖") -> None:
        """Shows a toast notification.[span_57](start_span)[span_57](end_span)"""
        st.toast(message, icon=icon)


class Divider:
    """
    Horizontal divider.[span_58](start_span)[span_58](end_span)
    """
    @staticmethod
    def render() -> None:
        """Renders a styled divider.[span_59](start_span)[span_59](end_span)"""
        st.markdown('<hr style="border: 0; border-top: 1px dashed #2B2B2B; margin: 1.5rem 0;">', unsafe_allow_html=True)


class SectionHeader:
    """
    Support Title, Subtitle, Icon.[span_60](start_span)[span_60](end_span)
    """
    def __init__(self, title: str, subtitle: Optional[str] = None, icon: str = ""):
        self.title = title
        self.subtitle = subtitle
        self.icon = icon

    def render(self) -> None:
        """Renders the section header.[span_61](start_span)[span_61](end_span)"""
        sub_html = f'<div style="color: #9A9A9A; font-size: 0.85rem;">{self.subtitle}</div>' if self.subtitle else ""
        html = f"""
        <div style="margin: 1.5rem 0 1rem 0;">
            <h2 style="color: #F2F2F2; margin: 0; display: flex; align-items: center; gap: 0.5rem;">
                <span>{self.icon}</span> {self.title}
            </h2>
            {sub_html}
        </div>
        """
        st.markdown(html, unsafe_allow_html=True)


class LoadingSpinner:
    """
    Support Terminal spinner, Dots, Progress.[span_62](start_span)[span_62](end_span)
    """
    def __init__(self, message: str = "Processing"):
        self.message = message
        self.placeholder = st.empty()

    def render(self) -> None:
        """Renders the spinner state.[span_63](start_span)[span_63](end_span)"""
        self.start()

    def start(self) -> None:
        """Starts the spinner visual in the placeholder.[span_64](start_span)[span_64](end_span)"""
        html = f"""
        <div style="color: #4FC3F7; font-family: monospace; display: flex; align-items: center; gap: 0.5rem;">
            <span class="badge badge-info" style="animation: glow-pulse 1.5s infinite;">RUNNING</span>
            {self.message}<span style="animation: loading-dots 1.5s infinite;"></span>
        </div>
        """
        self.placeholder.markdown(html, unsafe_allow_html=True)

    def stop(self) -> None:
        """Clears the spinner.[span_65](start_span)[span_65](end_span)"""
        self.placeholder.empty()


class TypingIndicator:
    """
    Blinking cursor. Typing animation.[span_66](start_span)[span_66](end_span)
    """
    def __init__(self, text: str):
        self.text = text

    def render(self) -> None:
        """Renders the typing effect.[span_67](start_span)[span_67](end_span)"""
        html = f"""
        <div style="font-family: monospace; white-space: nowrap; overflow: hidden; border-right: 2px solid #00FF88; animation: typing 2s steps(40, end), blink .75s step-end infinite; color: #00FF88;">
            {self.text}
        </div>
        """
        st.markdown(html, unsafe_allow_html=True)


class EmptyState:
    """
    Professional empty state.[span_68](start_span)[span_68](end_span)
    """
    def __init__(self, message: str = "No data available."):
        self.message = message

    def render(self) -> None:
        """Renders the empty state.[span_69](start_span)[span_69](end_span)"""
        html = f"""
        <div class="terminal-card" style="text-align: center; padding: 3rem 1rem; border-style: dashed;">
            <div style="font-size: 2rem; color: #2B2B2B; margin-bottom: 1rem;">∅</div>
            <div style="color: #9A9A9A;">{self.message}</div>
        </div>
        """
        st.markdown(html, unsafe_allow_html=True)


class ErrorPanel:
    """
    Display Title, Description, Error Code, Recovery suggestion.[span_70](start_span)[span_70](end_span)
    """
    def __init__(self, title: str, description: str, error_code: str, recovery: str):
        self.title = title
        self.description = description
        self.error_code = error_code
        self.recovery = recovery

    def render(self) -> None:
        """Renders the error panel.[span_71](start_span)[span_71](end_span)"""
        html = f"""
        <div class="terminal-card" style="border-color: #FF5252; background-color: rgba(255, 82, 82, 0.05);">
            <h3 style="color: #FF5252; margin-top: 0;">✖ {self.title}</h3>
            <p style="color: #F2F2F2;">{self.description}</p>
            <div style="margin: 1rem 0; font-family: monospace; color: #FF9800;">
                <strong>ERR_CODE:</strong> <code>{self.error_code}</code>
            </div>
            <div style="color: #9A9A9A; font-size: 0.85rem;">
                <strong>Suggestion:</strong> {self.recovery}
            </div>
        </div>
        """
        st.markdown(html, unsafe_allow_html=True)


class SuccessPanel:
    """
    Display Completion summary.[span_72](start_span)[span_72](end_span)
    """
    def __init__(self, summary: str):
        self.summary = summary

    def render(self) -> None:
        """Renders the success panel.[span_73](start_span)[span_73](end_span)"""
        html = f"""
        <div class="terminal-card" style="border-color: #00FF88; background-color: rgba(0, 255, 136, 0.05);">
            <h3 style="color: #00FF88; margin-top: 0;">✔ Success</h3>
            <p style="color: #F2F2F2;">{self.summary}</p>
        </div>
        """
        st.markdown(html, unsafe_allow_html=True)


class TerminalTable:
    """
    Support Headers, Rows, Alignment, ANSI-like styling.[span_74](start_span)[span_74](end_span)
    """
    def __init__(self, headers: List[str], rows: List[List[str]]):
        self.headers = headers
        self.rows = rows

    def render(self) -> None:
        """Renders the terminal table.[span_75](start_span)[span_75](end_span)"""
        headers_html = "".join([f"<th>{h}</th>" for h in self.headers])
        rows_html = ""
        for row in self.rows:
            tds = "".join([f"<td>{cell}</td>" for cell in row])
            rows_html += f"<tr>{tds}</tr>"
        
        html = f"""
        <div style="overflow-x: auto;">
            <table>
                <thead>
                    <tr>{headers_html}</tr>
                </thead>
                <tbody>
                    {rows_html}
                </tbody>
            </table>
        </div>
        """
        st.markdown(html, unsafe_allow_html=True)


class MetricCard:
    """
    Display Label, Value, Trend, Color.[span_76](start_span)[span_76](end_span)
    """
    def __init__(self, label: str, value: str, trend: str = "", color: str = "#00FF88"):
        self.label = label
        self.value = value
        self.trend = trend
        self.color = color

    def render(self) -> None:
        """Renders the metric card.[span_77](start_span)[span_77](end_span)"""
        trend_html = f'<div style="font-size: 0.75rem; color: #9A9A9A;">{self.trend}</div>' if self.trend else ""
        html = f"""
        <div class="terminal-card" style="text-align: center;">
            <div style="color: #9A9A9A; font-size: 0.85rem; text-transform: uppercase;">{self.label}</div>
            <div style="font-size: 2rem; color: {self.color}; font-weight: bold; margin: 0.5rem 0;">{self.value}</div>
            {trend_html}
        </div>
        """
        st.markdown(html, unsafe_allow_html=True)


class FileUploaderCard:
    """
    Accept GitHub URL, ZIP Archive, Rust Source, Wallet Address, Transaction Signature. Validation messages.[span_78](start_span)[span_78](end_span)
    """
    def __init__(self, input_type: str, key: str):
        self.input_type = input_type
        self.key = key

    def render(self) -> Any:
        """Renders the appropriate input mechanism based on type.[span_79](start_span)[span_79](end_span)"""
        st.markdown(f"<div class='terminal-card'><strong>Provide Input:</strong> {self.input_type}</div>", unsafe_allow_html=True)
        
        if self.input_type in ["GitHub URL", "Wallet Address", "Transaction Signature"]:
            return st.text_input(f"Enter {self.input_type}", key=self.key)
        elif self.input_type in ["ZIP Archive", "Rust Source"]:
            return st.file_uploader(f"Upload {self.input_type}", key=self.key)
        return None


class CopyToClipboardButton:
    """
    Support Wallet Address, Transaction Signature, Report ID, Invoice ID.[span_80](start_span)[span_80](end_span)
    """
    def __init__(self, text_to_copy: str, label: str = "Copy"):
        self.text_to_copy = text_to_copy
        self.label = label

    def render(self) -> None:
        """Renders the copy button. Uses streamlit clipboard capabilities if available, or visual fallback.[span_81](start_span)[span_81](end_span)"""
        if st.button(self.label, key=f"copy_{hash(self.text_to_copy)}"):
            st.toast(f"Copied: {self.text_to_copy}", icon="📋")


class DownloadButtons:
    """
    Support Markdown, PDF.[span_82](start_span)[span_82](end_span)
    """
    def __init__(self, md_content: str, pdf_bytes: bytes, file_prefix: str):
        self.md_content = md_content
        self.pdf_bytes = pdf_bytes
        self.file_prefix = file_prefix

    def render(self) -> None:
        """Renders the download buttons.[span_83](start_span)[span_83](end_span)"""
        col1, col2 = st.columns(2)
        with col1:
            st.download_button(
                label="Download Markdown",
                data=self.md_content,
                file_name=f"{self.file_prefix}.md",
                mime="text/markdown",
                use_container_width=True
            )
        with col2:
            st.download_button(
                label="Download PDF",
                data=self.pdf_bytes,
                file_name=f"{self.file_prefix}.pdf",
                mime="application/pdf",
                use_container_width=True
            )
