"""Qt5 Spotlight-style Window for Ruty"""
from PyQt5.QtWidgets import (
    QWidget, QVBoxLayout, QLineEdit, QLabel, QGraphicsDropShadowEffect
)
from PyQt5.QtCore import Qt, QThread, pyqtSignal, QPropertyAnimation, QEasingCurve
from PyQt5.QtGui import QFont, QPalette, QColor
from langchain_core.messages import HumanMessage


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
            from langchain_core.messages import AIMessage, ToolMessage
            
            tool_names = []
            ai_responses = []  # Collect all AI responses
            
            # Stream events from agent
            for event in self.agent.stream(
                {"messages": [HumanMessage(content=self.user_input)]},
                config=self.config,
                stream_mode="values"
            ):
                if "messages" in event:
                    for msg in event["messages"]:
                        # Track tool usage
                        if hasattr(msg, "tool_calls") and msg.tool_calls:
                            for tool_call in msg.tool_calls:
                                tool_name = tool_call.get("name", "unknown")
                                if tool_name not in tool_names:
                                    tool_names.append(tool_name)
                                    print(f"DEBUG: Tool called: {tool_name}")
                        
                        # Capture AI responses (final answers, not tool call requests)
                        if isinstance(msg, AIMessage):
                            # Only capture if it has content and is NOT followed by tool calls
                            if msg.content and not (hasattr(msg, "tool_calls") and msg.tool_calls):
                                print(f"DEBUG: AI Response captured: {msg.content[:100]}...")
                                ai_responses.append(msg.content)
            
            print(f"DEBUG: Total AI responses: {len(ai_responses)}")
            print(f"DEBUG: Tools used: {tool_names}")
            
            # Format output - use the LAST AI response (most final)
            output_parts = []
            if tool_names:
                output_parts.append(f"<i>🔧 {', '.join(tool_names)}</i>")
            
            if ai_responses:
                # Use last AI response (the final answer)
                output_parts.append(ai_responses[-1])
            
            full_response = "<br><br>".join(output_parts) if output_parts else "No response"
            print(f"DEBUG: Final response length: {len(full_response)}")
            self.response_ready.emit(full_response)
            
        except Exception as e:
            import traceback
            print(f"DEBUG: Exception: {e}")
            traceback.print_exc()
            error_msg = f"<b>❌ Error:</b> {str(e)}<br><small><pre>{traceback.format_exc()}</pre></small>"
            self.response_ready.emit(error_msg)


