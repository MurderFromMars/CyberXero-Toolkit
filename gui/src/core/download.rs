//! HTTP file transfers with pause, resume, cancel, and rolling-average
//! progress reporting. Used exclusively by the Arch-ISO download dialog
//! but kept in `core` so anything else that needs a big background fetch
//! can reuse the pause/cancel machinery.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use log::info;
use regex::Regex;

/// Live view of a transfer in flight.
#[derive(Clone, Debug)]
pub struct Progress {
    pub bytes_received: u64,
    /// 0 when the server hasn't disclosed a size yet.
    pub bytes_total: u64,
    pub bytes_per_second: f64,
}

/// Shared switches the UI holds on to so it can pause/resume/cancel the
/// transfer from the main thread while the worker thread drives I/O.
#[derive(Clone, Default)]
pub struct TransferFlags {
    pause: Arc<AtomicBool>,
    cancel: Arc<AtomicBool>,
}

impl TransferFlags {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_paused(&self, on: bool) {
        self.pause.store(on, Ordering::Relaxed);
    }

    pub fn is_paused(&self) -> bool {
        self.pause.load(Ordering::Relaxed)
    }

    pub fn request_cancel(&self) {
        self.cancel.store(true, Ordering::Relaxed);
    }

    fn is_cancelled(&self) -> bool {
        self.cancel.load(Ordering::Relaxed)
    }
}

// ---------------------------------------------------------------------------
// Mirror lookup
// ---------------------------------------------------------------------------

const MIRROR_URL: &str = "https://fastly.mirror.pkgbuild.com/iso/latest/";
const ARCH_ISO_PATTERN: &str = r"archlinux-\d{4}\.\d{2}\.\d{2}-x86_64\.iso";

/// Resolve the latest Arch Linux ISO to `(filename, absolute url)`.
pub async fn latest_arch_iso() -> Result<(String, String)> {
    info!("resolving latest Arch ISO");

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .context("build http client")?;

    let listing = client
        .get(MIRROR_URL)
        .send()
        .await
        .context("fetch mirror index")?
        .text()
        .await
        .context("read mirror index body")?;

    let re = Regex::new(ARCH_ISO_PATTERN)?;
    let filename = re
        .find(&listing)
        .map(|m| m.as_str().to_owned())
        .context("no ISO filename matched in mirror listing")?;

    let url = format!("{MIRROR_URL}{filename}");
    info!("latest ISO: {filename}");
    Ok((filename, url))
}

// ---------------------------------------------------------------------------
// File transfer
// ---------------------------------------------------------------------------

const SPEED_WINDOW: usize = 20;
const PROGRESS_TICK: Duration = Duration::from_millis(100);
const RETRY_BACKOFF: Duration = Duration::from_secs(2);

