//! Typst compilation engine
//!
//! Full implementation using the typst crate for document compilation.

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use typst::diag::{FileError, FileResult, SourceDiagnostic, Severity};
use typst::foundations::{Bytes, Datetime, Smart};
use typst::syntax::{FileId, Source, VirtualPath};
use typst::text::{Font, FontBook};
use typst::{Library, World};
use typst::utils::LazyHash;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompileResult {
    pub success: bool,
    pub output_path: Option<String>,
    pub pdf_data: Option<Vec<u8>>,
    pub errors: Vec<CompileError>,
    pub warnings: Vec<CompileWarning>,
    pub compile_time_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompileError {
    pub message: String,
    pub file: Option<String>,
    pub line: Option<u32>,
    pub column: Option<u32>,
    pub hint: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompileWarning {
    pub message: String,
    pub file: Option<String>,
    pub line: Option<u32>,
    pub column: Option<u32>,
}

/// RFPMaker's Typst world implementation
pub struct RfpWorld {
    /// The root directory for file resolution
    root: PathBuf,
    /// The main source file
    main: Source,
    /// Cached sources
    sources: RwLock<HashMap<FileId, Source>>,
    /// The standard library
    library: LazyHash<Library>,
    /// Font book
    book: LazyHash<FontBook>,
    /// Loaded fonts
    fonts: Vec<Font>,
    /// Current datetime
    now: chrono::DateTime<chrono::Utc>,
}

impl RfpWorld {
    pub fn new(root: PathBuf, main_path: &Path) -> Result<Self, String> {
        // Read the main source file
        let main_content = fs::read_to_string(main_path)
            .map_err(|e| format!("Failed to read main file: {}", e))?;

        let main_id = FileId::new(None, VirtualPath::new(main_path.file_name().unwrap()));
        let main = Source::new(main_id, main_content);

        // Load system fonts
        let (book, fonts) = Self::load_fonts();

        Ok(Self {
            root,
            main,
            sources: RwLock::new(HashMap::new()),
            library: LazyHash::new(Library::default()),
            book: LazyHash::new(book),
            fonts,
            now: chrono::Utc::now(),
        })
    }

    pub fn load_fonts() -> (FontBook, Vec<Font>) {
        let mut book = FontBook::new();
        let mut fonts = Vec::new();

        // Search for system fonts
        let font_paths = Self::get_font_paths();

        for font_path in font_paths {
            Self::load_fonts_from_dir(&font_path, &mut book, &mut fonts);
        }

        (book, fonts)
    }

    fn load_fonts_from_dir(dir: &Path, book: &mut FontBook, fonts: &mut Vec<Font>) {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    // Recurse into subdirectories
                    Self::load_fonts_from_dir(&path, book, fonts);
                } else if Self::is_font_file(&path) {
                    if let Ok(data) = fs::read(&path) {
                        let buffer = Bytes::new(data);
                        for font in Font::iter(buffer) {
                            book.push(font.info().clone());
                            fonts.push(font);
                        }
                    }
                }
            }
        }
    }

    fn get_font_paths() -> Vec<PathBuf> {
        let mut paths = Vec::new();

        // Linux font directories
        paths.push(PathBuf::from("/usr/share/fonts"));
        paths.push(PathBuf::from("/usr/local/share/fonts"));

        if let Some(home) = std::env::var_os("HOME") {
            let home = PathBuf::from(home);
            paths.push(home.join(".fonts"));
            paths.push(home.join(".local/share/fonts"));
        }

        paths
    }

    fn is_font_file(path: &Path) -> bool {
        match path.extension().and_then(|e| e.to_str()) {
            Some("ttf" | "otf" | "ttc" | "otc" | "woff" | "woff2") => true,
            _ => false,
        }
    }

    fn resolve_path(&self, path: &Path) -> PathBuf {
        if path.is_absolute() {
            path.to_path_buf()
        } else {
            self.root.join(path)
        }
    }
}

impl World for RfpWorld {
    fn library(&self) -> &LazyHash<Library> {
        &self.library
    }

    fn book(&self) -> &LazyHash<FontBook> {
        &self.book
    }

