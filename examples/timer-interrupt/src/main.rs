#![no_main]
#![no_std]

use panic_halt as _;
use core::cell::RefCell;
use cortex_m::{
    asm,
    interrupt::{Mutex},
    peripheral::NVIC,
};
use cortex_m_rt::entry;
use stm32f4xx_hal as hal;
use hal::{
    gpio::{self, Output, PushPull},
    pac::{Interrupt, Peripherals, TIM2, interrupt},
    prelude::*,
    rcc::Config,
    timer::{CounterUs, Event},
};

type LedPin = gpio::PA5<Output<PushPull>>;

// Делаем светодиод доступным глобально, обернув в мьютекс
static G_LED: Mutex<RefCell<Option<LedPin>>> = Mutex::new(RefCell::new(None));

// Делаем регистры прерывания таймера доступными глобально, обернув в мьютекс
static G_TIMER: Mutex<RefCell<Option<CounterUs<TIM2>>>> = Mutex::new(RefCell::new(None));

#[entry]
fn main() -> ! {
    // Получаем периферию платы
    let dp = Peripherals::take().unwrap();

    // Настраиваем тактирование
    let rcc_cfg = Config::hsi().sysclk(16.MHz()).pclk1(8.MHz());
    let mut rcc = dp.RCC.freeze(rcc_cfg);

    // Настраиваем пин PA5 для светодиода
    let gpioa = dp.GPIOA.split(&mut rcc);
    let led = gpioa.pa5.into_push_pull_output();

    // Конфигурируем 32-битный таймер
    let mut timer = dp.TIM2.counter_us(&mut rcc);
    // Устанавливаем таймер, срок действия которого истекает через 1 секунду
    timer.start(500.millis()).unwrap();
    // Генерируем прерывание по истечении таймера
    timer.listen(Event::Update);

    // Поместить пин светодиода и таймер в глобальный контекст
    cortex_m::interrupt::free(|cs| {
        G_LED.borrow(cs).replace(Some(led));
        G_TIMER.borrow(cs).replace(Some(timer));
    });

    // Включить прерывание ранее настроенного таймера
    unsafe {
        NVIC::unmask(Interrupt::TIM2);
    }

    loop {
        // Дать команду микроконтроллеру для ожидания прерывания
        asm::wfi();
    }
}

// Определяем обработчик прерываний, т.е. функцию, которая будет вызвана
// когда произойдет прерывание.
// Конкретно это прерывание сработает по истечении таймера.
#[interrupt]
fn TIM2() {
    static mut LED: Option<LedPin> = None;
    static mut TIMER: Option<CounterUs<TIM2>> = None;

    cortex_m::interrupt::free(|cs| {
        // Лениво забираем светодиод из глобального Mutex
        let led = LED.get_or_insert_with(|| {
            G_LED.borrow(cs).take().unwrap()
        });

        // Лениво забираем таймер из глобального Mutex
        let timer = TIMER.get_or_insert_with(|| {
            G_TIMER.borrow(cs).take().unwrap()
        });

        // Переключаем состояние светодиода
        led.toggle();

        // Сбрасываем event таймера
        timer.wait().unwrap();
    });
}