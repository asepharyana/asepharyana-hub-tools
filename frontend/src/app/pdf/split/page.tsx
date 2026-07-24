"use client";

import { useState, useCallback } from "react";
import { Split } from "lucide-react";
import { ToolLayout } from "@/components/tools/tool-layout";
import { UploadZone } from "@/components/tools/upload-zone";
import { useUpload } from "@/hooks/use-upload";
import { useJobStatus } from "@/hooks/use-job-status";
import { ProgressBar } from "@/components/tools/progress-bar";
import { ResultPreview } from "@/components/tools/result-preview";

type PageState = "upload" | "processing" | "result" | "error";

export default function PdfSplitPage() {
  const [pageState, setPageState] = useState<PageState>("upload");
  const [jobId, setJobId] = useState<string | null>(null);
  const [errorMsg, setErrorMsg] = useState<string | null>(null);
  const [pages, setPages] = useState("1,3-5");

  const { upload } = useUpload({
    tool: "pdf-split",
    options: { pages },
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
      title="Split PDF"
      description="Ekstrak halaman tertentu dari PDF"
      icon={Split}
      phase={1}
    >
      {pageState === "upload" && (
        <div className="space-y-6">
          <div className="p-6 rounded-xl border glass space-y-3">
            <h3 className="font-semibold">Page Range</h3>
            <p className="text-xs text-muted-foreground">
              Contoh: <code className="bg-muted px-1 rounded">1-3,5,7-9</code> ambil halaman 1-3, 5, dan 7-9
            </p>
            <input
              type="text"
              value={pages}
              onChange={(e) => setPages(e.target.value)}
              className="w-full bg-muted border rounded px-3 py-2 text-sm font-mono"
              placeholder="1-3,5,7-9"
            />
          </div>

          <UploadZone
            accept=".pdf,application/pdf"
            tool="pdf-split"
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