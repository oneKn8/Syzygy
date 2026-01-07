//! Pipeline Execution Engine
//!
//! Executes document processing pipelines defined as DAGs (Directed Acyclic Graphs).
//! Supports parallel execution where possible and handles dependencies correctly.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::PathBuf;
use tokio::sync::mpsc;
use uuid::Uuid;

/// A node in the pipeline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineNode {
    pub id: String,
    pub node_type: NodeType,
    pub data: NodeData,
    pub position: Position,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NodeType {
    Source,
    Transform,
    Output,
    Condition,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeData {
    pub label: String,
    // Source node fields
    pub source_type: Option<String>,
    pub path: Option<String>,
    // Transform node fields
    pub operation: Option<String>,
    pub parameters: Option<serde_json::Value>,
    // Output node fields
    pub format: Option<String>,
    pub output_path: Option<String>,
    // Condition node fields
    pub condition: Option<String>,
}

/// An edge connecting two nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineEdge {
    pub id: String,
    pub source: String,
    pub target: String,
    pub source_handle: Option<String>,
    pub target_handle: Option<String>,
}

/// A complete pipeline definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pipeline {
    pub id: String,
    pub name: String,
    pub nodes: Vec<PipelineNode>,
    pub edges: Vec<PipelineEdge>,
    pub variables: Option<HashMap<String, serde_json::Value>>,
}

/// Result of executing a single node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeResult {
    pub node_id: String,
    pub status: ExecutionStatus,
    pub output: Option<NodeOutput>,
    pub error: Option<String>,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExecutionStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Skipped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeOutput {
    pub content: Option<String>,
    pub path: Option<String>,
    pub format: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

/// Result of executing the entire pipeline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineResult {
    pub pipeline_id: String,
    pub status: ExecutionStatus,
    pub node_results: Vec<NodeResult>,
    pub outputs: Vec<PipelineOutput>,
    pub total_duration_ms: u64,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineOutput {
    pub node_id: String,
    pub format: String,
    pub path: Option<String>,
    pub content: Option<Vec<u8>>,
}

/// Progress update during pipeline execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineProgress {
    pub pipeline_id: String,
    pub current_node: String,
    pub completed_nodes: usize,
    pub total_nodes: usize,
    pub status: ExecutionStatus,
    pub message: String,
}

/// Pipeline execution context
struct ExecutionContext {
    pipeline: Pipeline,
    node_outputs: HashMap<String, NodeOutput>,
    variables: HashMap<String, serde_json::Value>,
    root_path: Option<PathBuf>,
}

impl ExecutionContext {
    fn new(pipeline: Pipeline, root_path: Option<PathBuf>) -> Self {
        let variables = pipeline.variables.clone().unwrap_or_default();
        Self {
            pipeline,
            node_outputs: HashMap::new(),
            variables,
            root_path,
        }
    }

    fn get_input(&self, node_id: &str) -> Option<&NodeOutput> {
        // Find the edge that leads to this node
        for edge in &self.pipeline.edges {
            if edge.target == node_id {
                return self.node_outputs.get(&edge.source);
            }
        }
        None
    }

    fn get_inputs(&self, node_id: &str) -> Vec<&NodeOutput> {
        let mut inputs = Vec::new();
        for edge in &self.pipeline.edges {
            if edge.target == node_id {
                if let Some(output) = self.node_outputs.get(&edge.source) {
                    inputs.push(output);
                }
            }
        }
        inputs
    }

    fn set_output(&mut self, node_id: &str, output: NodeOutput) {
        self.node_outputs.insert(node_id.to_string(), output);
    }

    fn resolve_path(&self, path: &str) -> PathBuf {
        let path = PathBuf::from(path);
        if path.is_absolute() {
            path
        } else if let Some(ref root) = self.root_path {
            root.join(path)
        } else {
            path
        }
    }
}

