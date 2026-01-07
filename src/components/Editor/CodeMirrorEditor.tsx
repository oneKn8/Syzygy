import { useEffect, useRef, useCallback } from "react";
import { EditorState } from "@codemirror/state";
import { EditorView, keymap, lineNumbers, highlightActiveLine, highlightSpecialChars, drawSelection, dropCursor, rectangularSelection, crosshairCursor, highlightActiveLineGutter } from "@codemirror/view";
import { defaultKeymap, history, historyKeymap, indentWithTab } from "@codemirror/commands";
import { syntaxHighlighting, defaultHighlightStyle, bracketMatching, foldGutter, indentOnInput, foldKeymap } from "@codemirror/language";
import { searchKeymap, highlightSelectionMatches } from "@codemirror/search";
import { autocompletion, completionKeymap, closeBrackets, closeBracketsKeymap } from "@codemirror/autocomplete";
import { lintKeymap } from "@codemirror/lint";
import { markdown } from "@codemirror/lang-markdown";
import { oneDark } from "@codemirror/theme-one-dark";

interface CodeMirrorEditorProps {
  value: string;
  onChange?: (value: string) => void;
  language?: "typst" | "latex" | "markdown" | "plaintext";
  readOnly?: boolean;
  className?: string;
}

const getLanguageExtension = (language: string) => {
  switch (language) {
    case "markdown":
      return markdown();
    case "typst":
    case "latex":
    default:
      return markdown();
  }
};

export function CodeMirrorEditor({
  value,
  onChange,
  language = "typst",
  readOnly = false,
  className = "",
}: CodeMirrorEditorProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const viewRef = useRef<EditorView | null>(null);
  const onChangeRef = useRef(onChange);

  onChangeRef.current = onChange;

  const handleChange = useCallback(() => {
    if (viewRef.current && onChangeRef.current) {
      const doc = viewRef.current.state.doc.toString();
      onChangeRef.current(doc);
    }
  }, []);

  useEffect(() => {
    if (!containerRef.current) return;

    const updateListener = EditorView.updateListener.of((update) => {
      if (update.docChanged) {
        handleChange();
      }
    });

    const state = EditorState.create({
      doc: value,
      extensions: [
        lineNumbers(),
        highlightActiveLineGutter(),
        highlightSpecialChars(),
        history(),
        foldGutter(),
        drawSelection(),
        dropCursor(),
        EditorState.allowMultipleSelections.of(true),
        indentOnInput(),
        syntaxHighlighting(defaultHighlightStyle, { fallback: true }),
        bracketMatching(),
        closeBrackets(),
        autocompletion(),
        rectangularSelection(),
        crosshairCursor(),
        highlightActiveLine(),
        highlightSelectionMatches(),
        keymap.of([
          ...closeBracketsKeymap,
          ...defaultKeymap,
          ...searchKeymap,
          ...historyKeymap,
          ...foldKeymap,
          ...completionKeymap,
          ...lintKeymap,
          indentWithTab,
        ]),
        getLanguageExtension(language),
        oneDark,
        updateListener,
        EditorView.lineWrapping,
        EditorState.readOnly.of(readOnly),
        EditorView.theme({
          "&": {
            height: "100%",
            fontSize: "14px",
          },
          ".cm-scroller": {
            fontFamily: "'JetBrains Mono', 'Fira Code', 'Consolas', monospace",
          },
          ".cm-content": {
            padding: "10px 0",
          },
          ".cm-gutters": {
            backgroundColor: "#1a1a2e",
            borderRight: "1px solid #2a2a4a",
          },
        }),
      ],
    });

    const view = new EditorView({
      state,
      parent: containerRef.current,
    });

    viewRef.current = view;

    return () => {
      view.destroy();
      viewRef.current = null;
    };
  }, [language, readOnly, handleChange]);

  useEffect(() => {
    if (viewRef.current) {
      const currentValue = viewRef.current.state.doc.toString();
      if (currentValue !== value) {
        viewRef.current.dispatch({
          changes: {
            from: 0,
            to: currentValue.length,
            insert: value,
          },
        });
      }
    }
  }, [value]);

  return (
    <div
      ref={containerRef}
      className={`codemirror-container ${className}`}
      style={{ height: "100%", overflow: "hidden" }}
    />
  );
}

export default CodeMirrorEditor;
