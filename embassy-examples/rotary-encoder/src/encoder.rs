use embedded_hal::digital::InputPin;
use either::Either;

pub struct RotaryEncoder<A, B, P> {
    pin_a: A,
    pin_b: B,
    state: u8,
    phase: P
}

pub enum Direction {
    Clockwise,
    CounterClockwise,
    None,
}

pub trait Phase {
    fn direction(&mut self, s: u8) -> Direction;
}

pub struct DefaultPhase;

impl Phase for DefaultPhase {
    fn direction(&mut self, s: u8) -> Direction {
        match s {
            0x17 => Direction::Clockwise,
            0x2b => Direction::CounterClockwise,
            _ => Direction::None,
        }   
    }
}

impl<A, B> RotaryEncoder<A, B, DefaultPhase>
where 
    A: InputPin,
    B: InputPin,
{
    pub fn new(pin_a: A, pin_b: B) -> Self {
        Self {
            pin_a,
            pin_b,
            state: 0_u8,
            phase: DefaultPhase,
        }
    }

    pub fn pins(&mut self) -> (&mut A, &mut B) {
        (&mut self.pin_a, &mut self.pin_b)
    }

    // https://www.best-microcontroller-projects.com/rotary-encoder.html
    pub fn update(&mut self) -> Result<Direction, Either<A::Error, B::Error>> {
        let (a_is_high, b_is_high) = (self.pin_a.is_high(), self.pin_b.is_high());

        let mut prev_next = (self.state << 2) & 0xF;
        
        if a_is_high.map_err(Either::Left)? {
            prev_next |= 0x01;
        }

        if b_is_high.map_err(Either::Right)? {
            prev_next |= 0x02;
        }

        match prev_next {
            1 | 2 | 4 | 7 | 8 | 11 | 13 | 14 => {
                let result = (self.state & 0xF0) | prev_next;
                self.state = prev_next << 4 | prev_next;
                Ok(self.phase.direction(result))
            }
            0 | 3 | 5 | 6 | 9 | 10 | 12 | 15 => {
                self.state = self.state & 0xF0 | prev_next;
                Ok(Direction::None)
            }
            0x10..=0xFF => Ok(Direction::None)
        }
    }
}
