"use client";

import { useState, useCallback } from "react";
import { Scan } from "lucide-react";
import { UploadZone } from "@/components/tools/upload-zone";
import { ProgressBar } from "@/components/tools/progress-bar";
import { ResultPreview } from "@/components/tools/result-preview";
import { useUpload } from "@/hooks/use-upload";
import { useJobStatus } from "@/hooks/use-job-status";

type PageState = "upload" | "processing" | "result" | "error";

interface ScanOptions {
  ocr: boolean;
  enhance: boolean;
  output_format: "pdf" | "jpeg" | "png";
  dpi: number;
  color_mode: "black_and_white" | "grayscale" | "color";
}

export default function ScanPage() {
  const [pageState, setPageState] = useState<PageState>("upload");
  const [jobId, setJobId] = useState<string | null>(null);
  const [errorMsg, setErrorMsg] = useState<string | null>(null);
  const [opts, setOpts] = useState<ScanOptions>({
    ocr: true,
    enhance: true,
    output_format: "pdf",
    dpi: 300,
    color_mode: "black_and_white",
  });

  const { upload, isUploading } = useUpload({
    tool: "scan",
    options: opts as unknown as Record<string, unknown>,
  });

  const handleComplete = useCallback(() => {
    setPageState("result");
  }, []);

  const handleError = useCallback((err: string) => {
    setErrorMsg(err);
    setPageState("error");
  }, []);

  const { progress, stage, message, status, result } = useJobStatus(jobId, {
    onComplete: handleComplete,
    onError: handleError,
  });

  const handleUpload = useCallback(
    async (file: File) => {
      setErrorMsg(null);
      const res = await upload(file);
      if (res) {
        setJobId(res.job_id);
        setPageState("processing");
      }
    },
    [upload],
  );

  const handleRetry = useCallback(() => {
    setPageState("upload");
    setJobId(null);
    setErrorMsg(null);
  }, []);

  return (
    <div className="container mx-auto px-4 py-8 max-w-3xl">
      <div className="flex items-center gap-3 mb-8">
        <div className="p-2 rounded-lg bg-primary/10 text-primary">
          <Scan className="h-6 w-6" />
        </div>
        <div>
          <h1 className="text-2xl font-bold">Document Scanner</h1>
          <p className="text-sm text-muted-foreground">
            Foto dokumen pake HP — auto-detect, lurusin, enhance, OCR
          </p>
        </div>
      </div>

      {pageState === "upload" && (
        <div className="space-y-6">
          <UploadZone
            accept="image/*"
            tool="scan"
            onUpload={handleUpload}
            maxSizeMB={50}
          />

          {/* Interactive Options */}
          <div className="p-6 rounded-xl border glass space-y-4">
            <h3 className="font-semibold">Scan Options</h3>

            {/* OCR Toggle */}
            <label className="flex items-center justify-between">
              <span className="text-sm">OCR (Tesseract)</span>
              <input
                type="checkbox"
                checked={opts.ocr}
                onChange={(e) => setOpts({ ...opts, ocr: e.target.checked })}
                className="toggle"
              />
            </label>

            {/* Enhance Toggle */}
            <label className="flex items-center justify-between">
              <span className="text-sm">Auto-enhance</span>
              <input
                type="checkbox"
                checked={opts.enhance}
                onChange={(e) => setOpts({ ...opts, enhance: e.target.checked })}
                className="toggle"
              />
            </label>

            {/* Output Format */}
            <label className="flex items-center justify-between">
              <span className="text-sm">Output Format</span>
              <select
                value={opts.output_format}
                onChange={(e) =>
                  setOpts({
                    ...opts,
                    output_format: e.target.value as ScanOptions["output_format"],
                  })
                }
                className="bg-muted border rounded px-2 py-1 text-sm"
              >
                <option value="pdf">Searchable PDF</option>
                <option value="jpeg">JPEG Image</option>
                <option value="png">PNG Image</option>
              </select>
            </label>

            {/* Color Mode */}
            <label className="flex items-center justify-between">
              <span className="text-sm">Color Mode</span>
              <select
                value={opts.color_mode}
                onChange={(e) =>
                  setOpts({
                    ...opts,
                    color_mode: e.target.value as ScanOptions["color_mode"],
                  })
                }
                className="bg-muted border rounded px-2 py-1 text-sm"
              >
                <option value="black_and_white">Black & White</option>
                <option value="grayscale">Grayscale</option>
                <option value="color">Color</option>
              </select>
            </label>

            {/* DPI */}
            <label className="flex items-center justify-between">
              <span className="text-sm">DPI</span>
              <select
                value={opts.dpi}
                onChange={(e) => setOpts({ ...opts, dpi: Number(e.target.value) })}
                className="bg-muted border rounded px-2 py-1 text-sm"
              >
                <option value={150}>150 (draft)</option>
                <option value={300}>300 (standard)</option>
                <option value={600}>600 (high)</option>
              </select>
            </label>
          </div>
        </div>
      )}

      {pageState === "processing" && jobId && (
        <ProgressBar
          progress={progress}
          stage={stage}
          message={message}
          status={status}
          onRetry={handleRetry}
        />
      )}

      {pageState === "result" && result && (
        <ResultPreview
          result={{
            download_url: result.download_url,
            file_size: result.file_size,
            file_name: result.file_name,
            preview_url: result.preview_url,
          }}
          ocrText={result.ocr_text}
          onProcessAnother={handleRetry}
        />
      )}

      {pageState === "error" && (
        <div className="p-6 rounded-xl border border-destructive/20 bg-destructive/5 text-center space-y-4">
          <p className="text-destructive font-medium">
            {errorMsg || "Terjadi kesalahan"}
          </p>
          <button
            onClick={handleRetry}
            className="px-4 py-2 bg-primary text-primary-foreground rounded-lg hover:opacity-90"
          >
            Coba Lagi
          </button>
        </div>
      )}
    </div>
  );
}