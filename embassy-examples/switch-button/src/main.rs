#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_stm32::{
    Config,
    gpio::{Level, Output, Pull, Speed},
};
use embassy_stm32::gpio::Input;
use embassy_time::{Timer};
use panic_halt as _;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    // Получаем периферию платы
    let p = embassy_stm32::init(Config::default());

    // Инициализируем пины для светодиодов
    let mut led = Output::new(p.PA5, Level::Low, Speed::Low);

    // Инициализируем пин для кнопки
    let button = Input::new(p.PC13, Pull::Down);

    loop {
        // Если кнопка нажата
        if button.is_low() {
            // Мигаем светодиодом
            led.toggle();
            // Устанавливаем задержку
            Timer::after_millis(200).await;
        }

        Timer::after_millis(6).await;
    }
}
