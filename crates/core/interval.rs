// Copyright (c) 2026 Blacknon. All rights reserved.
// Use of this source code is governed by an MIT license
// that can be found in the LICENSE file.

#[derive(Clone, Debug)]
pub struct RunInterval {
    pub interval: f64,
    pub paused: bool,
}

impl RunInterval {
    pub fn new(interval: f64) -> Self {
        Self {
            interval,
            paused: false,
        }
    }

    pub fn increase(&mut self, seconds: f64) {
        self.interval += seconds;
    }

    pub fn decrease(&mut self, seconds: f64) {
        if self.interval > seconds {
            self.interval -= seconds;
        }
    }

    pub fn toggle_pause(&mut self) {
        self.paused = !self.paused;
    }
}

impl Default for RunInterval {
    fn default() -> Self {
        Self::new(2.0)
    }
}

#[cfg(test)]
mod tests {
    use super::RunInterval;

    #[test]
    fn test_run_interval() {
        let mut actual = RunInterval::default();
        assert!(!actual.paused);
        assert_eq!(actual.interval, 2.0);
        actual.increase(1.5);
        actual.toggle_pause();
        assert!(actual.paused);
        assert_eq!(actual.interval, 3.5);
        actual.decrease(0.5);
        actual.toggle_pause();
        assert!(!actual.paused);
        assert_eq!(actual.interval, 3.0);
    }

    #[test]
    fn run_interval_decrease_does_not_go_below_threshold() {
        let mut actual = RunInterval::new(1.0);
        actual.decrease(1.0);
        actual.decrease(2.0);

        assert_eq!(actual.interval, 1.0);
        assert!(!actual.paused);
    }
}
