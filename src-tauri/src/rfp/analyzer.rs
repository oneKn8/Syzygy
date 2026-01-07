//! RFP Document Analyzer
//!
//! Extracts requirements, deadlines, and critical information from RFP documents.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use regex::Regex;

/// Represents a requirement extracted from an RFP
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Requirement {
    pub id: String,
    pub text: String,
    pub category: RequirementCategory,
    pub priority: RequirementPriority,
    pub section: Option<String>,
    pub page: Option<u32>,
    pub keywords: Vec<String>,
    pub compliance_status: ComplianceStatus,
    pub response_text: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum RequirementCategory {
    Technical,
    Management,
    Financial,
    Legal,
    Staffing,
    Timeline,
    Deliverable,
    Qualification,
    Experience,
    Security,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum RequirementPriority {
    Critical,
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ComplianceStatus {
    NotStarted,
    InProgress,
    Compliant,
    PartiallyCompliant,
    NonCompliant,
    NotApplicable,
}

/// Deadline or milestone extracted from an RFP
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deadline {
    pub id: String,
    pub description: String,
    pub date: Option<String>,
    pub deadline_type: DeadlineType,
    pub is_mandatory: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DeadlineType {
    Submission,
    Question,
    SiteVisit,
    Award,
    ContractStart,
    Deliverable,
    Other,
}

/// Critical term found in the document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CriticalTerm {
    pub term: String,
    pub term_type: CriticalTermType,
    pub occurrences: Vec<TermOccurrence>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TermOccurrence {
    pub context: String,
    pub page: Option<u32>,
    pub position: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CriticalTermType {
    MustRequirement,
    ShallRequirement,
    WillRequirement,
    MayRequirement,
    Penalty,
    Bonus,
    Disqualification,
    Evaluation,
    Legal,
}

/// RFP Analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RfpAnalysis {
    pub document_name: String,
    pub requirements: Vec<Requirement>,
    pub deadlines: Vec<Deadline>,
    pub critical_terms: Vec<CriticalTerm>,
    pub evaluation_criteria: Vec<EvaluationCriterion>,
    pub summary: DocumentSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationCriterion {
    pub name: String,
    pub description: Option<String>,
    pub weight: Option<f32>,
    pub max_points: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentSummary {
    pub total_requirements: usize,
    pub critical_requirements: usize,
    pub total_deadlines: usize,
    pub has_mandatory_deadlines: bool,
    pub estimated_effort: Option<String>,
    pub risk_level: RiskLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// Analyzes text content and extracts RFP requirements
pub fn analyze_rfp_text(content: &str, document_name: &str) -> RfpAnalysis {
    let requirements = extract_requirements(content);
    let deadlines = extract_deadlines(content);
    let critical_terms = extract_critical_terms(content);
    let evaluation_criteria = extract_evaluation_criteria(content);

    let critical_count = requirements
        .iter()
        .filter(|r| r.priority == RequirementPriority::Critical)
        .count();

    let has_mandatory_deadlines = deadlines.iter().any(|d| d.is_mandatory);

    let risk_level = calculate_risk_level(&requirements, &deadlines);

    RfpAnalysis {
        document_name: document_name.to_string(),
        summary: DocumentSummary {
            total_requirements: requirements.len(),
            critical_requirements: critical_count,
            total_deadlines: deadlines.len(),
            has_mandatory_deadlines,
            estimated_effort: estimate_effort(&requirements),
            risk_level,
        },
        requirements,
        deadlines,
        critical_terms,
        evaluation_criteria,
    }
}

fn extract_requirements(content: &str) -> Vec<Requirement> {
    let mut requirements = Vec::new();
    let mut req_id = 1;

    // Patterns for different types of requirements
    let patterns = [
        // "shall" requirements (strongest)
        (r"(?i)(?:the\s+)?(?:contractor|vendor|offeror|bidder|proposer|provider)\s+shall\s+([^.]+\.)", RequirementPriority::Critical, CriticalTermType::ShallRequirement),
        // "must" requirements
        (r"(?i)(?:the\s+)?(?:contractor|vendor|offeror|bidder|proposer|provider)\s+must\s+([^.]+\.)", RequirementPriority::Critical, CriticalTermType::MustRequirement),
        // "will" requirements
        (r"(?i)(?:the\s+)?(?:contractor|vendor|offeror|bidder|proposer|provider)\s+will\s+([^.]+\.)", RequirementPriority::High, CriticalTermType::WillRequirement),
        // "required to" phrases
        (r"(?i)(?:is|are)\s+required\s+to\s+([^.]+\.)", RequirementPriority::High, CriticalTermType::MustRequirement),
        // Numbered requirements (e.g., "1.1 The system shall...")
        (r"(?m)^\s*\d+\.[\d.]*\s+(?:the\s+)?(?:system|solution|application)\s+(?:shall|must|will)\s+([^.]+\.)", RequirementPriority::High, CriticalTermType::ShallRequirement),
        // "mandatory" items
        (r"(?i)(?:mandatory|required):\s*([^.]+\.)", RequirementPriority::Critical, CriticalTermType::MustRequirement),
    ];

    for (pattern, priority, _term_type) in patterns {
        if let Ok(re) = Regex::new(pattern) {
            for cap in re.captures_iter(content) {
                if let Some(text) = cap.get(0) {
                    let req_text = text.as_str().trim().to_string();

                    // Skip if we already have this requirement
                    if requirements.iter().any(|r: &Requirement| r.text == req_text) {
                        continue;
                    }

                    let category = categorize_requirement(&req_text);
                    let keywords = extract_keywords(&req_text);

                    requirements.push(Requirement {
                        id: format!("REQ-{:04}", req_id),
                        text: req_text,
                        category,
                        priority: priority.clone(),
                        section: None,
                        page: None,
                        keywords,
                        compliance_status: ComplianceStatus::NotStarted,
                        response_text: None,
                        notes: None,
                    });
                    req_id += 1;
                }
            }
        }
    }

    requirements
}

fn categorize_requirement(text: &str) -> RequirementCategory {
    let text_lower = text.to_lowercase();

    if text_lower.contains("security") || text_lower.contains("encrypt") || text_lower.contains("authentication") || text_lower.contains("clearance") {
        RequirementCategory::Security
    } else if text_lower.contains("staff") || text_lower.contains("personnel") || text_lower.contains("team") || text_lower.contains("resource") {
        RequirementCategory::Staffing
    } else if text_lower.contains("deliver") || text_lower.contains("report") || text_lower.contains("document") {
        RequirementCategory::Deliverable
    } else if text_lower.contains("schedule") || text_lower.contains("timeline") || text_lower.contains("milestone") || text_lower.contains("deadline") {
        RequirementCategory::Timeline
    } else if text_lower.contains("cost") || text_lower.contains("price") || text_lower.contains("budget") || text_lower.contains("payment") {
        RequirementCategory::Financial
    } else if text_lower.contains("experience") || text_lower.contains("past performance") || text_lower.contains("reference") {
        RequirementCategory::Experience
    } else if text_lower.contains("certif") || text_lower.contains("license") || text_lower.contains("qualified") {
        RequirementCategory::Qualification
    } else if text_lower.contains("manage") || text_lower.contains("plan") || text_lower.contains("approach") || text_lower.contains("methodology") {
        RequirementCategory::Management
    } else if text_lower.contains("law") || text_lower.contains("regulation") || text_lower.contains("compliance") || text_lower.contains("federal") {
        RequirementCategory::Legal
    } else if text_lower.contains("system") || text_lower.contains("software") || text_lower.contains("hardware") || text_lower.contains("technical") || text_lower.contains("implement") {
        RequirementCategory::Technical
    } else {
        RequirementCategory::Other
    }
}

fn extract_keywords(text: &str) -> Vec<String> {
    let stop_words: std::collections::HashSet<&str> = [
        "the", "a", "an", "and", "or", "but", "in", "on", "at", "to", "for",
        "of", "with", "by", "from", "as", "is", "was", "are", "were", "been",
        "be", "have", "has", "had", "do", "does", "did", "will", "would",
        "could", "should", "may", "might", "must", "shall", "can", "this",
        "that", "these", "those", "it", "its", "their", "they", "them",
        "contractor", "vendor", "offeror", "bidder", "proposer", "provider",
    ].iter().cloned().collect();

    let word_re = Regex::new(r"\b[a-zA-Z]{3,}\b").unwrap();

    word_re
        .find_iter(text)
        .map(|m| m.as_str().to_lowercase())
        .filter(|w| !stop_words.contains(w.as_str()))
        .take(10)
        .collect()
}

fn extract_deadlines(content: &str) -> Vec<Deadline> {
    let mut deadlines = Vec::new();
    let mut dl_id = 1;

    // Date patterns
    let date_patterns = [
        r"(?i)((?:submission|proposal|response|bid)\s+(?:deadline|due\s+date)[:\s]+)([^.\n]+)",
        r"(?i)((?:questions?\s+)?(?:due|deadline|must\s+be\s+(?:received|submitted))\s+(?:by|on|no\s+later\s+than)[:\s]+)([^.\n]+)",
        r"(?i)((?:award|contract\s+start|period\s+of\s+performance)[:\s]+)([^.\n]+)",
        r"(?i)(\d{1,2}/\d{1,2}/\d{2,4})",
        r"(?i)((?:january|february|march|april|may|june|july|august|september|october|november|december)\s+\d{1,2},?\s+\d{4})",
    ];

    for pattern in date_patterns {
        if let Ok(re) = Regex::new(pattern) {
            for cap in re.captures_iter(content) {
                let full_match = cap.get(0).map(|m| m.as_str()).unwrap_or("");
                let full_lower = full_match.to_lowercase();

                let deadline_type = if full_lower.contains("submission") || full_lower.contains("proposal") || full_lower.contains("response") || full_lower.contains("bid") {
                    DeadlineType::Submission
                } else if full_lower.contains("question") {
                    DeadlineType::Question
                } else if full_lower.contains("site visit") {
                    DeadlineType::SiteVisit
                } else if full_lower.contains("award") {
                    DeadlineType::Award
                } else if full_lower.contains("contract start") || full_lower.contains("period of performance") {
                    DeadlineType::ContractStart
                } else {
                    DeadlineType::Other
                };

                let is_mandatory = full_lower.contains("mandatory") ||
                    full_lower.contains("must be") ||
                    full_lower.contains("no later than") ||
                    matches!(deadline_type, DeadlineType::Submission);

                deadlines.push(Deadline {
                    id: format!("DL-{:04}", dl_id),
                    description: full_match.trim().to_string(),
                    date: Some(full_match.to_string()),
                    deadline_type,
                    is_mandatory,
                });
                dl_id += 1;
            }
        }
    }

    deadlines
}

fn extract_critical_terms(content: &str) -> Vec<CriticalTerm> {
    let mut terms: HashMap<String, CriticalTerm> = HashMap::new();

    let term_patterns = [
        ("shall", CriticalTermType::ShallRequirement),
        ("must", CriticalTermType::MustRequirement),
        ("will", CriticalTermType::WillRequirement),
        ("may", CriticalTermType::MayRequirement),
        ("penalty", CriticalTermType::Penalty),
        ("liquidated damages", CriticalTermType::Penalty),
        ("bonus", CriticalTermType::Bonus),
        ("incentive", CriticalTermType::Bonus),
        ("disqualif", CriticalTermType::Disqualification),
        ("non-responsive", CriticalTermType::Disqualification),
        ("evaluation criteria", CriticalTermType::Evaluation),
        ("scoring", CriticalTermType::Evaluation),
        ("points", CriticalTermType::Evaluation),
    ];

    for (term, term_type) in term_patterns {
        let pattern = format!(r"(?i)\b{}\b", regex::escape(term));
        if let Ok(re) = Regex::new(&pattern) {
            let occurrences: Vec<TermOccurrence> = re
                .find_iter(content)
                .map(|m| {
                    let start = m.start().saturating_sub(50);
                    let end = (m.end() + 50).min(content.len());
                    let context = content[start..end].to_string();
                    TermOccurrence {
                        context,
                        page: None,
                        position: m.start(),
                    }
                })
                .take(10)
                .collect();

            if !occurrences.is_empty() {
                terms.insert(
                    term.to_string(),
                    CriticalTerm {
                        term: term.to_string(),
                        term_type: term_type.clone(),
                        occurrences,
                    },
                );
            }
        }
    }

    terms.into_values().collect()
}

fn extract_evaluation_criteria(content: &str) -> Vec<EvaluationCriterion> {
    let mut criteria = Vec::new();

    // Pattern for evaluation criteria sections
    let patterns = [
        r"(?i)(?:evaluation\s+(?:criteria|factors?))[:\s]+([^.]+\.)",
        r"(?i)(\d+)\s*(?:points?|%)\s*[-:]\s*([^.\n]+)",
        r"(?i)(technical|management|cost|price|past\s+performance|experience)\s*[-:]\s*(\d+)\s*(?:points?|%)?",
    ];

    for pattern in patterns {
        if let Ok(re) = Regex::new(pattern) {
            for cap in re.captures_iter(content) {
                let name = cap.get(1).or(cap.get(2)).map(|m| m.as_str().trim().to_string());
                let weight = cap.get(2).or(cap.get(1)).and_then(|m| m.as_str().parse::<f32>().ok());

                if let Some(name) = name {
                    criteria.push(EvaluationCriterion {
                        name,
                        description: None,
                        weight,
                        max_points: weight.map(|w| w as u32),
                    });
                }
            }
        }
    }

    criteria
}

fn calculate_risk_level(requirements: &[Requirement], deadlines: &[Deadline]) -> RiskLevel {
    let critical_count = requirements
        .iter()
        .filter(|r| r.priority == RequirementPriority::Critical)
        .count();

    let mandatory_deadlines = deadlines.iter().filter(|d| d.is_mandatory).count();

    let total_reqs = requirements.len();

    if critical_count > 20 || mandatory_deadlines > 5 || total_reqs > 100 {
        RiskLevel::Critical
    } else if critical_count > 10 || mandatory_deadlines > 3 || total_reqs > 50 {
        RiskLevel::High
    } else if critical_count > 5 || mandatory_deadlines > 1 || total_reqs > 20 {
        RiskLevel::Medium
    } else {
        RiskLevel::Low
    }
}

fn estimate_effort(requirements: &[Requirement]) -> Option<String> {
    let count = requirements.len();

    let effort = if count > 100 {
        "Very High (100+ requirements)"
    } else if count > 50 {
        "High (50-100 requirements)"
    } else if count > 20 {
        "Medium (20-50 requirements)"
    } else if count > 0 {
        "Low (< 20 requirements)"
    } else {
        return None;
    };

    Some(effort.to_string())
}

// === Tauri Commands ===

/// Analyze RFP document text
#[tauri::command]
pub fn analyze_rfp_document(content: String, document_name: String) -> Result<RfpAnalysis, String> {
    Ok(analyze_rfp_text(&content, &document_name))
}

/// Extract requirements only
#[tauri::command]
pub fn extract_rfp_requirements(content: String) -> Result<Vec<Requirement>, String> {
    Ok(extract_requirements(&content))
}

/// Extract deadlines only
#[tauri::command]
pub fn extract_rfp_deadlines(content: String) -> Result<Vec<Deadline>, String> {
    Ok(extract_deadlines(&content))
}

/// Extract critical terms only
#[tauri::command]
pub fn extract_rfp_critical_terms(content: String) -> Result<Vec<CriticalTerm>, String> {
    Ok(extract_critical_terms(&content))
}

/// Update requirement compliance status
#[tauri::command]
pub fn update_requirement_status(
    mut requirement: Requirement,
    status: ComplianceStatus,
    response_text: Option<String>,
    notes: Option<String>,
) -> Result<Requirement, String> {
    requirement.compliance_status = status;
    if response_text.is_some() {
        requirement.response_text = response_text;
    }
    if notes.is_some() {
        requirement.notes = notes;
    }
    Ok(requirement)
}
