import { useState, useCallback, useRef, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import "./AIAssistant.css";

interface ChatMessage {
  role: "user" | "assistant" | "system";
  content: string;
}

interface OllamaModel {
  name: string;
  modified_at: string;
  size: number;
}

interface AIChatResponse {
  content: string;
  done: boolean;
  error: string | null;
}

interface AIAssistantProps {
  selectedText?: string;
  onInsertText?: (text: string) => void;
}

export function AIAssistant({ selectedText, onInsertText }: AIAssistantProps) {
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [input, setInput] = useState("");
  const [isLoading, setIsLoading] = useState(false);
  const [ollamaStatus, setOllamaStatus] = useState<"checking" | "connected" | "disconnected">("checking");
  const [models, setModels] = useState<OllamaModel[]>([]);
  const [selectedModel, setSelectedModel] = useState<string>("");
  const [streamingContent, setStreamingContent] = useState("");
  const chatEndRef = useRef<HTMLDivElement>(null);
  const unlistenRef = useRef<UnlistenFn | null>(null);

  // Check Ollama status and load models on mount
  useEffect(() => {
    checkOllamaStatus();
  }, []);

  // Scroll to bottom when messages change
  useEffect(() => {
    chatEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [messages, streamingContent]);

  const checkOllamaStatus = async () => {
    setOllamaStatus("checking");
    try {
      const isConnected = await invoke<boolean>("check_ollama_status");
      if (isConnected) {
        setOllamaStatus("connected");
        loadModels();
      } else {
        setOllamaStatus("disconnected");
      }
    } catch {
      setOllamaStatus("disconnected");
    }
  };

  const loadModels = async () => {
    try {
      const modelList = await invoke<OllamaModel[]>("list_ollama_models");
      setModels(modelList);
      if (modelList.length > 0 && !selectedModel) {
        setSelectedModel(modelList[0].name);
      }
    } catch {
      // Failed to load models
    }
  };

  const sendMessage = useCallback(async () => {
    if (!input.trim() || !selectedModel || isLoading) return;

    const userMessage: ChatMessage = { role: "user", content: input };
    setMessages((prev) => [...prev, userMessage]);
    setInput("");
    setIsLoading(true);
    setStreamingContent("");

    const requestId = `${Date.now()}`;

    // Set up streaming listener
    unlistenRef.current = await listen<AIChatResponse>(
      `ollama-stream-${requestId}`,
      (event) => {
        const { content, done, error } = event.payload;

        if (error) {
          setMessages((prev) => [
            ...prev,
            { role: "assistant", content: `Error: ${error}` },
          ]);
          setIsLoading(false);
          return;
        }

        if (content) {
          setStreamingContent((prev) => prev + content);
        }

        if (done) {
          setStreamingContent((prev) => {
            if (prev) {
              setMessages((msgs) => [
                ...msgs,
                { role: "assistant", content: prev },
              ]);
            }
            return "";
          });
          setIsLoading(false);
        }
      }
    );

    try {
      const chatHistory = messages.map((m) => ({
        role: m.role,
        content: m.content,
      }));
      chatHistory.push({ role: "user", content: input });

      await invoke("chat_with_ollama_stream", {
        model: selectedModel,
        messages: chatHistory,
        requestId,
      });
    } catch {
      setMessages((prev) => [
        ...prev,
        { role: "assistant", content: "Failed to send message. Is Ollama running?" },
      ]);
      setIsLoading(false);
    }

    return () => {
      if (unlistenRef.current) {
        unlistenRef.current();
      }
    };
  }, [input, selectedModel, isLoading, messages]);

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      sendMessage();
    }
  };

  const suggestImprovements = async (type: string) => {
    if (!selectedText || !selectedModel) return;

    setIsLoading(true);
    try {
      const result = await invoke<AIChatResponse>("suggest_improvements", {
        model: selectedModel,
        content: selectedText,
        suggestionType: type,
      });

      if (result.error) {
        setMessages((prev) => [
          ...prev,
          { role: "assistant", content: `Error: ${result.error}` },
        ]);
      } else {
        setMessages((prev) => [
          ...prev,
          { role: "user", content: `Suggest ${type} improvements for selected text` },
          { role: "assistant", content: result.content },
        ]);
      }
    } catch {
      setMessages((prev) => [
        ...prev,
        { role: "assistant", content: "Failed to get suggestions. Is Ollama running?" },
      ]);
    }
    setIsLoading(false);
  };

  const clearChat = () => {
    setMessages([]);
    setStreamingContent("");
  };

  return (
    <div className="ai-assistant">
      <div className="ai-header">
        <h3>AI Assistant</h3>
        <div className={`status-indicator ${ollamaStatus}`}>
          {ollamaStatus === "checking" && "Checking..."}
          {ollamaStatus === "connected" && "Connected"}
          {ollamaStatus === "disconnected" && "Disconnected"}
        </div>
      </div>

      {ollamaStatus === "disconnected" ? (
        <div className="ai-disconnected">
          <p>Ollama is not running.</p>
          <p className="hint">
            Start Ollama with: <code>ollama serve</code>
          </p>
          <button onClick={checkOllamaStatus}>Retry Connection</button>
        </div>
      ) : (
        <>
          <div className="model-selector">
            <label>Model:</label>
            <select
              value={selectedModel}
              onChange={(e) => setSelectedModel(e.target.value)}
              disabled={isLoading || models.length === 0}
            >
              {models.length === 0 ? (
                <option>No models available</option>
              ) : (
                models.map((m) => (
                  <option key={m.name} value={m.name}>
                    {m.name}
                  </option>
                ))
              )}
            </select>
          </div>

          {selectedText && (
            <div className="quick-actions">
              <span className="action-label">Quick Actions:</span>
              <button onClick={() => suggestImprovements("grammar")}>
                Grammar
              </button>
              <button onClick={() => suggestImprovements("style")}>
                Style
              </button>
              <button onClick={() => suggestImprovements("rfp")}>
                RFP
              </button>
            </div>
          )}

          <div className="chat-messages">
            {messages.length === 0 && !streamingContent ? (
              <div className="empty-chat">
                <p>Ask me anything about your document.</p>
                <p className="hint">
                  I can help with grammar, style, RFP analysis, and more.
                </p>
              </div>
            ) : (
              <>
                {messages.map((msg, idx) => (
                  <div key={idx} className={`message ${msg.role}`}>
                    <div className="message-content">{msg.content}</div>
                    {msg.role === "assistant" && onInsertText && (
                      <button
                        className="insert-btn"
                        onClick={() => onInsertText(msg.content)}
                        title="Insert into document"
                      >
                        Insert
                      </button>
                    )}
                  </div>
                ))}
                {streamingContent && (
                  <div className="message assistant streaming">
                    <div className="message-content">{streamingContent}</div>
                  </div>
                )}
              </>
            )}
            <div ref={chatEndRef} />
          </div>

          <div className="chat-input-area">
            <textarea
              value={input}
              onChange={(e) => setInput(e.target.value)}
              onKeyDown={handleKeyDown}
              placeholder="Type a message..."
              disabled={isLoading || ollamaStatus !== "connected"}
              rows={2}
            />
            <div className="input-actions">
              <button onClick={clearChat} className="clear-btn" title="Clear chat">
                Clear
              </button>
              <button
                onClick={sendMessage}
                disabled={!input.trim() || isLoading || ollamaStatus !== "connected"}
                className="send-btn"
              >
                {isLoading ? "..." : "Send"}
              </button>
            </div>
          </div>
        </>
      )}
    </div>
  );
}

export default AIAssistant;
