#![deny(clippy::all)]

use cpal::traits::HostTrait;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AudioError {
    #[error("cpal device error: {0}")]
    DeviceError(String),
    #[error("Audio stream error")]
    StreamError,
    #[error("Sample not found: {0}")]
    SampleNotFound(String),
}

pub enum AudioBus {
    Sfx,
    Music,
    Voice,
    Ambient,
}

impl AudioBus {
    pub fn name(&self) -> &str {
        match self { AudioBus::Sfx => "sfx", AudioBus::Music => "music", AudioBus::Voice => "voice", AudioBus::Ambient => "ambient" }
    }
    pub fn index(&self) -> usize {
        match self { AudioBus::Sfx => 0, AudioBus::Music => 1, AudioBus::Voice => 2, AudioBus::Ambient => 3 }
    }
}

pub struct AudioClip {
    pub samples: Vec<f32>,
    pub sample_rate: u32,
    pub channels: u16,
    pub duration: f32,
}

impl AudioClip {
    pub fn new(samples: Vec<f32>, sample_rate: u32, channels: u16) -> Self {
        let duration = if sample_rate > 0 { samples.len() as f32 / (sample_rate * channels as u32) as f32 } else { 0.0 };
        Self { samples, sample_rate, channels, duration }
    }

    pub fn sample_count(&self) -> usize { self.samples.len() }

    pub fn sine_wave(freq: f32, duration: f32, sample_rate: u32) -> Self {
        let num_samples = (sample_rate as f32 * duration) as usize;
        let samples: Vec<f32> = (0..num_samples).map(|i| {
            let t = i as f32 / sample_rate as f32;
            (2.0 * std::f32::consts::PI * freq * t).sin() * 0.5
        }).collect();
        Self::new(samples, sample_rate, 1)
    }
}

pub struct SpatialAudioSource {
    pub position: [f32; 3],
    pub velocity: [f32; 3],
    pub min_distance: f32,
    pub max_distance: f32,
    pub volume: f32,
    pub pitch: f32,
    pub looping: bool,
    pub clip: Option<AudioClip>,
}

impl SpatialAudioSource {
    pub fn new() -> Self {
        Self { position: [0.0; 3], velocity: [0.0; 3], min_distance: 1.0, max_distance: 50.0, volume: 1.0, pitch: 1.0, looping: false, clip: None }
    }

    pub fn set_position(&mut self, pos: [f32; 3]) { self.position = pos; }

    pub fn set_velocity(&mut self, vel: [f32; 3]) { self.velocity = vel; }

    pub fn calculate_attenuation(&self, listener_pos: [f32; 3]) -> f32 {
        let dx = self.position[0] - listener_pos[0];
        let dy = self.position[1] - listener_pos[1];
        let dz = self.position[2] - listener_pos[2];
        let dist = (dx * dx + dy * dy + dz * dz).sqrt();
        if dist <= self.min_distance { 1.0 }
        else if dist >= self.max_distance { 0.0 }
        else { 1.0 - (dist - self.min_distance) / (self.max_distance - self.min_distance) }
    }

    pub fn doppler_pitch(&self, listener_vel: [f32; 3], listener_pos: [f32; 3], speed_of_sound: f32) -> f32 {
        let dx = self.position[0] - listener_pos[0];
        let dy = self.position[1] - listener_pos[1];
        let dz = self.position[2] - listener_pos[2];
        let dist = (dx * dx + dy * dy + dz * dz).sqrt().max(0.001);
        let dir = [dx / dist, dy / dist, dz / dist];
        let v_rel = (self.velocity[0] - listener_vel[0]) * dir[0] + (self.velocity[1] - listener_vel[1]) * dir[1] + (self.velocity[2] - listener_vel[2]) * dir[2];
        (speed_of_sound + v_rel) / speed_of_sound
    }
}

impl Default for SpatialAudioSource { fn default() -> Self { Self::new() } }

pub struct AudioBusChannel {
    pub volume: f32,
    pub sources: Vec<SpatialAudioSource>,
}

impl AudioBusChannel {
    pub fn new() -> Self { Self { volume: 1.0, sources: Vec::new() } }

    pub fn get_volume(&self) -> f32 { self.volume }

    pub fn set_volume(&mut self, vol: f32) { self.volume = vol.clamp(0.0, 1.0); }
}

impl Default for AudioBusChannel { fn default() -> Self { Self::new() } }

pub struct AudioEngine {
    pub master_volume: f32,
    pub listener_position: [f32; 3],
    pub listener_velocity: [f32; 3],
    pub listener_forward: [f32; 3],
    pub speed_of_sound: f32,
    pub buses: [AudioBusChannel; 4],
    initialized: bool,
}

