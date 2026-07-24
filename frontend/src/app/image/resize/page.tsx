"use client";

import { useState, useCallback } from "react";
import { Crop } from "lucide-react";
import { ToolLayout } from "@/components/tools/tool-layout";
import { UploadZone } from "@/components/tools/upload-zone";
import { useUpload } from "@/hooks/use-upload";
import { useJobStatus } from "@/hooks/use-job-status";
import { ProgressBar } from "@/components/tools/progress-bar";
import { ResultPreview } from "@/components/tools/result-preview";

type PageState = "upload" | "processing" | "result" | "error";

export default function ImageResizePage() {
  const [pageState, setPageState] = useState<PageState>("upload");
  const [jobId, setJobId] = useState<string | null>(null);
  const [errorMsg, setErrorMsg] = useState<string | null>(null);

  const [width, setWidth] = useState(1920);
  const [height, setHeight] = useState(1080);
  const [lockAspect, setLockAspect] = useState(true);
  const [quality, setQuality] = useState(85);

  const { upload } = useUpload({
    tool: "image-resize",
    options: { width, height, quality, fit: lockAspect ? "inside" : "fill" },
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
      title="Resize Image"
      description="Ubah dimensi gambar — atur lebar, tinggi, dan kualitas"
      icon={Crop}
      phase={1}
    >
      {pageState === "upload" && (
        <div className="space-y-6">
          {/* Dimensions */}
          <div className="p-6 rounded-xl border glass space-y-4">
            <h3 className="font-semibold">Dimensions</h3>
            <div className="grid grid-cols-2 gap-4">
              <label className="space-y-1">
                <span className="text-sm text-muted-foreground">Width (px)</span>
                <input
                  type="number"
                  value={width}
                  onChange={(e) => setWidth(Number(e.target.value))}
                  min={1}
                  max={10000}
                  className="w-full bg-muted border rounded px-3 py-2 text-sm"
                />
              </label>
              <label className="space-y-1">
                <span className="text-sm text-muted-foreground">Height (px)</span>
                <input
                  type="number"
                  value={height}
                  onChange={(e) => setHeight(Number(e.target.value))}
                  min={1}
                  max={10000}
                  className="w-full bg-muted border rounded px-3 py-2 text-sm"
                />
              </label>
            </div>
            <label className="flex items-center gap-2 text-sm cursor-pointer">
              <input
                type="checkbox"
                checked={lockAspect}
                onChange={(e) => setLockAspect(e.target.checked)}
              />
              Lock aspect ratio
            </label>
          </div>

          {/* Quality */}
          <div className="p-6 rounded-xl border glass space-y-3">
            <div className="flex items-center justify-between">
              <span className="text-sm font-medium">Quality</span>
              <span className="text-sm font-mono text-muted-foreground">{quality}%</span>
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

          <UploadZone
            accept="image/*"
            tool="image-resize"
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