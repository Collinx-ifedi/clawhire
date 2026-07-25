"""
ClawHire - Terminal Styling and UI Architecture
Version: 1.0.0

This module is responsible for styling the ClawHire Streamlit frontend.
ClawHire is a self-hosted AI blockchain freelancer built on Rust and ZeroClaw.[span_0](start_span)[span_0](end_span)[span_1](start_span)[span_1](end_span)
The frontend is built entirely with Streamlit but completely overrides the default appearance
to look like a modern Linux terminal (e.g., Warp, Hyper, iTerm2).[span_2](start_span)[span_2](end_span)

Provides Theme, Typography, Spacing, Animations, and the CSSBuilder class.
No business logic or API calls are contained in this module.[span_3](start_span)[span_3](end_span)
"""

import streamlit as st
from dataclasses import dataclass
from typing import Dict, Any


@dataclass
class Theme:
    """
    Defines the primary color palette and layout theme for the application.[span_4](start_span)[span_4](end_span)
    """
    # Backgrounds
    bg_primary: str = "#090909"
    bg_secondary: str = "#111111"
    bg_panel: str = "#161616"
    
    # Borders and UI elements
    border_color: str = "#2B2B2B"
    
    # Text
    text_primary: str = "#F2F2F2"
    text_muted: str = "#9A9A9A"
    
    # Core Colors
    green: str = "#00FF88"
    blue: str = "#4FC3F7"
    yellow: str = "#FFD54F"
    orange: str = "#FF9800"
    red: str = "#FF5252"
    purple: str = "#B388FF"
    cyan: str = "#00E5FF"
    white: str = "#FFFFFF"

    # Terminal ANSI Colors (Standard and Bright)
    ansi_black: str = "#000000"
    ansi_red: str = "#FF5252"
    ansi_green: str = "#00FF88"
    ansi_yellow: str = "#FFD54F"
    ansi_blue: str = "#4FC3F7"
    ansi_magenta: str = "#B388FF"
    ansi_cyan: str = "#00E5FF"
    ansi_white: str = "#FFFFFF"
    
    ansi_bright_black: str = "#666666"
    ansi_bright_red: str = "#FF8A80"
    ansi_bright_green: str = "#B9F6CA"
    ansi_bright_yellow: str = "#FFE57F"
    ansi_bright_blue: str = "#80D8FF"
    ansi_bright_magenta: str = "#EA80FC"
    ansi_bright_cyan: str = "#84FFFF"
    ansi_bright_white: str = "#FFFFFF"

    # Layout dimensions
    border_radius: str = "8px"
    terminal_shadow: str = "0 10px 30px rgba(0, 0, 0, 0.8), 0 0 15px rgba(0, 255, 136, 0.1)"
    transition_timing: str = "0.2s ease-in-out"


@dataclass
class Typography:
    """
    Defines the font families and sizing scales.[span_5](start_span)[span_5](end_span)
    """
    primary_font: str = "'JetBrains Mono', 'Fira Code', 'IBM Plex Mono', monospace"
    
    size_h1: str = "2rem"
    size_h2: str = "1.5rem"
    size_h3: str = "1.25rem"
    size_body: str = "0.9rem"
    size_code: str = "0.85rem"
    size_small: str = "0.75rem"
    
    line_height: str = "1.6"


@dataclass
class Spacing:
    """
    Defines padding, margins, and gaps for layouts.[span_6](start_span)[span_6](end_span)
    """
    xs: str = "0.25rem"
    sm: str = "0.5rem"
    md: str = "1rem"
    lg: str = "1.5rem"
    xl: str = "2rem"
    container: str = "2rem"


