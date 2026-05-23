#![no_std]
#![no_main]

use panic_halt as _;
use cortex_m_rt::entry;
use stm32f4xx_hal as hal;
use hal::{
    pac,
    prelude::*,
};
use stm32f4xx_hal::rcc::Config;

#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();

    // Настройка тактирования
    let mut rcc = dp.RCC.freeze(Config::hsi().sysclk(16.MHz()));

    // Подключаем светодиод
    let gpioa = dp.GPIOA.split(&mut rcc);
    let mut led = gpioa.pa10.into_push_pull_output();

    // Создаем базовый 16-битовый таймер для задержки,
    // передав ему настройки тактирования
    let mut delay = dp.TIM6.delay_us(&mut rcc);

    loop {
        // Мигаем светодиодом с задержкой 0.5 сек.
        led.toggle();
        delay.delay(500.millis());
    }
}
