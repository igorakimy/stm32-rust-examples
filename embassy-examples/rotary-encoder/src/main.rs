#![no_std]
#![no_main]

mod encoder;
use crate::encoder::{Direction, RotaryEncoder};

use core::{fmt::Write, sync::atomic::{AtomicIsize, Ordering}};
use embassy_executor::Spawner;
use embassy_futures::select::select;
use embassy_stm32::{
    Config, 
    bind_interrupts, 
    exti::{self, ExtiInput}, 
    gpio::{Level, Output, Pull, Speed}, 
    interrupt, 
    mode::{Async, Blocking},
    rcc::PllSource::HSE, 
    time::mhz,
    usart::{Config as UartConfig, Uart},
};
use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, signal::Signal};
use embassy_time::Timer;
use heapless::String;
use panic_halt as _;

// Данный пример демонстрирует асинхронную реализацию с опросом GPIO.
// 
// Более правильный и идеоматичный подход - использовать квадратурное кодирование таймера
// (embassy_stm32::timer::qei), установив таймеру режим энкодера, где:
// - канал 1(CH1) подключается к A-сигналу энкодера
// - канал 2(СР2) подключается к B-сигналу энкодера
// - регистр счетчика таймера автоматически увеличиватся/уменьшается 
// в зависимости от сигналов энкодера.
// 
// Первый вариант реализации был выбран по той причине, что эмулятор Renode 
// не поддерживает режим энкодера для таймеров. Подробнее о преимуществах 
// и недостатках каждого из подходов можно почитать в файле info.md

// Храним глобально текущую позицию энкодера
static ENCODER_POSITION: AtomicIsize = AtomicIsize::new(0);
// Сигнал о том, что позиция энкодера была изменена (регулятор повернули)
static ENCODER_CHANGE: Signal<ThreadModeRawMutex, isize> = Signal::new();

bind_interrupts!(struct Irqs {
   EXTI9_5 => exti::InterruptHandler<interrupt::typelevel::EXTI9_5>;
});

#[embassy_executor::main]
async fn main(spawner: Spawner) {   
    let p = embassy_stm32::init(init_config());

    let pin_a = ExtiInput::new(p.PB6, p.EXTI6, Pull::Up, Irqs);
    let pin_b = ExtiInput::new(p.PB7, p.EXTI7, Pull::Up, Irqs);

    let led = Output::new(p.PA5, Level::Low, Speed::Low);
    
    let uart = Uart::new_blocking(p.USART2, p.PA3, p.PA2, UartConfig::default()).unwrap();

    spawner.spawn(rotate_encoder(pin_a, pin_b).unwrap());
    spawner.spawn(blink_led(led).unwrap());
    spawner.spawn(uart_log(uart).unwrap());
}

#[embassy_executor::task]
async fn rotate_encoder(pin_a: ExtiInput<'static, Async>, pin_b: ExtiInput<'static, Async>) {
    // Создаем энкодер, передавая пины A и B
    let mut encoder = RotaryEncoder::new(pin_a, pin_b);
    // Текущаа позиция энкодера.
    let mut position: isize = 0;
    
    loop { 
        // Получаем пины энкодера
        let (pin_a, pin_b) = encoder.pins();

        // Ждем, когда логический уровень одного из пинов будет изменен
        select(
            pin_a.wait_for_any_edge(),
            pin_b.wait_for_any_edge(),
        ).await;

        // Определеяем в каком направлении была повернута ручка энкодера
        let direction = encoder.update().unwrap();
        match direction {
            // По часовой стрелке
            Direction::Clockwise => position += 1,
            // Против часовой стрелки
            Direction::CounterClockwise => position -= 1,
            // Направление не изменилось
            Direction::None => ()
        }

        // Сохраняем глобально позицию энкодера
        ENCODER_POSITION.store(position, Ordering::Relaxed);
        // Сигнализируем о том, что состояние энкодера изменилось
        ENCODER_CHANGE.signal(position);
    }
}

#[embassy_executor::task]
async fn blink_led(mut led: Output<'static>) {
    loop {
        // Получаем текущую позицию энкодера
        let mut position = ENCODER_POSITION.load(Ordering::Relaxed);
        if position < -4 {
            position = -4;
        }
        // Изменяем задержку таймера на основе показателя позиции энкодера
        let millis = (500 as isize + position * 100) as u64;
        led.toggle();
        Timer::after_millis(millis).await;
    }
}

// Логирует позицию энкодера в UART
#[embassy_executor::task]
async fn uart_log(mut uart: Uart<'static, Blocking>) {
    loop {
        // Ждем, когда позиция энкодера изменится и получаем эту позицию.
        let position = ENCODER_CHANGE.wait().await;

        // Формируем строку для записи по UART
        let mut buf: String<64> = String::new();
        writeln!(&mut buf, "position: {}", position).unwrap();

        // Записать строку в UART
        uart.blocking_write(buf.as_bytes()).unwrap();
    }
}

// Тактирование таймера.
fn init_config() -> Config {
    let mut config = Config::default();

    {
        use embassy_stm32::rcc::*;
        config.rcc.hse = Some(Hse { 
            freq: mhz(8), 
            mode: HseMode::Bypass,
        });

        config.rcc.pll_src = HSE;
        config.rcc.pll = Some(Pll { 
            prediv: PllPreDiv::DIV4, 
            mul: PllMul::MUL80, 
            divp: Some(PllPDiv::DIV8), 
            divq: None, 
            divr: None
        });

        config.rcc.ahb_pre = AHBPrescaler::DIV2;
        config.rcc.sys = Sysclk::PLL1_P;
    }

    config
}
