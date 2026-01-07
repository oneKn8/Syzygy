//! Document Export Module
//!
//! Exports documents to various formats including PDF, HTML, Markdown, and more.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Export configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportConfig {
    pub format: ExportFormat,
    pub output_path: Option<String>,
    pub options: ExportOptions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExportFormat {
    Pdf,
    Html,
    Markdown,
    Docx,
    Epub,
    Latex,
    PlainText,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExportOptions {
    /// Include table of contents
    #[serde(default)]
    pub include_toc: bool,
    /// Include page numbers (for PDF/DOCX)
    #[serde(default)]
    pub include_page_numbers: bool,
    /// Document title
    pub title: Option<String>,
    /// Document author
    pub author: Option<String>,
    /// Paper size for PDF
    pub paper_size: Option<String>,
    /// Custom CSS for HTML export
    pub custom_css: Option<String>,
    /// Page margins
    pub margins: Option<PageMargins>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageMargins {
    pub top: f64,
    pub bottom: f64,
    pub left: f64,
    pub right: f64,
}

/// Export result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportResult {
    pub success: bool,
    pub format: ExportFormat,
    pub output_path: Option<String>,
    pub content: Option<String>,
    pub binary_data: Option<Vec<u8>>,
    pub error: Option<String>,
}

/// Export Typst source to HTML
pub fn export_to_html(source: &str, options: &ExportOptions) -> ExportResult {
    let title = options.title.as_deref().unwrap_or("Document");
    let custom_css = options.custom_css.as_deref().unwrap_or("");

    let default_css = r#"
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, sans-serif;
            max-width: 800px;
            margin: 0 auto;
            padding: 2rem;
            line-height: 1.6;
            color: #333;
        }
        h1, h2, h3, h4, h5, h6 { color: #1a1a1a; margin-top: 1.5em; }
        h1 { border-bottom: 2px solid #eee; padding-bottom: 0.3em; }
        code { background: #f4f4f4; padding: 0.2em 0.4em; border-radius: 3px; }
        pre { background: #f4f4f4; padding: 1em; border-radius: 5px; overflow-x: auto; }
        blockquote { border-left: 4px solid #ddd; margin-left: 0; padding-left: 1em; color: #666; }
        table { border-collapse: collapse; width: 100%; }
        th, td { border: 1px solid #ddd; padding: 0.5em; text-align: left; }
        th { background: #f4f4f4; }
    "#;

    // Simple Typst to HTML conversion
    // Note: A full implementation would parse Typst AST
    let body_content = convert_typst_to_html_basic(source);

    let html = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{}</title>
    <style>
        {}
        {}
    </style>
</head>
<body>
{}
</body>
</html>"#,
        html_escape(title),
        default_css,
        custom_css,
        body_content
    );

    ExportResult {
        success: true,
        format: ExportFormat::Html,
        output_path: None,
        content: Some(html),
        binary_data: None,
        error: None,
    }
}

/// Basic Typst to HTML conversion (simplified)
fn convert_typst_to_html_basic(source: &str) -> String {
    let mut html = String::new();
    let mut in_code_block = false;

    for line in source.lines() {
        let trimmed = line.trim();

        // Code blocks
        if trimmed.starts_with("```") {
            if in_code_block {
                html.push_str("</code></pre>\n");
                in_code_block = false;
            } else {
                let lang = trimmed.strip_prefix("```").unwrap_or("");
                html.push_str(&format!("<pre><code class=\"language-{}\">", lang));
                in_code_block = true;
            }
            continue;
        }

        if in_code_block {
            html.push_str(&html_escape(line));
            html.push('\n');
            continue;
        }

        // Headings
        if trimmed.starts_with("= ") {
            html.push_str(&format!("<h1>{}</h1>\n", html_escape(&trimmed[2..])));
        } else if trimmed.starts_with("== ") {
            html.push_str(&format!("<h2>{}</h2>\n", html_escape(&trimmed[3..])));
        } else if trimmed.starts_with("=== ") {
            html.push_str(&format!("<h3>{}</h3>\n", html_escape(&trimmed[4..])));
        } else if trimmed.starts_with("==== ") {
            html.push_str(&format!("<h4>{}</h4>\n", html_escape(&trimmed[5..])));
        }
        // Lists
        else if trimmed.starts_with("- ") {
            html.push_str(&format!("<li>{}</li>\n", html_escape(&trimmed[2..])));
        } else if trimmed.starts_with("+ ") {
            html.push_str(&format!("<li>{}</li>\n", html_escape(&trimmed[2..])));
        }
        // Blockquote
        else if trimmed.starts_with("> ") {
            html.push_str(&format!("<blockquote>{}</blockquote>\n", html_escape(&trimmed[2..])));
        }
        // Empty line
        else if trimmed.is_empty() {
            html.push_str("<br>\n");
        }
        // Regular paragraph
        else if !trimmed.starts_with('#') && !trimmed.starts_with("//") {
            // Convert inline formatting
            let formatted = convert_inline_formatting(trimmed);
            html.push_str(&format!("<p>{}</p>\n", formatted));
        }
    }

    html
}

/// Convert inline Typst formatting to HTML
fn convert_inline_formatting(text: &str) -> String {
    let mut result = html_escape(text);

    // Bold: *text* -> <strong>text</strong>
    result = regex_replace(&result, r"\*([^*]+)\*", "<strong>$1</strong>");

    // Italic: _text_ -> <em>text</em>
    result = regex_replace(&result, r"_([^_]+)_", "<em>$1</em>");

    // Code: `text` -> <code>text</code>
    result = regex_replace(&result, r"`([^`]+)`", "<code>$1</code>");

    // Links: #link("url")[text] -> <a href="url">text</a>
    result = regex_replace(&result, r#"#link\("([^"]+)"\)\[([^\]]+)\]"#, "<a href=\"$1\">$2</a>");

    result
}

fn regex_replace(text: &str, pattern: &str, replacement: &str) -> String {
    if let Ok(re) = regex::Regex::new(pattern) {
        re.replace_all(text, replacement).to_string()
    } else {
        text.to_string()
    }
}

fn html_escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Export to Markdown
pub fn export_to_markdown(source: &str, options: &ExportOptions) -> ExportResult {
    let mut md = String::new();

    // Add frontmatter if title/author provided
    if options.title.is_some() || options.author.is_some() {
        md.push_str("---\n");
        if let Some(ref title) = options.title {
            md.push_str(&format!("title: \"{}\"\n", title));
        }
        if let Some(ref author) = options.author {
            md.push_str(&format!("author: \"{}\"\n", author));
        }
        md.push_str("---\n\n");
    }

    // Convert Typst to Markdown
    for line in source.lines() {
        let trimmed = line.trim();

        // Headings: = -> #
        if trimmed.starts_with("==== ") {
            md.push_str(&format!("#### {}\n", &trimmed[5..]));
        } else if trimmed.starts_with("=== ") {
            md.push_str(&format!("### {}\n", &trimmed[4..]));
        } else if trimmed.starts_with("== ") {
            md.push_str(&format!("## {}\n", &trimmed[3..]));
        } else if trimmed.starts_with("= ") {
            md.push_str(&format!("# {}\n", &trimmed[2..]));
        }
        // Lists: keep as-is (Markdown compatible)
        else if trimmed.starts_with("- ") || trimmed.starts_with("+ ") {
            md.push_str(line);
            md.push('\n');
        }
        // Blockquote: keep as-is
        else if trimmed.starts_with("> ") {
            md.push_str(line);
            md.push('\n');
        }
        // Code blocks: keep as-is
        else if trimmed.starts_with("```") {
            md.push_str(line);
            md.push('\n');
        }
        // Skip Typst-specific directives
        else if trimmed.starts_with("#set") || trimmed.starts_with("#show") || trimmed.starts_with("#import") {
            continue;
        }
        // Regular content
        else {
            // Convert Typst inline formatting to Markdown
            let converted = line
                .replace("#strong[", "**")
                .replace("#emph[", "*")
                .replace("]", "");
            md.push_str(&converted);
            md.push('\n');
        }
    }

    ExportResult {
        success: true,
        format: ExportFormat::Markdown,
        output_path: None,
        content: Some(md),
        binary_data: None,
        error: None,
    }
}

/// Export to LaTeX
pub fn export_to_latex(source: &str, options: &ExportOptions) -> ExportResult {
    let mut latex = String::new();

    // Document preamble
    latex.push_str("\\documentclass[12pt]{article}\n");
    latex.push_str("\\usepackage[utf8]{inputenc}\n");
    latex.push_str("\\usepackage{graphicx}\n");
    latex.push_str("\\usepackage{hyperref}\n");
    latex.push_str("\\usepackage{listings}\n");

    // Page margins
    if let Some(ref margins) = options.margins {
        latex.push_str(&format!(
            "\\usepackage[top={}cm,bottom={}cm,left={}cm,right={}cm]{{geometry}}\n",
            margins.top, margins.bottom, margins.left, margins.right
        ));
    }

    latex.push('\n');

    // Title and author
    if let Some(ref title) = options.title {
        latex.push_str(&format!("\\title{{{}}}\n", latex_escape(title)));
    }
    if let Some(ref author) = options.author {
        latex.push_str(&format!("\\author{{{}}}\n", latex_escape(author)));
    }

    latex.push_str("\n\\begin{document}\n\n");

    if options.title.is_some() {
        latex.push_str("\\maketitle\n\n");
    }

    if options.include_toc {
        latex.push_str("\\tableofcontents\n\\newpage\n\n");
    }

    // Convert Typst to LaTeX
    let mut in_code_block = false;

    for line in source.lines() {
        let trimmed = line.trim();

        // Code blocks
        if trimmed.starts_with("```") {
            if in_code_block {
                latex.push_str("\\end{lstlisting}\n");
                in_code_block = false;
            } else {
                let lang = trimmed.strip_prefix("```").unwrap_or("");
                latex.push_str(&format!("\\begin{{lstlisting}}[language={}]\n", lang));
                in_code_block = true;
            }
            continue;
        }

        if in_code_block {
            latex.push_str(line);
            latex.push('\n');
            continue;
        }

        // Headings
        if trimmed.starts_with("= ") {
            latex.push_str(&format!("\\section{{{}}}\n", latex_escape(&trimmed[2..])));
        } else if trimmed.starts_with("== ") {
            latex.push_str(&format!("\\subsection{{{}}}\n", latex_escape(&trimmed[3..])));
        } else if trimmed.starts_with("=== ") {
            latex.push_str(&format!("\\subsubsection{{{}}}\n", latex_escape(&trimmed[4..])));
        }
        // Lists
        else if trimmed.starts_with("- ") {
            latex.push_str(&format!("\\item {}\n", latex_escape(&trimmed[2..])));
        }
        // Blockquote
        else if trimmed.starts_with("> ") {
            latex.push_str(&format!("\\begin{{quote}}\n{}\n\\end{{quote}}\n", latex_escape(&trimmed[2..])));
        }
        // Skip Typst directives
        else if trimmed.starts_with("#set") || trimmed.starts_with("#show") || trimmed.starts_with("#import") {
            continue;
        }
        // Empty line
        else if trimmed.is_empty() {
            latex.push_str("\n");
        }
        // Regular paragraph
        else {
            latex.push_str(&latex_escape(line));
            latex.push_str("\n\n");
        }
    }

    latex.push_str("\\end{document}\n");

    ExportResult {
        success: true,
        format: ExportFormat::Latex,
        output_path: None,
        content: Some(latex),
        binary_data: None,
        error: None,
    }
}

fn latex_escape(text: &str) -> String {
    text.replace('\\', "\\textbackslash{}")
        .replace('&', "\\&")
        .replace('%', "\\%")
        .replace('$', "\\$")
        .replace('#', "\\#")
        .replace('_', "\\_")
        .replace('{', "\\{")
        .replace('}', "\\}")
        .replace('~', "\\textasciitilde{}")
        .replace('^', "\\textasciicircum{}")
}

/// Export to plain text
pub fn export_to_plain_text(source: &str, _options: &ExportOptions) -> ExportResult {
    let mut text = String::new();

    for line in source.lines() {
        let trimmed = line.trim();

        // Skip Typst directives
        if trimmed.starts_with("#set") || trimmed.starts_with("#show") || trimmed.starts_with("#import") {
            continue;
        }

        // Remove Typst markup
        let cleaned = trimmed
            .trim_start_matches("= ")
            .trim_start_matches("== ")
            .trim_start_matches("=== ")
            .trim_start_matches("==== ")
            .trim_start_matches("- ")
            .trim_start_matches("+ ")
            .trim_start_matches("> ");

        // Remove inline formatting
        let cleaned = cleaned
            .replace("*", "")
            .replace("_", "")
            .replace("`", "");

        text.push_str(&cleaned);
        text.push('\n');
    }

    ExportResult {
        success: true,
        format: ExportFormat::PlainText,
        output_path: None,
        content: Some(text),
        binary_data: None,
        error: None,
    }
}

/// Export document to specified format
pub fn export_document(source: &str, config: &ExportConfig) -> ExportResult {
    let result = match config.format {
        ExportFormat::Html => export_to_html(source, &config.options),
        ExportFormat::Markdown => export_to_markdown(source, &config.options),
        ExportFormat::Latex => export_to_latex(source, &config.options),
        ExportFormat::PlainText => export_to_plain_text(source, &config.options),
        ExportFormat::Pdf => {
            // PDF export is handled by the Typst engine
            return ExportResult {
                success: false,
                format: ExportFormat::Pdf,
                output_path: None,
                content: None,
                binary_data: None,
                error: Some("Use compile_typst_source for PDF export".to_string()),
            };
        }
        ExportFormat::Docx | ExportFormat::Epub => {
            return ExportResult {
                success: false,
                format: config.format.clone(),
                output_path: None,
                content: None,
                binary_data: None,
                error: Some(format!("{:?} export not yet implemented", config.format)),
            };
        }
    };

    // Save to file if output path specified
    if let Some(ref output_path) = config.output_path {
        if let Some(ref content) = result.content {
            if let Err(e) = std::fs::write(output_path, content) {
                return ExportResult {
                    success: false,
                    format: config.format.clone(),
                    output_path: None,
                    content: None,
                    binary_data: None,
                    error: Some(format!("Failed to write file: {}", e)),
                };
            }
            return ExportResult {
                success: true,
                format: config.format.clone(),
                output_path: Some(output_path.clone()),
                content: Some(content.clone()),
                binary_data: None,
                error: None,
            };
        }
    }

    result
}

// === Tauri Commands ===

/// Export document to specified format
#[tauri::command]
pub fn export_to_format(
    source: String,
    format: String,
    output_path: Option<String>,
    options: Option<ExportOptions>,
) -> Result<ExportResult, String> {
    let export_format = match format.to_lowercase().as_str() {
        "pdf" => ExportFormat::Pdf,
        "html" => ExportFormat::Html,
        "markdown" | "md" => ExportFormat::Markdown,
        "latex" | "tex" => ExportFormat::Latex,
        "docx" | "word" => ExportFormat::Docx,
        "epub" => ExportFormat::Epub,
        "text" | "txt" | "plain" => ExportFormat::PlainText,
        _ => return Err(format!("Unknown export format: {}", format)),
    };

    let config = ExportConfig {
        format: export_format,
        output_path,
        options: options.unwrap_or_default(),
    };

    Ok(export_document(&source, &config))
}

/// Get available export formats
#[tauri::command]
pub fn get_export_formats() -> Vec<serde_json::Value> {
    vec![
        serde_json::json!({
            "id": "pdf",
            "name": "PDF",
            "description": "Portable Document Format - best for printing and sharing",
            "extension": ".pdf",
            "binary": true
        }),
        serde_json::json!({
            "id": "html",
            "name": "HTML",
            "description": "Web page format - viewable in any browser",
            "extension": ".html",
            "binary": false
        }),
        serde_json::json!({
            "id": "markdown",
            "name": "Markdown",
            "description": "Plain text with formatting - great for documentation",
            "extension": ".md",
            "binary": false
        }),
        serde_json::json!({
            "id": "latex",
            "name": "LaTeX",
            "description": "LaTeX source - for academic and scientific documents",
            "extension": ".tex",
            "binary": false
        }),
        serde_json::json!({
            "id": "text",
            "name": "Plain Text",
            "description": "Plain text without formatting",
            "extension": ".txt",
            "binary": false
        }),
        serde_json::json!({
            "id": "docx",
            "name": "Word (DOCX)",
            "description": "Microsoft Word format - coming soon",
            "extension": ".docx",
            "binary": true,
            "available": false
        }),
        serde_json::json!({
            "id": "epub",
            "name": "EPUB",
            "description": "E-book format - coming soon",
            "extension": ".epub",
            "binary": true,
            "available": false
        }),
    ]
}

/// Batch export to multiple formats
#[tauri::command]
pub fn batch_export(
    source: String,
    formats: Vec<String>,
    output_dir: String,
    base_name: String,
    options: Option<ExportOptions>,
) -> Result<Vec<ExportResult>, String> {
    let mut results = Vec::new();
    let opts = options.unwrap_or_default();

    for format in formats {
        let extension = match format.to_lowercase().as_str() {
            "pdf" => ".pdf",
            "html" => ".html",
            "markdown" | "md" => ".md",
            "latex" | "tex" => ".tex",
            "text" | "txt" | "plain" => ".txt",
            _ => continue,
        };

        let output_path = PathBuf::from(&output_dir)
            .join(format!("{}{}", base_name, extension))
            .to_string_lossy()
            .to_string();

        match export_to_format(
            source.clone(),
            format,
            Some(output_path),
            Some(opts.clone()),
        ) {
            Ok(result) => results.push(result),
            Err(e) => results.push(ExportResult {
                success: false,
                format: ExportFormat::PlainText,
                output_path: None,
                content: None,
                binary_data: None,
                error: Some(e),
            }),
        }
    }

    Ok(results)
}