impl AudioEngine {
    pub fn new() -> Self {
        Self {
            master_volume: 1.0, listener_position: [0.0; 3], listener_velocity: [0.0; 3],
            listener_forward: [0.0, 0.0, -1.0], speed_of_sound: 343.0,
            buses: [AudioBusChannel::new(), AudioBusChannel::new(), AudioBusChannel::new(), AudioBusChannel::new()],
            initialized: false,
        }
    }

    pub fn initialize(&mut self) -> Result<(), AudioError> {
        let host = cpal::default_host();
        match host.default_output_device() {
            Some(_device) => {
                log::info!("[Audio] Output device found");
                self.initialized = true;
                Ok(())
            }
            None => {
                log::warn!("[Audio] No output device found — running silent");
                self.initialized = true;
                Ok(())
            }
        }
    }

    pub fn play(&mut self, clip: AudioClip, bus: AudioBus) {
        let mut source = SpatialAudioSource::new();
        source.clip = Some(clip);
        self.buses[bus.index()].sources.push(source);
    }

    pub fn play_3d(&mut self, clip: AudioClip, position: [f32; 3], bus: AudioBus) {
        let mut source = SpatialAudioSource::new();
        source.position = position;
        source.clip = Some(clip);
        self.buses[bus.index()].sources.push(source);
    }

    pub fn set_listener_position(&mut self, pos: [f32; 3]) {
        self.listener_position = pos;
    }

    pub fn get_listener_position(&self) -> [f32; 3] {
        self.listener_position
    }

    pub fn set_listener_velocity(&mut self, vel: [f32; 3]) {
        self.listener_velocity = vel;
    }

    pub fn set_master_volume(&mut self, volume: f32) {
        self.master_volume = volume.clamp(0.0, 1.0);
    }

    pub fn set_speed_of_sound(&mut self, speed: f32) {
        self.speed_of_sound = speed;
    }

    pub fn stop_all(&mut self) {
        for bus in &mut self.buses {
            bus.sources.clear();
        }
    }

    pub fn num_active_sources(&self) -> usize {
        self.buses.iter().map(|b| b.sources.len()).sum()
    }

    pub fn bus_source_count(&self, bus: &AudioBus) -> usize {
        self.buses[bus.index()].sources.len()
    }

    pub fn get_master_volume(&self) -> f32 {
        self.master_volume
    }

    pub fn get_bus_volume(&self, bus: &AudioBus) -> f32 { self.buses[bus.index()].volume }

    pub fn set_bus_volume(&mut self, bus: AudioBus, volume: f32) {
        self.buses[bus.index()].volume = volume.clamp(0.0, 1.0);
    }

    pub fn mix_sample(&self, bus_idx: usize, _sample_index: usize) -> f32 {
        let bus = &self.buses[bus_idx];
        if bus.sources.is_empty() { return 0.0; }
        let mut mixed = 0.0f32;
        for source in &bus.sources {
            if let Some(ref _clip) = source.clip {
                let attenuation = source.calculate_attenuation(self.listener_position);
                mixed += attenuation * source.volume * bus.volume * self.master_volume;
            }
        }
        mixed.clamp(-1.0, 1.0)
    }

    pub fn is_initialized(&self) -> bool { self.initialized }
}

