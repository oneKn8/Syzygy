import { Handle, Position, NodeProps } from "@xyflow/react";
import "./nodes.css";

interface ConditionNodeData {
  label: string;
  condition: string;
}

export function ConditionNode({ data, selected }: NodeProps) {
  const nodeData = data as unknown as ConditionNodeData;

  return (
    <div className={`pipeline-node condition-node ${selected ? "selected" : ""}`}>
      <Handle
        type="target"
        position={Position.Left}
        className="node-handle target-handle"
      />
      <div className="node-header">
        <span className="node-icon">?</span>
        <span className="node-label">{nodeData.label}</span>
      </div>
      <div className="node-content">
        <div className="node-field">
          <label>If:</label>
          <span className="condition-expr">{nodeData.condition || "Not set"}</span>
        </div>
      </div>
      <div className="condition-handles">
        <Handle
          type="source"
          position={Position.Right}
          id="true"
          className="node-handle source-handle true-handle"
          style={{ top: "35%" }}
        />
        <Handle
          type="source"
          position={Position.Right}
          id="false"
          className="node-handle source-handle false-handle"
          style={{ top: "65%" }}
        />
      </div>
      <div className="condition-labels">
        <span className="true-label">True</span>
        <span className="false-label">False</span>
      </div>
    </div>
  );
}

export default ConditionNode;
