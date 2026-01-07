import { Handle, Position, NodeProps } from "@xyflow/react";
import "./nodes.css";

interface TransformNodeData {
  label: string;
  operation: string;
}

const OPERATIONS: Record<string, string> = {
  merge: "Merge Documents",
  split: "Split Pages",
  inject: "Inject Variables",
  filter: "Filter Content",
  template: "Apply Template",
};

export function TransformNode({ data, selected }: NodeProps) {
  const nodeData = data as unknown as TransformNodeData;

  return (
    <div className={`pipeline-node transform-node ${selected ? "selected" : ""}`}>
      <Handle
        type="target"
        position={Position.Left}
        className="node-handle target-handle"
      />
      <div className="node-header">
        <span className="node-icon">T</span>
        <span className="node-label">{nodeData.label}</span>
      </div>
      <div className="node-content">
        <div className="node-field">
          <label>Operation:</label>
          <span>{OPERATIONS[nodeData.operation] || nodeData.operation}</span>
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

export default TransformNode;
