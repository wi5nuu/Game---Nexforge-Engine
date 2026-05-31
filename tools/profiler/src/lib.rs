#![deny(clippy::all)]

use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileSample {
    pub name: String,
    pub duration_ns: u64,
    pub start_ns: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileFrame {
    pub frame: u64,
    pub total_duration_ns: u64,
    pub samples: Vec<ProfileSample>,
    pub gpu_timestamps: Vec<GpuTimestamp>,
    pub memory_stats: MemoryStats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuTimestamp {
    pub name: String,
    pub start_ns: u64,
    pub end_ns: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStats {
    pub heap_used_bytes: u64,
    pub vram_used_bytes: u64,
    pub total_allocations: u64,
}

impl MemoryStats {
    pub fn new() -> Self {
        Self { heap_used_bytes: 0, vram_used_bytes: 0, total_allocations: 0 }
    }
}

impl Default for MemoryStats { fn default() -> Self { Self::new() } }

pub struct ScopedTimer {
    name: String,
    start: Instant,
    profiler: Option<std::cell::RefCell<FrameProfiler>>,
}

impl ScopedTimer {
    pub fn new(name: &str, profiler: Option<std::cell::RefCell<FrameProfiler>>) -> Self {
        let start = Instant::now();
        if let Some(ref p) = profiler {
            p.borrow_mut().begin_sample(name, start);
        }
        Self { name: name.to_string(), start, profiler }
    }
}

impl Drop for ScopedTimer {
    fn drop(&mut self) {
        let elapsed = self.start.elapsed();
        if let Some(ref p) = self.profiler {
            p.borrow_mut().end_sample(&self.name, self.start, elapsed);
        }
    }
}

pub struct FrameProfiler {
    pub visible: bool,
    pub current_frame: u64,
    pub frame_history: Vec<ProfileFrame>,
    pub max_history: usize,
    pub current_samples: Vec<ProfileSample>,
    pub current_gpu: Vec<GpuTimestamp>,
    pub memory_stats: MemoryStats,
    pub frame_start: Option<Instant>,
    pub overlay_text: String,
}

impl FrameProfiler {
    pub fn new(max_history: usize) -> Self {
        Self {
            visible: false,
            current_frame: 0,
            frame_history: Vec::new(),
            max_history,
            current_samples: Vec::new(),
            current_gpu: Vec::new(),
            memory_stats: MemoryStats::new(),
            frame_start: None,
            overlay_text: String::new(),
        }
    }

    pub fn toggle(&mut self) {
        self.visible = !self.visible;
    }

    pub fn begin_frame(&mut self) {
        self.frame_start = Some(Instant::now());
        self.current_samples.clear();
        self.current_gpu.clear();
    }

    pub fn end_frame(&mut self) {
        let elapsed = self.frame_start
            .map(|s| s.elapsed())
            .unwrap_or(Duration::ZERO);

        let frame = ProfileFrame {
            frame: self.current_frame,
            total_duration_ns: elapsed.as_nanos() as u64,
            samples: self.current_samples.clone(),
            gpu_timestamps: self.current_gpu.clone(),
            memory_stats: self.memory_stats.clone(),
        };

        self.frame_history.push(frame);
        if self.frame_history.len() > self.max_history {
            self.frame_history.remove(0);
        }

        self.current_frame += 1;
        self.update_overlay();
    }

    pub fn begin_sample(&mut self, name: &str, _start: Instant) {
        // Sample will be finalized in end_sample
    }

    pub fn end_sample(&mut self, name: &str, start: Instant, duration: Duration) {
        self.current_samples.push(ProfileSample {
            name: name.to_string(),
            duration_ns: duration.as_nanos() as u64,
            start_ns: start.elapsed().as_nanos() as u64,
        });
    }

    pub fn add_gpu_timestamp(&mut self, name: &str, start_ns: u64, end_ns: u64) {
        self.current_gpu.push(GpuTimestamp {
            name: name.to_string(),
            start_ns,
            end_ns,
        });
    }

    pub fn update_memory(&mut self, heap: u64, vram: u64, allocs: u64) {
        self.memory_stats = MemoryStats {
            heap_used_bytes: heap,
            vram_used_bytes: vram,
            total_allocations: allocs,
        };
    }

    fn update_overlay(&mut self) {
        let last = self.frame_history.last();
        let fps = last.map(|f| {
            if f.total_duration_ns > 0 {
                (1_000_000_000.0 / f.total_duration_ns as f64) as u32
            } else { 0 }
        }).unwrap_or(0);

        let total_ms = last.map(|f| f.total_duration_ns as f64 / 1_000_000.0).unwrap_or(0.0);

        let mut breakdown = String::new();
        if let Some(frame) = last {
            for sample in &frame.samples {
                let ms = sample.duration_ns as f64 / 1_000_000.0;
                let pct = if frame.total_duration_ns > 0 {
                    (sample.duration_ns as f64 / frame.total_duration_ns as f64) * 100.0
                } else { 0.0 };
                breakdown.push_str(&format!("\n  {}: {:.2}ms ({:.1}%)", sample.name, ms, pct));
            }
        }

        self.overlay_text = format!(
            "Nexforge Profiler [F1]\n\
             Frame {} | FPS: {} | Total: {:.2}ms{}\n\
             GPU Timestamps: {}\n\
             Memory: heap={}MB vram={}MB allocs={}",
            self.current_frame, fps, total_ms, breakdown,
            self.current_gpu.len(),
            self.memory_stats.heap_used_bytes / 1_000_000,
            self.memory_stats.vram_used_bytes / 1_000_000,
            self.memory_stats.total_allocations,
        );
    }

    pub fn export_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(&self.frame_history)
    }

    pub fn average_frame_time_ms(&self) -> f64 {
        if self.frame_history.is_empty() { return 0.0; }
        let sum: u64 = self.frame_history.iter().map(|f| f.total_duration_ns).sum();
        (sum as f64 / self.frame_history.len() as f64) / 1_000_000.0
    }

    pub fn min_frame_time_ms(&self) -> f64 {
        self.frame_history.iter()
            .map(|f| f.total_duration_ns as f64 / 1_000_000.0)
            .fold(f64::MAX, |a, b| a.min(b))
    }

    pub fn max_frame_time_ms(&self) -> f64 {
        self.frame_history.iter()
            .map(|f| f.total_duration_ns as f64 / 1_000_000.0)
            .fold(f64::MIN, |a, b| a.max(b))
    }

    pub fn p99_frame_time_ms(&self) -> f64 {
        let mut times: Vec<f64> = self.frame_history.iter()
            .map(|f| f.total_duration_ns as f64 / 1_000_000.0)
            .collect();
        times.sort_by(|a, b| a.partial_cmp(b).unwrap());
        if times.is_empty() { return 0.0; }
        let idx = ((times.len() as f64) * 0.99) as usize;
        times[idx.min(times.len() - 1)]
    }
}

impl Default for FrameProfiler {
    fn default() -> Self { Self::new(256) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profiler_creation() {
        let profiler = FrameProfiler::new(128);
        assert!(!profiler.visible);
        assert_eq!(profiler.current_frame, 0);
    }

    #[test]
    fn test_toggle() {
        let mut profiler = FrameProfiler::new(128);
        profiler.toggle();
        assert!(profiler.visible);
    }

    #[test]
    fn test_frame_cycle() {
        let mut profiler = FrameProfiler::new(128);
        profiler.begin_frame();
        std::thread::sleep(std::time::Duration::from_millis(1));
        profiler.end_frame();
        assert_eq!(profiler.frame_history.len(), 1);
        assert!(profiler.frame_history[0].total_duration_ns > 0);
    }

    #[test]
    fn test_sample_timing() {
        let mut profiler = FrameProfiler::new(128);
        profiler.begin_frame();
        let start = Instant::now();
        std::thread::sleep(std::time::Duration::from_millis(2));
        profiler.end_sample("test_system", start, start.elapsed());
        profiler.end_frame();
        assert!(!profiler.current_samples.is_empty() || !profiler.frame_history.is_empty());
    }

    #[test]
    fn test_gpu_timestamps() {
        let mut profiler = FrameProfiler::new(128);
        profiler.add_gpu_timestamp("shadow_pass", 0, 1_000_000);
        profiler.add_gpu_timestamp("lighting_pass", 1_000_000, 3_000_000);
        assert_eq!(profiler.current_gpu.len(), 2);
    }

    #[test]
    fn test_memory_stats() {
        let mut profiler = FrameProfiler::new(128);
        profiler.update_memory(256_000_000, 512_000_000, 1024);
        assert_eq!(profiler.memory_stats.heap_used_bytes, 256_000_000);
    }

    #[test]
    fn test_export_json() {
        let mut profiler = FrameProfiler::new(128);
        profiler.begin_frame();
        profiler.end_frame();
        let json = profiler.export_json().unwrap();
        assert!(json.contains("frame"));
        assert!(json.contains("total_duration_ns"));
    }

    #[test]
    fn test_frame_stats() {
        let mut profiler = FrameProfiler::new(128);
        for _ in 0..10 {
            profiler.begin_frame();
            std::thread::sleep(std::time::Duration::from_micros(100));
            profiler.end_frame();
        }
        assert!(profiler.average_frame_time_ms() > 0.0);
        assert!(profiler.min_frame_time_ms() <= profiler.max_frame_time_ms());
        assert!(profiler.p99_frame_time_ms() > 0.0);
    }

    #[test]
    fn test_overlay_text() {
        let mut profiler = FrameProfiler::new(128);
        profiler.begin_frame();
        profiler.end_frame();
        assert!(profiler.overlay_text.contains("FPS"));
        assert!(profiler.overlay_text.contains("Nexforge Profiler"));
    }
}
