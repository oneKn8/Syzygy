//! Project templates for quick project setup

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub doc_type: String,
    pub files: Vec<TemplateFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateFile {
    pub path: String,
    pub content: String,
}

/// Get all available project templates
#[tauri::command]
pub fn get_templates() -> Vec<ProjectTemplate> {
    vec![
        // Basic Typst Document
        ProjectTemplate {
            id: "typst-basic".to_string(),
            name: "Basic Typst Document".to_string(),
            description: "A simple Typst document with basic formatting".to_string(),
            doc_type: "typst".to_string(),
            files: vec![TemplateFile {
                path: "main.typ".to_string(),
                content: r#"#set document(title: "{{PROJECT_NAME}}")
#set page(paper: "us-letter", margin: 1in)
#set text(font: "New Computer Modern", size: 11pt)

= {{PROJECT_NAME}}

Your content starts here.

== Section One

Write your first section here.

== Section Two

Write your second section here.
"#.to_string(),
            }],
        },

        // RFP Response Template
        ProjectTemplate {
            id: "rfp-response".to_string(),
            name: "RFP Response".to_string(),
            description: "Professional RFP response template with cover letter and sections".to_string(),
            doc_type: "typst".to_string(),
            files: vec![
                TemplateFile {
                    path: "main.typ".to_string(),
                    content: r#"#set document(
  title: "{{PROJECT_NAME}}",
  author: "Your Company Name"
)
#set page(
  paper: "us-letter",
  margin: (top: 1in, bottom: 1in, left: 1.25in, right: 1in),
  header: [
    #set text(8pt)
    #h(1fr) Response to RFP
  ],
  footer: [
    #set text(8pt)
    Your Company Name #h(1fr) Page #counter(page).display()
  ]
)
#set text(font: "New Computer Modern", size: 11pt)
#set par(justify: true)

#include "cover-letter.typ"
#pagebreak()

#include "executive-summary.typ"
#pagebreak()

#include "technical-approach.typ"
#pagebreak()

#include "past-performance.typ"
#pagebreak()

#include "pricing.typ"
"#.to_string(),
                },
                TemplateFile {
                    path: "cover-letter.typ".to_string(),
                    content: r#"#align(center)[
  #text(size: 18pt, weight: "bold")[Response to Request for Proposal]

  #v(0.5em)

  #text(size: 14pt)[{{PROJECT_NAME}}]
]

#v(2em)

#text(weight: "bold")[Submitted to:]

Agency/Organization Name \
Address Line 1 \
City, State ZIP

#v(1em)

#text(weight: "bold")[Submitted by:]

Your Company Name \
Your Address \
City, State ZIP

#v(1em)

#text(weight: "bold")[Date:] #datetime.today().display()

#v(2em)

Dear Selection Committee,

We are pleased to submit this proposal in response to your Request for Proposal. Our team brings extensive experience and expertise to deliver exceptional results for your organization.

Please find enclosed our complete response addressing all requirements outlined in your RFP. We are confident that our approach will meet and exceed your expectations.

We look forward to the opportunity to discuss our proposal in more detail.

Sincerely,

#v(3em)

Authorized Representative \
Title \
Your Company Name
"#.to_string(),
                },
                TemplateFile {
                    path: "executive-summary.typ".to_string(),
                    content: r#"= Executive Summary

== Understanding of Requirements

Provide a clear summary of your understanding of the client's needs and requirements.

== Proposed Solution

Describe your proposed solution at a high level.

== Key Benefits

- Benefit 1
- Benefit 2
- Benefit 3

== Why Choose Us

Explain why your organization is the best choice for this project.
"#.to_string(),
                },
                TemplateFile {
                    path: "technical-approach.typ".to_string(),
                    content: r#"= Technical Approach

== Methodology

Describe your technical methodology and approach.

== Implementation Plan

=== Phase 1: Discovery
Timeline and deliverables

=== Phase 2: Development
Timeline and deliverables

=== Phase 3: Deployment
Timeline and deliverables

== Team Structure

Describe your team's organization and key personnel.

== Quality Assurance

Describe your quality assurance processes.
"#.to_string(),
                },
                TemplateFile {
                    path: "past-performance.typ".to_string(),
                    content: r#"= Past Performance

== Relevant Experience

=== Project 1: [Project Name]
- *Client:* Client Name
- *Duration:* Start - End
- *Scope:* Brief description
- *Results:* Key outcomes

=== Project 2: [Project Name]
- *Client:* Client Name
- *Duration:* Start - End
- *Scope:* Brief description
- *Results:* Key outcomes

== References

Contact information for references available upon request.
"#.to_string(),
                },
                TemplateFile {
                    path: "pricing.typ".to_string(),
                    content: r#"= Pricing

== Cost Summary

#table(
  columns: (auto, 1fr, auto),
  inset: 10pt,
  align: (left, left, right),
  [*Item*], [*Description*], [*Cost*],
  [Phase 1], [Discovery and Planning], [\$X,XXX],
  [Phase 2], [Development], [\$X,XXX],
  [Phase 3], [Deployment], [\$X,XXX],
  table.hline(),
  [*Total*], [], [*\$XX,XXX*],
)

== Payment Terms

Describe your payment terms and conditions.

== Assumptions

List any assumptions underlying your pricing.
"#.to_string(),
                },
            ],
        },

        // Academic Paper Template
        ProjectTemplate {
            id: "academic-paper".to_string(),
            name: "Academic Paper".to_string(),
            description: "Template for academic papers with citations".to_string(),
            doc_type: "typst".to_string(),
            files: vec![
                TemplateFile {
                    path: "main.typ".to_string(),
                    content: r#"#set document(
  title: "{{PROJECT_NAME}}",
  author: ("Author Name",)
)
#set page(paper: "us-letter", margin: 1in)
#set text(font: "New Computer Modern", size: 12pt)
#set par(justify: true, leading: 1.5em)
#set heading(numbering: "1.1")

#align(center)[
  #text(size: 16pt, weight: "bold")[{{PROJECT_NAME}}]

  #v(1em)

  Author Name \
  Institution Name \
  #link("mailto:author@email.com")

  #v(0.5em)

  #datetime.today().display("[month repr:long] [day], [year]")
]

#v(2em)

#heading(outlined: false, numbering: none)[Abstract]

Write your abstract here. This should be a concise summary of your paper, typically 150-300 words.

#v(1em)

*Keywords:* keyword1, keyword2, keyword3

#pagebreak()

= Introduction

Introduce your research topic and objectives.

= Literature Review

Review relevant prior work in the field.

= Methodology

Describe your research methodology.

= Results

Present your findings.

= Discussion

Discuss the implications of your results.

= Conclusion

Summarize your conclusions and suggest future work.

#pagebreak()

#heading(outlined: false, numbering: none)[References]

Add your references here.
"#.to_string(),
                },
            ],
        },

        // Technical Documentation Template
        ProjectTemplate {
            id: "technical-docs".to_string(),
            name: "Technical Documentation".to_string(),
            description: "Template for software/technical documentation".to_string(),
            doc_type: "typst".to_string(),
            files: vec![
                TemplateFile {
                    path: "main.typ".to_string(),
                    content: r#"#set document(title: "{{PROJECT_NAME}}")
#set page(paper: "us-letter", margin: 1in)
#set text(font: "New Computer Modern", size: 11pt)
#set heading(numbering: "1.1")

#align(center)[
  #text(size: 20pt, weight: "bold")[{{PROJECT_NAME}}]

  #v(0.5em)

  Technical Documentation

  #v(0.5em)

  Version 1.0
]

#v(2em)

#outline(indent: auto)

#pagebreak()

= Overview

Brief overview of the system/software.

= Getting Started

== Requirements

List system requirements.

== Installation

Installation instructions.

== Quick Start

Quick start guide.

= Architecture

== System Architecture

Describe the overall architecture.

== Components

=== Component A

Description and responsibilities.

=== Component B

Description and responsibilities.

= API Reference

== Endpoints

=== GET /api/resource

Description of the endpoint.

*Parameters:*
- `param1` - Description

*Response:*
```json
{
  "status": "success"
}
```

= Troubleshooting

Common issues and solutions.

= Changelog

== Version 1.0
- Initial release
"#.to_string(),
                },
            ],
        },

        // Business Proposal Template
        ProjectTemplate {
            id: "business-proposal".to_string(),
            name: "Business Proposal".to_string(),
            description: "Professional business proposal template".to_string(),
            doc_type: "typst".to_string(),
            files: vec![TemplateFile {
                path: "main.typ".to_string(),
                content: r#"#set document(title: "{{PROJECT_NAME}}")
#set page(
  paper: "us-letter",
  margin: 1in,
  footer: [
    #set text(8pt)
    #h(1fr) Page #counter(page).display() of #locate(loc => counter(page).final(loc).first())
  ]
)
#set text(font: "New Computer Modern", size: 11pt)
#set par(justify: true)

#v(4em)

#align(center)[
  #text(size: 24pt, weight: "bold")[Business Proposal]

  #v(1em)

  #text(size: 16pt)[{{PROJECT_NAME}}]

  #v(2em)

  Prepared for: Client Name

  #v(1em)

  Prepared by: Your Company Name

  #v(1em)

  #datetime.today().display("[month repr:long] [day], [year]")
]

#pagebreak()

#outline(indent: auto)

#pagebreak()

= Executive Summary

Provide a compelling summary of your proposal.

= Problem Statement

Describe the challenge or opportunity.

= Proposed Solution

Detail your proposed solution.

= Benefits

== For Your Organization

- Benefit 1
- Benefit 2

== Return on Investment

Describe expected ROI.

= Implementation Timeline

#table(
  columns: (auto, 1fr, auto),
  [*Phase*], [*Activities*], [*Timeline*],
  [Phase 1], [Planning], [Week 1-2],
  [Phase 2], [Execution], [Week 3-6],
  [Phase 3], [Review], [Week 7-8],
)

