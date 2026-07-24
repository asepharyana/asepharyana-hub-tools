"use client";

import { useState, useCallback } from "react";
import { Repeat } from "lucide-react";
import { ToolLayout } from "@/components/tools/tool-layout";
import { UploadZone } from "@/components/tools/upload-zone";
import { useUpload } from "@/hooks/use-upload";
import { useJobStatus } from "@/hooks/use-job-status";
import { ProgressBar } from "@/components/tools/progress-bar";
import { ResultPreview } from "@/components/tools/result-preview";

type PageState = "upload" | "processing" | "result" | "error";

const FORMATS = [
  { value: "jpeg", label: "JPEG (.jpg)" },
  { value: "png", label: "PNG (.png)" },
  { value: "webp", label: "WebP (.webp)" },
  { value: "gif", label: "GIF (.gif)" },
  { value: "bmp", label: "BMP (.bmp)" },
];

export default function ImageConvertPage() {
  const [pageState, setPageState] = useState<PageState>("upload");
  const [jobId, setJobId] = useState<string | null>(null);
  const [errorMsg, setErrorMsg] = useState<string | null>(null);

  const [format, setFormat] = useState("jpeg");
  const [quality, setQuality] = useState(85);

  const { upload } = useUpload({
    tool: "image-convert",
    options: { format, quality },
  });

  const handleComplete = useCallback(() => setPageState("result"), []);
  const handleError = useCallback(
    (err: string) => {
      setErrorMsg(err);
      setPageState("error");
    },
    [],
  );

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
    <ToolLayout
      title="Convert Image"
      description="Konversi antar format gambar — atur format tujuan dan kualitas"
      icon={Repeat}
      phase={1}
    >
      {pageState === "upload" && (
        <div className="space-y-6">
          {/* Format + Quality */}
          <div className="p-6 rounded-xl border glass space-y-4">
            <h3 className="font-semibold">Conversion Settings</h3>

            <label className="space-y-1">
              <span className="text-sm text-muted-foreground">Target Format</span>
              <select
                value={format}
                onChange={(e) => setFormat(e.target.value)}
                className="w-full bg-muted border rounded px-3 py-2 text-sm"
              >
                {FORMATS.map((f) => (
                  <option key={f.value} value={f.value}>
                    {f.label}
                  </option>
                ))}
              </select>
            </label>

            <div className="space-y-2">
              <div className="flex items-center justify-between text-sm">
                <span className="text-muted-foreground">Quality</span>
                <span className="font-mono">{quality}%</span>
              </div>
              <input
                type="range"
                min={1}
                max={100}
                value={quality}
                onChange={(e) => setQuality(Number(e.target.value))}
                className="w-full"
              />
              <div className="flex justify-between text-xs text-muted-foreground">
                <span>Small file</span>
                <span>Best quality</span>
              </div>
            </div>
          </div>

          <UploadZone
            accept="image/*"
            tool="image-convert"
            onUpload={handleUpload}
          />
        </div>
      )}

      {pageState === "processing" && jobId && (
        <ProgressBar progress={progress} stage={stage} message={message} status={status} onRetry={handleRetry} />
      )}

      {pageState === "result" && result && (
        <ResultPreview result={result} onProcessAnother={handleRetry} />
      )}

      {pageState === "error" && (
        <div className="p-6 rounded-xl border border-destructive/20 bg-destructive/5 text-center">
          <p className="text-destructive font-medium mb-4">{errorMsg || "Terjadi kesalahan"}</p>
          <button onClick={handleRetry} className="px-4 py-2 bg-primary text-primary-foreground rounded-lg hover:opacity-90">Coba Lagi</button>
        </div>
      )}
    </ToolLayout>
  );
}