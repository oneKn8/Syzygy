//! Compliance Matrix Generation and Management
//!
//! Creates and manages compliance matrices for RFP responses.

use super::analyzer::{
    ComplianceStatus, RequirementCategory, RequirementPriority, RfpAnalysis,
};
use serde::{Deserialize, Serialize};

/// A compliance matrix entry linking requirement to response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceEntry {
    pub requirement_id: String,
    pub requirement_text: String,
    pub category: RequirementCategory,
    pub priority: RequirementPriority,
    pub status: ComplianceStatus,
    pub response_section: Option<String>,
    pub response_page: Option<u32>,
    pub response_summary: Option<String>,
    pub evidence: Option<String>,
    pub reviewer: Option<String>,
    pub review_date: Option<String>,
    pub comments: Option<String>,
}

/// A complete compliance matrix
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceMatrix {
    pub project_name: String,
    pub rfp_name: String,
    pub created_at: String,
    pub updated_at: String,
    pub entries: Vec<ComplianceEntry>,
    pub summary: ComplianceSummary,
}

/// Summary statistics for a compliance matrix
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceSummary {
    pub total_requirements: usize,
    pub compliant: usize,
    pub partially_compliant: usize,
    pub non_compliant: usize,
    pub not_started: usize,
    pub in_progress: usize,
    pub not_applicable: usize,
    pub compliance_percentage: f32,
    pub by_category: Vec<CategorySummary>,
    pub by_priority: Vec<PrioritySummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategorySummary {
    pub category: RequirementCategory,
    pub total: usize,
    pub compliant: usize,
    pub percentage: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrioritySummary {
    pub priority: RequirementPriority,
    pub total: usize,
    pub compliant: usize,
    pub percentage: f32,
}

/// Gap analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GapAnalysis {
    pub total_gaps: usize,
    pub critical_gaps: Vec<GapItem>,
    pub high_gaps: Vec<GapItem>,
    pub medium_gaps: Vec<GapItem>,
    pub low_gaps: Vec<GapItem>,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GapItem {
    pub requirement_id: String,
    pub requirement_text: String,
    pub category: RequirementCategory,
    pub priority: RequirementPriority,
    pub current_status: ComplianceStatus,
    pub gap_description: String,
    pub suggested_action: String,
}

/// Generate a compliance matrix from RFP analysis
pub fn generate_compliance_matrix(
    analysis: &RfpAnalysis,
    project_name: &str,
) -> ComplianceMatrix {
    let now = chrono::Utc::now().to_rfc3339();

    let entries: Vec<ComplianceEntry> = analysis
        .requirements
        .iter()
        .map(|req| ComplianceEntry {
            requirement_id: req.id.clone(),
            requirement_text: req.text.clone(),
            category: req.category.clone(),
            priority: req.priority.clone(),
            status: req.compliance_status.clone(),
            response_section: None,
            response_page: None,
            response_summary: req.response_text.clone(),
            evidence: None,
            reviewer: None,
            review_date: None,
            comments: req.notes.clone(),
        })
        .collect();

    let summary = calculate_summary(&entries);

    ComplianceMatrix {
        project_name: project_name.to_string(),
        rfp_name: analysis.document_name.clone(),
        created_at: now.clone(),
        updated_at: now,
        entries,
        summary,
    }
}

fn calculate_summary(entries: &[ComplianceEntry]) -> ComplianceSummary {
    let total = entries.len();

    let compliant = entries
        .iter()
        .filter(|e| e.status == ComplianceStatus::Compliant)
        .count();
    let partially_compliant = entries
        .iter()
        .filter(|e| e.status == ComplianceStatus::PartiallyCompliant)
        .count();
    let non_compliant = entries
        .iter()
        .filter(|e| e.status == ComplianceStatus::NonCompliant)
        .count();
    let not_started = entries
        .iter()
        .filter(|e| e.status == ComplianceStatus::NotStarted)
        .count();
    let in_progress = entries
        .iter()
        .filter(|e| e.status == ComplianceStatus::InProgress)
        .count();
    let not_applicable = entries
        .iter()
        .filter(|e| e.status == ComplianceStatus::NotApplicable)
        .count();

    let applicable_total = total - not_applicable;
    let compliance_percentage = if applicable_total > 0 {
        ((compliant as f32 + partially_compliant as f32 * 0.5) / applicable_total as f32) * 100.0
    } else {
        0.0
    };

    // Calculate by category
    let categories = [
        RequirementCategory::Technical,
        RequirementCategory::Management,
        RequirementCategory::Financial,
        RequirementCategory::Legal,
        RequirementCategory::Staffing,
        RequirementCategory::Timeline,
        RequirementCategory::Deliverable,
        RequirementCategory::Qualification,
        RequirementCategory::Experience,
        RequirementCategory::Security,
        RequirementCategory::Other,
    ];

    let by_category: Vec<CategorySummary> = categories
        .iter()
        .filter_map(|cat| {
            let cat_entries: Vec<&ComplianceEntry> =
                entries.iter().filter(|e| e.category == *cat).collect();
            let cat_total = cat_entries.len();
            if cat_total == 0 {
                return None;
            }
            let cat_compliant = cat_entries
                .iter()
                .filter(|e| e.status == ComplianceStatus::Compliant)
                .count();
            Some(CategorySummary {
                category: cat.clone(),
                total: cat_total,
                compliant: cat_compliant,
                percentage: (cat_compliant as f32 / cat_total as f32) * 100.0,
            })
        })
        .collect();

    // Calculate by priority
    let priorities = [
        RequirementPriority::Critical,
        RequirementPriority::High,
        RequirementPriority::Medium,
        RequirementPriority::Low,
    ];

    let by_priority: Vec<PrioritySummary> = priorities
        .iter()
        .filter_map(|pri| {
            let pri_entries: Vec<&ComplianceEntry> =
                entries.iter().filter(|e| e.priority == *pri).collect();
            let pri_total = pri_entries.len();
            if pri_total == 0 {
                return None;
            }
            let pri_compliant = pri_entries
                .iter()
                .filter(|e| e.status == ComplianceStatus::Compliant)
                .count();
            Some(PrioritySummary {
                priority: pri.clone(),
                total: pri_total,
                compliant: pri_compliant,
                percentage: (pri_compliant as f32 / pri_total as f32) * 100.0,
            })
        })
        .collect();

    ComplianceSummary {
        total_requirements: total,
        compliant,
        partially_compliant,
        non_compliant,
        not_started,
        in_progress,
        not_applicable,
        compliance_percentage,
        by_category,
        by_priority,
    }
}

/// Perform gap analysis on a compliance matrix
pub fn perform_gap_analysis(matrix: &ComplianceMatrix) -> GapAnalysis {
    let gaps: Vec<GapItem> = matrix
        .entries
        .iter()
        .filter(|e| {
            e.status == ComplianceStatus::NonCompliant
                || e.status == ComplianceStatus::PartiallyCompliant
                || e.status == ComplianceStatus::NotStarted
        })
        .map(|e| GapItem {
            requirement_id: e.requirement_id.clone(),
            requirement_text: e.requirement_text.clone(),
            category: e.category.clone(),
            priority: e.priority.clone(),
            current_status: e.status.clone(),
            gap_description: generate_gap_description(e),
            suggested_action: generate_suggested_action(e),
        })
        .collect();

    let critical_gaps: Vec<GapItem> = gaps
        .iter()
        .filter(|g| g.priority == RequirementPriority::Critical)
        .cloned()
        .collect();

    let high_gaps: Vec<GapItem> = gaps
        .iter()
        .filter(|g| g.priority == RequirementPriority::High)
        .cloned()
        .collect();

    let medium_gaps: Vec<GapItem> = gaps
        .iter()
        .filter(|g| g.priority == RequirementPriority::Medium)
        .cloned()
        .collect();

    let low_gaps: Vec<GapItem> = gaps
        .iter()
        .filter(|g| g.priority == RequirementPriority::Low)
        .cloned()
        .collect();

    let recommendations = generate_recommendations(&matrix.entries, &gaps);

    GapAnalysis {
        total_gaps: gaps.len(),
        critical_gaps,
        high_gaps,
        medium_gaps,
        low_gaps,
        recommendations,
    }
}

fn generate_gap_description(entry: &ComplianceEntry) -> String {
    match entry.status {
        ComplianceStatus::NonCompliant => {
            format!(
                "Requirement {} is not met. No compliant solution has been identified.",
                entry.requirement_id
            )
        }
        ComplianceStatus::PartiallyCompliant => {
            format!(
                "Requirement {} is only partially addressed. Additional work needed.",
                entry.requirement_id
            )
        }
        ComplianceStatus::NotStarted => {
            format!(
                "Requirement {} has not been addressed yet.",
                entry.requirement_id
            )
        }
        _ => String::new(),
    }
}

fn generate_suggested_action(entry: &ComplianceEntry) -> String {
    match (&entry.status, &entry.category) {
        (ComplianceStatus::NonCompliant, RequirementCategory::Technical) => {
            "Evaluate technical alternatives or propose exception with mitigation plan.".to_string()
        }
        (ComplianceStatus::NonCompliant, RequirementCategory::Staffing) => {
            "Identify qualified personnel or plan for recruitment/subcontracting.".to_string()
        }
        (ComplianceStatus::NonCompliant, RequirementCategory::Experience) => {
            "Consider teaming arrangements or highlight transferable experience.".to_string()
        }
        (ComplianceStatus::PartiallyCompliant, _) => {
            "Document existing compliance and propose plan to address remaining gaps.".to_string()
        }
        (ComplianceStatus::NotStarted, _) => {
            "Assign owner and develop response strategy for this requirement.".to_string()
        }
        _ => "Review requirement and determine appropriate response strategy.".to_string(),
    }
}

fn generate_recommendations(entries: &[ComplianceEntry], gaps: &[GapItem]) -> Vec<String> {
    let mut recommendations = Vec::new();

    // Critical gaps
    let critical_count = gaps
        .iter()
        .filter(|g| g.priority == RequirementPriority::Critical)
        .count();
    if critical_count > 0 {
        recommendations.push(format!(
            "URGENT: {} critical requirement(s) have compliance gaps. Address these first.",
            critical_count
        ));
    }

    // Category-specific recommendations
    let security_gaps = gaps
        .iter()
        .filter(|g| matches!(g.category, RequirementCategory::Security))
        .count();
    if security_gaps > 0 {
        recommendations.push(format!(
            "Security: {} gap(s) identified. Engage security team for review.",
            security_gaps
        ));
    }

    let staffing_gaps = gaps
        .iter()
        .filter(|g| matches!(g.category, RequirementCategory::Staffing))
        .count();
    if staffing_gaps > 0 {
        recommendations.push(format!(
            "Staffing: {} gap(s) identified. Review resource availability and consider partners.",
            staffing_gaps
        ));
    }

    // Not started items
    let not_started = entries
        .iter()
        .filter(|e| e.status == ComplianceStatus::NotStarted)
        .count();
    if not_started > 0 {
        recommendations.push(format!(
            "{} requirement(s) not yet started. Assign owners to ensure coverage.",
            not_started
        ));
    }

    // Overall progress
    let total = entries.len();
    let complete = entries
        .iter()
        .filter(|e| {
            e.status == ComplianceStatus::Compliant || e.status == ComplianceStatus::NotApplicable
        })
        .count();
    let progress_pct = (complete as f32 / total as f32) * 100.0;

    if progress_pct < 50.0 {
        recommendations.push(format!(
            "Overall progress at {:.0}%. Accelerate response development.",
            progress_pct
        ));
    } else if progress_pct < 80.0 {
        recommendations.push(format!(
            "Good progress at {:.0}%. Focus on closing remaining gaps.",
            progress_pct
        ));
    }

    if recommendations.is_empty() {
        recommendations.push("All requirements are being addressed. Continue current approach.".to_string());
    }

    recommendations
}

/// Export compliance matrix to various formats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceExport {
    pub format: ExportFormat,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExportFormat {
    Csv,
    Json,
    Markdown,
    Html,
}

pub fn export_compliance_matrix(matrix: &ComplianceMatrix, format: ExportFormat) -> ComplianceExport {
    let content = match format {
        ExportFormat::Csv => export_to_csv(matrix),
        ExportFormat::Json => serde_json::to_string_pretty(matrix).unwrap_or_default(),
        ExportFormat::Markdown => export_to_markdown(matrix),
        ExportFormat::Html => export_to_html(matrix),
    };

    ComplianceExport {
        format,
        content,
    }
}

fn export_to_csv(matrix: &ComplianceMatrix) -> String {
    let mut csv = String::from("ID,Requirement,Category,Priority,Status,Response Section,Response Summary,Comments\n");

    for entry in &matrix.entries {
        csv.push_str(&format!(
            "\"{}\",\"{}\",\"{:?}\",\"{:?}\",\"{:?}\",\"{}\",\"{}\",\"{}\"\n",
            entry.requirement_id,
            entry.requirement_text.replace('"', "\"\""),
            entry.category,
            entry.priority,
            entry.status,
            entry.response_section.as_deref().unwrap_or(""),
            entry.response_summary.as_deref().unwrap_or("").replace('"', "\"\""),
            entry.comments.as_deref().unwrap_or("").replace('"', "\"\""),
        ));
    }

    csv
}

fn export_to_markdown(matrix: &ComplianceMatrix) -> String {
    let mut md = format!("# Compliance Matrix: {}\n\n", matrix.project_name);
    md.push_str(&format!("**RFP:** {}\n\n", matrix.rfp_name));
    md.push_str(&format!("**Generated:** {}\n\n", matrix.created_at));

    // Summary
    md.push_str("## Summary\n\n");
    md.push_str(&format!("- Total Requirements: {}\n", matrix.summary.total_requirements));
    md.push_str(&format!("- Compliant: {}\n", matrix.summary.compliant));
    md.push_str(&format!("- Partially Compliant: {}\n", matrix.summary.partially_compliant));
    md.push_str(&format!("- Non-Compliant: {}\n", matrix.summary.non_compliant));
    md.push_str(&format!("- Compliance Rate: {:.1}%\n\n", matrix.summary.compliance_percentage));

    // Table
    md.push_str("## Requirements\n\n");
    md.push_str("| ID | Requirement | Category | Priority | Status |\n");
    md.push_str("|:---|:------------|:---------|:---------|:-------|\n");

    for entry in &matrix.entries {
        let short_text = if entry.requirement_text.len() > 80 {
            format!("{}...", &entry.requirement_text[..77])
        } else {
            entry.requirement_text.clone()
        };

        md.push_str(&format!(
            "| {} | {} | {:?} | {:?} | {:?} |\n",
            entry.requirement_id,
            short_text.replace('|', "\\|"),
            entry.category,
            entry.priority,
            entry.status,
        ));
    }

    md
}

fn export_to_html(matrix: &ComplianceMatrix) -> String {
    let mut html = String::from("<!DOCTYPE html>\n<html>\n<head>\n");
    html.push_str("<meta charset=\"UTF-8\">\n");
    html.push_str(&format!("<title>Compliance Matrix: {}</title>\n", matrix.project_name));
    html.push_str("<style>\n");
    html.push_str("body { font-family: Arial, sans-serif; margin: 20px; }\n");
    html.push_str("table { border-collapse: collapse; width: 100%; }\n");
    html.push_str("th, td { border: 1px solid #ddd; padding: 8px; text-align: left; }\n");
    html.push_str("th { background-color: #4a5568; color: white; }\n");
    html.push_str("tr:nth-child(even) { background-color: #f2f2f2; }\n");
    html.push_str(".compliant { color: #22c55e; }\n");
    html.push_str(".partially { color: #eab308; }\n");
    html.push_str(".non-compliant { color: #ef4444; }\n");
    html.push_str(".critical { font-weight: bold; color: #dc2626; }\n");
    html.push_str("</style>\n</head>\n<body>\n");

    html.push_str(&format!("<h1>Compliance Matrix: {}</h1>\n", matrix.project_name));
    html.push_str(&format!("<p><strong>RFP:</strong> {}</p>\n", matrix.rfp_name));
    html.push_str(&format!("<p><strong>Generated:</strong> {}</p>\n", matrix.created_at));

    // Summary
    html.push_str("<h2>Summary</h2>\n");
    html.push_str("<ul>\n");
    html.push_str(&format!("<li>Total Requirements: {}</li>\n", matrix.summary.total_requirements));
    html.push_str(&format!("<li class=\"compliant\">Compliant: {}</li>\n", matrix.summary.compliant));
    html.push_str(&format!("<li class=\"partially\">Partially Compliant: {}</li>\n", matrix.summary.partially_compliant));
    html.push_str(&format!("<li class=\"non-compliant\">Non-Compliant: {}</li>\n", matrix.summary.non_compliant));
    html.push_str(&format!("<li>Compliance Rate: {:.1}%</li>\n", matrix.summary.compliance_percentage));
    html.push_str("</ul>\n");

    // Table
    html.push_str("<h2>Requirements</h2>\n");
    html.push_str("<table>\n<thead>\n<tr>\n");
    html.push_str("<th>ID</th><th>Requirement</th><th>Category</th><th>Priority</th><th>Status</th>\n");
    html.push_str("</tr>\n</thead>\n<tbody>\n");

    for entry in &matrix.entries {
        let status_class = match entry.status {
            ComplianceStatus::Compliant => "compliant",
            ComplianceStatus::PartiallyCompliant => "partially",
            ComplianceStatus::NonCompliant => "non-compliant",
            _ => "",
        };

        let priority_class = if entry.priority == RequirementPriority::Critical {
            "critical"
        } else {
            ""
        };

        html.push_str(&format!(
            "<tr><td>{}</td><td>{}</td><td>{:?}</td><td class=\"{}\">{:?}</td><td class=\"{}\">{:?}</td></tr>\n",
            entry.requirement_id,
            html_escape(&entry.requirement_text),
            entry.category,
            priority_class,
            entry.priority,
            status_class,
            entry.status,
        ));
    }

    html.push_str("</tbody>\n</table>\n</body>\n</html>");
    html
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

// === Tauri Commands ===

/// Generate compliance matrix from analysis
#[tauri::command]
pub fn generate_matrix(analysis: RfpAnalysis, project_name: String) -> Result<ComplianceMatrix, String> {
    Ok(generate_compliance_matrix(&analysis, &project_name))
}

/// Update a compliance entry
#[tauri::command]
pub fn update_compliance_entry(
    mut matrix: ComplianceMatrix,
    requirement_id: String,
    status: ComplianceStatus,
    response_section: Option<String>,
    response_summary: Option<String>,
    comments: Option<String>,
) -> Result<ComplianceMatrix, String> {
    if let Some(entry) = matrix.entries.iter_mut().find(|e| e.requirement_id == requirement_id) {
        entry.status = status;
        if response_section.is_some() {
            entry.response_section = response_section;
        }
        if response_summary.is_some() {
            entry.response_summary = response_summary;
        }
        if comments.is_some() {
            entry.comments = comments;
        }
    }

    // Recalculate summary
    matrix.summary = calculate_summary(&matrix.entries);
    matrix.updated_at = chrono::Utc::now().to_rfc3339();

    Ok(matrix)
}

/// Perform gap analysis
#[tauri::command]
pub fn analyze_gaps(matrix: ComplianceMatrix) -> Result<GapAnalysis, String> {
    Ok(perform_gap_analysis(&matrix))
}

/// Export compliance matrix
#[tauri::command]
pub fn export_matrix(matrix: ComplianceMatrix, format: String) -> Result<ComplianceExport, String> {
    let export_format = match format.to_lowercase().as_str() {
        "csv" => ExportFormat::Csv,
        "json" => ExportFormat::Json,
        "markdown" | "md" => ExportFormat::Markdown,
        "html" => ExportFormat::Html,
        _ => return Err("Unsupported format. Use: csv, json, markdown, or html".to_string()),
    };

    Ok(export_compliance_matrix(&matrix, export_format))
}

/// Calculate compliance summary
#[tauri::command]
pub fn get_compliance_summary(matrix: ComplianceMatrix) -> Result<ComplianceSummary, String> {
    Ok(matrix.summary)
}
