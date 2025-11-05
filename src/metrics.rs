/// Metrics and monitoring for manga scraper sources
///
/// Tracks success rates, error counts, and performance metrics for each source

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceMetrics {
    pub source_name: String,
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub last_success: Option<DateTime<Utc>>,
    pub last_failure: Option<DateTime<Utc>>,
    pub last_error: Option<String>,
    pub average_response_time_ms: f64,
    pub total_response_time_ms: u64,
    pub retry_count: u64,
    pub cloudflare_challenges: u64,
    pub rate_limit_hits: u64,
    pub timeout_count: u64,
}

impl SourceMetrics {
    #[allow(dead_code)]
    pub fn new(source_name: String) -> Self {
        Self {
            source_name,
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            last_success: None,
            last_failure: None,
            last_error: None,
            average_response_time_ms: 0.0,
            total_response_time_ms: 0,
            retry_count: 0,
            cloudflare_challenges: 0,
            rate_limit_hits: 0,
            timeout_count: 0,
        }
    }

    pub fn success_rate(&self) -> f64 {
        if self.total_requests == 0 {
            0.0
        } else {
            (self.successful_requests as f64 / self.total_requests as f64) * 100.0
        }
    }

    #[allow(dead_code)]
    pub fn record_success(&mut self, response_time: Duration) {
        self.total_requests += 1;
        self.successful_requests += 1;
        self.last_success = Some(Utc::now());

        let response_ms = response_time.as_millis() as u64;
        self.total_response_time_ms += response_ms;
        self.average_response_time_ms =
            self.total_response_time_ms as f64 / self.successful_requests as f64;
    }

    #[allow(dead_code)]
    pub fn record_failure(&mut self, error: String) {
        self.total_requests += 1;
        self.failed_requests += 1;
        self.last_failure = Some(Utc::now());
        self.last_error = Some(error.clone());

        // Categorize errors
        if error.contains("429") || error.to_lowercase().contains("rate limit") {
            self.rate_limit_hits += 1;
        } else if error.to_lowercase().contains("cloudflare") ||
                  error.contains("503") || error.contains("520") {
            self.cloudflare_challenges += 1;
        } else if error.to_lowercase().contains("timeout") {
            self.timeout_count += 1;
        }
    }

    #[allow(dead_code)]
    pub fn record_retry(&mut self) {
        self.retry_count += 1;
    }
}

/// Global metrics tracker
pub struct MetricsTracker {
    metrics: Arc<Mutex<HashMap<String, SourceMetrics>>>,
}

impl MetricsTracker {
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    #[allow(dead_code)]
    pub fn get_or_create(&self, source_name: &str) -> SourceMetrics {
        let mut metrics = self.metrics.lock().unwrap();
        metrics
            .entry(source_name.to_string())
            .or_insert_with(|| SourceMetrics::new(source_name.to_string()))
            .clone()
    }

    #[allow(dead_code)]
    pub fn record_success(&self, source_name: &str, response_time: Duration) {
        let mut metrics = self.metrics.lock().unwrap();
        let source_metrics = metrics
            .entry(source_name.to_string())
            .or_insert_with(|| SourceMetrics::new(source_name.to_string()));
        source_metrics.record_success(response_time);

        log::info!(
            "[{}] Success - Response time: {}ms - Success rate: {:.2}%",
            source_name,
            response_time.as_millis(),
            source_metrics.success_rate()
        );
    }

    #[allow(dead_code)]
    pub fn record_failure(&self, source_name: &str, error: String) {
        let mut metrics = self.metrics.lock().unwrap();
        let source_metrics = metrics
            .entry(source_name.to_string())
            .or_insert_with(|| SourceMetrics::new(source_name.to_string()));
        source_metrics.record_failure(error.clone());

        log::warn!(
            "[{}] Failure - Error: {} - Success rate: {:.2}%",
            source_name,
            error,
            source_metrics.success_rate()
        );
    }

    #[allow(dead_code)]
    pub fn record_retry(&self, source_name: &str) {
        let mut metrics = self.metrics.lock().unwrap();
        let source_metrics = metrics
            .entry(source_name.to_string())
            .or_insert_with(|| SourceMetrics::new(source_name.to_string()));
        source_metrics.record_retry();

        log::debug!("[{}] Retry attempt - Total retries: {}", source_name, source_metrics.retry_count);
    }

