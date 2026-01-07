import { useState, useCallback, useRef, useEffect } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";
import {
  Panel,
  Group as PanelGroup,
  Separator as PanelResizeHandle,
} from "react-resizable-panels";
import { Editor } from "./components/Editor";
import { FileTree } from "./components/FileTree";
import { Preview } from "./components/Preview";

const SAMPLE_TYPST = `#set document(title: "Sample Document")
#set page(paper: "us-letter", margin: 1in)
#set text(font: "New Computer Modern", size: 11pt)

= Welcome to RFPMaker

This is a sample Typst document. Start editing to see the power of RFPMaker!

== Features

- *Live Preview*: See your changes instantly
- *AI Assistant*: Get help writing and editing
- *Visual Pipelines*: Connect documents visually
- *Multi-format Export*: PDF, DOCX, HTML, and more

== Getting Started

1. Open a project folder
2. Create or edit .typ files
3. Watch the preview update in real-time

#lorem(50)
`;

interface FileContent {
  path: string;
  content: string;
  language: string;
}

interface CompileResult {
  success: boolean;
  errors: Array<{ message: string }>;
  warnings: Array<{ message: string }>;
  compile_time_ms: number;
}

function App() {
  const [projectPath, setProjectPath] = useState<string | null>(null);
  const [currentFile, setCurrentFile] = useState<FileContent | null>(null);
  const [editorContent, setEditorContent] = useState(SAMPLE_TYPST);
  const [compileStatus, setCompileStatus] = useState<{
    errors: number;
    warnings: number;
    time: number;
  } | null>(null);
  const editorContentRef = useRef(editorContent);

  // Keep ref in sync with state
  useEffect(() => {
    editorContentRef.current = editorContent;
  }, [editorContent]);

  const handleOpenProject = useCallback(async () => {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: "Open Project Folder",
      });
      if (selected && typeof selected === "string") {
        setProjectPath(selected);
      }
    } catch (_err) {
      // User cancelled
    }
  }, []);

  const handleFileSelect = useCallback(async (path: string) => {
    try {
      const result = await invoke<FileContent>("read_file", { path });
      setCurrentFile(result);
      setEditorContent(result.content);
    } catch (_err) {
      // Failed to read file
    }
  }, []);

  const handleSave = useCallback(
    async (content: string) => {
      if (currentFile) {
        try {
          await invoke("write_file", { path: currentFile.path, content });
        } catch (_err) {
          // Failed to save
        }
      }
    },
    [currentFile]
  );

  const handleEditorChange = useCallback((content: string) => {
    setEditorContent(content);
  }, []);

  const handleCompileResult = useCallback((result: CompileResult) => {
    setCompileStatus({
      errors: result.errors.length,
      warnings: result.warnings.length,
      time: result.compile_time_ms,
    });
  }, []);

  return (
    <div className="app">
      <header className="app-header">
        <h1>RFPMaker</h1>
        <p>Premium Document Engineering IDE</p>
        <div className="header-status">
          {compileStatus && (
            <span className={`compile-status ${compileStatus.errors > 0 ? "error" : "success"}`}>
              {compileStatus.errors > 0
                ? `${compileStatus.errors} error${compileStatus.errors > 1 ? "s" : ""}`
                : "OK"}{" "}
              | {compileStatus.time}ms
            </span>
          )}
        </div>
        <button className="open-project-btn" onClick={handleOpenProject}>
          Open Project
        </button>
      </header>
      <main className="app-main">
        <PanelGroup orientation="horizontal">
          {/* Sidebar */}
          <Panel defaultSize={15} minSize={10} maxSize={30}>
            <aside className="sidebar">
              <div className="sidebar-section">
                <h3>Project</h3>
                {projectPath ? (
                  <FileTree
                    rootPath={projectPath}
                    onFileSelect={handleFileSelect}
                    selectedPath={currentFile?.path}
                  />
                ) : (
                  <p className="placeholder">
                    Click "Open Project" to select a folder
                  </p>
                )}
              </div>
            </aside>
          </Panel>

          <PanelResizeHandle className="resize-handle" />

          {/* Editor */}
          <Panel defaultSize={40} minSize={20}>
            <div className="editor-pane">
              <Editor
                initialValue={editorContent}
                language={
                  (currentFile?.language as "typst" | "latex" | "markdown") ||
                  "typst"
                }
                filePath={currentFile?.path}
                onSave={handleSave}
                onChange={handleEditorChange}
              />
            </div>
          </Panel>

          <PanelResizeHandle className="resize-handle" />

          {/* Preview */}
          <Panel defaultSize={30} minSize={20}>
            <div className="preview-pane">
              <Preview
                source={editorContent}
                rootPath={projectPath || "."}
                onCompileResult={handleCompileResult}
              />
            </div>
          </Panel>

          <PanelResizeHandle className="resize-handle" />

          {/* AI Panel */}
          <Panel defaultSize={15} minSize={10} maxSize={30}>
            <aside className="ai-panel">
              <h3>AI Assistant</h3>
              <div className="ai-chat-placeholder">
                <p className="placeholder">
                  AI chat assistant coming soon...
                </p>
                <p className="placeholder hint">
                  Will support: grammar checking, content suggestions, RFP analysis
                </p>
              </div>
            </aside>
          </Panel>
        </PanelGroup>
      </main>
    </div>
  );
}

export default App;