class RutyChatWindow(QWidget):
    """Spotlight-style popup window for Ruty agent"""
    
    def __init__(self, agent, config):
        super().__init__()
        self.agent = agent
        self.config = config
        self.worker = None
        
        # Window properties - Spotlight style
        self.setWindowTitle("Ruty")
        self.setMinimumSize(600, 120)
        self.resize(600, 200)  # Compact like Spotlight
        
        # Frameless, always on top - use Popup for Wayland centering
        self.setWindowFlags(
            Qt.FramelessWindowHint | 
            Qt.WindowStaysOnTopHint | 
            Qt.Tool
        )
        
        # Enable true transparency (required for translucent backgrounds)
        self.setAttribute(Qt.WA_TranslucentBackground, False)
        
        # Build UI
        self.init_ui()
        
    def center_on_screen(self):
        """Center the window on the screen"""
        from PyQt5.QtWidgets import QApplication, QDesktopWidget
        # Get screen geometry
        desktop = QDesktopWidget()
        screen = desktop.availableGeometry(desktop.primaryScreen())
        # Calculate center position (upper third for Spotlight feel)
        x = screen.x() + (screen.width() - self.width()) // 2
        y = screen.y() + (screen.height() - self.height()) // 3
        self.setGeometry(x, y, self.width(), self.height())
        
    def init_ui(self):
        """Construct the Spotlight-style interface"""
        # Main container with rounded corners and slight transparency
        self.setStyleSheet("""
            RutyChatWindow {
                background-color: rgba(35, 35, 40, 245);
                border-radius: 12px;
                border: 1px solid rgba(80, 80, 90, 180);
            }
        """)
        
        layout = QVBoxLayout()
        layout.setContentsMargins(16, 16, 16, 16)
        layout.setSpacing(12)
        
        # Search input (at top, Spotlight style)
        self.input_field = QLineEdit()
        self.input_field.setPlaceholderText("Ask Ruty anything...")
        self.input_field.setFont(QFont("Ubuntu", 16))
        self.input_field.setStyleSheet("""
            QLineEdit {
                background-color: rgba(80, 80, 80, 200);
                border: none;
                border-radius: 8px;
                padding: 14px 18px;
                color: white;
                font-size: 16px;
            }
            QLineEdit:focus {
                background-color: rgba(100, 100, 100, 220);
            }
        """)
        self.input_field.returnPressed.connect(self.send_message)
        layout.addWidget(self.input_field)
        
        # Scrollable result area
        from PyQt5.QtWidgets import QTextEdit
        
        self.result_area = QTextEdit()
        self.result_area.setReadOnly(True)
        self.result_area.setFont(QFont("Ubuntu", 12))
        self.result_area.setStyleSheet("""
            QTextEdit {
                background-color: transparent;
                color: #E8E8E8;
                border: none;
                padding: 8px;
            }
        """)
        self.result_area.setVerticalScrollBarPolicy(Qt.ScrollBarAsNeeded)
        self.result_area.setHorizontalScrollBarPolicy(Qt.ScrollBarAlwaysOff)
        layout.addWidget(self.result_area, 1)
        
        self.setLayout(layout)
        
    def send_message(self):
        """Handle Enter key press"""
        user_input = self.input_field.text().strip()
        if not user_input:
            return
        
        # Clear previous result and show thinking
        self.result_area.clear()
        self.result_area.append("⚡ <i>Thinking...</i>")
        
        # Disable input while processing
        self.input_field.setEnabled(False)
        
        # Process in background thread
        self.worker = AgentWorker(self.agent, self.config, user_input)
        self.worker.response_ready.connect(self.on_response_ready)
        self.worker.start()
        
    def on_response_ready(self, response):
        """Handle agent response"""
        # Clear thinking message
        self.result_area.clear()
        
        # Show response with proper formatting
        self.result_area.setHtml(f"<div style='line-height: 1.6;'>{response}</div>")
        
        # Force document layout update
        self.result_area.document().setTextWidth(self.result_area.viewport().width())
        
        # Auto-resize window based on content
        doc_height = self.result_area.document().size().height()
        input_height = 60  # Input field + margins
        padding = 50  # Extra padding
        needed_height = min(int(doc_height) + input_height + padding, 500)  # Cap at 500px
        needed_height = max(needed_height, 200)  # Min 200px
        
        self.resize(600, needed_height)
        self.center_on_screen()
        
        # Clear input and re-enable
        self.input_field.clear()
        self.input_field.setEnabled(True)
        self.input_field.setFocus()
        
    def animate_resize(self, width, height):
        """Smoothly resize the window"""
        self.animation = QPropertyAnimation(self, b"size")
        self.animation.setDuration(200)
        self.animation.setStartValue(self.size())
        self.animation.setEndValue(self.size().__class__(width, height))
        self.animation.setEasingCurve(QEasingCurve.OutCubic)
        self.animation.start()
        
    def keyPressEvent(self, event):
        """Handle keyboard shortcuts"""
        if event.key() == Qt.Key_Escape:
            # Reset size and hide
            self.resize(600, 200)
            self.result_area.clear()
            self.hide()
        else:
            super().keyPressEvent(event)
            
    def closeEvent(self, event):
        """Prevent window close, just hide instead"""
        event.ignore()
        self.hide()
        
    def showEvent(self, event):
        """Handle show event"""
        super().showEvent(event)
        # Delay centering slightly for Wayland to process the window
        from PyQt5.QtCore import QTimer
        QTimer.singleShot(10, self.center_on_screen)
        
    def toggle_visibility(self):
        """Show or hide the window"""
        if self.isVisible():
            # Reset to default size when hiding
            self.resize(600, 200)
            self.result_area.clear()
            self.hide()
        else:
            self.show()
            # Center after show with multiple attempts for Wayland
            from PyQt5.QtCore import QTimer
            QTimer.singleShot(50, self.center_on_screen)
            QTimer.singleShot(150, self.center_on_screen)
            self.activateWindow()
            self.raise_()
            self.input_field.setFocus()


def create_chat_window(agent, config):
    """Factory function to create chat window"""
    return RutyChatWindow(agent, config)
