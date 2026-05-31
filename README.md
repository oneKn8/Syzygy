<p align="center">
  <img src="assets/logo.svg" alt="Syzygy" width="120" height="120" />
</p>

<h1 align="center">Syzygy</h1>

<p align="center">
  <strong>When documents align perfectly.</strong>
</p>

<p align="center">
  An RFP and proposal authoring tool that brings celestial precision to professional proposal creation.
  <br />
  RFP intelligence. Visual pipelines. Typst power.
</p>

<p align="center">
  <a href="#features">Features</a> |
  <a href="#quick-start">Quick Start</a> |
  <a href="#architecture">Architecture</a> |
  <a href="#api-reference">API</a> |
  <a href="#roadmap">Roadmap</a>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/version-0.1.0-blue?style=flat-square" alt="Version" />
  <img src="https://img.shields.io/badge/license-MIT-green?style=flat-square" alt="License" />
  <img src="https://img.shields.io/badge/rust-1.77+-orange?style=flat-square" alt="Rust" />
  <img src="https://img.shields.io/badge/tauri-2.0-purple?style=flat-square" alt="Tauri" />
  <img src="https://img.shields.io/badge/typst-0.13-red?style=flat-square" alt="Typst" />
</p>

---

<p align="center">
  <img src="assets/screenshot-main.png" alt="Syzygy Interface" width="800" />
</p>

## The Problem

Proposal and RFP creation for enterprises is broken:

- **Word processors** produce inconsistent, unprofessional output
- **LaTeX** has a brutal learning curve and glacial compile times
- **Existing RFP tools** cost $25K-100K/year and lock you into their cloud
- **Manual compliance tracking** means missed requirements and lost bids

## The Solution

**Syzygy** (noun): *The alignment of three or more celestial bodies* - when everything comes together perfectly.

We built the RFP and proposal authoring tool that proposal teams, technical writers, and publishers actually need:

```
+------------------+     +------------------+     +------------------+
|   Visual Flow    | --> |   Typst Engine   | --> |   Perfect PDFs   |
|   Pipeline       |     |   Sub-100ms      |     |   Every Time     |
+------------------+     +------------------+     +------------------+
         ^                        ^                        |
         |                        |                        v
+------------------+     +------------------+     +------------------+
|   AI Assistant   |     |   Content        |     |   Compliance     |
|   Local + Cloud  |     |   Library        |     |   Matrix         |
+------------------+     +------------------+     +------------------+
```

---

## Features

### Document Engineering

<table>
<tr>
<td width="50%">

**Typst-Powered Compilation**

Sub-100ms incremental builds. Write in a modern markup language that compiles instantly. No more waiting for LaTeX.

```typst
#set page(margin: 2cm)
#set text(font: "IBM Plex Sans")

= Proposal Executive Summary

#lorem(100)
```

</td>
<td width="50%">

**Live Preview**

See changes as you type. Synchronized scrolling between editor and preview. Click-to-source navigation.

</td>
</tr>
</table>

### Visual Document Pipelines

Build complex document workflows like you build in n8n or Zapier:

```
[Source: chapters/*.typ] --> [Transform: Merge] --> [Transform: Inject Variables]
                                                            |
                                                            v
[Output: proposal.pdf] <-- [Condition: Client Type] <-- [Template: Cover Page]
```

**Node Types:**
| Type | Purpose | Examples |
|------|---------|----------|
| **Source** | Input data | Files, folders, JSON, CSV |
| **Transform** | Modify content | Merge, split, inject, filter |
| **Condition** | Branch logic | If/else, switch |
| **Output** | Generate files | PDF, HTML, Markdown, LaTeX |

### RFP Intelligence

Drop an RFP. Get instant analysis.

```
+---------------------------+     +---------------------------+
|     RFP Document          | --> |     Extracted Data        |
+---------------------------+     +---------------------------+
| 47-page Government RFP    |     | - 127 Requirements        |
| Complex compliance needs  |     | - 12 Deadlines            |
| Buried critical terms     |     | - 34 Critical Terms       |
|                           |     | - Compliance Matrix       |
+---------------------------+     +---------------------------+
```

**What we extract:**
- Requirements with priority levels (SHALL, MUST, WILL)
- Submission deadlines and milestones
- Evaluation criteria and scoring weights
- Compliance gaps with recommendations

### AI-Powered Assistance

**Local-First Privacy:**
```bash
# Your sensitive documents never leave your machine
ollama pull llama3.2
# Syzygy auto-connects to localhost:11434
```

