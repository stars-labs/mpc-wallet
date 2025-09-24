//! Loading States & Progress Indicators
//!
//! This module provides loading states, spinners, and progress bars
//! for asynchronous operations in the TUI.

use std::time::{Duration, Instant};
use std::collections::HashMap;

/// Loading state for async operations
#[derive(Debug, Clone)]
pub enum LoadingState {
    /// No operation in progress
    Idle,
    
    /// Operation in progress
    Loading {
        message: String,
        progress: Option<f32>,
        started_at: Instant,
        estimated_completion: Option<Instant>,
        cancelable: bool,
    },
    
    /// Operation completed successfully
    Success {
        message: String,
        completed_at: Instant,
        duration: Duration,
    },
    
    /// Operation failed
    Error {
        message: String,
        error: String,
        failed_at: Instant,
        recoverable: bool,
    },
}

impl LoadingState {
    /// Create a new loading state
    pub fn loading(message: impl Into<String>) -> Self {
        Self::Loading {
            message: message.into(),
            progress: None,
            started_at: Instant::now(),
            estimated_completion: None,
            cancelable: false,
        }
    }
    
    /// Create a cancelable loading state
    pub fn loading_cancelable(message: impl Into<String>) -> Self {
        Self::Loading {
            message: message.into(),
            progress: None,
            started_at: Instant::now(),
            estimated_completion: None,
            cancelable: true,
        }
    }
    
    /// Update progress
    pub fn with_progress(mut self, progress: f32) -> Self {
        if let Self::Loading { progress: ref mut p, started_at, estimated_completion: ref mut est, .. } = self {
            *p = Some(progress.clamp(0.0, 1.0));
            
            // Estimate completion time based on progress
            if progress > 0.0 && progress < 1.0 {
                let elapsed = started_at.elapsed();
                let total_estimated = elapsed.as_secs_f32() / progress;
                let remaining = Duration::from_secs_f32(total_estimated - elapsed.as_secs_f32());
                *est = Some(Instant::now() + remaining);
            }
        }
        self
    }
    
    /// Mark as success
    pub fn success(message: impl Into<String>, started_at: Instant) -> Self {
        Self::Success {
            message: message.into(),
            completed_at: Instant::now(),
            duration: started_at.elapsed(),
        }
    }
    
    /// Mark as error
    pub fn error(message: impl Into<String>, error: impl Into<String>, recoverable: bool) -> Self {
        Self::Error {
            message: message.into(),
            error: error.into(),
            failed_at: Instant::now(),
            recoverable,
        }
    }
    
    /// Check if operation is in progress
    pub fn is_loading(&self) -> bool {
        matches!(self, Self::Loading { .. })
    }
    
    /// Get elapsed time for loading operation
    pub fn elapsed(&self) -> Option<Duration> {
        match self {
            Self::Loading { started_at, .. } => Some(started_at.elapsed()),
            Self::Success { duration, .. } => Some(*duration),
            _ => None,
        }
    }
    
    /// Get estimated time remaining
    pub fn time_remaining(&self) -> Option<Duration> {
        if let Self::Loading { estimated_completion: Some(est), .. } = self {
            let now = Instant::now();
            if *est > now {
                Some(*est - now)
            } else {
                Some(Duration::ZERO)
            }
        } else {
            None
        }
    }
}

/// Animated spinner for loading states
#[derive(Debug, Clone)]
pub struct Spinner {
    frames: Vec<&'static str>,
    current_frame: usize,
    last_update: Instant,
    update_interval: Duration,
}

impl Spinner {
    /// Create a new spinner with default style
    pub fn new() -> Self {
        Self::dots()
    }
    
    /// Dots spinner: ⠋ ⠙ ⠹ ⠸ ⠼ ⠴ ⠦ ⠧ ⠇ ⠏
    pub fn dots() -> Self {
        Self {
            frames: vec!["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"],
            current_frame: 0,
            last_update: Instant::now(),
            update_interval: Duration::from_millis(80),
        }
    }
    
    /// Line spinner: - \ | /
    pub fn line() -> Self {
        Self {
            frames: vec!["-", "\\", "|", "/"],
            current_frame: 0,
            last_update: Instant::now(),
            update_interval: Duration::from_millis(100),
        }
    }
    
    /// Arrow spinner: ← ↖ ↑ ↗ → ↘ ↓ ↙
    pub fn arrow() -> Self {
        Self {
            frames: vec!["←", "↖", "↑", "↗", "→", "↘", "↓", "↙"],
            current_frame: 0,
            last_update: Instant::now(),
            update_interval: Duration::from_millis(100),
        }
    }
    
    /// Get current frame and advance if needed
    pub fn tick(&mut self) -> &str {
        let now = Instant::now();
        if now.duration_since(self.last_update) >= self.update_interval {
            self.current_frame = (self.current_frame + 1) % self.frames.len();
            self.last_update = now;
        }
        self.frames[self.current_frame]
    }
    
