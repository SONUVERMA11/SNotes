//! Pressure normalization with configurable curves

/// Normalizes raw pressure values to 0.0–1.0 range with adjustable curve
pub struct PressureNormalizer {
    gamma: f32,
    threshold: f32,
}

impl PressureNormalizer {
    pub fn new(gamma: f32, threshold: f32) -> Self {
        Self {
            gamma: gamma.max(0.1),
            threshold: threshold.clamp(0.0, 1.0),
        }
    }

    /// Normalize a raw pressure value (already in 0.0-1.0 from libinput)
    pub fn normalize(&self, raw: f32) -> f32 {
        let clamped = raw.clamp(0.0, 1.0);
        if clamped < self.threshold {
            return 0.0;
        }
        // Apply gamma curve for pressure sensitivity adjustment
        // gamma < 1.0 = more sensitive to light pressure
        // gamma > 1.0 = less sensitive to light pressure
        clamped.powf(self.gamma)
    }

    /// Normalize from device-specific raw range to 0.0-1.0
    pub fn normalize_from_range(&self, raw: f32, min: f32, max: f32) -> f32 {
        if max <= min {
            return 0.0;
        }
        let normalized = (raw - min) / (max - min);
        self.normalize(normalized)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear_normalization() {
        let norm = PressureNormalizer::new(1.0, 0.0);
        assert!((norm.normalize(0.0) - 0.0).abs() < f32::EPSILON);
        assert!((norm.normalize(0.5) - 0.5).abs() < f32::EPSILON);
        assert!((norm.normalize(1.0) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_threshold() {
        let norm = PressureNormalizer::new(1.0, 0.1);
        assert!((norm.normalize(0.05) - 0.0).abs() < f32::EPSILON);
        assert!(norm.normalize(0.2) > 0.0);
    }

    #[test]
    fn test_gamma_curve() {
        let norm = PressureNormalizer::new(2.0, 0.0);
        assert!((norm.normalize(0.5) - 0.25).abs() < 0.01);
    }

    #[test]
    fn test_range_normalization() {
        let norm = PressureNormalizer::new(1.0, 0.0);
        let result = norm.normalize_from_range(4096.0, 0.0, 8192.0);
        assert!((result - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_clamp() {
        let norm = PressureNormalizer::new(1.0, 0.0);
        assert!((norm.normalize(1.5) - 1.0).abs() < f32::EPSILON);
        assert!((norm.normalize(-0.5) - 0.0).abs() < f32::EPSILON);
    }
}
