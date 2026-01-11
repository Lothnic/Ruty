"""Qt5 Chat Window for Ruty"""
from PyQt5.QtWidgets import (
    QWidget, QVBoxLayout, QHBoxLayout, QTextEdit, 
    QLineEdit, QPushButton, QLabel
)
from PyQt5.QtCore import Qt, QThread, pyqtSignal
from PyQt5.QtGui import QFont
from datetime import datetime
from langchain_core.messages import HumanMessage
import threading


class AgentWorker(QThread):
    """Background thread for agent processing"""
    response_ready = pyqtSignal(str)
    
    def __init__(self, agent, config, user_input):
        super().__init__()
        self.agent = agent
        self.config = config
        self.user_input = user_input
        
    def run(self):
        """Process input with agent in background thread"""
        try:
            final_response = None
            for event in self.agent.stream(
                {"messages": [HumanMessage(content=self.user_input)]},
                config=self.config,
                stream_mode="values"
            ):
                if "messages" in event:
                    last_msg = event["messages"][-1]
                    if hasattr(last_msg, "content") and last_msg.content:
                        if not hasattr(last_msg, "tool_calls") or not last_msg.tool_calls:
                            final_response = last_msg.content
            
            self.response_ready.emit(final_response or "No response")
        except Exception as e:
            self.response_ready.emit(f"Error: {e}")


class RutyChatWindow(QWidget):
    """Popup chat window for Ruty agent"""
    
    def __init__(self, agent, config):
        super().__init__()
        self.agent = agent
        self.config = config
        self.worker = None
        
        # Window properties
        self.setWindowTitle("🧠 Ruty")
        self.setGeometry(100, 100, 600, 700)
        # Qt.Tool makes it a floating window, often bypassing focus rules
        self.setWindowFlags(Qt.Window | Qt.WindowStaysOnTopHint | Qt.Tool | Qt.FramelessWindowHint)
        
        # Build UI
        self.init_ui()
        
    def init_ui(self):
        """Construct the Qt interface"""
        layout = QVBoxLayout()
        layout.setContentsMargins(15, 15, 15, 15)
        layout.setSpacing(10)
        
        # Header
        header = QLabel("🧠 Ruty Assistant")
        header_font = QFont()
        header_font.setPointSize(14)
        header_font.setBold(True)
        header.setFont(header_font)
        header.setAlignment(Qt.AlignCenter)
        layout.addWidget(header)
        
        # Chat history view
        self.chat_view = QTextEdit()
        self.chat_view.setReadOnly(True)
        self.chat_view.setFont(QFont("Monospace", 10))
        self.chat_view.setPlaceholderText("Your conversation will appear here...")
        layout.addWidget(self.chat_view)
        
        # Input area
        input_layout = QHBoxLayout()
        
        self.input_field = QLineEdit()
        self.input_field.setPlaceholderText("Ask me anything... (Press Enter to send, Esc to hide)")
        self.input_field.setFont(QFont("Sans", 11))
        self.input_field.returnPressed.connect(self.send_message)
        input_layout.addWidget(self.input_field)
        
        send_button = QPushButton("Send")
        send_button.setFixedWidth(80)
        send_button.clicked.connect(self.send_message)
        input_layout.addWidget(send_button)
        
        layout.addLayout(input_layout)
        
        # Hint label
        hint = QLabel("💡 Tip: Press Esc to hide window | Super+Space to toggle")
        hint.setStyleSheet("color: gray; font-size: 10px;")
        hint.setAlignment(Qt.AlignCenter)
        layout.addWidget(hint)
        
        self.setLayout(layout)
        
    def append_message(self, role, content):
        """Add a message to the chat history"""
        timestamp = datetime.now().strftime("%H:%M")
        self.chat_view.append(f"<b>[{timestamp}] {role}:</b> {content}<br>")
        
    def send_message(self):
        """Handle send button click or Enter press"""
        user_input = self.input_field.text().strip()
        if not user_input:
            return
        
        # Clear input
        self.input_field.clear()
        
        # Display user message
        self.append_message("You", user_input)
        
        # Show thinking indicator
        self.append_message("Ruty", "<i>thinking...</i>")
        
        # Disable input while processing
        self.input_field.setEnabled(False)
        
        # Process in background thread
        self.worker = AgentWorker(self.agent, self.config, user_input)
        self.worker.response_ready.connect(self.on_response_ready)
        self.worker.start()
        
    def on_response_ready(self, response):
        """Handle agent response"""
        # Remove "thinking..." message
        cursor = self.chat_view.textCursor()
        cursor.movePosition(cursor.End)
        cursor.select(cursor.BlockUnderCursor)
        cursor.removeSelectedText()
        cursor.deletePreviousChar()  # Remove extra newline
        
        # Add real response
        self.append_message("Ruty", response)
        
        # Re-enable input
        self.input_field.setEnabled(True)
        self.input_field.setFocus()
        
    def keyPressEvent(self, event):
        """Handle keyboard shortcuts"""
        if event.key() == Qt.Key_Escape:
            self.hide()
        else:
            super().keyPressEvent(event)
            
    def closeEvent(self, event):
        """Prevent window close, just hide instead"""
        event.ignore()
        self.hide()
        
    def toggle_visibility(self):
        """Show or hide the window"""
        if self.isVisible():
            self.hide()
        else:
            self.show()
            self.activateWindow()
            self.raise_()
            self.input_field.setFocus()


def create_chat_window(agent, config):
    """Factory function to create chat window"""
    return RutyChatWindow(agent, config)
