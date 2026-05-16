#![no_std]
#![no_main]

use panic_halt as _;

use cortex_m_rt::entry;
use stm32f4xx_hal as hal;
use hal::{pac, prelude::*};

#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();
    let core = cortex_m::Peripherals::take().unwrap();
    let mut rcc = dp.RCC.constrain();
    let mut delay = core.SYST.delay(&rcc.clocks);

    let gpioa = dp.GPIOA.split(&mut rcc);
    let gpiob = dp.GPIOB.split(&mut rcc);

    // Инициализация светодиодов
    let mut blue_led = gpioa.pa10.into_push_pull_output();
    let mut red_led = gpiob.pb5.into_push_pull_output();
    let mut yellow_led = gpioa.pa8.into_push_pull_output();

    // Инициализация пинов для кнопок(только на input, т.к. для кнопок не нужен output)
    // с внутренними подтягивающими резисторами PULL DOWN и PULL UP соответственно
    let button_1 = gpiob.pb9.into_pull_down_input();
    let button_2 = gpiob.pb8.into_pull_up_input();

    loop {
        // Если на кнопке высокий логический уровень (кнопка нажата)
        if button_1.is_high() {
            // Переключить логический уровень на синем светодиоде
            blue_led.toggle();
            delay.delay(2.secs());
        }

        // Если на кнопке высокий логический уровень
        // (кнопка отжата, т.к. установлен PULL UP подтягивающий резистор)
        if button_2.is_high() {
            // Переключить логический уровень на красном светодиоде
            red_led.toggle();
            delay.delay(500.millis());
        }

        // Если на обеих кнопках низкий логический уровень
        // (первая кнопка отжата, а вторая нажата)
        if button_1.is_low() && button_2.is_low() {
            // Переключить логический уровень на желтом светодиоде
            yellow_led.toggle();
            delay.delay(1.secs());
        }
    }
}
