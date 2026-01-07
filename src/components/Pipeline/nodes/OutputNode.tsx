import { Handle, Position, NodeProps } from "@xyflow/react";
import "./nodes.css";

interface OutputNodeData {
  label: string;
  format: string;
}

const FORMATS: Record<string, string> = {
  pdf: "PDF",
  docx: "Word (DOCX)",
  html: "HTML",
  epub: "EPUB",
  md: "Markdown",
};

export function OutputNode({ data, selected }: NodeProps) {
  const nodeData = data as unknown as OutputNodeData;

  return (
    <div className={`pipeline-node output-node ${selected ? "selected" : ""}`}>
      <Handle
        type="target"
        position={Position.Left}
        className="node-handle target-handle"
      />
      <div className="node-header">
        <span className="node-icon">O</span>
        <span className="node-label">{nodeData.label}</span>
      </div>
      <div className="node-content">
        <div className="node-field">
          <label>Format:</label>
          <span className="format-badge">{FORMATS[nodeData.format] || nodeData.format}</span>
        </div>
      </div>
    </div>
  );
}

export default OutputNode;
