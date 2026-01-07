import { useState, useEffect, useRef, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import * as pdfjsLib from "pdfjs-dist";
import "./Preview.css";

// Configure PDF.js worker
pdfjsLib.GlobalWorkerOptions.workerSrc = new URL(
  "pdfjs-dist/build/pdf.worker.min.mjs",
  import.meta.url
).toString();

interface CompileResult {
  success: boolean;
  output_path: string | null;
  pdf_data: number[] | null;
  errors: CompileError[];
  warnings: CompileWarning[];
  compile_time_ms: number;
}

interface CompileError {
  message: string;
  file: string | null;
  line: number | null;
  column: number | null;
  hint: string | null;
}

interface CompileWarning {
  message: string;
  file: string | null;
  line: number | null;
  column: number | null;
}

interface PreviewProps {
  source: string;
  rootPath: string;
  onCompileResult?: (result: CompileResult) => void;
}

export function Preview({ source, rootPath, onCompileResult }: PreviewProps) {
  const [pages, setPages] = useState<string[]>([]);
  const [currentPage, setCurrentPage] = useState(0);
  const [totalPages, setTotalPages] = useState(0);
  const [isCompiling, setIsCompiling] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [compileTime, setCompileTime] = useState<number | null>(null);
  const [scale, setScale] = useState(1.0);
  const containerRef = useRef<HTMLDivElement>(null);
  const debounceRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const pdfDocRef = useRef<pdfjsLib.PDFDocumentProxy | null>(null);

  const renderPage = useCallback(
    async (
      pdf: pdfjsLib.PDFDocumentProxy,
      pageNum: number
    ): Promise<string> => {
      const page = await pdf.getPage(pageNum);

      // Get container width for responsive scaling
      const containerWidth = containerRef.current?.clientWidth || 600;
      const viewport = page.getViewport({ scale: 1 });
      const fitScale = ((containerWidth - 40) / viewport.width) * scale;
      const scaledViewport = page.getViewport({ scale: fitScale });

      // Create canvas
      const canvas = document.createElement("canvas");
      const context = canvas.getContext("2d");
      if (!context) throw new Error("Cannot get canvas context");

      canvas.height = scaledViewport.height;
      canvas.width = scaledViewport.width;

      // Render PDF page to canvas
      await page.render({
        canvasContext: context,
        viewport: scaledViewport,
        canvas: canvas,
      }).promise;

      return canvas.toDataURL("image/png");
    },
    [scale]
  );

  const renderPdf = useCallback(
    async (pdfData: Uint8Array) => {
      try {
        // Clean up previous document
        if (pdfDocRef.current) {
          await pdfDocRef.current.destroy();
        }

        // Load PDF document
        const loadingTask = pdfjsLib.getDocument({ data: pdfData });
        const pdf = await loadingTask.promise;
        pdfDocRef.current = pdf;

        setTotalPages(pdf.numPages);

        // Render all pages
        const renderedPages: string[] = [];
        for (let i = 1; i <= pdf.numPages; i++) {
          const pageDataUrl = await renderPage(pdf, i);
          renderedPages.push(pageDataUrl);
        }

        setPages(renderedPages);
        setError(null);
      } catch (err) {
        setError(`PDF rendering error: ${err}`);
      }
    },
    [renderPage]
  );

  const compile = useCallback(async () => {
    if (!source.trim()) {
      setPages([]);
      setError(null);
      return;
    }

    setIsCompiling(true);
    setError(null);

    try {
      const result = await invoke<CompileResult>("compile_typst_source", {
        source,
        rootPath: rootPath || ".",
      });

      setCompileTime(result.compile_time_ms);
      onCompileResult?.(result);

      if (result.success && result.pdf_data) {
        const pdfBytes = new Uint8Array(result.pdf_data);
        await renderPdf(pdfBytes);
      } else if (result.errors.length > 0) {
        const errorMessages = result.errors
          .map((e) => {
            let msg = e.message;
            if (e.file && e.line) {
              msg = `${e.file}:${e.line}:${e.column || 0}: ${msg}`;
            }
            if (e.hint) {
              msg += `\nHint: ${e.hint}`;
            }
            return msg;
          })
          .join("\n\n");
        setError(errorMessages);
        setPages([]);
      }
    } catch (err) {
      setError(`Compilation failed: ${err}`);
      setPages([]);
    } finally {
      setIsCompiling(false);
    }
  }, [source, rootPath, renderPdf, onCompileResult]);

  // Debounced compilation on source change
  useEffect(() => {
    if (debounceRef.current) {
      clearTimeout(debounceRef.current);
    }

    debounceRef.current = setTimeout(() => {
      compile();
    }, 300);

    return () => {
      if (debounceRef.current) {
        clearTimeout(debounceRef.current);
      }
    };
  }, [source, compile]);

  // Clean up on unmount
  useEffect(() => {
    return () => {
      if (pdfDocRef.current) {
        pdfDocRef.current.destroy();
      }
    };
  }, []);

  // Re-render when scale changes
  useEffect(() => {
    if (pdfDocRef.current && pages.length > 0) {
      const rerender = async () => {
        const renderedPages: string[] = [];
        for (let i = 1; i <= pdfDocRef.current!.numPages; i++) {
          const pageDataUrl = await renderPage(pdfDocRef.current!, i);
          renderedPages.push(pageDataUrl);
        }
        setPages(renderedPages);
      };
      rerender();
    }
  }, [scale, renderPage]);

  const handleZoomIn = () => setScale((s) => Math.min(s + 0.25, 3));
  const handleZoomOut = () => setScale((s) => Math.max(s - 0.25, 0.25));
  const handleZoomReset = () => setScale(1);

  const handlePrevPage = () => setCurrentPage((p) => Math.max(p - 1, 0));
  const handleNextPage = () =>
    setCurrentPage((p) => Math.min(p + 1, totalPages - 1));

  return (
    <div className="preview-container">
      <div className="preview-toolbar">
        <div className="preview-status">
          {isCompiling && <span className="compiling">Compiling...</span>}
          {!isCompiling && compileTime !== null && (
            <span className="compile-time">{compileTime}ms</span>
          )}
        </div>
        <div className="preview-pagination">
          <button
            onClick={handlePrevPage}
            disabled={currentPage === 0 || totalPages === 0}
          >
            Prev
          </button>
          <span className="page-info">
            {totalPages > 0 ? `${currentPage + 1} / ${totalPages}` : "- / -"}
          </span>
          <button
            onClick={handleNextPage}
            disabled={currentPage >= totalPages - 1 || totalPages === 0}
          >
            Next
          </button>
        </div>
        <div className="preview-zoom">
          <button onClick={handleZoomOut}>-</button>
          <span className="zoom-level">{Math.round(scale * 100)}%</span>
          <button onClick={handleZoomIn}>+</button>
          <button onClick={handleZoomReset}>Reset</button>
        </div>
      </div>
      <div className="preview-viewport" ref={containerRef}>
        {error ? (
          <div className="preview-error">
            <h4>Compilation Error</h4>
            <pre>{error}</pre>
          </div>
        ) : pages.length > 0 ? (
          <div className="preview-page">
            <img
              src={pages[currentPage]}
              alt={`Page ${currentPage + 1}`}
              style={{ maxWidth: "100%" }}
            />
          </div>
        ) : (
          <div className="preview-placeholder">
            {isCompiling ? "Compiling..." : "No preview available"}
          </div>
        )}
      </div>
    </div>
  );
}

export default Preview;