class Animation:
    """
    CSS animation definitions for terminal effects, cursors, and loading states.[span_7](start_span)[span_7](end_span)
    """
    
    @staticmethod
    def get_keyframes() -> str:
        """Returns the raw CSS keyframe animations."""
        return """
        @keyframes blink {
            0%, 100% { opacity: 1; }
            50% { opacity: 0; }
        }
        @keyframes fade-in {
            from { opacity: 0; }
            to { opacity: 1; }
        }
        @keyframes slide-up {
            from { opacity: 0; transform: translateY(10px); }
            to { opacity: 1; transform: translateY(0); }
        }
        @keyframes typing {
            from { width: 0; }
            to { width: 100%; }
        }
        @keyframes glow-pulse {
            0%, 100% { box-shadow: 0 0 5px rgba(0, 255, 136, 0.2); }
            50% { box-shadow: 0 0 15px rgba(0, 255, 136, 0.6); }
        }
        @keyframes scanline {
            0% { transform: translateY(-100%); }
            100% { transform: translateY(100vh); }
        }
        @keyframes boot {
            0% { opacity: 0; transform: scale(0.98); filter: blur(4px); }
            100% { opacity: 1; transform: scale(1); filter: blur(0); }
        }
        @keyframes loading-dots {
            0%, 20% { content: "."; }
            40% { content: ".."; }
            60%, 100% { content: "..."; }
        }
        """