    /// Reset spinner to first frame
    pub fn reset(&mut self) {
        self.current_frame = 0;
        self.last_update = Instant::now();
    }
}

impl Default for Spinner {
    fn default() -> Self {
        Self::new()
    }
}

/// Progress bar for operations with known progress
#[derive(Debug, Clone)]
pub struct ProgressBar {
    pub current: f32,
    pub total: f32,
    pub width: usize,
    pub filled_char: char,
    pub empty_char: char,
    pub show_percentage: bool,
    pub show_eta: bool,
    pub started_at: Option<Instant>,
}

impl ProgressBar {
    /// Create a new progress bar
    pub fn new() -> Self {
        Self {
            current: 0.0,
            total: 100.0,
            width: 20,
            filled_char: '█',
            empty_char: '░',
            show_percentage: true,
            show_eta: true,
            started_at: None,
        }
    }
    
    /// Set progress (0.0 to 1.0)
    pub fn set_progress(&mut self, progress: f32) {
        self.current = (progress * self.total).clamp(0.0, self.total);
        if self.started_at.is_none() && progress > 0.0 {
            self.started_at = Some(Instant::now());
        }
    }
    
    /// Get progress as percentage
    pub fn percentage(&self) -> f32 {
        if self.total > 0.0 {
            (self.current / self.total * 100.0).clamp(0.0, 100.0)
        } else {
            0.0
        }
    }
    
    /// Render the progress bar as a string
    pub fn render(&self) -> String {
        let progress = self.current / self.total;
        let filled_width = (self.width as f32 * progress) as usize;
        let empty_width = self.width - filled_width;
        
        let mut result = String::new();
        result.push('[');
        
        for _ in 0..filled_width {
            result.push(self.filled_char);
        }
        
        for _ in 0..empty_width {
            result.push(self.empty_char);
        }
        
        result.push(']');
        
        if self.show_percentage {
            result.push_str(&format!(" {:.0}%", self.percentage()));
        }
        
        if self.show_eta {
            if let Some(eta) = self.estimate_time_remaining() {
                result.push_str(&format!(" ETA: {}", format_duration(eta)));
            }
        }
        
        result
    }
    
    /// Estimate time remaining based on current progress
    pub fn estimate_time_remaining(&self) -> Option<Duration> {
        if let Some(started_at) = self.started_at {
            if self.current > 0.0 && self.current < self.total {
                let elapsed = started_at.elapsed();
                let total_estimated = elapsed.as_secs_f32() * (self.total / self.current);
                let remaining = Duration::from_secs_f32(total_estimated - elapsed.as_secs_f32());
                Some(remaining)
            } else {
                None
            }
        } else {
            None
        }
    }
}

impl Default for ProgressBar {
    fn default() -> Self {
        Self::new()
    }
}

/// Multi-stage progress tracker
#[derive(Debug, Clone)]
pub struct MultiStageProgress {
    pub stages: Vec<Stage>,
    pub current_stage: usize,
}

#[derive(Debug, Clone)]
pub struct Stage {
    pub name: String,
    pub status: StageStatus,
    pub progress: Option<f32>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StageStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    Skipped,
}

impl MultiStageProgress {
    /// Create a new multi-stage progress tracker
    pub fn new(stage_names: Vec<String>) -> Self {
        let stages = stage_names
            .into_iter()
            .map(|name| Stage {
                name,
                status: StageStatus::Pending,
                progress: None,
            })
            .collect();
        
        Self {
            stages,
            current_stage: 0,
        }
    }
    
    /// Start the current stage
    pub fn start_stage(&mut self) {
        if self.current_stage < self.stages.len() {
            self.stages[self.current_stage].status = StageStatus::InProgress;
        }
    }
    
    /// Update progress of current stage
    pub fn update_progress(&mut self, progress: f32) {
        if self.current_stage < self.stages.len() {
            self.stages[self.current_stage].progress = Some(progress.clamp(0.0, 1.0));
        }
    }
    
    /// Complete current stage and move to next
    pub fn complete_stage(&mut self) {
        if self.current_stage < self.stages.len() {
            self.stages[self.current_stage].status = StageStatus::Completed;
            self.stages[self.current_stage].progress = Some(1.0);
            self.current_stage += 1;
            
            // Start next stage if available
            if self.current_stage < self.stages.len() {
                self.start_stage();
            }
        }
    }
    
    /// Mark current stage as failed
    pub fn fail_stage(&mut self) {
        if self.current_stage < self.stages.len() {
            self.stages[self.current_stage].status = StageStatus::Failed;
        }
    }
    
    /// Skip current stage
    pub fn skip_stage(&mut self) {
        if self.current_stage < self.stages.len() {
            self.stages[self.current_stage].status = StageStatus::Skipped;
            self.current_stage += 1;
            
            // Start next stage if available
            if self.current_stage < self.stages.len() {
                self.start_stage();
            }
        }
    }
    
