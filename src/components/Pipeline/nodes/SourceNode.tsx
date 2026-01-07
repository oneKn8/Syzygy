import { Handle, Position, NodeProps } from "@xyflow/react";
import "./nodes.css";

interface SourceNodeData {
  label: string;
  file: string;
}

export function SourceNode({ data, selected }: NodeProps) {
  const nodeData = data as unknown as SourceNodeData;

  return (
    <div className={`pipeline-node source-node ${selected ? "selected" : ""}`}>
      <div className="node-header">
        <span className="node-icon">F</span>
        <span className="node-label">{nodeData.label}</span>
      </div>
      <div className="node-content">
        <div className="node-field">
          <label>File:</label>
          <span>{nodeData.file || "Not set"}</span>
        </div>
      </div>
      <Handle
        type="source"
        position={Position.Right}
        className="node-handle source-handle"
      />
    </div>
  );
}

export default SourceNode;