class CSSBuilder:
    """
    Constructs the master CSS string to override Streamlit and style the terminal UI.[span_8](start_span)[span_8](end_span)
    """
    
    def __init__(self):
        self.theme = Theme()
        self.typography = Typography()
        self.spacing = Spacing()

    def build_header(self) -> str:
        """Overrides Streamlit header."""
        return """
        [data-testid="stHeader"] {
            display: none !important;
        }
        .stDeployButton {
            display: none !important;
        }
        #MainMenu {
            display: none !important;
        }
        """

    def build_footer(self) -> str:
        """Overrides Streamlit footer."""
        return """
        footer {
            display: none !important;
        }
        """

    def build_terminal(self) -> str:
        """Styles the main application container to look like a terminal window."""
        return f"""
        .stApp {{
            background-color: {self.theme.bg_primary} !important;
            color: {self.theme.text_primary} !important;
            font-family: {self.typography.primary_font} !important;
            background-image: radial-gradient(circle at 50% 50%, rgba(0, 255, 136, 0.02) 0%, transparent 100%);
        }}
        [data-testid="stAppViewContainer"] {{
            background-color: transparent !important;
            animation: boot 0.5s ease-out forwards;
        }}
        [data-testid="stBlock"] {{
            gap: {self.spacing.md};
        }}
        /* Terminal Window Container */
        .terminal-window {{
            background-color: {self.theme.bg_secondary};
            border: 1px solid {self.theme.border_color};
            border-radius: {self.theme.border_radius};
            box-shadow: {self.theme.terminal_shadow};
            margin: {self.spacing.lg} auto;
            max-width: 1200px;
            overflow: hidden;
            position: relative;
        }}
        /* Title bar with traffic lights */
        .terminal-titlebar {{
            background-color: {self.theme.bg_panel};
            border-bottom: 1px solid {self.theme.border_color};
            padding: {self.spacing.sm} {self.spacing.md};
            display: flex;
            align-items: center;
            height: 32px;
        }}
        .terminal-traffic-lights {{
            display: flex;
            gap: 8px;
        }}
        .traffic-light {{
            width: 12px;
            height: 12px;
            border-radius: 50%;
        }}
        .tl-red {{ background-color: {self.theme.red}; }}
        .tl-yellow {{ background-color: {self.theme.yellow}; }}
        .tl-green {{ background-color: {self.theme.green}; }}
        
        .terminal-title {{
            margin: 0 auto;
            color: {self.theme.text_muted};
            font-size: {self.typography.size_small};
            user-select: none;
        }}
        /* Scanline Overlay */
        .scanline-overlay {{
            position: fixed;
            top: 0;
            left: 0;
            width: 100vw;
            height: 100vh;
            background: linear-gradient(
                to bottom,
                rgba(255,255,255,0),
                rgba(255,255,255,0) 50%,
                rgba(0,0,0,0.1) 50%,
                rgba(0,0,0,0.1)
            );
            background-size: 100% 4px;
            pointer-events: none;
            z-index: 9999;
            opacity: 0.3;
        }}
        """

    def build_buttons(self) -> str:
        """Styles interactive buttons."""
        return f"""
        .stButton > button {{
            background-color: transparent !important;
            color: {self.theme.green} !important;
            border: 1px solid {self.theme.green} !important;
            border-radius: {self.theme.border_radius} !important;
            font-family: {self.typography.primary_font} !important;
            text-transform: uppercase;
            letter-spacing: 1px;
            padding: {self.spacing.sm} {self.spacing.md} !important;
            transition: all {self.theme.transition_timing} !important;
        }}
        .stButton > button:hover {{
            background-color: rgba(0, 255, 136, 0.1) !important;
            box-shadow: 0 0 10px rgba(0, 255, 136, 0.3) !important;
            color: {self.theme.white} !important;
            border-color: {self.theme.green} !important;
        }}
        .stButton > button:active {{
            transform: scale(0.98);
        }}
        """

    def build_cards(self) -> str:
        """Styles general card containers."""
        return f"""
        .terminal-card {{
            background-color: {self.theme.bg_panel};
            border: 1px solid {self.theme.border_color};
            border-radius: {self.theme.border_radius};
            padding: {self.spacing.md};
            margin-bottom: {self.spacing.md};
            animation: slide-up 0.4s ease-out;
        }}
        """

    def build_badges(self) -> str:
        """Styles inline status badges."""
        return f"""
        .badge {{
            display: inline-block;
            padding: 0.15rem 0.5rem;
            border-radius: 12px;
            font-size: {self.typography.size_small};
            font-weight: bold;
            text-transform: uppercase;
        }}
        .badge-success {{ background-color: rgba(0, 255, 136, 0.2); color: {self.theme.green}; border: 1px solid {self.theme.green}; }}
        .badge-warning {{ background-color: rgba(255, 213, 79, 0.2); color: {self.theme.yellow}; border: 1px solid {self.theme.yellow}; }}
        .badge-danger {{ background-color: rgba(255, 82, 82, 0.2); color: {self.theme.red}; border: 1px solid {self.theme.red}; }}
        .badge-info {{ background-color: rgba(79, 195, 247, 0.2); color: {self.theme.blue}; border: 1px solid {self.theme.blue}; }}
        """

    def build_tables(self) -> str:
        """Styles data tables to look like modern terminal outputs."""
        return f"""
        .stDataFrame, .stTable {{
            font-family: {self.typography.primary_font} !important;
            font-size: {self.typography.size_code} !important;
        }}
        table {{
            width: 100%;
            border-collapse: collapse;
            background-color: {self.theme.bg_secondary};
            border-radius: {self.theme.border_radius};
            overflow: hidden;
            border: 1px solid {self.theme.border_color};
        }}
        th {{
            background-color: {self.theme.bg_panel};
            color: {self.theme.blue};
            text-align: left;
            padding: {self.spacing.sm} {self.spacing.md};
            border-bottom: 1px solid {self.theme.border_color};
            text-transform: uppercase;
            font-size: {self.typography.size_small};
        }}
        td {{
            padding: {self.spacing.sm} {self.spacing.md};
            border-bottom: 1px dashed {self.theme.border_color};
            color: {self.theme.text_primary};
        }}
        tr:hover td {{
            background-color: rgba(255, 255, 255, 0.02);
        }}
        """

    def build_scrollbars(self) -> str:
        """Styles custom scrollbars."""
        return f"""
        ::-webkit-scrollbar {{
            width: 8px;
            height: 8px;
        }}
        ::-webkit-scrollbar-track {{
            background: {self.theme.bg_primary};
        }}
        ::-webkit-scrollbar-thumb {{
            background: {self.theme.border_color};
            border-radius: 4px;
        }}
        ::-webkit-scrollbar-thumb:hover {{
            background: {self.theme.text_muted};
        }}
        """

    def build_progress(self) -> str:
        """Styles animated progress bars."""
        return f"""
        .stProgress > div > div > div > div {{
            background: linear-gradient(90deg, {self.theme.blue}, {self.theme.green});
            box-shadow: 0 0 10px rgba(0, 255, 136, 0.5);
            border-radius: 4px;
        }}
        """

    def build_reports(self) -> str:
        """Styles markdown output and report cards."""
        return f"""
        .report-card {{
            border-left: 4px solid {self.theme.green};
            background: {self.theme.bg_panel};
            padding: {self.spacing.md};
            margin-top: {self.spacing.md};
            border-radius: 0 {self.theme.border_radius} {self.theme.border_radius} 0;
        }}
        code {{
            background-color: rgba(255, 255, 255, 0.1) !important;
            color: {self.theme.yellow} !important;
            padding: 0.2rem 0.4rem !important;
            border-radius: 4px !important;
            font-family: {self.typography.primary_font} !important;
        }}
        pre {{
            background-color: {self.theme.bg_secondary} !important;
            border: 1px solid {self.theme.border_color} !important;
            border-radius: {self.theme.border_radius} !important;
            padding: {self.spacing.md} !important;
        }}
        """

    def build_invoice(self) -> str:
        """Styles Solana payment invoices."""
        return f"""
        .invoice-card {{
            background: {self.theme.bg_panel};
            border: 1px solid {self.theme.blue};
            border-radius: {self.theme.border_radius};
            padding: {self.spacing.lg};
            text-align: center;
            box-shadow: 0 0 15px rgba(79, 195, 247, 0.1);
        }}
        .invoice-amount {{
            font-size: {self.typography.size_h1};
            color: {self.theme.blue};
            font-weight: bold;
            margin: {self.spacing.md} 0;
        }}
        """

    def build_modals(self) -> str:
        """Styles dialogs and popups."""
        return f"""
        div[data-testid="stDialog"] > div {{
            background-color: {self.theme.bg_secondary};
            border: 1px solid {self.theme.border_color};
            border-radius: {self.theme.border_radius};
            box-shadow: {self.theme.terminal_shadow};
        }}
        """

    def build_forms(self) -> str:
        """Styles general form wrappers."""
        return f"""
        form {{
            background: {self.theme.bg_panel};
            padding: {self.spacing.md};
            border-radius: {self.theme.border_radius};
            border: 1px solid {self.theme.border_color};
        }}
        """

    def build_inputs(self) -> str:
        """Styles text inputs, textareas, and selects."""
        return f"""
        .stTextInput > div > div > input,
        .stTextArea > div > textarea,
        .stSelectbox > div > div > div {{
            background-color: {self.theme.bg_secondary} !important;
            color: {self.theme.text_primary} !important;
            border: 1px solid {self.theme.border_color} !important;
            border-radius: 4px !important;
            font-family: {self.typography.primary_font} !important;
            padding: {self.spacing.sm} {self.spacing.md} !important;
        }}
        .stTextInput > div > div > input:focus,
        .stTextArea > div > textarea:focus {{
            border-color: {self.theme.green} !important;
            box-shadow: 0 0 5px rgba(0, 255, 136, 0.3) !important;
        }}
        /* Hide label if desired, or style it */
        .stTextInput > label, .stTextArea > label, .stSelectbox > label {{
            color: {self.theme.text_muted} !important;
            font-family: {self.typography.primary_font} !important;
        }}
        """

    def build_sidebar(self) -> str:
        """Overrides and styles the sidebar if used."""
        return f"""
        [data-testid="stSidebar"] {{
            background-color: {self.theme.bg_secondary} !important;
            border-right: 1px solid {self.theme.border_color} !important;
        }}
        [data-testid="stSidebar"] .stMarkdown {{
            color: {self.theme.text_muted};
        }}
        """

    def build_downloads(self) -> str:
        """Styles file download components."""
        return f"""
        .stDownloadButton > button {{
            background-color: rgba(79, 195, 247, 0.1) !important;
            border-color: {self.theme.blue} !important;
            color: {self.theme.blue} !important;
        }}
        .stDownloadButton > button:hover {{
            background-color: {self.theme.blue} !important;
            color: {self.theme.bg_primary} !important;
            box-shadow: 0 0 10px rgba(79, 195, 247, 0.4) !important;
        }}
        """

    def build_mobile(self) -> str:
        """Responsive adjustments for mobile."""
        return f"""
        @media (max-width: 768px) {{
            .terminal-window {{
                margin: 0;
                border-radius: 0;
                border: none;
            }}
            [data-testid="stBlock"] {{
                padding: {self.spacing.sm};
            }}
        }}
        """

    def build_desktop(self) -> str:
        """Responsive adjustments for desktop."""
        return """
        @media (min-width: 769px) {
            /* Standard desktop view inherits base window styles */
        }
        """

    def build(self) -> str:
        """
        Compiles all CSS components into a single stylesheet string.[span_9](start_span)[span_9](end_span)
        """
        styles = [
            Animation.get_keyframes(),
            self.build_header(),
            self.build_footer(),
            self.build_terminal(),
            self.build_buttons(),
            self.build_cards(),
            self.build_badges(),
            self.build_tables(),
            self.build_scrollbars(),
            self.build_progress(),
            self.build_reports(),
            self.build_invoice(),
            self.build_modals(),
            self.build_forms(),
            self.build_inputs(),
            self.build_sidebar(),
            self.build_downloads(),
            self.build_mobile(),
            self.build_desktop()
        ]
        return "\n".join(styles)


