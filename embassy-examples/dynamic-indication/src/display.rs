use embassy_stm32::gpio::Output;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::signal::Signal;
use embedded_hal::digital::{OutputPin, PinState};

pub enum CommonPinType {
    Anode,
    Cathode,
}

pub struct FourDigitSevenSegmentDisplay {
    pub segments: [Output<'static>; 8],
    pub digits: [Output<'static>; 4],
    pub common_pin: CommonPinType,
    pub data: &'static Signal<CriticalSectionRawMutex, [u8; 4]>
}

impl FourDigitSevenSegmentDisplay {
    fn get_common_pin(&mut self) -> PinState {
        match self.common_pin {
            CommonPinType::Anode => PinState::High,
            CommonPinType::Cathode => PinState::Low,
        }
    }

    fn get_segments_by_digit(&mut self, digit: u8) -> [u8; 8] {
        match digit {
            0 => [1, 1, 1, 1, 1, 1, 0, 0],
            1 => [0, 1, 1, 0, 0, 0, 0, 0],
            2 => [1, 1, 0, 1, 1, 0, 1, 0],
            3 => [1, 1, 1, 1, 0, 0, 1, 0],
            4 => [0, 1, 1, 0, 0, 0, 1, 0],
            5 => [1, 0, 1, 1, 0, 1, 1, 0],
            6 => [1, 0, 1, 1, 1, 1, 1, 0],
            7 => [1, 1, 1, 0, 0, 0, 0, 0],
            8 => [1, 1, 1, 1, 1, 1, 1, 0],
            9 => [1, 1, 1, 1, 0, 1, 1, 0],
            _ => [0, 0, 0, 0, 0, 0, 0, 0],
        }
    }

    pub fn clear(&mut self) {
        let state = self.get_common_pin();
        for digit in self.digits.iter_mut() {
            digit.set_state(state).ok();
        }
    }

    pub fn set_segments(&mut self, digit: u8) {
        self.clear();
        let pin_state = self.get_common_pin();
        for (i, &seg_state) in self.get_segments_by_digit(digit).iter().enumerate() {
            let state = PinState::from(seg_state != (bool::from(pin_state) as u8));
            self.segments[i].set_state(state).ok();
        }
    }

    pub fn set_active_digit_idx(&mut self, idx: usize) {
        let state = self.get_common_pin();
        self.digits[idx].set_state(!state).ok();
    }
}