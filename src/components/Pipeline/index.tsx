import { useCallback, useState, useMemo } from "react";
import {
  ReactFlow,
  MiniMap,
  Controls,
  Background,
  useNodesState,
  useEdgesState,
  addEdge,
  Connection,
  Edge,
  Node,
  NodeTypes,
  BackgroundVariant,
  Panel,
} from "@xyflow/react";
import "@xyflow/react/dist/style.css";
import "./Pipeline.css";
import { SourceNode } from "./nodes/SourceNode";
import { TransformNode } from "./nodes/TransformNode";
import { OutputNode } from "./nodes/OutputNode";
import { ConditionNode } from "./nodes/ConditionNode";

// Define custom node types
const nodeTypes: NodeTypes = {
  source: SourceNode,
  transform: TransformNode,
  output: OutputNode,
  condition: ConditionNode,
};

// Initial nodes for demo
const initialNodes: Node[] = [
  {
    id: "1",
    type: "source",
    data: { label: "Main Document", file: "main.typ" },
    position: { x: 50, y: 100 },
  },
  {
    id: "2",
    type: "source",
    data: { label: "Appendix", file: "appendix.typ" },
    position: { x: 50, y: 250 },
  },
  {
    id: "3",
    type: "transform",
    data: { label: "Merge Documents", operation: "merge" },
    position: { x: 300, y: 150 },
  },
  {
    id: "4",
    type: "condition",
    data: { label: "Client Type", condition: "client == 'government'" },
    position: { x: 550, y: 150 },
  },
  {
    id: "5",
    type: "output",
    data: { label: "PDF Output", format: "pdf" },
    position: { x: 800, y: 100 },
  },
  {
    id: "6",
    type: "output",
    data: { label: "DOCX Output", format: "docx" },
    position: { x: 800, y: 250 },
  },
];

// Initial edges
const initialEdges: Edge[] = [
  { id: "e1-3", source: "1", target: "3", animated: true },
  { id: "e2-3", source: "2", target: "3", animated: true },
  { id: "e3-4", source: "3", target: "4", animated: true },
  { id: "e4-5", source: "4", target: "5", sourceHandle: "true", animated: true },
  { id: "e4-6", source: "4", target: "6", sourceHandle: "false", animated: true },
];

interface PipelineProps {
  onNodeSelect?: (node: Node | null) => void;
  onRunPipeline?: () => void;
}

export function Pipeline({ onNodeSelect, onRunPipeline }: PipelineProps) {
  const [nodes, setNodes, onNodesChange] = useNodesState(initialNodes);
  const [edges, setEdges, onEdgesChange] = useEdgesState(initialEdges);
  const [selectedNode, setSelectedNode] = useState<Node | null>(null);

  const onConnect = useCallback(
    (params: Connection) => setEdges((eds) => addEdge({ ...params, animated: true }, eds)),
    [setEdges]
  );

  const onNodeClick = useCallback(
    (_event: React.MouseEvent, node: Node) => {
      setSelectedNode(node);
      onNodeSelect?.(node);
    },
    [onNodeSelect]
  );

  const onPaneClick = useCallback(() => {
    setSelectedNode(null);
    onNodeSelect?.(null);
  }, [onNodeSelect]);

  const addNode = useCallback(
    (type: string) => {
      const newNode: Node = {
        id: `node-${Date.now()}`,
        type,
        data: getDefaultNodeData(type),
        position: { x: Math.random() * 400 + 100, y: Math.random() * 300 + 100 },
      };
      setNodes((nds) => [...nds, newNode]);
    },
    [setNodes]
  );

  const deleteSelected = useCallback(() => {
    if (selectedNode) {
      setNodes((nds) => nds.filter((n) => n.id !== selectedNode.id));
      setEdges((eds) =>
        eds.filter((e) => e.source !== selectedNode.id && e.target !== selectedNode.id)
      );
      setSelectedNode(null);
    }
  }, [selectedNode, setNodes, setEdges]);

  // Memoize the flow styles
  const flowStyles = useMemo(
    () => ({
      width: "100%",
      height: "100%",
      background: "var(--bg-tertiary)",
    }),
    []
  );

  return (
    <div className="pipeline-container">
      <div className="pipeline-toolbar">
        <div className="toolbar-group">
          <span className="toolbar-label">Add Node:</span>
          <button onClick={() => addNode("source")} title="Add Source Node">
            Source
          </button>
          <button onClick={() => addNode("transform")} title="Add Transform Node">
            Transform
          </button>
          <button onClick={() => addNode("condition")} title="Add Condition Node">
            Condition
          </button>
          <button onClick={() => addNode("output")} title="Add Output Node">
            Output
          </button>
        </div>
        <div className="toolbar-group">
          <button
            onClick={deleteSelected}
            disabled={!selectedNode}
            className="delete-btn"
          >
            Delete
          </button>
          <button onClick={onRunPipeline} className="run-btn">
            Run Pipeline
          </button>
        </div>
      </div>
      <div className="pipeline-canvas">
        <ReactFlow
          nodes={nodes}
          edges={edges}
          onNodesChange={onNodesChange}
          onEdgesChange={onEdgesChange}
          onConnect={onConnect}
          onNodeClick={onNodeClick}
          onPaneClick={onPaneClick}
          nodeTypes={nodeTypes}
          fitView
          style={flowStyles}
          proOptions={{ hideAttribution: true }}
        >
          <Controls />
          <MiniMap
            nodeColor={(node) => {
              switch (node.type) {
                case "source":
                  return "#4caf50";
                case "transform":
                  return "#2196f3";
                case "condition":
                  return "#ff9800";
                case "output":
                  return "#9c27b0";
                default:
                  return "#666";
              }
            }}
          />
          <Background variant={BackgroundVariant.Dots} gap={12} size={1} />
          <Panel position="bottom-right" className="pipeline-info">
            {nodes.length} nodes | {edges.length} connections
          </Panel>
        </ReactFlow>
      </div>
    </div>
  );
}

function getDefaultNodeData(type: string): Record<string, string> {
  switch (type) {
    case "source":
      return { label: "New Source", file: "" };
    case "transform":
      return { label: "Transform", operation: "merge" };
    case "condition":
      return { label: "Condition", condition: "" };
    case "output":
      return { label: "Output", format: "pdf" };
    default:
      return { label: "Node" };
  }
}

export default Pipeline;