/// Build execution order using topological sort
fn build_execution_order(pipeline: &Pipeline) -> Result<Vec<String>, String> {
    let mut in_degree: HashMap<String, usize> = HashMap::new();
    let mut adjacency: HashMap<String, Vec<String>> = HashMap::new();

    // Initialize
    for node in &pipeline.nodes {
        in_degree.insert(node.id.clone(), 0);
        adjacency.insert(node.id.clone(), Vec::new());
    }

    // Build graph
    for edge in &pipeline.edges {
        *in_degree.get_mut(&edge.target).unwrap() += 1;
        adjacency.get_mut(&edge.source).unwrap().push(edge.target.clone());
    }

    // Kahn's algorithm for topological sort
    let mut queue: VecDeque<String> = VecDeque::new();
    for (node_id, &degree) in &in_degree {
        if degree == 0 {
            queue.push_back(node_id.clone());
        }
    }

    let mut order = Vec::new();
    while let Some(node_id) = queue.pop_front() {
        order.push(node_id.clone());

        if let Some(neighbors) = adjacency.get(&node_id) {
            for neighbor in neighbors {
                let degree = in_degree.get_mut(neighbor).unwrap();
                *degree -= 1;
                if *degree == 0 {
                    queue.push_back(neighbor.clone());
                }
            }
        }
    }

    if order.len() != pipeline.nodes.len() {
        return Err("Pipeline contains a cycle".to_string());
    }

    Ok(order)
}

/// Execute a source node
async fn execute_source_node(
    node: &PipelineNode,
    ctx: &ExecutionContext,
) -> Result<NodeOutput, String> {
    let source_type = node.data.source_type.as_deref().unwrap_or("file");

    match source_type {
        "file" => {
            let path = node.data.path.as_ref().ok_or("Source path not specified")?;
            let resolved_path = ctx.resolve_path(path);

            let content = tokio::fs::read_to_string(&resolved_path)
                .await
                .map_err(|e| format!("Failed to read file: {}", e))?;

            let format = resolved_path
                .extension()
                .and_then(|e| e.to_str())
                .map(String::from);

            Ok(NodeOutput {
                content: Some(content),
                path: Some(resolved_path.to_string_lossy().to_string()),
                format,
                metadata: None,
            })
        }
        "folder" => {
            let path = node.data.path.as_ref().ok_or("Folder path not specified")?;
            let resolved_path = ctx.resolve_path(path);

            let mut files = Vec::new();
            let mut entries = tokio::fs::read_dir(&resolved_path)
                .await
                .map_err(|e| format!("Failed to read directory: {}", e))?;

            while let Some(entry) = entries.next_entry().await.map_err(|e| e.to_string())? {
                let path = entry.path();
                if path.is_file() {
                    files.push(path.to_string_lossy().to_string());
                }
            }

            Ok(NodeOutput {
                content: None,
                path: Some(resolved_path.to_string_lossy().to_string()),
                format: Some("folder".to_string()),
                metadata: Some(serde_json::json!({ "files": files })),
            })
        }
        "variable" => {
            let var_name = node.data.path.as_ref().ok_or("Variable name not specified")?;
            let value = ctx.variables.get(var_name)
                .ok_or(format!("Variable '{}' not found", var_name))?;

            Ok(NodeOutput {
                content: Some(serde_json::to_string(value).unwrap_or_default()),
                path: None,
                format: Some("json".to_string()),
                metadata: None,
            })
        }
        _ => Err(format!("Unknown source type: {}", source_type)),
    }
}