impl Default for AudioEngine { fn default() -> Self { Self::new() } }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_init() {
        let mut engine = AudioEngine::new();
        assert!(!engine.is_initialized());
        assert!(engine.initialize().is_ok());
        assert!(engine.is_initialized());
    }

    #[test]
    fn test_default_volume() {
        let engine = AudioEngine::new();
        assert!((engine.master_volume - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_sine_wave() {
        let clip = AudioClip::sine_wave(440.0, 1.0, 44100);
        assert_eq!(clip.sample_rate, 44100);
        assert!((clip.duration - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_spatial_attenuation() {
        let source = SpatialAudioSource::new();
        let atten = source.calculate_attenuation([100.0, 0.0, 0.0]);
        assert!((atten - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_doppler_shift() {
        let mut source = SpatialAudioSource::new();
        source.velocity = [10.0, 0.0, 0.0];
        let pitch = source.doppler_pitch([0.0; 3], [0.0; 3], 343.0);
        assert!(pitch > 0.0);
    }

    #[test]
    fn test_audio_bus_system() {
        let mut engine = AudioEngine::new();
        engine.set_bus_volume(AudioBus::Music, 0.5);
        assert!((engine.buses[AudioBus::Music.index()].volume - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_listener_position() {
        let mut engine = AudioEngine::new();
        engine.listener_position = [10.0, 5.0, 0.0];
        assert_eq!(engine.listener_position, [10.0, 5.0, 0.0]);
    }

    #[test]
    fn test_mix_sample_empty_bus() {
        let engine = AudioEngine::new();
        let sample = engine.mix_sample(AudioBus::Sfx.index(), 0);
        assert_eq!(sample, 0.0);
    }

    #[test]
    fn test_speed_of_sound() {
        let engine = AudioEngine::new();
        assert!((engine.speed_of_sound - 343.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_set_listener_position() {
        let mut engine = AudioEngine::new();
        engine.set_listener_position([10.0, 20.0, 30.0]);
        assert_eq!(engine.get_listener_position(), [10.0, 20.0, 30.0]);
    }

    #[test]
    fn test_set_master_volume() {
        let mut engine = AudioEngine::new();
        engine.set_master_volume(0.5);
        assert!((engine.master_volume - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_set_listener_velocity() {
        let mut engine = AudioEngine::new();
        engine.set_listener_velocity([1.0, 2.0, 3.0]);
        assert_eq!(engine.listener_velocity, [1.0, 2.0, 3.0]);
    }

    #[test]
    fn test_set_speed_of_sound() {
        let mut engine = AudioEngine::new();
        engine.set_speed_of_sound(300.0);
        assert!((engine.speed_of_sound - 300.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_stop_all() {
        let mut engine = AudioEngine::new();
        let clip = AudioClip::sine_wave(440.0, 0.1, 44100);
        engine.play(clip, AudioBus::Sfx);
        assert_eq!(engine.buses[AudioBus::Sfx.index()].sources.len(), 1);
        engine.stop_all();
        assert_eq!(engine.buses[AudioBus::Sfx.index()].sources.len(), 0);
    }

    #[test]
    fn test_play_3d_creates_source() {
        let mut engine = AudioEngine::new();
        let clip = AudioClip::sine_wave(440.0, 0.1, 44100);
        engine.play_3d(clip, [5.0, 0.0, 0.0], AudioBus::Sfx);
        assert_eq!(engine.buses[AudioBus::Sfx.index()].sources.len(), 1);
    }

    #[test]
    fn test_num_active_sources() {
        let mut engine = AudioEngine::new();
        assert_eq!(engine.num_active_sources(), 0);
        let clip = AudioClip::sine_wave(440.0, 0.1, 44100);
        engine.play(clip, AudioBus::Sfx);
        assert_eq!(engine.num_active_sources(), 1);
    }

    #[test]
    fn test_get_master_volume() {
        let engine = AudioEngine::new();
        assert!((engine.get_master_volume() - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_bus_source_count() {
        let mut engine = AudioEngine::new();
        assert_eq!(engine.bus_source_count(&AudioBus::Sfx), 0);
        let clip = AudioClip::sine_wave(440.0, 0.1, 44100);
        engine.play(clip, AudioBus::Music);
        assert_eq!(engine.bus_source_count(&AudioBus::Music), 1);
        assert_eq!(engine.bus_source_count(&AudioBus::Sfx), 0);
    }

    #[test]
    fn test_sample_count() {
        let clip = AudioClip::sine_wave(440.0, 1.0, 44100);
        assert_eq!(clip.sample_count(), 44100);
    }

    #[test]
    fn test_audio_bus_channel_volume() {
        let mut channel = AudioBusChannel::new();
        assert!((channel.get_volume() - 1.0).abs() < f32::EPSILON);
        channel.set_volume(0.5);
        assert!((channel.get_volume() - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_get_bus_volume() {
        let mut engine = AudioEngine::new();
        assert!((engine.get_bus_volume(&AudioBus::Music) - 1.0).abs() < f32::EPSILON);
        engine.set_bus_volume(AudioBus::Music, 0.3);
        assert!((engine.get_bus_volume(&AudioBus::Music) - 0.3).abs() < f32::EPSILON);
    }

    #[test]
    fn test_spatial_source_set_position() {
        let mut source = SpatialAudioSource::new();
        source.set_position([1.0, 2.0, 3.0]);
        assert_eq!(source.position, [1.0, 2.0, 3.0]);
    }

    #[test]
    fn test_spatial_source_set_velocity() {
        let mut source = SpatialAudioSource::new();
        source.set_velocity([4.0, 5.0, 6.0]);
        assert_eq!(source.velocity, [4.0, 5.0, 6.0]);
    }

    #[test]
    fn test_channel_volume_clamp() {
        let mut engine = AudioEngine::new();
        engine.set_bus_volume(AudioBus::Music, 1.5);
        assert!((engine.buses[AudioBus::Music.index()].volume - 1.0).abs() < f32::EPSILON);
        engine.set_bus_volume(AudioBus::Music, -0.5);
        assert!((engine.buses[AudioBus::Music.index()].volume).abs() < f32::EPSILON);
    }
}