/// Stream `url` into `dest`, calling `on_progress` roughly every 100ms.
///
/// - Reconnects automatically on transient errors, using HTTP `Range` to
///   resume from where we left off.
/// - Honours [`TransferFlags::set_paused`] by dropping the current
///   connection and sleeping until the flag clears.
/// - Honours [`TransferFlags::request_cancel`] by bailing out and deleting
///   the partial file.
pub async fn stream_to_file<F>(
    url: String,
    dest: String,
    mut on_progress: F,
    flags: TransferFlags,
) -> Result<()>
where
    F: FnMut(Progress) + Send + 'static,
{
    use futures_util::StreamExt;
    use reqwest::header::RANGE;
    use tokio::io::AsyncWriteExt;

    info!("stream_to_file: {url} -> {dest}");

    let client = reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(30))
        .build()
        .context("build http client")?;

    let mut file = tokio::fs::File::create(&dest)
        .await
        .context("create destination file")?;

    // Try HEAD first; a failure here is non-fatal, since the first GET
    // response will usually report `Content-Length` too.
    let mut total: u64 = 0;
    if let Ok(head) = client.head(&url).send().await {
        if let Some(len) = head.content_length() {
            total = len;
            info!("total size from HEAD: {total}");
        }
    }

    let mut received: u64 = 0;
    let mut window = SpeedWindow::with_capacity(SPEED_WINDOW);
    let mut last_tick = Instant::now();
    let mut last_bytes: u64 = 0;

    loop {
        if flags.is_cancelled() {
            cleanup_partial(file, &dest).await;
            anyhow::bail!("Download cancelled");
        }
        if flags.is_paused() {
            tokio::time::sleep(Duration::from_millis(100)).await;
            continue;
        }
        if total > 0 && received >= total {
            break;
        }

        let mut request = client.get(&url);
        if received > 0 {
            info!("resuming at byte {received}");
            request = request.header(RANGE, format!("bytes={received}-"));
        }

        let response = match request.send().await {
            Ok(r) => r,
            Err(e) => {
                info!("connect error: {e}; retrying in {:?}", RETRY_BACKOFF);
                tokio::time::sleep(RETRY_BACKOFF).await;
                continue;
            }
        };

        if total == 0 {
            if let Some(len) = response.content_length() {
                total = received + len;
                info!("total size from GET: {total}");
            }
        }

        let status = response.status();
        if !status.is_success() {
            info!("HTTP {status}");
            if status == reqwest::StatusCode::RANGE_NOT_SATISFIABLE
                && total > 0
                && received >= total
            {
                break;
            }
            tokio::time::sleep(RETRY_BACKOFF).await;
            continue;
        }

        let mut stream = response.bytes_stream();
        let mut interrupted = false;

        while let Some(chunk) = stream.next().await {
            if flags.is_cancelled() {
                cleanup_partial(file, &dest).await;
                anyhow::bail!("Download cancelled");
            }
            if flags.is_paused() {
                info!("paused mid-stream; dropping connection");
                break;
            }

            match chunk {
                Ok(bytes) => {
                    file.write_all(&bytes).await?;
                    received += bytes.len() as u64;

                    let now = Instant::now();
                    if now.duration_since(last_tick) >= PROGRESS_TICK {
                        let elapsed = now.duration_since(last_tick).as_secs_f64();
                        let instant = (received - last_bytes) as f64 / elapsed;
                        window.push(instant);
                        on_progress(Progress {
                            bytes_received: received,
                            bytes_total: total,
                            bytes_per_second: window.average(),
                        });
                        last_tick = now;
                        last_bytes = received;
                    }
                }
                Err(e) => {
                    info!("chunk error: {e}");
                    interrupted = true;
                    break;
                }
            }
        }

        if !interrupted && !flags.is_paused() && (total == 0 || received >= total) {
            break;
        }
    }

    file.flush().await?;
    drop(file);

    on_progress(Progress {
        bytes_received: received,
        bytes_total: total,
        bytes_per_second: 0.0,
    });

    info!("transfer complete: {dest}");
    Ok(())
}

async fn cleanup_partial(file: tokio::fs::File, path: &str) {
    drop(file);
    let _ = tokio::fs::remove_file(path).await;
}

/// Rolling window of recent byte-rate samples used to smooth the speed
/// readout. Samples older than `capacity` entries are evicted.
struct SpeedWindow {
    samples: Vec<f64>,
    capacity: usize,
}

impl SpeedWindow {
    fn with_capacity(capacity: usize) -> Self {
        Self {
            samples: Vec::with_capacity(capacity),
            capacity,
        }
    }

    fn push(&mut self, sample: f64) {
        if self.samples.len() == self.capacity {
            self.samples.remove(0);
        }
        self.samples.push(sample);
    }

    fn average(&self) -> f64 {
        if self.samples.is_empty() {
            return 0.0;
        }
        self.samples.iter().sum::<f64>() / self.samples.len() as f64
    }
}

// ---------------------------------------------------------------------------
// Formatting helpers for the progress UI
// ---------------------------------------------------------------------------

const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];

/// Turn a byte count into a short human string (`4.20 GB`). Whole bytes
/// render without decimals so the UI doesn't flicker at small sizes.
pub fn humanize_bytes(bytes: u64) -> String {
    let mut value = bytes as f64;
    let mut idx = 0usize;
    while value >= 1024.0 && idx + 1 < UNITS.len() {
        value /= 1024.0;
        idx += 1;
    }
    if idx == 0 {
        format!("{bytes} {}", UNITS[idx])
    } else {
        format!("{value:.2} {}", UNITS[idx])
    }
}

pub fn humanize_rate(bytes_per_sec: f64) -> String {
    format!("{}/s", humanize_bytes(bytes_per_sec as u64))
}

/// Render an ETA. Zero is treated as "just about done" rather than
/// "calculating" so the UI doesn't flash placeholder text at the end.
pub fn humanize_eta(seconds: u64) -> String {
    if seconds == 0 {
        return String::from("Less than 1s");
    }
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;
    if hours > 0 {
        format!("{hours}h {minutes}m {secs}s")
    } else if minutes > 0 {
        format!("{minutes}m {secs}s")
    } else {
        format!("{secs}s")
    }
}
