import { useState, useCallback, useEffect, useRef } from "react";
import { CodeMirrorEditor } from "./CodeMirrorEditor";
import "./Editor.css";

interface EditorProps {
  initialValue?: string;
  language?: "typst" | "latex" | "markdown" | "plaintext";
  filePath?: string;
  onSave?: (content: string) => void;
  onChange?: (content: string) => void;
}

export function Editor({
  initialValue = "",
  language = "typst",
  filePath,
  onSave,
  onChange,
}: EditorProps) {
  const [content, setContent] = useState(initialValue);
  const [isDirty, setIsDirty] = useState(false);
  const prevInitialValue = useRef(initialValue);

  // Update content when initialValue changes (e.g., when switching files)
  useEffect(() => {
    if (initialValue !== prevInitialValue.current) {
      setContent(initialValue);
      setIsDirty(false);
      prevInitialValue.current = initialValue;
    }
  }, [initialValue]);

  const handleChange = useCallback(
    (value: string) => {
      setContent(value);
      setIsDirty(true);
      onChange?.(value);
    },
    [onChange]
  );

  const handleSave = useCallback(() => {
    if (onSave) {
      onSave(content);
      setIsDirty(false);
    }
  }, [content, onSave]);

  // Handle Ctrl+S keyboard shortcut
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key === "s") {
        e.preventDefault();
        if (isDirty && onSave) {
          onSave(content);
          setIsDirty(false);
        }
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [content, isDirty, onSave]);

  return (
    <div className="editor-container">
      <div className="editor-header">
        <div className="editor-file-info">
          <span className="editor-language">{language.toUpperCase()}</span>
          {filePath && (
            <span className="editor-filepath">
              {filePath}
              {isDirty && <span className="editor-dirty">*</span>}
            </span>
          )}
        </div>
        <div className="editor-actions">
          <button
            className="editor-save-btn"
            onClick={handleSave}
            disabled={!isDirty}
          >
            Save
          </button>
        </div>
      </div>
      <div className="editor-content">
        <CodeMirrorEditor
          value={content}
          onChange={handleChange}
          language={language}
        />
      </div>
    </div>
  );
}

export { CodeMirrorEditor };
export default Editor;
