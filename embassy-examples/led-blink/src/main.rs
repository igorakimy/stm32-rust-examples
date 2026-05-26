#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_time::Timer;
use embassy_stm32::{self, gpio::{Output, Level, Speed}, Config, Peri};
use embassy_stm32::gpio::AnyPin;
use panic_halt as _;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    // Получаем конфигурацию по умолчанию
    let config = Config::default();

    // Получаем синглтон с периферией платы
    let p = embassy_stm32::init(config);

    // Создаем задачу в исполнителе задач
    spawner.spawn(led_blink(p.PA5.into()).unwrap());
}

#[embassy_executor::task]
async fn led_blink(pin: Peri<'static, AnyPin>) {
    // Инициализируем новый GPIO-пин вывода для связи со светодиодом
    let mut led = Output::new(pin, Level::Low, Speed::Low);

    loop {
        // Устанавливаем высокий логический уровень на пине(включаем светодиод)
        led.set_high();
        Timer::after_millis(500).await;

        // Устанавливаем низкий логический уровень на пине(выключаем светодиод)
        led.set_low();
        Timer::after_millis(500).await;
    }
}
