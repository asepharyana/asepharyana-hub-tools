use std::path::Path;

use crate::config::WorkerConfig;
use crate::nats::progress::ProgressReporter;
use tools_common::types::{Job, JobStatus, Tool};

mod compress;
mod convert;
mod resize;

/// Process an image tool job.
pub async fn process_job(
    job: Job,
    redis: &redis::Client,
    config: &WorkerConfig,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing::info!(job_id = %job.id, tool = %job.tool.as_str(), "Processing image job");

    let nats = async_nats::connect(&config.nats_url).await?;
    let progress = ProgressReporter::new(redis.clone(), nats, job.id, job.tool.clone());

    progress
        .report(JobStatus::Processing { stage: "load".to_string(), progress: 10 }, "load", 10, "Memuat gambar...")
        .await?;

    let input_path = Path::new(&job.file_path);
    let img = image::open(input_path)
        .map_err(|e| format!("Failed to load image: {}", e))?;

    progress
        .report(JobStatus::Processing { stage: "process".to_string(), progress: 50 }, "process", 50, "Memproses...")
        .await?;

    let output_dir = config.storage_path.join("output");
    tokio::fs::create_dir_all(&output_dir).await?;

    let result_path = match job.tool {
        Tool::ImageCompress => compress::process(&img, &job.options, &output_dir, &progress).await?,
        Tool::ImageResize => resize::process(&img, &job.options, &output_dir, &progress).await?,
        Tool::ImageConvert => convert::process(&img, &job.options, &output_dir, &progress).await?,
        _ => {
            // Fallback: save as-is
            let output_path = output_dir.join(format!("{}.png", job.id));
            img.save(&output_path)?;
            output_path
        }
    };

    progress
        .report(JobStatus::Completed, "complete", 100, "Selesai")
        .await?;

    // Update Redis with result
    let mut conn = redis.get_multiplexed_async_connection().await?;
    crate::nats::consumer::JobConsumer::update_job_result(
        &mut conn,
        job.id,
        &result_path.to_string_lossy(),
        job.ttl_seconds,
    )
    .await?;

    tracing::info!(job_id = %job.id, output = %result_path.display(), "Image job completed");
    Ok(())
}