@st.cache_data
def get_css() -> str:
    """
    Returns the compiled CSS stylesheet. Cached for performance.[span_10](start_span)[span_10](end_span)
    """
    builder = CSSBuilder()
    return f"<style>{builder.build()}</style>"


def inject_css() -> None:
    """
    Injects the compiled CSS into the Streamlit application.[span_11](start_span)[span_11](end_span)
    """
    st.markdown(get_css(), unsafe_allow_html=True)


def terminal_style(text: str, color: str = "green", bold: bool = False, blink: bool = False) -> str:
    """
    Formats text to look like terminal output with ANSI-like colors.[span_12](start_span)[span_12](end_span)
    
    Args:
        text: The string to format.
        color: The color key (e.g., 'green', 'blue', 'red').
        bold: Whether to bold the text.
        blink: Whether to apply a blinking animation.
        
    Returns:
        HTML formatted string.
    """
    theme = Theme()
    hex_color = getattr(theme, color, theme.green)
    
    styles = [f"color: {hex_color};"]
    if bold:
        styles.append("font-weight: bold;")
    if blink:
        styles.append("animation: blink 1s step-end infinite;")
        
    style_str = " ".join(styles)
    return f"<span style='{style_str}'>{text}</span>"


def status_color(status: str) -> str:
    """
    Maps a job or payment status to a corresponding terminal color.[span_13](start_span)[span_13](end_span)
    
    Args:
        status: The status string (e.g., 'completed', 'pending').
        
    Returns:
        The color key string.
    """
    status_map = {
        "completed": "green",
        "pending": "yellow",
        "executing": "cyan",
        "error": "red",
        "failed": "red",
        "generating": "purple",
        "awaiting payment": "blue"
    }
    return status_map.get(status.lower(), "white")