= Investment

Detail pricing and payment terms.

= Next Steps

Outline the path forward.

= About Us

Brief company background and qualifications.
"#.to_string(),
            }],
        },
    ]
}

/// Create a project from a template
#[tauri::command]
pub async fn create_from_template(
    template_id: String,
    project_path: String,
    project_name: String,
) -> Result<(), String> {
    let templates = get_templates();
    let template = templates
        .iter()
        .find(|t| t.id == template_id)
        .ok_or_else(|| format!("Template not found: {}", template_id))?;

    let path = PathBuf::from(&project_path);

    // Create project directory
    fs::create_dir_all(&path).map_err(|e| format!("Failed to create directory: {}", e))?;

    // Create output directory
    fs::create_dir_all(path.join("output")).map_err(|e| format!("Failed to create output directory: {}", e))?;

    // Create template files
    for file in &template.files {
        let file_path = path.join(&file.path);

        // Create parent directories if needed
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("Failed to create parent directory: {}", e))?;
        }

        // Replace template variables
        let content = file
            .content
            .replace("{{PROJECT_NAME}}", &project_name);

        fs::write(&file_path, content).map_err(|e| format!("Failed to write file: {}", e))?;
    }

    // Create project config
    let config = serde_json::json!({
        "name": project_name,
        "doc_type": template.doc_type,
        "template": template_id,
        "main_file": template.files.first().map(|f| &f.path).unwrap_or(&String::new()),
        "output_dir": "output"
    });

    let config_path = path.join("rfpmaker.json");
    let config_json = serde_json::to_string_pretty(&config).map_err(|e| e.to_string())?;
    fs::write(&config_path, config_json).map_err(|e| format!("Failed to write config: {}", e))?;

    log::info!("Created project from template '{}' at {:?}", template_id, path);

    Ok(())
}
