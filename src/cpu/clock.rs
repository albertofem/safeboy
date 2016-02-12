pub struct Clock {
    pub m: u8,
    pub t: u8
}

impl Clock {
    pub fn new() -> Clock {
        Clock {
            m: 0,
            t: 0
        }
    }


    pub fn reset(&mut self) {
        self.m = 0;
        self.t = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_instantiates() {
        let clock = Clock::new();

        assert_eq!(0, clock.m);
        assert_eq!(0, clock.t);
    }

    #[test]
    fn it_resets() {
        let mut clock = Clock::new();

        clock.m = 1;
        clock.t = 4;

        clock.reset();

        assert_eq!(0, clock.m);
        assert_eq!(0, clock.t);
    }
}