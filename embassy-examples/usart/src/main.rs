#![no_std]
#![no_main]

use core::fmt::Write;
use core::str;
use core::sync::atomic::{AtomicU32, Ordering};
use embassy_executor::Spawner;
use embassy_stm32::usart::{DataBits, Uart, UartTx};
use embassy_stm32::{
    self as _, Config, bind_interrupts,
    exti::{self, ExtiInput},
    gpio::{Level, Output, Pull, Speed},
    interrupt,
    mode::{Async, Blocking},
    time::mhz,
    usart::{Config as UartConfig, Parity, StopBits},
};
use embassy_time::Timer;
use heapless::String;
use panic_halt as _;

// Привязываем обработчик прерываний
bind_interrupts!(struct Irqs {
    EXTI15_10 => exti::InterruptHandler<interrupt::typelevel::EXTI15_10>;
});

// Определяем глобальную переменную для хранения
// значения задержки мигания светодиода, в миллисекундах
static BLINK_MS: AtomicU32 = AtomicU32::new(0);

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    // Получаем периферию
    let p = embassy_stm32::init(init_peripheral_config());

    // Инициализируем светодиод на плате
    let led = Output::new(p.PA5, Level::Low, Speed::Low);

    // Инициализируем пользовательскую кнопку на плате
    let button = ExtiInput::new(p.PC13, p.EXTI13, Pull::None, Irqs);

    // Инициализируем двунаправленный блокирующий UART
    let uart = Uart::new_blocking(p.USART2, p.PA3, p.PA2, init_uart_config()).unwrap();

    // Разделить UART на передатчик и приемник
    let (mut tx, mut rx) = uart.split();

    tx.blocking_write(b"Please, enter start delay(ms):\n")
        .unwrap();

    let mut buf = [0_u8; 4];

    // Читаем UART в буфер
    rx.blocking_read(&mut buf).unwrap();

    // Преобразовываем значение из буфера в целочисленное,
    // которое будет являться начальной задержкой таймера в миллисекундах
    let delay_ms = str::from_utf8(&buf)
        .ok()
        .and_then(|x| x.trim().parse::<u32>().ok())
        .unwrap_or(1000);

    // Записываем начальное значение задержки в миллисекундах
    BLINK_MS.store(delay_ms, Ordering::Relaxed);

    // Добавить задачи на исполнение
    spawner.spawn(led_blink(led).unwrap());
    spawner.spawn(show_btn_click_count(button, tx).unwrap());
}

// Мигает светодиодом
#[embassy_executor::task]
async fn led_blink(mut led: Output<'static>) {
    loop {
        // Получаем задержку таймера в миллисвекундах из глобальной переменной
        let delay = BLINK_MS.load(Ordering::Relaxed);
        led.toggle();
        Timer::after_millis(delay as u64).await;
    }
}

// Выводит в UART количество нажатий кнопки
#[embassy_executor::task]
async fn show_btn_click_count(
    mut button: ExtiInput<'static, Async>,
    mut uart: UartTx<'static, Blocking>,
) {
    let mut value: u8 = 0;
    let mut delay_ms: u32 = BLINK_MS.load(Ordering::Relaxed);
    let mut buffer: String<8> = String::new();

    loop {
        // Ждем нажатия кнопки
        button.wait_for_falling_edge().await;

        // Уменьшаем задержку таймера каждое нажатие
        delay_ms -= 300;
        if delay_ms < 500 {
            delay_ms = 2000;
        }

        // Сохраняем значение задержки в глобальной переменной
        BLINK_MS.store(delay_ms, Ordering::Relaxed);

        // Увеличиваем счетчик нажатий кнопки
        value = value.wrapping_add(1);

        // Выводим количество нажатий в UART
        writeln!(&mut buffer, "{}", value).unwrap();
        uart.blocking_write(buffer.as_bytes()).unwrap();

        // Очищаем буфер
        buffer.clear();
    }
}

// Инициализирует и возвращает конфигурацию UART
fn init_uart_config() -> UartConfig {
    let mut config = UartConfig::default();

    config.baudrate = 9600_u32; // Скорость передачи данных в бодах(бит/с), битрейт
    config.data_bits = DataBits::DataBits8; // Количество битов данных
    config.stop_bits = StopBits::STOP1; // Количество стоп-битов
    config.parity = Parity::ParityNone; // Наличие бита четности

    config
}

// Инициализирует и возвращает конфигурацию периферии
fn init_peripheral_config() -> Config {
    let mut config = Config::default();
    {
        use embassy_stm32::rcc::*;
        config.rcc.hse = Some(Hse {
            freq: mhz(8),
            mode: HseMode::Bypass,
        });
        config.rcc.pll_src = PllSource::HSE;
        config.rcc.pll = Some(Pll {
            prediv: PllPreDiv::DIV4,
            mul: PllMul::MUL80,
            divp: Some(PllPDiv::DIV8),
            divq: None,
            divr: None,
        });
        config.rcc.ahb_pre = AHBPrescaler::DIV2;
        config.rcc.sys = Sysclk::PLL1_P;
    }

    config
}
