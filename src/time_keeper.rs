use time;

pub struct TimeKeeper {
    accumulated: f64,
    produced: f64,
    consumed: f64,
    timestamp: f64
}

impl TimeKeeper {
    pub fn new() -> Self {
        TimeKeeper {
            accumulated: 0.0,
            produced: 0.0,
            consumed: 0.0,
            timestamp: time::precise_time_s()
        }
    }
    pub fn produce(&mut self, time: f64) {
        self.accumulated += time;
        self.produced += time;
    }

    pub fn produce_frame(&mut self) -> f64 {
        let new_timestamp = time::precise_time_s();
        let elapsed = new_timestamp - self.timestamp;
        self.produce(elapsed);
        self.timestamp = new_timestamp;
        elapsed
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

    #[allow(dead_code)]
    pub fn accumulated(&self) -> f64 {
        self.accumulated
    }

    #[allow(dead_code)]
    pub fn consumed(&self) -> f64 {
        self.consumed
    }

    #[allow(dead_code)]
    pub fn produced(&self) -> f64 {
        self.produced
    }
}