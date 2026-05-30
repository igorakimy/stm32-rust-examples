#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_stm32::{
    bind_interrupts,
    interrupt,
    exti::{self, ExtiInput}
};
use embassy_stm32::gpio::{Output, Pull, Level, Speed};
use embassy_stm32::mode::Async;
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::signal::Signal;
use panic_halt as _;

// Создаем сигнальную переменную с типом ButtonEvent,
// которая необходима, чтобы сообщать текущее состояние кнопки
static BUTTON_EVENTS: Signal<ThreadModeRawMutex, ButtonEvent> = Signal::new();

// Создаем перечисления для удобной сигнализации
enum ButtonEvent {
    Pressed,
    Released,
}

// Используем макрос прерываний, который связывает прерывание
// с обработчиком прерываний.
bind_interrupts!(pub struct Irqs {
    EXTI15_10 => exti::InterruptHandler<interrupt::typelevel::EXTI15_10>;
});

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    // Инициализируем периферию
    let p = embassy_stm32::init(Default::default());

    // Инициализируем кнопку с функциональностью внешних прерываний
    let button = ExtiInput::new(p.PC13, p.EXTI13, Pull::Up, Irqs);

    // Инициализируем светодиод
    let led = Output::new(p.PA5, Level::Low, Speed::Low);

    // Добавляем задачи на исполнение в executor
    spawner.spawn(led_task(led).unwrap());
    spawner.spawn(button_task(button).unwrap());
}

// Задача переключает логический уровень светодиода
// в зависимости от поступившего сигнала из задачи кнопки
#[embassy_executor::task]
async fn led_task(mut led: Output<'static>) {
    loop {
        // Ожидаем, пока поступит сигнал
        match BUTTON_EVENTS.wait().await {
            ButtonEvent::Pressed => led.set_high(),
            ButtonEvent::Released => led.set_low(),
        }
    }
}

// Задача ожидает нажатия и отпускания кнопки, и, соответствующим
// образом реагирует на эти события, отправляя сигнал в задачу, которая
// переключает светодиод в зависимости от состояния кнопки
#[embassy_executor::task]
async fn button_task(mut button: ExtiInput<'static, Async>) {
    loop {
        // Приостанавливаем выполнение до тех пор, пока логический уровень
        // не опустится с высокого на низкий (будет нажата кнопка).
        // Когда кнопка будет нажата микроконтроллер автоматически
        // сгенерирует прерывание и начнется выполнение обработчика
        // прерываний, который в свою очередь разбудит executor и вернет
        // управление задаче в это место.
        button.wait_for_falling_edge().await;
        // Убеждаемся, что логический уровень теперь низкий (кнопка нажата)
        if button.is_low() {
            // Отправляем сигнал о том, что кнопка была нажата
            BUTTON_EVENTS.signal(ButtonEvent::Pressed);
        }

        button.wait_for_rising_edge().await;
        if button.is_high() {
            BUTTON_EVENTS.signal(ButtonEvent::Released);
        }
    }
}