    fn main(&self) -> FileId {
        self.main.id()
    }

    fn source(&self, id: FileId) -> FileResult<Source> {
        if id == self.main.id() {
            return Ok(self.main.clone());
        }

        // Check cache first
        if let Some(source) = self.sources.read().get(&id) {
            return Ok(source.clone());
        }

        // Load from disk
        let vpath = id.vpath();
        let path = self.resolve_path(vpath.as_rooted_path());

        let content = fs::read_to_string(&path).map_err(|e| {
            FileError::from_io(e, &path)
        })?;

        let source = Source::new(id, content);
        self.sources.write().insert(id, source.clone());
        Ok(source)
    }

    fn file(&self, id: FileId) -> FileResult<Bytes> {
        let vpath = id.vpath();
        let path = self.resolve_path(vpath.as_rooted_path());

        let data = fs::read(&path).map_err(|e| {
            FileError::from_io(e, &path)
        })?;

        Ok(Bytes::new(data))
    }

    fn font(&self, index: usize) -> Option<Font> {
        self.fonts.get(index).cloned()
    }

    fn today(&self, offset: Option<i64>) -> Option<Datetime> {
        use chrono::{Datelike, Timelike};
        let offset = offset.unwrap_or(0);
        let dt = self.now + chrono::Duration::hours(offset);
        Datetime::from_ymd_hms(
            dt.year(),
            dt.month().try_into().ok()?,
            dt.day().try_into().ok()?,
            dt.hour().try_into().ok()?,
            dt.minute().try_into().ok()?,
            dt.second().try_into().ok()?,
        )
    }
}

fn diagnostic_to_error(diag: &SourceDiagnostic, world: &RfpWorld) -> CompileError {
    let span = diag.span;
    let (file, line, column) = if !span.is_detached() {
        if let Some(id) = span.id() {
            if let Ok(source) = world.source(id) {
                let range = source.range(span).unwrap_or(0..0);
                let line = source.byte_to_line(range.start).map(|l| l as u32 + 1);
                let column = source.byte_to_column(range.start).map(|c| c as u32 + 1);
                (Some(source.id().vpath().as_rooted_path().display().to_string()), line, column)
            } else {
                (None, None, None)
            }
        } else {
            (None, None, None)
        }
    } else {
        (None, None, None)
    };

    CompileError {
        message: diag.message.to_string(),
        file,
        line,
        column,
        hint: diag.hints.first().map(|h| h.to_string()),
    }
}

fn diagnostic_to_warning(diag: &SourceDiagnostic, world: &RfpWorld) -> CompileWarning {
    let span = diag.span;
    let (file, line, column) = if !span.is_detached() {
        if let Some(id) = span.id() {
            if let Ok(source) = world.source(id) {
                let range = source.range(span).unwrap_or(0..0);
                let line = source.byte_to_line(range.start).map(|l| l as u32 + 1);
                let column = source.byte_to_column(range.start).map(|c| c as u32 + 1);
                (Some(source.id().vpath().as_rooted_path().display().to_string()), line, column)
            } else {
                (None, None, None)
            }
        } else {
            (None, None, None)
        }
    } else {
        (None, None, None)
    };

    CompileWarning {
        message: diag.message.to_string(),
        file,
        line,
        column,
    }
}