/// Execute a transform node
async fn execute_transform_node(
    node: &PipelineNode,
    ctx: &ExecutionContext,
) -> Result<NodeOutput, String> {
    let operation = node.data.operation.as_deref().unwrap_or("passthrough");
    let inputs = ctx.get_inputs(&node.id);

    if inputs.is_empty() {
        return Err("Transform node has no inputs".to_string());
    }

    let first_format = inputs.first().and_then(|i| i.format.clone());

    match operation {
        "merge" => {
            // Merge multiple inputs into one
            let mut merged_content = String::new();
            for input in &inputs {
                if let Some(ref content) = input.content {
                    merged_content.push_str(content);
                    merged_content.push_str("\n\n");
                }
            }

            Ok(NodeOutput {
                content: Some(merged_content),
                path: None,
                format: first_format,
                metadata: None,
            })
        }
        "inject" => {
            // Inject variables into content
            let input = inputs.first().ok_or("No input for inject operation")?;
            let mut content = input.content.clone().unwrap_or_default();

            // Replace {{variable}} placeholders
            for (key, value) in &ctx.variables {
                let placeholder = format!("{{{{{}}}}}", key);
                let replacement = match value {
                    serde_json::Value::String(s) => s.clone(),
                    _ => serde_json::to_string(value).unwrap_or_default(),
                };
                content = content.replace(&placeholder, &replacement);
            }

            Ok(NodeOutput {
                content: Some(content),
                path: input.path.clone(),
                format: input.format.clone(),
                metadata: None,
            })
        }
        "filter" => {
            // Filter content based on parameters
            let input = inputs.first().ok_or("No input for filter operation")?;
            let content = input.content.clone().unwrap_or_default();

            // Simple line filter based on pattern
            let pattern = node.data.parameters
                .as_ref()
                .and_then(|p| p.get("pattern"))
                .and_then(|v| v.as_str())
                .unwrap_or("");

            let filtered: String = content
                .lines()
                .filter(|line| line.contains(pattern))
                .collect::<Vec<_>>()
                .join("\n");

            Ok(NodeOutput {
                content: Some(filtered),
                path: input.path.clone(),
                format: input.format.clone(),
                metadata: None,
            })
        }
        "template" => {
            // Apply a template transformation
            let input = inputs.first().ok_or("No input for template operation")?;
            let content = input.content.clone().unwrap_or_default();

            // Get template from parameters
            let template = node.data.parameters
                .as_ref()
                .and_then(|p| p.get("template"))
                .and_then(|v| v.as_str())
                .unwrap_or("{{content}}");

            let result = template.replace("{{content}}", &content);

            Ok(NodeOutput {
                content: Some(result),
                path: input.path.clone(),
                format: input.format.clone(),
                metadata: None,
            })
        }
        "split" => {
            // Split content by delimiter
            let input = inputs.first().ok_or("No input for split operation")?;
            let content = input.content.clone().unwrap_or_default();

            let delimiter = node.data.parameters
                .as_ref()
                .and_then(|p| p.get("delimiter"))
                .and_then(|v| v.as_str())
                .unwrap_or("\n---\n");

            let parts: Vec<String> = content.split(delimiter).map(String::from).collect();
            let parts_count = parts.len();

            Ok(NodeOutput {
                content: Some(content),
                path: input.path.clone(),
                format: input.format.clone(),
                metadata: Some(serde_json::json!({
                    "parts": parts,
                    "count": parts_count
                })),
            })
        }
        "passthrough" | _ => {
            // Pass through first input unchanged
            let input = inputs.first().ok_or("No input")?;
            Ok((*input).clone())
        }
    }
}

