import { useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./RFPAnalyzer.css";

interface Requirement {
  id: string;
  text: string;
  category: string;
  priority: string;
  section?: string;
  page?: number;
  keywords: string[];
  compliance_status: string;
  response_text?: string;
  notes?: string;
}

interface Deadline {
  id: string;
  description: string;
  date?: string;
  deadline_type: string;
  is_mandatory: boolean;
}

interface CriticalTerm {
  term: string;
  term_type: string;
  occurrences: Array<{
    context: string;
    page?: number;
    position: number;
  }>;
}

interface EvaluationCriterion {
  name: string;
  description?: string;
  weight?: number;
  max_points?: number;
}

interface DocumentSummary {
  total_requirements: number;
  critical_requirements: number;
  total_deadlines: number;
  has_mandatory_deadlines: boolean;
  estimated_effort?: string;
  risk_level: string;
}

interface RfpAnalysis {
  document_name: string;
  requirements: Requirement[];
  deadlines: Deadline[];
  critical_terms: CriticalTerm[];
  evaluation_criteria: EvaluationCriterion[];
  summary: DocumentSummary;
}

interface ComplianceMatrix {
  project_name: string;
  rfp_name: string;
  created_at: string;
  updated_at: string;
  entries: ComplianceEntry[];
  summary: ComplianceSummary;
}

interface ComplianceEntry {
  requirement_id: string;
  requirement_text: string;
  category: string;
  priority: string;
  status: string;
  response_section?: string;
  response_page?: number;
  response_summary?: string;
  evidence?: string;
  reviewer?: string;
  review_date?: string;
  comments?: string;
}

interface ComplianceSummary {
  total_requirements: number;
  compliant: number;
  partially_compliant: number;
  non_compliant: number;
  not_started: number;
  in_progress: number;
  not_applicable: number;
  compliance_percentage: number;
}

interface GapAnalysis {
  total_gaps: number;
  critical_gaps: GapItem[];
  high_gaps: GapItem[];
  medium_gaps: GapItem[];
  low_gaps: GapItem[];
  recommendations: string[];
}

interface GapItem {
  requirement_id: string;
  requirement_text: string;
  category: string;
  priority: string;
  current_status: string;
  gap_description: string;
  suggested_action: string;
}

interface RFPAnalyzerProps {
  documentContent?: string;
  documentName?: string;
  onRequirementSelect?: (requirement: Requirement) => void;
}

type TabType = "overview" | "requirements" | "deadlines" | "compliance" | "gaps";

export function RFPAnalyzer({ documentContent, documentName, onRequirementSelect }: RFPAnalyzerProps) {
  const [analysis, setAnalysis] = useState<RfpAnalysis | null>(null);
  const [complianceMatrix, setComplianceMatrix] = useState<ComplianceMatrix | null>(null);
  const [gapAnalysis, setGapAnalysis] = useState<GapAnalysis | null>(null);
  const [activeTab, setActiveTab] = useState<TabType>("overview");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [inputText, setInputText] = useState("");
  const [projectName, setProjectName] = useState("My RFP Response");

  const analyzeDocument = useCallback(async (content: string, name: string) => {
    try {
      setLoading(true);
      setError(null);

      const result = await invoke<RfpAnalysis>("analyze_rfp_document", {
        content,
        documentName: name,
      });

      setAnalysis(result);

      // Auto-generate compliance matrix
      const matrix = await invoke<ComplianceMatrix>("generate_matrix", {
        analysis: result,
        projectName,
      });
      setComplianceMatrix(matrix);

      // Auto-generate gap analysis
      const gaps = await invoke<GapAnalysis>("analyze_gaps", {
        matrix,
      });
      setGapAnalysis(gaps);

    } catch (err) {
      console.error("Failed to analyze RFP:", err);
      setError(err instanceof Error ? err.message : "Failed to analyze document");
    } finally {
      setLoading(false);
    }
  }, [projectName]);

  const handleAnalyze = () => {
    const content = documentContent || inputText;
    const name = documentName || "Uploaded RFP";
    if (content) {
      analyzeDocument(content, name);
    }
  };

  const updateComplianceStatus = async (requirementId: string, status: string) => {
    if (!complianceMatrix) return;

    try {
      const updatedMatrix = await invoke<ComplianceMatrix>("update_compliance_entry", {
        matrix: complianceMatrix,
        requirementId,
        status,
        responseSection: null,
        responseSummary: null,
        comments: null,
      });
      setComplianceMatrix(updatedMatrix);

      // Refresh gap analysis
      const gaps = await invoke<GapAnalysis>("analyze_gaps", {
        matrix: updatedMatrix,
      });
      setGapAnalysis(gaps);
    } catch (err) {
      console.error("Failed to update compliance:", err);
    }
  };

  const exportMatrix = async (format: string) => {
    if (!complianceMatrix) return;

    try {
      const result = await invoke<{ format: string; content: string }>("export_matrix", {
        matrix: complianceMatrix,
        format,
      });

      // Create download
      const blob = new Blob([result.content], { type: "text/plain" });
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = `compliance_matrix.${format}`;
      a.click();
      URL.revokeObjectURL(url);
    } catch (err) {
      console.error("Failed to export:", err);
    }
  };

  const getRiskColor = (level: string) => {
    switch (level.toLowerCase()) {
      case "critical": return "var(--error)";
      case "high": return "#f97316";
      case "medium": return "#eab308";
      case "low": return "#22c55e";
      default: return "var(--text-secondary)";
    }
  };

  const getStatusColor = (status: string) => {
    switch (status.toLowerCase()) {
      case "compliant": return "#22c55e";
      case "partiallycompliant": case "partially_compliant": return "#eab308";
      case "noncompliant": case "non_compliant": return "#ef4444";
      case "inprogress": case "in_progress": return "#3b82f6";
      case "notapplicable": case "not_applicable": return "#6b7280";
      default: return "var(--text-secondary)";
    }
  };

  const getPriorityColor = (priority: string) => {
    switch (priority.toLowerCase()) {
      case "critical": return "#dc2626";
      case "high": return "#f97316";
      case "medium": return "#eab308";
      case "low": return "#22c55e";
      default: return "var(--text-secondary)";
    }
  };

  return (
    <div className="rfp-analyzer">
      <div className="rfp-analyzer-header">
        <h3>RFP Analyzer</h3>
        {analysis && (
          <div className="export-buttons">
            <button onClick={() => exportMatrix("csv")}>CSV</button>
            <button onClick={() => exportMatrix("markdown")}>MD</button>
            <button onClick={() => exportMatrix("html")}>HTML</button>
          </div>
        )}
      </div>

      {!analysis && (
        <div className="rfp-input-section">
          <div className="input-field">
            <label>Project Name</label>
            <input
              type="text"
              value={projectName}
              onChange={(e) => setProjectName(e.target.value)}
              placeholder="My RFP Response"
            />
          </div>
          <div className="input-field">
            <label>RFP Content</label>
            <textarea
              value={inputText}
              onChange={(e) => setInputText(e.target.value)}
              placeholder="Paste RFP document content here for analysis..."
              rows={10}
            />
          </div>
          <button
            className="analyze-button"
            onClick={handleAnalyze}
            disabled={loading || (!documentContent && !inputText)}
          >
            {loading ? "Analyzing..." : "Analyze RFP"}
          </button>
        </div>
      )}

      {error && <div className="rfp-error">{error}</div>}

      {analysis && (
        <>
          <div className="rfp-tabs">
            <button
              className={activeTab === "overview" ? "active" : ""}
              onClick={() => setActiveTab("overview")}
            >
              Overview
            </button>
            <button
              className={activeTab === "requirements" ? "active" : ""}
              onClick={() => setActiveTab("requirements")}
            >
              Requirements ({analysis.requirements.length})
            </button>
            <button
              className={activeTab === "deadlines" ? "active" : ""}
              onClick={() => setActiveTab("deadlines")}
            >
              Deadlines ({analysis.deadlines.length})
            </button>
            <button
              className={activeTab === "compliance" ? "active" : ""}
              onClick={() => setActiveTab("compliance")}
            >
              Compliance
            </button>
            <button
              className={activeTab === "gaps" ? "active" : ""}
              onClick={() => setActiveTab("gaps")}
            >
              Gaps {gapAnalysis && `(${gapAnalysis.total_gaps})`}
            </button>
          </div>

          <div className="rfp-content">
            {activeTab === "overview" && (
              <div className="overview-section">
                <div className="summary-cards">
                  <div className="summary-card">
                    <div className="card-value">{analysis.summary.total_requirements}</div>
                    <div className="card-label">Requirements</div>
                  </div>
                  <div className="summary-card critical">
                    <div className="card-value">{analysis.summary.critical_requirements}</div>
                    <div className="card-label">Critical</div>
                  </div>
                  <div className="summary-card">
                    <div className="card-value">{analysis.summary.total_deadlines}</div>
                    <div className="card-label">Deadlines</div>
                  </div>
                  <div className="summary-card" style={{ borderColor: getRiskColor(analysis.summary.risk_level) }}>
                    <div className="card-value" style={{ color: getRiskColor(analysis.summary.risk_level) }}>
                      {analysis.summary.risk_level}
                    </div>
                    <div className="card-label">Risk Level</div>
                  </div>
                </div>

                {analysis.summary.estimated_effort && (
                  <div className="effort-estimate">
                    <strong>Estimated Effort:</strong> {analysis.summary.estimated_effort}
                  </div>
                )}

                {complianceMatrix && (
                  <div className="compliance-overview">
                    <h4>Compliance Progress</h4>
                    <div className="progress-bar">
                      <div
                        className="progress-fill"
                        style={{ width: `${complianceMatrix.summary.compliance_percentage}%` }}
                      />
                    </div>
                    <div className="progress-label">
                      {complianceMatrix.summary.compliance_percentage.toFixed(1)}% Complete
                    </div>
                    <div className="compliance-stats">
                      <span className="stat compliant">
                        {complianceMatrix.summary.compliant} Compliant
                      </span>
                      <span className="stat partial">
                        {complianceMatrix.summary.partially_compliant} Partial
                      </span>
                      <span className="stat non-compliant">
                        {complianceMatrix.summary.non_compliant} Non-Compliant
                      </span>
                      <span className="stat not-started">
                        {complianceMatrix.summary.not_started} Not Started
                      </span>
                    </div>
                  </div>
                )}

                {analysis.evaluation_criteria.length > 0 && (
                  <div className="evaluation-section">
                    <h4>Evaluation Criteria</h4>
                    <ul>
                      {analysis.evaluation_criteria.map((criterion, idx) => (
                        <li key={idx}>
                          {criterion.name}
                          {criterion.weight && ` (${criterion.weight}%)`}
                          {criterion.max_points && ` - ${criterion.max_points} pts`}
                        </li>
                      ))}
                    </ul>
                  </div>
                )}

                {analysis.critical_terms.length > 0 && (
                  <div className="critical-terms-section">
                    <h4>Critical Terms Found</h4>
                    <div className="term-badges">
                      {analysis.critical_terms.map((term, idx) => (
                        <span
                          key={idx}
                          className="term-badge"
                          title={`${term.occurrences.length} occurrences`}
                        >
                          {term.term} ({term.occurrences.length})
                        </span>
                      ))}
                    </div>
                  </div>
                )}
              </div>
            )}

            {activeTab === "requirements" && (
              <div className="requirements-section">
                {analysis.requirements.map((req) => (
                  <div
                    key={req.id}
                    className="requirement-item"
                    onClick={() => onRequirementSelect?.(req)}
                  >
                    <div className="req-header">
                      <span className="req-id">{req.id}</span>
                      <span
                        className="req-priority"
                        style={{ backgroundColor: getPriorityColor(req.priority) }}
                      >
                        {req.priority}
                      </span>
                      <span className="req-category">{req.category}</span>
                    </div>
                    <div className="req-text">{req.text}</div>
                    {req.keywords.length > 0 && (
                      <div className="req-keywords">
                        {req.keywords.slice(0, 5).map((kw, idx) => (
                          <span key={idx} className="keyword">{kw}</span>
                        ))}
                      </div>
                    )}
                  </div>
                ))}
              </div>
            )}

            {activeTab === "deadlines" && (
              <div className="deadlines-section">
                {analysis.deadlines.length === 0 ? (
                  <div className="empty-state">No deadlines detected</div>
                ) : (
                  analysis.deadlines.map((deadline) => (
                    <div key={deadline.id} className={`deadline-item ${deadline.is_mandatory ? "mandatory" : ""}`}>
                      <div className="deadline-header">
                        <span className="deadline-id">{deadline.id}</span>
                        <span className="deadline-type">{deadline.deadline_type}</span>
                        {deadline.is_mandatory && (
                          <span className="mandatory-badge">Mandatory</span>
                        )}
                      </div>
                      <div className="deadline-description">{deadline.description}</div>
                    </div>
                  ))
                )}
              </div>
            )}

            {activeTab === "compliance" && complianceMatrix && (
              <div className="compliance-section">
                {complianceMatrix.entries.map((entry) => (
                  <div key={entry.requirement_id} className="compliance-entry">
                    <div className="entry-header">
                      <span className="entry-id">{entry.requirement_id}</span>
                      <select
                        value={entry.status}
                        onChange={(e) => updateComplianceStatus(entry.requirement_id, e.target.value)}
                        style={{ borderColor: getStatusColor(entry.status) }}
                      >
                        <option value="notstarted">Not Started</option>
                        <option value="inprogress">In Progress</option>
                        <option value="compliant">Compliant</option>
                        <option value="partiallycompliant">Partially Compliant</option>
                        <option value="noncompliant">Non-Compliant</option>
                        <option value="notapplicable">Not Applicable</option>
                      </select>
                    </div>
                    <div className="entry-text">{entry.requirement_text}</div>
                    <div className="entry-meta">
                      <span className="entry-category">{entry.category}</span>
                      <span
                        className="entry-priority"
                        style={{ color: getPriorityColor(entry.priority) }}
                      >
                        {entry.priority}
                      </span>
                    </div>
                  </div>
                ))}
              </div>
            )}

            {activeTab === "gaps" && gapAnalysis && (
              <div className="gaps-section">
                <div className="recommendations">
                  <h4>Recommendations</h4>
                  <ul>
                    {gapAnalysis.recommendations.map((rec, idx) => (
                      <li key={idx}>{rec}</li>
                    ))}
                  </ul>
                </div>

                {gapAnalysis.critical_gaps.length > 0 && (
                  <div className="gap-group critical">
                    <h4>Critical Gaps ({gapAnalysis.critical_gaps.length})</h4>
                    {gapAnalysis.critical_gaps.map((gap) => (
                      <div key={gap.requirement_id} className="gap-item">
                        <div className="gap-header">
                          <span className="gap-id">{gap.requirement_id}</span>
                          <span className="gap-status">{gap.current_status}</span>
                        </div>
                        <div className="gap-text">{gap.requirement_text}</div>
                        <div className="gap-action">
                          <strong>Action:</strong> {gap.suggested_action}
                        </div>
                      </div>
                    ))}
                  </div>
                )}

                {gapAnalysis.high_gaps.length > 0 && (
                  <div className="gap-group high">
                    <h4>High Priority Gaps ({gapAnalysis.high_gaps.length})</h4>
                    {gapAnalysis.high_gaps.map((gap) => (
                      <div key={gap.requirement_id} className="gap-item">
                        <div className="gap-header">
                          <span className="gap-id">{gap.requirement_id}</span>
                          <span className="gap-status">{gap.current_status}</span>
                        </div>
                        <div className="gap-text">{gap.requirement_text}</div>
                        <div className="gap-action">
                          <strong>Action:</strong> {gap.suggested_action}
                        </div>
                      </div>
                    ))}
                  </div>
                )}

                {gapAnalysis.medium_gaps.length > 0 && (
                  <div className="gap-group medium">
                    <h4>Medium Priority Gaps ({gapAnalysis.medium_gaps.length})</h4>
                    {gapAnalysis.medium_gaps.slice(0, 5).map((gap) => (
                      <div key={gap.requirement_id} className="gap-item">
                        <div className="gap-header">
                          <span className="gap-id">{gap.requirement_id}</span>
                        </div>
                        <div className="gap-text">{gap.requirement_text}</div>
                      </div>
                    ))}
                    {gapAnalysis.medium_gaps.length > 5 && (
                      <div className="more-gaps">
                        +{gapAnalysis.medium_gaps.length - 5} more
                      </div>
                    )}
                  </div>
                )}
              </div>
            )}
          </div>

          <button className="reset-button" onClick={() => {
            setAnalysis(null);
            setComplianceMatrix(null);
            setGapAnalysis(null);
            setInputText("");
          }}>
            Analyze New Document
          </button>
        </>
      )}
    </div>
  );
}

export default RFPAnalyzer;