    #[allow(dead_code)]
    pub fn get_metrics(&self, source_name: &str) -> Option<SourceMetrics> {
        let metrics = self.metrics.lock().unwrap();
        metrics.get(source_name).cloned()
    }

    pub fn get_all_metrics(&self) -> Vec<SourceMetrics> {
        let metrics = self.metrics.lock().unwrap();
        metrics.values().cloned().collect()
    }

    #[allow(dead_code)]
    pub fn print_summary(&self) {
        let metrics = self.metrics.lock().unwrap();
        println!("\n=== Source Performance Summary ===\n");

        let mut sorted_metrics: Vec<_> = metrics.values().collect();
        sorted_metrics.sort_by(|a, b| {
            b.success_rate().partial_cmp(&a.success_rate()).unwrap()
        });

        for m in sorted_metrics {
            println!("Source: {}", m.source_name);
            println!("  Success Rate: {:.2}%", m.success_rate());
            println!("  Total Requests: {}", m.total_requests);
            println!("  Successful: {}", m.successful_requests);
            println!("  Failed: {}", m.failed_requests);
            println!("  Avg Response Time: {:.2}ms", m.average_response_time_ms);
            println!("  Retries: {}", m.retry_count);
            println!("  Rate Limit Hits: {}", m.rate_limit_hits);
            println!("  Cloudflare Challenges: {}", m.cloudflare_challenges);
            println!("  Timeouts: {}", m.timeout_count);
            if let Some(last_error) = &m.last_error {
                println!("  Last Error: {}", last_error);
            }
            println!();
        }
    }

    #[allow(dead_code)]
    pub fn export_json(&self) -> String {
        let metrics = self.metrics.lock().unwrap();
        serde_json::to_string_pretty(&*metrics).unwrap_or_else(|_| "{}".to_string())
    }
}

impl Default for MetricsTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper function to time an operation and record metrics
#[allow(dead_code)]
pub async fn track_request<F, T, E>(
    tracker: &MetricsTracker,
    source_name: &str,
    operation: F,
) -> Result<T, E>
where
    F: std::future::Future<Output = Result<T, E>>,
    E: std::fmt::Display,
{
    let start = Instant::now();
    let result = operation.await;
    let duration = start.elapsed();

    match &result {
        Ok(_) => tracker.record_success(source_name, duration),
        Err(e) => tracker.record_failure(source_name, e.to_string()),
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_creation() {
        let metrics = SourceMetrics::new("test_source".to_string());
        assert_eq!(metrics.source_name, "test_source");
        assert_eq!(metrics.total_requests, 0);
        assert_eq!(metrics.success_rate(), 0.0);
    }

    #[test]
    fn test_record_success() {
        let mut metrics = SourceMetrics::new("test_source".to_string());
        metrics.record_success(Duration::from_millis(100));

        assert_eq!(metrics.total_requests, 1);
        assert_eq!(metrics.successful_requests, 1);
        assert_eq!(metrics.success_rate(), 100.0);
        assert!(metrics.last_success.is_some());
    }

    #[test]
    fn test_record_failure() {
        let mut metrics = SourceMetrics::new("test_source".to_string());
        metrics.record_failure("Test error".to_string());

        assert_eq!(metrics.total_requests, 1);
        assert_eq!(metrics.failed_requests, 1);
        assert_eq!(metrics.success_rate(), 0.0);
        assert_eq!(metrics.last_error, Some("Test error".to_string()));
    }

    #[test]
    fn test_success_rate_calculation() {
        let mut metrics = SourceMetrics::new("test_source".to_string());

        metrics.record_success(Duration::from_millis(100));
        metrics.record_success(Duration::from_millis(200));
        metrics.record_failure("Error".to_string());

        assert_eq!(metrics.total_requests, 3);
        assert_eq!(metrics.successful_requests, 2);
        assert_eq!(metrics.failed_requests, 1);
        assert!((metrics.success_rate() - 66.66).abs() < 0.1);
    }

    #[test]
    fn test_tracker() {
        let tracker = MetricsTracker::new();

        tracker.record_success("source1", Duration::from_millis(100));
        tracker.record_failure("source2", "Error".to_string());

        let metrics1 = tracker.get_metrics("source1").unwrap();
        let metrics2 = tracker.get_metrics("source2").unwrap();

        assert_eq!(metrics1.success_rate(), 100.0);
        assert_eq!(metrics2.success_rate(), 0.0);

        let all_metrics = tracker.get_all_metrics();
        assert_eq!(all_metrics.len(), 2);
    }
}
