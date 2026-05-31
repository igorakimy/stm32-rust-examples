#![no_std]
#![no_main]

mod display;

use embassy_executor::Spawner;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::signal::Signal;
use embassy_time::Timer;
use panic_halt as _;
use crate::display::{CommonPinType, FourDigitSevenSegmentDisplay};

static DATA: Signal<CriticalSectionRawMutex, [u8; 4]> = Signal::new();

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    // Инициализируем периферию
    let p = embassy_stm32::init(Default::default());

    // Инициализируем сегменты дисплея
    let seg_a = Output::new(p.PA10, Level::Low, Speed::Low);
    let seg_b = Output::new(p.PB3, Level::Low, Speed::Low);
    let seg_c = Output::new(p.PB5, Level::Low, Speed::Low);
    let seg_d = Output::new(p.PB4, Level::Low, Speed::Low);
    let seg_e = Output::new(p.PB10, Level::Low, Speed::Low);
    let seg_f = Output::new(p.PA8, Level::Low, Speed::Low);
    let seg_g = Output::new(p.PA9, Level::Low, Speed::Low);
    let dp = Output::new(p.PC7, Level::Low, Speed::Low);

    // Инициализируем разряды дисплея
    let dig_1 = Output::new(p.PB6, Level::Low, Speed::Low);
    let dig_2 = Output::new(p.PA7, Level::Low, Speed::Low);
    let dig_3 = Output::new(p.PA6, Level::Low, Speed::Low);
    let dig_4 = Output::new(p.PA5, Level::Low, Speed::Low);

    // Создаем 4-х разрядный 7-сегментный дисплей
    // с общим катодом и инициализируем его
    let display = FourDigitSevenSegmentDisplay{
        segments: [seg_a, seg_b, seg_c, seg_d, seg_e, seg_f, seg_g, dp],
        digits: [dig_1, dig_2, dig_3, dig_4],
        common_pin: CommonPinType::Cathode,
        data: &DATA,
    };

    spawner.spawn(display_task(display).unwrap());
    spawner.spawn(change_data_task().unwrap());
}

#[embassy_executor::task]
async fn display_task(mut display: FourDigitSevenSegmentDisplay) {
    loop {
        // Ждем, пока не поступит сигнал, содержащий значение каждого разряда
        let current_values = display.data.wait().await;

        // Отображаем цифру на каждом из разрядов 7-сегментного дисплея
        for i in 0..4 {
            display.set_segments(current_values[i]);
            display.set_active_digit_idx(i);
            Timer::after_millis(20).await;
        }
    }
}

#[embassy_executor::task]
async fn change_data_task() {
    let mut digit: i16 = 1;
    loop {
        DATA.signal([
            ((digit / 1000) % 10) as u8,
            ((digit / 100) % 10) as u8,
            ((digit / 10) % 10) as u8,
            (digit % 10) as u8,
        ]);
        digit += 1;
        if digit > 9 {
            digit = 1;
        }
        Timer::after_millis(4).await;
    }
}