#![no_std]
#![no_main]

use core::sync::atomic::{AtomicBool, Ordering};
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Input, Level, Output, Pull, Speed};
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::signal::Signal;
use embassy_time::Timer;
use embedded_hal::digital::OutputPin;
use panic_halt as _;

// Атомарная статическая переменная для определения
// удерживается ли кнопка или нет
static BUTTON_PRESSED: AtomicBool = AtomicBool::new(false);
// Сигнальная переменная для определения текущего состояния кнопки:
// нажата ли она и не удерживается ли
static BUTTON_STATE: Signal<ThreadModeRawMutex, bool> = Signal::new();

// Структура, реализующая 7-сегментный дисплей и управление им
struct SevenSegmentDisplay<T: OutputPin> {
    a: T,
    b: T,
    c: T,
    d: T,
    e: T,
    f: T,
    g: T,
    dp: T,
}

impl<T: OutputPin> SevenSegmentDisplay<T> {
    pub fn new(a: T, b: T, c: T, d: T, e: T, f: T, g: T, dp: T) -> Self {
        Self {
            a,
            b,
            c,
            d,
            e,
            f,
            g,
            dp,
        }
    }

    pub fn clear(&mut self) {
        self.a.set_low().ok();
        self.b.set_low().ok();
        self.c.set_low().ok();
        self.d.set_low().ok();
        self.e.set_low().ok();
        self.f.set_low().ok();
        self.g.set_low().ok();
        self.dp.set_low().ok();
    }

    pub fn show(&mut self, digit: u8) {
        self.clear();
        match digit {
            0 => {
                self.a.set_high().ok();
                self.b.set_high().ok();
                self.c.set_high().ok();
                self.d.set_high().ok();
                self.e.set_high().ok();
                self.f.set_high().ok();
            }
            1 => {
                self.b.set_high().ok();
                self.c.set_high().ok();
            }
            2 => {
                self.a.set_high().ok();
                self.b.set_high().ok();
                self.g.set_high().ok();
                self.e.set_high().ok();
                self.d.set_high().ok();
            }
            3 => {
                self.a.set_high().ok();
                self.b.set_high().ok();
                self.c.set_high().ok();
                self.d.set_high().ok();
                self.g.set_high().ok();
            }
            4 => {
                self.b.set_high().ok();
                self.c.set_high().ok();
                self.f.set_high().ok();
                self.g.set_high().ok();
            }
            5 => {
                self.a.set_high().ok();
                self.c.set_high().ok();
                self.d.set_high().ok();
                self.f.set_high().ok();
                self.g.set_high().ok();
            }
            6 => {
                self.a.set_high().ok();
                self.c.set_high().ok();
                self.d.set_high().ok();
                self.e.set_high().ok();
                self.f.set_high().ok();
                self.g.set_high().ok();
            }
            7 => {
                self.a.set_high().ok();
                self.b.set_high().ok();
                self.c.set_high().ok();
            }
            8 => {
                self.a.set_high().ok();
                self.b.set_high().ok();
                self.c.set_high().ok();
                self.d.set_high().ok();
                self.e.set_high().ok();
                self.f.set_high().ok();
                self.g.set_high().ok();
            }
            9 => {
                self.a.set_high().ok();
                self.b.set_high().ok();
                self.c.set_high().ok();
                self.d.set_high().ok();
                self.f.set_high().ok();
                self.g.set_high().ok();
            }
            _ => (),
        }
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    // Инициализируем периферию
    let p = embassy_stm32::init(Default::default());

    // Инициализируем сегменты(диоды) дисплея
    let seg_a = Output::new(p.PA10, Level::Low, Speed::Low);
    let seg_b = Output::new(p.PB3, Level::Low, Speed::Low);
    let seg_c = Output::new(p.PB5, Level::Low, Speed::Low);
    let seg_d = Output::new(p.PB4, Level::Low, Speed::Low);
    let seg_e = Output::new(p.PB10, Level::Low, Speed::Low);
    let seg_f = Output::new(p.PA8, Level::Low, Speed::Low);
    let seg_g = Output::new(p.PA9, Level::Low, Speed::Low);
    let dp = Output::new(p.PC7, Level::Low, Speed::Low);

    // Инициализируем кнопку
    let button = Input::new(p.PC0, Pull::Down);

    // Создаем 7-сегментный дисплей
    let display = SevenSegmentDisplay::new(
        seg_a, seg_b, seg_c, seg_d,
        seg_e, seg_f, seg_g, dp,
    );

    // Добавляем задачи в executor
    spawner.spawn(button_press(button).unwrap());
    spawner.spawn(show_digit_on_display(display).unwrap());
}

// Функция отображает цифру на 7-сегментном дисплее при нажатии кнопки
#[embassy_executor::task]
async fn show_digit_on_display(mut display: SevenSegmentDisplay<Output<'static>>) {
    let mut current_digit = 0;
    display.show(current_digit);

    loop {
        // Ждём и проверяем, когда будет нажата кнопка
        if BUTTON_STATE.wait().await {
            current_digit += 1;
            if current_digit > 9 {
                current_digit = 0;
            }
            // Отображаем число на дисплее
            display.show(current_digit);
        }
    }
}

// Функция отслеживает состояние кнопки и отправляет сигнал о нажатой кнопке
// каждые 10 мс в другую функцию для реагирования
#[embassy_executor::task]
async fn button_press(button: Input<'static>) {
    loop {
        // Получаем текущее состояние кнопки: удерживается или нет
        let btn_pressed = BUTTON_PRESSED.load(Ordering::Relaxed);

        // Записываем(сигнализируем) что текущий логический уровень
        // кнопки - низкий(кнопка была нажата)
        BUTTON_STATE.signal(button.is_low() && !btn_pressed);

        if button.is_low() && !btn_pressed {
            // Если кнопка была нажата, то записываем
            // в глобальный контекст о том, что она удерживается
            BUTTON_PRESSED.store(true, Ordering::Relaxed);
        } else if button.is_high() && btn_pressed {
            // Если же кнопка была отжата, то убираем состояние удержания
            BUTTON_PRESSED.store(false, Ordering::Relaxed);
        }

        Timer::after_millis(10).await;
    }
}