def severity_color(severity: str) -> str:
    """
    Maps a security vulnerability severity to a corresponding terminal color.[span_14](start_span)[span_14](end_span)
    
    Args:
        severity: 'critical', 'high', 'medium', 'low', 'info'.
        
    Returns:
        The color key string.
    """
    severity_map = {
        "critical": "red",
        "high": "orange",
        "medium": "yellow",
        "low": "blue",
        "info": "cyan"
    }
    return severity_map.get(severity.lower(), "white")


def badge_style(text: str, badge_type: str = "info") -> str:
    """
    Generates HTML for an inline status badge.[span_15](start_span)[span_15](end_span)
    
    Args:
        text: The text inside the badge.
        badge_type: 'success', 'warning', 'danger', 'info'.
        
    Returns:
        HTML formatted string for the badge.
    """
    return f"<span class='badge badge-{badge_type}'>{text}</span>"


def generate_theme() -> Dict[str, Any]:
    """
    Returns the theme dictionary for any internal component needing raw values.[span_16](start_span)[span_16](end_span)
    """
    theme = Theme()
    return {
        "primaryColor": theme.green,
        "backgroundColor": theme.bg_primary,
        "secondaryBackgroundColor": theme.bg_secondary,
        "textColor": theme.text_primary,
        "font": "monospace"
    }