**Cloud Power (Optional):**
- Claude API for maximum capability
- Per-project configuration
- Air-gap mode for classified work

### Content Library

Stop rewriting the same boilerplate:

```typescript
// Save once
await invoke('create_content_block', {
  input: {
    title: 'Company Overview - Federal',
    content: '...',
    category: 'Boilerplate',
    tags: ['federal', 'overview', 'about'],
  }
});

// Semantic search
const blocks = await invoke('search_content_blocks', {
  query: 'cloud migration capabilities'
});
```

---

## Quick Start

### Prerequisites

```bash
# Check versions
node --version  # 18+
rustc --version # 1.77+
```

### Installation

```bash
# Clone
git clone https://github.com/yourusername/syzygy.git
cd syzygy

# Install dependencies
npm install

# Run in development
npm run tauri dev
```

### First Document

1. **Create a project** - `Ctrl+Shift+N`
2. **Write Typst** - Start typing in the editor
3. **See live preview** - PDF updates instantly
4. **Build pipeline** - `Ctrl+Shift+P` to open pipeline view
5. **Export** - `Ctrl+E` for export options

---

## Architecture

```
+------------------------------------------------------------------+
|                           TAURI SHELL                             |
+------------------------------------------------------------------+
|                         RUST BACKEND                              |
|  +------------------+  +------------------+  +------------------+ |
|  |  Typst Engine    |  |  Pipeline DAG    |  |   AI Service     | |
|  |  - Compile       |  |  - Topo sort     |  |  - Ollama        | |
|  |  - Render        |  |  - Parallel exec |  |  - Claude API    | |
|  |  - Watch         |  |  - Streaming     |  |  - Embeddings    | |
|  +------------------+  +------------------+  +------------------+ |
|  +------------------+  +------------------+  +------------------+ |
|  |  SurrealDB       |  |  RFP Analyzer    |  |   Export         | |
|  |  - Content lib   |  |  - Extraction    |  |  - PDF           | |
|  |  - Settings      |  |  - Compliance    |  |  - HTML/MD       | |
|  |  - Projects      |  |  - Gap analysis  |  |  - LaTeX         | |
|  +------------------+  +------------------+  +------------------+ |
+------------------------------------------------------------------+
|                        REACT FRONTEND                             |
|  +------------------+  +------------------+  +------------------+ |
|  |  CodeMirror 6    |  |  React Flow      |  |   PDF.js         | |
|  |  - Typst syntax  |  |  - Node canvas   |  |  - Preview       | |
|  |  - LSP ready     |  |  - Connections   |  |  - Annotations   | |
|  +------------------+  +------------------+  +------------------+ |
+------------------------------------------------------------------+
```

### Tech Stack

| Layer | Technology | Purpose |
|-------|------------|---------|
| Shell | Tauri 2.0 | Cross-platform, secure, tiny bundle |
| Backend | Rust | Performance, safety, Typst native |
| Database | SurrealDB + RocksDB | Multi-model, embedded, fast |
| Editor | CodeMirror 6 | Lightweight, extensible |
| Pipeline | React Flow | Visual node editing |
| Preview | PDF.js | Accurate rendering |
| AI | Ollama / Claude | Local-first, cloud optional |

---

## Project Structure

```
syzygy/
+-- src/                          # React frontend
|   +-- components/
|   |   +-- AIAssistant/          # Chat interface
|   |   +-- CommandPalette/       # Ctrl+K actions
|   |   +-- ContentLibrary/       # Reusable blocks
|   |   +-- Editor/               # CodeMirror wrapper
|   |   +-- FileTree/             # Project navigator
|   |   +-- Pipeline/             # React Flow canvas
|   |   +-- Preview/              # PDF renderer
|   |   +-- RFPAnalyzer/          # Compliance tools
|   +-- hooks/                    # React hooks
|   +-- App.tsx
|   +-- main.tsx
+-- src-tauri/                    # Rust backend
|   +-- src/
|   |   +-- ai/                   # LLM integration
|   |   +-- commands/             # Tauri IPC handlers
|   |   +-- engine/
|   |   |   +-- typst_engine.rs   # Document compilation
|   |   |   +-- pipeline.rs       # DAG execution
|   |   |   +-- export.rs         # Format conversion
|   |   +-- rfp/
|   |   |   +-- analyzer.rs       # Requirement extraction
|   |   |   +-- compliance.rs     # Matrix generation
|   |   +-- storage/
|   |       +-- database.rs       # SurrealDB connection
|   |       +-- content_library.rs
|   |       +-- settings.rs
|   +-- Cargo.toml
+-- package.json
+-- README.md
```

