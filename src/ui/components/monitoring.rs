//! System monitoring UI components
//!
//! Displays RAM/VRAM usage and generation statistics like tokens-per-second.

use dioxus::prelude::*;

/// Generation statistics tracked during inference
#[derive(Debug, Clone, Default, PartialEq)]
pub struct GenerationStats {
    pub tokens_generated: u32,
    pub tokens_per_second: f32,
    pub context_used: u32,
    pub context_total: u32,
}

/// Monitoring panel component
///
/// Displays system resource usage and generation statistics.
/// Can be placed in sidebar or as a status bar.
#[component]
pub fn MonitoringPanel(
    ram_used_mb: u64,
    ram_total_mb: u64,
    vram_used_mb: u64,
    vram_total_mb: u64,
    generation_stats: Option<GenerationStats>,
) -> Element {
    rsx! {
        div { class: "monitoring-panel",
            // RAM usage
            div { class: "stat-row",
                span { class: "stat-label", "RAM:" }
                span { class: "stat-value", "{ram_used_mb} / {ram_total_mb} MB" }
            }

            // VRAM usage (if GPU available)
            if vram_total_mb > 0 {
                div { class: "stat-row",
                    span { class: "stat-label", "VRAM:" }
                    span { class: "stat-value", "{vram_used_mb} / {vram_total_mb} MB" }
                }
            }

            // Generation stats (during inference)
            if let Some(stats) = generation_stats {
                div { class: "stat-row",
                    span { class: "stat-label", "Speed:" }
                    span { class: "stat-value", "{stats.tokens_per_second:.1} tok/s" }
                }
                div { class: "stat-row",
                    span { class: "stat-label", "Context:" }
                    span { class: "stat-value", "{stats.context_used} / {stats.context_total}" }
                }
            }
        }
    }
}

/// Token timing utility for calculating tokens-per-second
#[derive(Debug)]
pub struct TokenTimer {
    start_time: std::time::Instant,
    token_count: u32,
}

impl TokenTimer {
    /// Create a new timer starting now
    pub fn new() -> Self {
        Self {
            start_time: std::time::Instant::now(),
            token_count: 0,
        }
    }

    /// Increment the token count
    pub fn add_token(&mut self) {
        self.token_count += 1;
    }

    /// Calculate current tokens per second
    pub fn tokens_per_second(&self) -> f32 {
        let elapsed = self.start_time.elapsed().as_secs_f32();
        if elapsed > 0.0 {
            self.token_count as f32 / elapsed
        } else {
            0.0
        }
    }

    /// Get total tokens counted
    pub fn token_count(&self) -> u32 {
        self.token_count
    }

    /// Get elapsed time in seconds
    pub fn elapsed_secs(&self) -> f32 {
        self.start_time.elapsed().as_secs_f32()
    }
}

impl Default for TokenTimer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_token_timer_new() {
        let timer = TokenTimer::new();
        assert_eq!(timer.token_count(), 0);
        assert!(timer.elapsed_secs() >= 0.0);
    }

    #[test]
    fn test_token_timer_add_token() {
        let mut timer = TokenTimer::new();
        assert_eq!(timer.token_count(), 0);

        timer.add_token();
        assert_eq!(timer.token_count(), 1);

        timer.add_token();
        assert_eq!(timer.token_count(), 2);
    }

    #[test]
    fn test_token_timer_tokens_per_second() {
        let mut timer = TokenTimer::new();

        // Add some tokens
        for _ in 0..10 {
            timer.add_token();
        }

        // Wait a bit
        thread::sleep(Duration::from_millis(100));

        let tps = timer.tokens_per_second();
        // Should have some reasonable rate (10 tokens over ~0.1s = ~100 tok/s)
        assert!(tps > 0.0);
        assert!(tps < 1000.0); // Sanity check
    }

    #[test]
    fn test_token_timer_zero_elapsed() {
        let timer = TokenTimer::new();
        // Immediately checking should return 0.0, not panic
        let tps = timer.tokens_per_second();
        assert_eq!(tps, 0.0);
    }

    #[test]
    fn test_generation_stats_default() {
        let stats = GenerationStats::default();
        assert_eq!(stats.tokens_generated, 0);
        assert_eq!(stats.tokens_per_second, 0.0);
        assert_eq!(stats.context_used, 0);
        assert_eq!(stats.context_total, 0);
    }
}