/// Execute an output node
async fn execute_output_node(
    node: &PipelineNode,
    ctx: &ExecutionContext,
) -> Result<NodeOutput, String> {
    let format = node.data.format.as_deref().unwrap_or("pdf");
    let input = ctx.get_input(&node.id).ok_or("Output node has no input")?;

    let content = input.content.clone().unwrap_or_default();

    match format {
        "pdf" => {
            // Compile Typst to PDF
            let root_path = ctx.root_path.as_ref()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|| ".".to_string());

            let result = super::compile_typst_source(content.clone(), root_path)
                .await
                .map_err(|e| format!("PDF compilation failed: {}", e))?;

            let pdf_data = result.pdf_data.ok_or("No PDF data generated")?;

            // Optionally save to file
            if let Some(ref output_path) = node.data.output_path {
                let resolved_path = ctx.resolve_path(output_path);
                tokio::fs::write(&resolved_path, &pdf_data)
                    .await
                    .map_err(|e| format!("Failed to write PDF: {}", e))?;

                return Ok(NodeOutput {
                    content: None,
                    path: Some(resolved_path.to_string_lossy().to_string()),
                    format: Some("pdf".to_string()),
                    metadata: Some(serde_json::json!({
                        "size_bytes": pdf_data.len(),
                        "compile_time_ms": result.compile_time_ms
                    })),
                });
            }

            Ok(NodeOutput {
                content: None,
                path: None,
                format: Some("pdf".to_string()),
                metadata: Some(serde_json::json!({
                    "size_bytes": pdf_data.len(),
                    "compile_time_ms": result.compile_time_ms,
                    "pdf_base64": base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &pdf_data)
                })),
            })
        }
        "html" => {
            // Simple Typst to HTML conversion (basic)
            let html_content = format!(
                "<!DOCTYPE html>\n<html>\n<head>\n<meta charset=\"UTF-8\">\n<title>{}</title>\n</head>\n<body>\n<pre>{}</pre>\n</body>\n</html>",
                node.data.label,
                content.replace('<', "&lt;").replace('>', "&gt;")
            );

            if let Some(ref output_path) = node.data.output_path {
                let resolved_path = ctx.resolve_path(output_path);
                tokio::fs::write(&resolved_path, &html_content)
                    .await
                    .map_err(|e| format!("Failed to write HTML: {}", e))?;

                return Ok(NodeOutput {
                    content: Some(html_content),
                    path: Some(resolved_path.to_string_lossy().to_string()),
                    format: Some("html".to_string()),
                    metadata: None,
                });
            }

            Ok(NodeOutput {
                content: Some(html_content),
                path: None,
                format: Some("html".to_string()),
                metadata: None,
            })
        }
        "md" | "markdown" => {
            // Output as markdown
            if let Some(ref output_path) = node.data.output_path {
                let resolved_path = ctx.resolve_path(output_path);
                tokio::fs::write(&resolved_path, &content)
                    .await
                    .map_err(|e| format!("Failed to write Markdown: {}", e))?;

                return Ok(NodeOutput {
                    content: Some(content),
                    path: Some(resolved_path.to_string_lossy().to_string()),
                    format: Some("markdown".to_string()),
                    metadata: None,
                });
            }

            Ok(NodeOutput {
                content: Some(content),
                path: None,
                format: Some("markdown".to_string()),
                metadata: None,
            })
        }
        _ => {
            // Default: save as-is
            if let Some(ref output_path) = node.data.output_path {
                let resolved_path = ctx.resolve_path(output_path);
                tokio::fs::write(&resolved_path, &content)
                    .await
                    .map_err(|e| format!("Failed to write output: {}", e))?;

                return Ok(NodeOutput {
                    content: Some(content),
                    path: Some(resolved_path.to_string_lossy().to_string()),
                    format: Some(format.to_string()),
                    metadata: None,
                });
            }

            Ok(NodeOutput {
                content: Some(content),
                path: None,
                format: Some(format.to_string()),
                metadata: None,
            })
        }
    }
}

/// Execute a condition node
async fn execute_condition_node(
    node: &PipelineNode,
    ctx: &ExecutionContext,
) -> Result<NodeOutput, String> {
    let condition = node.data.condition.as_deref().unwrap_or("true");
    let input = ctx.get_input(&node.id);

    // Simple condition evaluation
    let result = evaluate_condition(condition, input, &ctx.variables);

    Ok(NodeOutput {
        content: input.and_then(|i| i.content.clone()),
        path: input.and_then(|i| i.path.clone()),
        format: input.and_then(|i| i.format.clone()),
        metadata: Some(serde_json::json!({
            "condition_result": result,
            "condition": condition
        })),
    })
}

