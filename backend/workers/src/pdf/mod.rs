use std::path::Path;

use crate::config::WorkerConfig;
use crate::nats::progress::ProgressReporter;
use tools_common::types::{Job, JobStatus, Tool};

mod merge;
mod split;
mod images_to_pdf;
mod compress;

/// Process a PDF tool job.
pub async fn process_job(
    job: Job,
    redis: &redis::Client,
    config: &WorkerConfig,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing::info!(job_id = %job.id, tool = %job.tool.as_str(), "Processing PDF job");

    let nats = async_nats::connect(&config.nats_url).await?;
    let progress = ProgressReporter::new(redis.clone(), nats, job.id, job.tool.clone());

    progress
        .report(JobStatus::Processing { stage: "process".to_string(), progress: 30 }, "process", 30, "Memproses PDF...")
        .await?;

    let output_dir = config.storage_path.join("output");
    tokio::fs::create_dir_all(&output_dir).await?;

    let input_path = Path::new(&job.file_path);
    let result_path = match job.tool {
        Tool::PdfMerge => merge::process(&job, &output_dir, &progress).await?,
        Tool::PdfSplit => split::process(input_path, &job.options, &output_dir, &progress).await?,
        Tool::ImagesToPdf => images_to_pdf::process(&job, &output_dir, &progress).await?,
        Tool::PdfCompress => compress::process(input_path, &job.options, &output_dir, &progress).await?,
        _ => {
            // Fallback: copy input as-is
            let output_path = output_dir.join(format!("{}.pdf", job.id));
            tokio::fs::copy(input_path, &output_path).await?;
            output_path
        }
    };

    progress
        .report(JobStatus::Completed, "complete", 100, "Selesai")
        .await?;

    let mut conn = redis.get_multiplexed_async_connection().await?;
    crate::nats::consumer::JobConsumer::update_job_result(
        &mut conn,
        job.id,
        &result_path.to_string_lossy(),
        job.ttl_seconds,
    )
    .await?;

    tracing::info!(job_id = %job.id, output = %result_path.display(), "PDF job completed");
    Ok(())
}