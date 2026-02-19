#!/usr/bin/env python3
"""
Python client example for winux-ai-service

This example demonstrates how to use the AI service from Python applications,
which is useful for GTK/GI-based applications.

Requirements:
    pip install PyGObject dbus-python

Usage:
    python3 client.py
"""

import dbus
from dbus.mainloop.glib import DBusGMainLoop
from gi.repository import GLib


# D-Bus service details
SERVICE_NAME = "com.winux.AI"
OBJECT_PATH = "/com/winux/AI"
INTERFACE_NAME = "com.winux.AI"


class WinuxAIClient:
    """Client for the Winux AI Service"""

    def __init__(self, use_system_bus=True):
        """Initialize the AI client.

        Args:
            use_system_bus: If True, connect to system bus. If False, use session bus.
        """
        if use_system_bus:
            self.bus = dbus.SystemBus()
        else:
            self.bus = dbus.SessionBus()

        self.proxy = self.bus.get_object(SERVICE_NAME, OBJECT_PATH)
        self.interface = dbus.Interface(self.proxy, INTERFACE_NAME)

    def complete(self, prompt: str, model: str = "gpt-4o") -> str:
        """Complete text based on a prompt.

        Args:
            prompt: The text prompt to complete
            model: The model to use

        Returns:
            The completed text
        """
        return str(self.interface.Complete(prompt, model))

    def chat(self, messages: list, model: str = "gpt-4o") -> str:
        """Chat with message history.

        Args:
            messages: List of (role, content) tuples
            model: The model to use

        Returns:
            The assistant's response
        """
        # Convert to D-Bus array of structs
        dbus_messages = [(str(role), str(content)) for role, content in messages]
        return str(self.interface.Chat(dbus_messages, model))

    def summarize(self, text: str) -> str:
        """Summarize text.

        Args:
            text: The text to summarize

        Returns:
            A concise summary
        """
        return str(self.interface.Summarize(text))

    def translate(self, text: str, from_lang: str, to_lang: str) -> str:
        """Translate text between languages.

        Args:
            text: The text to translate
            from_lang: Source language code (or "auto")
            to_lang: Target language code

        Returns:
            The translated text
        """
        return str(self.interface.Translate(text, from_lang, to_lang))

    def analyze_code(self, code: str, language: str) -> str:
        """Analyze source code.

        Args:
            code: The code to analyze
            language: The programming language

        Returns:
            Code analysis
        """
        return str(self.interface.AnalyzeCode(code, language))

    def analyze_image(self, image_path: str, prompt: str) -> str:
        """Analyze an image.

        Args:
            image_path: Path to the image file
            prompt: What to analyze about the image

        Returns:
            Image analysis/description
        """
        return str(self.interface.AnalyzeImage(image_path, prompt))

    def version(self) -> str:
        """Get the service version.

        Returns:
            Version string
        """
        return str(self.interface.Version())

    def health_check(self) -> bool:
        """Check if the service is healthy.

        Returns:
            True if healthy
        """
        return bool(self.interface.HealthCheck())


class StreamingAIClient(WinuxAIClient):
    """Client with streaming support"""

    def __init__(self, use_system_bus=True):
        super().__init__(use_system_bus)
        DBusGMainLoop(set_as_default=True)
        self._callbacks = {}

    def chat_stream(self, messages: list, model: str = "gpt-4o",
                    on_chunk=None, on_done=None) -> str:
        """Start a streaming chat.

        Args:
            messages: List of (role, content) tuples
            model: The model to use
            on_chunk: Callback for each chunk (request_id, chunk)
            on_done: Callback when streaming is complete (request_id, full_response)

        Returns:
            The request ID
        """
        dbus_messages = [(str(role), str(content)) for role, content in messages]
        request_id = str(self.interface.ChatStream(dbus_messages, model))

        # Store callbacks
        self._callbacks[request_id] = {
            'on_chunk': on_chunk,
            'on_done': on_done,
            'response': ''
        }

        # Connect to signal
        self.bus.add_signal_receiver(
            self._handle_streaming_response,
            signal_name="StreamingResponse",
            dbus_interface=INTERFACE_NAME,
            bus_name=SERVICE_NAME,
            path=OBJECT_PATH
        )

        return request_id

    def _handle_streaming_response(self, request_id, chunk, done):
        """Handle streaming response signal"""
        if request_id not in self._callbacks:
            return

        callbacks = self._callbacks[request_id]
        callbacks['response'] += chunk

        if callbacks['on_chunk']:
            callbacks['on_chunk'](request_id, chunk)

        if done:
            if callbacks['on_done']:
                callbacks['on_done'](request_id, callbacks['response'])
            del self._callbacks[request_id]


def main():
    """Example usage"""
    print("Winux AI Service Python Client Example\n")

    # Create client
    try:
        client = WinuxAIClient(use_system_bus=True)
    except dbus.exceptions.DBusException as e:
        print(f"Failed to connect to system bus: {e}")
        print("Trying session bus...")
        client = WinuxAIClient(use_system_bus=False)

    # Health check
    print("Checking service health...")
    try:
        healthy = client.health_check()
        print(f"Service healthy: {healthy}\n")
    except dbus.exceptions.DBusException as e:
        print(f"Service not available: {e}")
        return

    # Get version
    version = client.version()
    print(f"Service version: {version}\n")

    # Text completion
    print("=== Text Completion ===")
    prompt = "Complete this sentence: The future of desktop computing is"
    print(f"Prompt: {prompt}")
    response = client.complete(prompt)
    print(f"Response: {response}\n")

    # Chat
    print("=== Chat ===")
    messages = [
        ("system", "You are a helpful assistant for the Winux operating system."),
        ("user", "What makes Winux special?")
    ]
    print(f"Messages: {messages}")
    response = client.chat(messages)
    print(f"Response: {response}\n")

    # Translation
    print("=== Translation ===")
    text = "Welcome to Winux!"
    print(f"English: {text}")
    translation = client.translate(text, "en", "es")
    print(f"Spanish: {translation}\n")

    # Summarization
    print("=== Summarization ===")
    long_text = """
    Linux is a family of open-source Unix-like operating systems based on the
    Linux kernel, an operating system kernel first released on September 17, 1991,
    by Linus Torvalds. Linux is typically packaged in a Linux distribution.
    Distributions include the Linux kernel and supporting system software and
    libraries, many of which are provided by the GNU Project. Many Linux distributions
    use the word "Linux" in their name, but the Free Software Foundation uses the
    name "GNU/Linux" to emphasize the importance of GNU software, causing some
    controversy.
    """
    print(f"Text: {long_text[:100]}...")
    summary = client.summarize(long_text)
    print(f"Summary: {summary}\n")

    # Code analysis
    print("=== Code Analysis ===")
    code = """
def fibonacci(n):
    if n <= 1:
        return n
    return fibonacci(n-1) + fibonacci(n-2)
"""
    print(f"Code:\n{code}")
    analysis = client.analyze_code(code, "python")
    print(f"Analysis: {analysis}\n")

    print("All examples completed successfully!")


if __name__ == "__main__":
    main()