---

## Keyboard Shortcuts

| Category | Shortcut | Action |
|----------|----------|--------|
| **Navigation** | `Ctrl+K` | Command palette |
| | `Ctrl+P` | Quick file open |
| | `Ctrl+B` | Toggle sidebar |
| | `Ctrl+Shift+E` | Focus explorer |
| **Document** | `Ctrl+S` | Save |
| | `Ctrl+Shift+S` | Save all |
| | `Ctrl+Enter` | Compile |
| | `Ctrl+E` | Export |
| **Views** | `Ctrl+Shift+P` | Pipeline editor |
| | `Ctrl+Shift+A` | AI assistant |
| | `Ctrl+Shift+L` | Content library |
| **General** | `Escape` | Close modal |
| | `F11` | Fullscreen |

---

## API Reference

### Document Compilation

```typescript
// Compile Typst to PDF
const result = await invoke('compile_typst_source', {
  source: typstContent,
  rootPath: '/path/to/project'
});

// result.pdf_data: Uint8Array
// result.compile_time_ms: number
// result.warnings: string[]
```

### Pipeline Execution

```typescript
const pipeline = {
  nodes: [
    { id: '1', type: 'source', data: { path: 'chapters/*.typ' }},
    { id: '2', type: 'transform', data: { operation: 'merge' }},
    { id: '3', type: 'output', data: { format: 'pdf', outputPath: 'out.pdf' }}
  ],
  edges: [
    { source: '1', target: '2' },
    { source: '2', target: '3' }
  ]
};

const result = await invoke('run_pipeline', { pipeline, rootPath: '.' });
```

### RFP Analysis

```typescript
// Analyze document
const analysis = await invoke('analyze_rfp_document', {
  content: rfpText,
  documentName: 'RFP-2024-001.pdf'
});

// Generate compliance matrix
const matrix = await invoke('generate_matrix', {
  analysis,
  projectName: 'Our Response'
});

// Find gaps
const gaps = await invoke('analyze_gaps', { matrix });
```

### Content Library

```typescript
// Create block
await invoke('create_content_block', {
  input: {
    title: 'Past Performance - Cloud',
    content: '...',
    category: 'Past Performance',
    tags: ['cloud', 'aws', 'migration'],
    format: 'typst'
  }
});

// Semantic search
const results = await invoke('search_content_blocks', {
  query: 'federal cloud migration experience',
  limit: 10
});

// Get popular blocks
const popular = await invoke('get_popular_blocks', { limit: 5 });
```

---

## Configuration

### Ollama (Local AI)

```bash
# Install
curl -fsSL https://ollama.ai/install.sh | sh

# Pull models
ollama pull llama3.2      # General purpose
ollama pull nomic-embed-text  # Embeddings

# Syzygy auto-detects at localhost:11434
```

### Database

Data is stored locally:

| Platform | Location |
|----------|----------|
| Linux | `~/.local/share/syzygy/database/` |
| macOS | `~/Library/Application Support/com.syzygy.Syzygy/database/` |
| Windows | `%APPDATA%\syzygy\Syzygy\database\` |

---

## Roadmap

### Now
- [x] Typst compilation with live preview
- [x] Visual pipeline editor
- [x] RFP requirement extraction
- [x] Compliance matrix generation
- [x] Ollama AI integration
- [x] Content library with search

### Next
- [ ] LaTeX support via Tectonic
- [ ] DOCX export
- [ ] LSP integration (Tinymist)
- [ ] Git integration
- [ ] Template marketplace

### Future
- [ ] Real-time collaboration (CRDT)
- [ ] Cloud sync
- [ ] Plugin system
- [ ] Team workspaces
- [ ] Enterprise SSO

---

## Why "Syzygy"?

In astronomy, a **syzygy** is a rare alignment of three or more celestial bodies - like a solar eclipse when the Sun, Moon, and Earth align perfectly.

That's what we do for documents: align your content, design, and compliance requirements into perfect harmony.

---

## Contributing

```bash
# Fork and clone
git clone https://github.com/yourusername/syzygy.git

# Create feature branch
git checkout -b feature/amazing-feature

# Make changes, then commit
git commit -m 'feat: add amazing feature'

# Push and open PR
git push origin feature/amazing-feature
```

Please follow [Conventional Commits](https://conventionalcommits.org).

---

## License

MIT License - see [LICENSE](LICENSE) for details.

---

<p align="center">
  <sub>Built with obsessive attention to detail.</sub>
</p>
