#![no_std]
#![no_main]

use core::sync::atomic::{AtomicBool, Ordering};
use embassy_executor::Spawner;
use embassy_stm32::{
    Config,
    gpio::{Level, Output, Pull, Speed},
    exti::{self, ExtiInput},
    bind_interrupts,
    mode::Async,
    interrupt,
};
use embassy_time::{Timer};
use panic_halt as _;

// Объявляем глобальные статические переменные для отслеживания
// состояния нажатия кнопок.
// Значения состояний являются атомарными
// для безопасного чтения/записи из нескольких задач
static BUTTON1_PRESSED: AtomicBool = AtomicBool::new(false);
static BUTTON2_PRESSED: AtomicBool = AtomicBool::new(false);

// Включаем внешние прерывания для GPIO пинов с 5 по 9
bind_interrupts!(pub struct Irqs {
    EXTI9_5 => exti::InterruptHandler<interrupt::typelevel::EXTI9_5>;
});

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    // Получаем периферию платы
    let p = embassy_stm32::init(Config::default());

    // Инициализируем пины для светодиодов
    let blue_led = Output::new(p.PA10, Level::Low, Speed::Low);
    let red_led = Output::new(p.PB5, Level::Low, Speed::Low);
    let yellow_led = Output::new(p.PA8, Level::Low, Speed::Low);

    // Инициализируем пины для кнопок
    let button_1 = ExtiInput::new(p.PB9, p.EXTI9, Pull::Down, Irqs);
    let button_2 = ExtiInput::new(p.PB8, p.EXTI8, Pull::Up, Irqs);

    // Помещаем задачи в executor, передавая ссылки на глобальные пины
    spawner.spawn(blue_led_blink(blue_led).unwrap());
    spawner.spawn(red_led_blink(red_led).unwrap());
    spawner.spawn(yellow_led_blink(yellow_led).unwrap());
    spawner.spawn(button1_click(button_1).unwrap());
    spawner.spawn(button2_click(button_2).unwrap());
}

// Асинхронная функция для мигания красного светодиода
#[embassy_executor::task]
async fn red_led_blink(mut led: Output<'static>) {
    loop {
        // Если нажата первая кнопка
        if BUTTON1_PRESSED.load(Ordering::Relaxed) {
            // Переключить состояние светодиода
            led.toggle();
            Timer::after_millis(500).await;
        } else {
            // Если кнопка отпущена, то погасить светодиод
            led.set_low();
            Timer::after_millis(10).await;
        }
    }
}

#[embassy_executor::task]
async fn blue_led_blink(mut led: Output<'static>) {
    loop {
        if BUTTON2_PRESSED.load(Ordering::Relaxed) {
            led.toggle();
            Timer::after_millis(2000).await;
        } else {
            led.set_low();
            Timer::after_millis(10).await;
        }
    }
}

#[embassy_executor::task]
async fn yellow_led_blink(mut led: Output<'static>) {
    loop {
        if !BUTTON1_PRESSED.load(Ordering::Relaxed) && !BUTTON2_PRESSED.load(Ordering::Relaxed) {
            led.toggle();
            Timer::after_millis(1000).await;
        } else {
            led.set_low();
            Timer::after_millis(10).await;
        }
    }
}

#[embassy_executor::task]
async fn button1_click(mut button: ExtiInput<'static, Async>) {
    loop {
        wait_debounced_press(&mut button).await;
        BUTTON1_PRESSED.store(true, Ordering::Relaxed);
        wait_debounced_release(&mut button).await;
        BUTTON1_PRESSED.store(false, Ordering::Relaxed);
    }
}

#[embassy_executor::task]
async fn button2_click(mut button: ExtiInput<'static, Async>) {
    loop {
        wait_debounced_release(&mut button).await;
        BUTTON2_PRESSED.store(true, Ordering::Relaxed);
        wait_debounced_press(&mut button).await;
        BUTTON2_PRESSED.store(false, Ordering::Relaxed);
    }
}

// Асинхронная функция для отслеживания, нажата ли кнопка
// и подавления дребезга контактов
async fn wait_debounced_press(button: &mut ExtiInput<'static, Async>) {
    loop {
        // Подождать пока состояние пина кнопки измениться на значение высокого потенциала
        button.wait_for_rising_edge().await;
        // Заснуть для подавления дребезга
        Timer::after_millis(6).await;
        // Проверить состояние кнопки
        if button.is_high() {
            return;
        }
    }
}

// Асинхронная функция для отслеживания, отжата ли кнопка
// и подавления дребезга контактов
async fn wait_debounced_release(button: &mut ExtiInput<'static, Async>) {
    loop {
        button.wait_for_falling_edge().await;
        Timer::after_millis(6).await;
        if button.is_low() {
            return;
        }
    }
}