/// Compile a Typst document to PDF
#[tauri::command]
pub async fn compile_typst(input_path: String, output_path: String) -> Result<CompileResult, String> {
    let start = std::time::Instant::now();

    log::info!("Compiling Typst document: {} -> {}", input_path, output_path);

    let input = PathBuf::from(&input_path);
    let output = PathBuf::from(&output_path);

    // Check if input file exists
    if !input.exists() {
        return Ok(CompileResult {
            success: false,
            output_path: None,
            pdf_data: None,
            errors: vec![CompileError {
                message: format!("Input file not found: {}", input_path),
                file: Some(input_path),
                line: None,
                column: None,
                hint: None,
            }],
            warnings: vec![],
            compile_time_ms: start.elapsed().as_millis() as u64,
        });
    }

    // Get root directory
    let root = input.parent().unwrap_or(Path::new(".")).to_path_buf();

    // Create world
    let world = match RfpWorld::new(root, &input) {
        Ok(w) => w,
        Err(e) => {
            return Ok(CompileResult {
                success: false,
                output_path: None,
                pdf_data: None,
                errors: vec![CompileError {
                    message: e,
                    file: Some(input_path),
                    line: None,
                    column: None,
                    hint: None,
                }],
                warnings: vec![],
                compile_time_ms: start.elapsed().as_millis() as u64,
            });
        }
    };

    // Compile the document
    let result = typst::compile(&world);
    let document = result.output;
    let compilation_warnings = result.warnings;

    match document {
        Ok(doc) => {
            // Generate PDF
            let pdf_options = typst_pdf::PdfOptions {
                ident: Smart::Auto,
                timestamp: None,
                page_ranges: None,
                standards: typst_pdf::PdfStandards::default(),
            };

            let pdf_result = typst_pdf::pdf(&doc, &pdf_options);

            match pdf_result {
                Ok(pdf_data) => {
                    // Write to file
                    if let Err(e) = fs::write(&output, &pdf_data) {
                        return Ok(CompileResult {
                            success: false,
                            output_path: None,
                            pdf_data: None,
                            errors: vec![CompileError {
                                message: format!("Failed to write PDF: {}", e),
                                file: None,
                                line: None,
                                column: None,
                                hint: None,
                            }],
                            warnings: vec![],
                            compile_time_ms: start.elapsed().as_millis() as u64,
                        });
                    }

                    // Collect warnings
                    let warnings: Vec<CompileWarning> = compilation_warnings
                        .iter()
                        .map(|d| diagnostic_to_warning(d, &world))
                        .collect();

                    log::info!("Compilation successful in {}ms", start.elapsed().as_millis());

                    Ok(CompileResult {
                        success: true,
                        output_path: Some(output.display().to_string()),
                        pdf_data: Some(pdf_data),
                        errors: vec![],
                        warnings,
                        compile_time_ms: start.elapsed().as_millis() as u64,
                    })
                }
                Err(pdf_errors) => {
                    let errors: Vec<CompileError> = pdf_errors
                        .iter()
                        .map(|d| diagnostic_to_error(d, &world))
                        .collect();

                    Ok(CompileResult {
                        success: false,
                        output_path: None,
                        pdf_data: None,
                        errors,
                        warnings: vec![],
                        compile_time_ms: start.elapsed().as_millis() as u64,
                    })
                }
            }
        }
        Err(diagnostics) => {
            let mut errors = Vec::new();
            let mut warnings = Vec::new();

            for diag in diagnostics.iter() {
                match diag.severity {
                    Severity::Error => errors.push(diagnostic_to_error(diag, &world)),
                    Severity::Warning => warnings.push(diagnostic_to_warning(diag, &world)),
                }
            }

            // Also collect compilation warnings
            for diag in compilation_warnings.iter() {
                warnings.push(diagnostic_to_warning(diag, &world));
            }

            log::warn!("Compilation failed with {} errors", errors.len());

            Ok(CompileResult {
                success: false,
                output_path: None,
                pdf_data: None,
                errors,
                warnings,
                compile_time_ms: start.elapsed().as_millis() as u64,
            })
        }
    }
}