/// Evaluate a simple condition expression
fn evaluate_condition(
    condition: &str,
    input: Option<&NodeOutput>,
    variables: &HashMap<String, serde_json::Value>,
) -> bool {
    // Simple condition evaluation
    let condition = condition.trim().to_lowercase();

    match condition.as_str() {
        "true" => true,
        "false" => false,
        _ => {
            // Check for variable comparisons like "variable == value"
            if condition.contains("==") {
                let parts: Vec<&str> = condition.split("==").collect();
                if parts.len() == 2 {
                    let var_name = parts[0].trim();
                    let expected = parts[1].trim().trim_matches('"');

                    if let Some(value) = variables.get(var_name) {
                        if let Some(s) = value.as_str() {
                            return s == expected;
                        }
                    }
                }
            }

            // Check for content existence
            if condition.contains("has_content") {
                return input.map(|i| i.content.is_some()).unwrap_or(false);
            }

            // Default to true
            true
        }
    }
}

/// Execute a single node
async fn execute_node(
    node: &PipelineNode,
    ctx: &mut ExecutionContext,
) -> NodeResult {
    let start = std::time::Instant::now();

    let result = match node.node_type {
        NodeType::Source => execute_source_node(node, ctx).await,
        NodeType::Transform => execute_transform_node(node, ctx).await,
        NodeType::Output => execute_output_node(node, ctx).await,
        NodeType::Condition => execute_condition_node(node, ctx).await,
    };

    let duration_ms = start.elapsed().as_millis() as u64;

    match result {
        Ok(output) => {
            ctx.set_output(&node.id, output.clone());
            NodeResult {
                node_id: node.id.clone(),
                status: ExecutionStatus::Completed,
                output: Some(output),
                error: None,
                duration_ms,
            }
        }
        Err(e) => NodeResult {
            node_id: node.id.clone(),
            status: ExecutionStatus::Failed,
            output: None,
            error: Some(e),
            duration_ms,
        },
    }
}

/// Execute a complete pipeline
pub async fn execute_pipeline(
    pipeline: Pipeline,
    root_path: Option<String>,
    progress_tx: Option<mpsc::Sender<PipelineProgress>>,
) -> PipelineResult {
    let start = std::time::Instant::now();
    let pipeline_id = pipeline.id.clone();

    // Build execution order
    let execution_order = match build_execution_order(&pipeline) {
        Ok(order) => order,
        Err(e) => {
            return PipelineResult {
                pipeline_id,
                status: ExecutionStatus::Failed,
                node_results: Vec::new(),
                outputs: Vec::new(),
                total_duration_ms: start.elapsed().as_millis() as u64,
                error: Some(e),
            };
        }
    };

    let total_nodes = execution_order.len();
    let root_path = root_path.map(PathBuf::from);
    let mut ctx = ExecutionContext::new(pipeline.clone(), root_path);
    let mut node_results = Vec::new();
    let mut outputs = Vec::new();

    // Execute nodes in order
    for (idx, node_id) in execution_order.iter().enumerate() {
        let node = pipeline.nodes.iter().find(|n| &n.id == node_id);
        let node = match node {
            Some(n) => n,
            None => continue,
        };

        // Send progress update
        if let Some(ref tx) = progress_tx {
            let _ = tx.send(PipelineProgress {
                pipeline_id: pipeline_id.clone(),
                current_node: node_id.clone(),
                completed_nodes: idx,
                total_nodes,
                status: ExecutionStatus::Running,
                message: format!("Executing {}", node.data.label),
            }).await;
        }

        // Execute the node
        let result = execute_node(node, &mut ctx).await;

        // Check for failure
        if matches!(result.status, ExecutionStatus::Failed) {
            node_results.push(result.clone());

            return PipelineResult {
                pipeline_id,
                status: ExecutionStatus::Failed,
                node_results,
                outputs,
                total_duration_ms: start.elapsed().as_millis() as u64,
                error: result.error,
            };
        }

        // Collect output node results
        if matches!(node.node_type, NodeType::Output) {
            if let Some(ref output) = result.output {
                outputs.push(PipelineOutput {
                    node_id: node_id.clone(),
                    format: output.format.clone().unwrap_or_default(),
                    path: output.path.clone(),
                    content: None, // Content stored in NodeOutput
                });
            }
        }

        node_results.push(result);
    }

    // Send completion progress
    if let Some(ref tx) = progress_tx {
        let _ = tx.send(PipelineProgress {
            pipeline_id: pipeline_id.clone(),
            current_node: String::new(),
            completed_nodes: total_nodes,
            total_nodes,
            status: ExecutionStatus::Completed,
            message: "Pipeline completed successfully".to_string(),
        }).await;
    }

    PipelineResult {
        pipeline_id,
        status: ExecutionStatus::Completed,
        node_results,
        outputs,
        total_duration_ms: start.elapsed().as_millis() as u64,
        error: None,
    }
}

