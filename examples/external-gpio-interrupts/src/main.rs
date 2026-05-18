#![no_std]
#![no_main]

mod display;

use panic_halt as _;
use core::cell::RefCell;
use cortex_m::{
    interrupt::{free, Mutex},
    peripheral::NVIC,
};
use cortex_m_rt::entry;
use stm32f4xx_hal as hal;
use hal::{
    pac::{self, interrupt},
    gpio::{Edge, Input, Output, PushPull, PC1, PA10, PA8, PA9, PB10, PB3, PB4, PB5, PC7},
    prelude::*
};

// Схема включения сегментов на дисплее для каждой цифры
// 0: a-b-c-d-e-f
// 1: b-c
// 2: a-b-g-e-d
// 3: a-b-c-d-g
// 4: b-c-f-g
// 5: a-c-d-f-g
// 6: a-c-d-e-f-g
// 7: a-b-c
// 8: a-b-c-d-e-f-g
// 9: a-b-c-d-f-g

type DisplayType = display::SevenSegmentDisplay<
    PA10<Output<PushPull>>,
    PB3<Output<PushPull>>,
    PB5<Output<PushPull>>,
    PB4<Output<PushPull>>,
    PB10<Output<PushPull>>,
    PA8<Output<PushPull>>,
    PA9<Output<PushPull>>,
    PC7<Output<PushPull>>,
>;
static G_BUTTON: Mutex<RefCell<Option<PC1<Input>>>> = Mutex::new(RefCell::new(None));
static G_7SEGMENT_DISPLAY: Mutex<RefCell<Option<DisplayType>>> = Mutex::new(RefCell::new(None));

#[entry]
fn main() -> ! {
    let mut dp = pac::Peripherals::take().unwrap();
    let mut cp = cortex_m::Peripherals::take().unwrap();
    let mut rcc = dp.RCC.constrain();
    let mut syscfg = dp.SYSCFG.constrain(&mut rcc);

    let gpioa = dp.GPIOA.split(&mut rcc);
    let gpiob = dp.GPIOB.split(&mut rcc);
    let gpioc = dp.GPIOC.split(&mut rcc);

    let seg_a = gpioa.pa10.into_push_pull_output();
    let seg_b = gpiob.pb3.into_push_pull_output();
    let seg_c = gpiob.pb5.into_push_pull_output();
    let seg_d = gpiob.pb4.into_push_pull_output();
    let seg_e = gpiob.pb10.into_push_pull_output();
    let seg_f = gpioa.pa8.into_push_pull_output();
    let seg_g = gpioa.pa9.into_push_pull_output();
    let decimal_point = gpioc.pc7.into_push_pull_output();

    let display = DisplayType {
        a: seg_a, b: seg_b, c: seg_c, d: seg_d,
        e: seg_e, f: seg_f, g: seg_g, dp: decimal_point,
    };

    let mut button = gpioc.pc1.into_pull_up_input();
    // Подключаем GPIO к EXTI подсистеме
    button.make_interrupt_source(&mut syscfg);
    // Задаем триггер срабатывания прерывания по спадающему фронту напряжения
    button.trigger_on_edge(&mut dp.EXTI, Edge::Falling);
    // Включаем GPIO прерывание для кнопки
    button.enable_interrupt(&mut dp.EXTI);

    // Переместить владение кнопки и дисплея в глобальный контекст
    free(|cs| {
        G_BUTTON.borrow(cs).replace(Some(button));
        G_7SEGMENT_DISPLAY.borrow(cs).replace(Some(display));
    });

    // Включаем внешнее прерывание в NVIC,
    // передав номер прерывания кнопки, например EXTI1
    unsafe {
        cp.NVIC.set_priority(interrupt::EXTI1, 0);
        NVIC::unmask::<interrupt>(interrupt::EXTI1);
    }

    loop {
        for digit in 0..=9 {
            free(|cs| {
                G_7SEGMENT_DISPLAY
                    .borrow(cs)
                    .borrow_mut()
                    .as_mut()
                    .unwrap()
                    .show(digit);
            });
            for _ in 0..=20_000_000 {}
        }
    }
}

// Прерывание по нажатии кнопки.
// EXTI# соответствует номеру контакта, который используется
// в качестве внешнего прерывания.
// Функция может быть вызвана более одного раза для каждого нажатия кнопки,
// поскольку дребезг кнопки не устранен.
#[interrupt]
fn EXTI1() {
    free(|cs| {
        // Отобразить цифру 1 на 7-сегментном дисплее
        if let Some(display) = G_7SEGMENT_DISPLAY.borrow(cs).borrow_mut().as_mut() {
            display.show(1);
            for _ in 0..=10_000_000 {}
        }

        // Очистить бит ожидания прерывания, чтобы не вызывать
        // эту процедуру бесконечно
        if let Some(button) = G_BUTTON.borrow(cs).borrow_mut().as_mut() {
            button.clear_interrupt_pending_bit();
        }
    });
}