
pub struct TimeKeeper {
    accumulated: f64,
    produced: f64,
    consumed: f64
}

impl TimeKeeper {
    pub fn new() -> Self {
        TimeKeeper {
            accumulated: 0.0,
            produced: 0.0,
            consumed: 0.0
        }
    }
    pub fn produce(&mut self, time: f64) {
        self.accumulated += time;
        self.produced += time;
    }

    pub fn consume(&mut self, time: f64) -> bool {
        if time <= self.accumulated {
            self.accumulated -= time;
            self.consumed += time;
            true
        } else {
            false
        }
    }

    pub fn accumulated(&self) -> f64 {
        self.accumulated
    }

    pub fn consumed(&self) -> f64 {
        self.consumed
    }

    pub fn produced(&self) -> f64 {
        self.produced
    }
}