// === Tauri Commands ===

/// Execute a pipeline
#[tauri::command]
pub async fn run_pipeline(
    pipeline: Pipeline,
    root_path: Option<String>,
) -> Result<PipelineResult, String> {
    Ok(execute_pipeline(pipeline, root_path, None).await)
}

/// Execute a pipeline with streaming progress
#[tauri::command]
pub async fn run_pipeline_stream(
    app: tauri::AppHandle,
    pipeline: Pipeline,
    root_path: Option<String>,
    request_id: String,
) -> Result<PipelineResult, String> {
    use tauri::Emitter;

    let (tx, mut rx) = mpsc::channel::<PipelineProgress>(32);

    // Spawn progress forwarder
    let app_clone = app.clone();
    let request_id_clone = request_id.clone();
    tokio::spawn(async move {
        while let Some(progress) = rx.recv().await {
            let _ = app_clone.emit(
                &format!("pipeline-progress-{}", request_id_clone),
                progress,
            );
        }
    });

    let result = execute_pipeline(pipeline, root_path, Some(tx)).await;

    // Emit completion
    let _ = app.emit(
        &format!("pipeline-complete-{}", request_id),
        &result,
    );

    Ok(result)
}

/// Validate a pipeline structure
#[tauri::command]
pub fn validate_pipeline(pipeline: Pipeline) -> Result<Vec<String>, String> {
    let mut warnings = Vec::new();

    // Check for cycles
    if let Err(e) = build_execution_order(&pipeline) {
        return Err(e);
    }

    // Check for disconnected nodes
    let mut connected: HashSet<String> = HashSet::new();
    for edge in &pipeline.edges {
        connected.insert(edge.source.clone());
        connected.insert(edge.target.clone());
    }

    for node in &pipeline.nodes {
        if !connected.contains(&node.id) && pipeline.nodes.len() > 1 {
            warnings.push(format!("Node '{}' is not connected to the pipeline", node.data.label));
        }
    }

    // Check for required fields
    for node in &pipeline.nodes {
        match node.node_type {
            NodeType::Source => {
                if node.data.path.is_none() && node.data.source_type.as_deref() != Some("variable") {
                    warnings.push(format!("Source node '{}' has no path specified", node.data.label));
                }
            }
            NodeType::Output => {
                if node.data.format.is_none() {
                    warnings.push(format!("Output node '{}' has no format specified", node.data.label));
                }
            }
            _ => {}
        }
    }

    // Check for output nodes
    let has_output = pipeline.nodes.iter().any(|n| matches!(n.node_type, NodeType::Output));
    if !has_output {
        warnings.push("Pipeline has no output nodes".to_string());
    }

    Ok(warnings)
}

/// Create a new pipeline with a unique ID
#[tauri::command]
pub fn create_pipeline(name: String) -> Pipeline {
    Pipeline {
        id: Uuid::new_v4().to_string(),
        name,
        nodes: Vec::new(),
        edges: Vec::new(),
        variables: Some(HashMap::new()),
    }
}