/// Compile Typst source directly from string (for live preview)
#[tauri::command]
pub async fn compile_typst_source(
    source: String,
    root_path: String,
) -> Result<CompileResult, String> {
    let start = std::time::Instant::now();

    log::debug!("Compiling Typst source from memory");

    let root = PathBuf::from(&root_path);

    // Create a temporary file ID for the source
    let main_id = FileId::new(None, VirtualPath::new("main.typ"));
    let main = Source::new(main_id, source);

    // Load fonts
    let (book, fonts) = RfpWorld::load_fonts();

    // Create a minimal world for in-memory compilation
    struct MemoryWorld {
        main: Source,
        root: PathBuf,
        library: LazyHash<Library>,
        book: LazyHash<FontBook>,
        fonts: Vec<Font>,
        sources: RwLock<HashMap<FileId, Source>>,
        now: chrono::DateTime<chrono::Utc>,
    }

    impl World for MemoryWorld {
        fn library(&self) -> &LazyHash<Library> {
            &self.library
        }

        fn book(&self) -> &LazyHash<FontBook> {
            &self.book
        }

        fn main(&self) -> FileId {
            self.main.id()
        }

        fn source(&self, id: FileId) -> FileResult<Source> {
            if id == self.main.id() {
                return Ok(self.main.clone());
            }

            if let Some(source) = self.sources.read().get(&id) {
                return Ok(source.clone());
            }

            let vpath = id.vpath();
            let path = if vpath.as_rooted_path().is_absolute() {
                vpath.as_rooted_path().to_path_buf()
            } else {
                self.root.join(vpath.as_rooted_path())
            };

            let content = fs::read_to_string(&path).map_err(|e| FileError::from_io(e, &path))?;
            let source = Source::new(id, content);
            self.sources.write().insert(id, source.clone());
            Ok(source)
        }

        fn file(&self, id: FileId) -> FileResult<Bytes> {
            let vpath = id.vpath();
            let path = if vpath.as_rooted_path().is_absolute() {
                vpath.as_rooted_path().to_path_buf()
            } else {
                self.root.join(vpath.as_rooted_path())
            };

            let data = fs::read(&path).map_err(|e| FileError::from_io(e, &path))?;
            Ok(Bytes::new(data))
        }

        fn font(&self, index: usize) -> Option<Font> {
            self.fonts.get(index).cloned()
        }

        fn today(&self, offset: Option<i64>) -> Option<Datetime> {
            use chrono::{Datelike, Timelike};
            let offset = offset.unwrap_or(0);
            let dt = self.now + chrono::Duration::hours(offset);
            Datetime::from_ymd_hms(
                dt.year(),
                dt.month().try_into().ok()?,
                dt.day().try_into().ok()?,
                dt.hour().try_into().ok()?,
                dt.minute().try_into().ok()?,
                dt.second().try_into().ok()?,
            )
        }
    }

    let world = MemoryWorld {
        main,
        root,
        library: LazyHash::new(Library::default()),
        book: LazyHash::new(book),
        fonts,
        sources: RwLock::new(HashMap::new()),
        now: chrono::Utc::now(),
    };

    let result = typst::compile(&world);
    let document = result.output;

    match document {
        Ok(doc) => {
            let pdf_options = typst_pdf::PdfOptions {
                ident: Smart::Auto,
                timestamp: None,
                page_ranges: None,
                standards: typst_pdf::PdfStandards::default(),
            };

            match typst_pdf::pdf(&doc, &pdf_options) {
                Ok(pdf_data) => {
                    Ok(CompileResult {
                        success: true,
                        output_path: None,
                        pdf_data: Some(pdf_data),
                        errors: vec![],
                        warnings: vec![],
                        compile_time_ms: start.elapsed().as_millis() as u64,
                    })
                }
                Err(pdf_errors) => {
                    let errors: Vec<CompileError> = pdf_errors
                        .iter()
                        .map(|d| CompileError {
                            message: d.message.to_string(),
                            file: None,
                            line: None,
                            column: None,
                            hint: d.hints.first().map(|h| h.to_string()),
                        })
                        .collect();

                    Ok(CompileResult {
                        success: false,
                        output_path: None,
                        pdf_data: None,
                        errors,
                        warnings: vec![],
                        compile_time_ms: start.elapsed().as_millis() as u64,
                    })
                }
            }
        }
        Err(diagnostics) => {
            let errors: Vec<CompileError> = diagnostics
                .iter()
                .filter(|d| d.severity == Severity::Error)
                .map(|d| CompileError {
                    message: d.message.to_string(),
                    file: None,
                    line: None,
                    column: None,
                    hint: d.hints.first().map(|h| h.to_string()),
                })
                .collect();

            Ok(CompileResult {
                success: false,
                output_path: None,
                pdf_data: None,
                errors,
                warnings: vec![],
                compile_time_ms: start.elapsed().as_millis() as u64,
            })
        }
    }
}

/// Get Typst version info
#[tauri::command]
pub fn get_typst_version() -> String {
    "Typst 0.13".to_string()
}
