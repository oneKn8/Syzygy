import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./FileTree.css";

interface FileEntry {
  name: string;
  path: string;
  is_dir: boolean;
  extension: string | null;
}

interface FileTreeProps {
  rootPath: string;
  onFileSelect?: (path: string) => void;
  selectedPath?: string;
}

interface FileNodeProps {
  entry: FileEntry;
  depth: number;
  onSelect: (path: string) => void;
  selectedPath?: string;
}

function FileNode({ entry, depth, onSelect, selectedPath }: FileNodeProps) {
  const [isExpanded, setIsExpanded] = useState(false);
  const [children, setChildren] = useState<FileEntry[]>([]);
  const [isLoading, setIsLoading] = useState(false);

  const isSelected = selectedPath === entry.path;

  const handleClick = useCallback(async () => {
    if (entry.is_dir) {
      if (!isExpanded && children.length === 0) {
        setIsLoading(true);
        try {
          const entries = await invoke<FileEntry[]>("read_directory", {
            path: entry.path,
          });
          setChildren(entries);
        } catch (err) {
          // Failed to read directory, ignore for now
        }
        setIsLoading(false);
      }
      setIsExpanded(!isExpanded);
    } else {
      onSelect(entry.path);
    }
  }, [entry, isExpanded, children.length, onSelect]);

  const getFileIcon = () => {
    if (entry.is_dir) {
      return isExpanded ? "[-]" : "[+]";
    }
    switch (entry.extension) {
      case "typ":
        return "T";
      case "tex":
        return "L";
      case "pdf":
        return "P";
      case "md":
        return "M";
      case "json":
        return "J";
      default:
        return "*";
    }
  };

  return (
    <div className="file-node">
      <div
        className={`file-node-row ${isSelected ? "selected" : ""} ${
          entry.is_dir ? "directory" : "file"
        }`}
        style={{ paddingLeft: `${depth * 16 + 8}px` }}
        onClick={handleClick}
      >
        <span className="file-icon">{getFileIcon()}</span>
        <span className="file-name">{entry.name}</span>
        {isLoading && <span className="loading">...</span>}
      </div>
      {isExpanded && children.length > 0 && (
        <div className="file-children">
          {children.map((child) => (
            <FileNode
              key={child.path}
              entry={child}
              depth={depth + 1}
              onSelect={onSelect}
              selectedPath={selectedPath}
            />
          ))}
        </div>
      )}
    </div>
  );
}

export function FileTree({ rootPath, onFileSelect, selectedPath }: FileTreeProps) {
  const [entries, setEntries] = useState<FileEntry[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    async function loadDirectory() {
      setIsLoading(true);
      setError(null);
      try {
        const result = await invoke<FileEntry[]>("read_directory", {
          path: rootPath,
        });
        setEntries(result);
      } catch (err) {
        setError(String(err));
      }
      setIsLoading(false);
    }

    if (rootPath) {
      loadDirectory();
    }
  }, [rootPath]);

  const handleSelect = useCallback(
    (path: string) => {
      if (onFileSelect) {
        onFileSelect(path);
      }
    },
    [onFileSelect]
  );

  if (isLoading) {
    return <div className="file-tree-loading">Loading...</div>;
  }

  if (error) {
    return <div className="file-tree-error">{error}</div>;
  }

  if (entries.length === 0) {
    return <div className="file-tree-empty">No files</div>;
  }

  return (
    <div className="file-tree">
      {entries.map((entry) => (
        <FileNode
          key={entry.path}
          entry={entry}
          depth={0}
          onSelect={handleSelect}
          selectedPath={selectedPath}
        />
      ))}
    </div>
  );
}

export default FileTree;