    /// Get overall progress (0.0 to 1.0)
    pub fn overall_progress(&self) -> f32 {
        if self.stages.is_empty() {
            return 0.0;
        }
        
        let mut completed = 0.0;
        
        for stage in &self.stages {
            match stage.status {
                StageStatus::Completed => completed += 1.0,
                StageStatus::InProgress => {
                    if let Some(progress) = stage.progress {
                        completed += progress;
                    }
                }
                StageStatus::Skipped => completed += 1.0,
                _ => {}
            }
        }
        
        completed / self.stages.len() as f32
    }
    
    /// Check if all stages are complete
    pub fn is_complete(&self) -> bool {
        self.stages.iter().all(|s| {
            matches!(s.status, StageStatus::Completed | StageStatus::Skipped)
        })
    }
    
    /// Check if any stage failed
    pub fn has_failed(&self) -> bool {
        self.stages.iter().any(|s| s.status == StageStatus::Failed)
    }
}

/// Progress manager for tracking multiple operations
#[derive(Debug, Clone)]
pub struct ProgressManager {
    operations: HashMap<String, LoadingState>,
}

impl ProgressManager {
    /// Create a new progress manager
    pub fn new() -> Self {
        Self {
            operations: HashMap::new(),
        }
    }
    
    /// Start a new operation
    pub fn start_operation(&mut self, id: impl Into<String>, message: impl Into<String>) {
        self.operations.insert(id.into(), LoadingState::loading(message));
    }
    
    /// Update operation progress
    pub fn update_progress(&mut self, id: &str, progress: f32) {
        if let Some(state) = self.operations.get_mut(id) {
            *state = state.clone().with_progress(progress);
        }
    }
    
    /// Complete an operation
    pub fn complete_operation(&mut self, id: &str, message: impl Into<String>) {
        if let Some(LoadingState::Loading { started_at, .. }) = self.operations.get(id) {
            let started = *started_at;
            self.operations.insert(
                id.to_string(),
                LoadingState::success(message, started)
            );
        }
    }
    
    /// Fail an operation
    pub fn fail_operation(&mut self, id: &str, error: impl Into<String>) {
        self.operations.insert(
            id.to_string(),
            LoadingState::error("Operation failed", error, true)
        );
    }
    
    /// Get operation state
    pub fn get_operation(&self, id: &str) -> Option<&LoadingState> {
        self.operations.get(id)
    }
    
    /// Remove completed operations older than duration
    pub fn cleanup_old_operations(&mut self, age: Duration) {
        let now = Instant::now();
        self.operations.retain(|_, state| {
            match state {
                LoadingState::Success { completed_at, .. } => {
                    now.duration_since(*completed_at) < age
                }
                LoadingState::Error { failed_at, .. } => {
                    now.duration_since(*failed_at) < age
                }
                _ => true, // Keep loading and idle states
            }
        });
    }
    
    /// Get all active operations
    pub fn active_operations(&self) -> Vec<(&String, &LoadingState)> {
        self.operations
            .iter()
            .filter(|(_, state)| state.is_loading())
            .collect()
    }
}

impl Default for ProgressManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Format duration in human-readable format
fn format_duration(duration: Duration) -> String {
    let secs = duration.as_secs();
    
    if secs < 60 {
        format!("{}s", secs)
    } else if secs < 3600 {
        format!("{}m {}s", secs / 60, secs % 60)
    } else {
        format!("{}h {}m", secs / 3600, (secs % 3600) / 60)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_loading_state() {
        let state = LoadingState::loading("Testing");
        assert!(state.is_loading());
        
        let state = state.with_progress(0.5);
        if let LoadingState::Loading { progress, .. } = state {
            assert_eq!(progress, Some(0.5));
        } else {
            panic!("Expected loading state");
        }
    }
    
    #[test]
    fn test_spinner() {
        let mut spinner = Spinner::dots();
        let frame1 = spinner.tick();
        assert_eq!(frame1, "⠋");
        
        // Force advance
        spinner.current_frame = 1;
        let frame2 = spinner.tick();
        assert_eq!(frame2, "⠙");
    }
    
    #[test]
    fn test_progress_bar() {
        let mut bar = ProgressBar::new();
        bar.set_progress(0.5);
        
        assert_eq!(bar.percentage(), 50.0);
        
        let rendered = bar.render();
        assert!(rendered.contains("50%"));
        assert!(rendered.contains('█'));
        assert!(rendered.contains('░'));
    }
    
    #[test]
    fn test_multi_stage_progress() {
        let mut progress = MultiStageProgress::new(vec![
            "Stage 1".to_string(),
            "Stage 2".to_string(),
            "Stage 3".to_string(),
        ]);
        
        assert_eq!(progress.overall_progress(), 0.0);
        
        progress.start_stage();
        progress.update_progress(0.5);
        assert!(progress.overall_progress() > 0.0 && progress.overall_progress() < 0.2);
        
        progress.complete_stage();
        assert!(progress.overall_progress() > 0.33);
        
        progress.complete_stage();
        progress.complete_stage();
        
        assert!(progress.is_complete());
        assert_eq!(progress.overall_progress(), 1.0);
    }
}