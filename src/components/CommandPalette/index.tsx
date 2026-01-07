import { useState, useEffect, useCallback, useRef, useMemo } from "react";
import "./CommandPalette.css";

export interface Command {
  id: string;
  title: string;
  description?: string;
  shortcut?: string;
  category?: string;
  action: () => void;
}

interface CommandPaletteProps {
  commands: Command[];
  isOpen: boolean;
  onClose: () => void;
}

export function CommandPalette({ commands, isOpen, onClose }: CommandPaletteProps) {
  const [search, setSearch] = useState("");
  const [selectedIndex, setSelectedIndex] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);
  const listRef = useRef<HTMLDivElement>(null);

  // Filter commands based on search
  const filteredCommands = useMemo(() => {
    if (!search.trim()) return commands;

    const lowerSearch = search.toLowerCase();
    return commands.filter(
      (cmd) =>
        cmd.title.toLowerCase().includes(lowerSearch) ||
        cmd.description?.toLowerCase().includes(lowerSearch) ||
        cmd.category?.toLowerCase().includes(lowerSearch)
    );
  }, [commands, search]);

  // Reset selection when search changes
  useEffect(() => {
    setSelectedIndex(0);
  }, [search]);

  // Focus input when opened
  useEffect(() => {
    if (isOpen) {
      setSearch("");
      setSelectedIndex(0);
      setTimeout(() => inputRef.current?.focus(), 0);
    }
  }, [isOpen]);

  // Scroll selected item into view
  useEffect(() => {
    if (listRef.current && filteredCommands.length > 0) {
      const selectedElement = listRef.current.children[selectedIndex] as HTMLElement;
      if (selectedElement) {
        selectedElement.scrollIntoView({ block: "nearest" });
      }
    }
  }, [selectedIndex, filteredCommands.length]);

  const executeCommand = useCallback(
    (command: Command) => {
      onClose();
      // Small delay to ensure modal closes before action
      setTimeout(() => command.action(), 50);
    },
    [onClose]
  );

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      switch (e.key) {
        case "ArrowDown":
          e.preventDefault();
          setSelectedIndex((i) => Math.min(i + 1, filteredCommands.length - 1));
          break;
        case "ArrowUp":
          e.preventDefault();
          setSelectedIndex((i) => Math.max(i - 1, 0));
          break;
        case "Enter":
          e.preventDefault();
          if (filteredCommands[selectedIndex]) {
            executeCommand(filteredCommands[selectedIndex]);
          }
          break;
        case "Escape":
          e.preventDefault();
          onClose();
          break;
      }
    },
    [filteredCommands, selectedIndex, executeCommand, onClose]
  );

  if (!isOpen) return null;

  return (
    <div className="command-palette-overlay" onClick={onClose}>
      <div className="command-palette" onClick={(e) => e.stopPropagation()}>
        <div className="command-palette-input">
          <input
            ref={inputRef}
            type="text"
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder="Type a command..."
            autoComplete="off"
            spellCheck={false}
          />
        </div>
        <div className="command-palette-list" ref={listRef}>
          {filteredCommands.length === 0 ? (
            <div className="command-palette-empty">No matching commands</div>
          ) : (
            filteredCommands.map((cmd, index) => (
              <div
                key={cmd.id}
                className={`command-item ${index === selectedIndex ? "selected" : ""}`}
                onClick={() => executeCommand(cmd)}
                onMouseEnter={() => setSelectedIndex(index)}
              >
                <div className="command-info">
                  <span className="command-title">{cmd.title}</span>
                  {cmd.description && (
                    <span className="command-description">{cmd.description}</span>
                  )}
                </div>
                {cmd.shortcut && (
                  <span className="command-shortcut">{cmd.shortcut}</span>
                )}
              </div>
            ))
          )}
        </div>
        <div className="command-palette-footer">
          <span>
            <kbd>Enter</kbd> to run
          </span>
          <span>
            <kbd>Esc</kbd> to close
          </span>
        </div>
      </div>
    </div>
  );
}

// Hook for managing global keyboard shortcuts
export function useKeyboardShortcuts(shortcuts: Record<string, () => void>) {
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      const key = [];

      if (e.ctrlKey || e.metaKey) key.push("Ctrl");
      if (e.altKey) key.push("Alt");
      if (e.shiftKey) key.push("Shift");

      // Normalize key name
      let keyName = e.key;
      if (keyName === " ") keyName = "Space";
      if (keyName.length === 1) keyName = keyName.toUpperCase();

      key.push(keyName);
      const combo = key.join("+");

      if (shortcuts[combo]) {
        e.preventDefault();
        shortcuts[combo]();
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [shortcuts]);
}

export default CommandPalette;
