#![no_std]
#![no_main]

use panic_halt as _;
use cortex_m_rt::entry;
use stm32f4xx_hal as hal;
use hal::{pac, prelude::*};
use embedded_hal::digital::{OutputPin, InputPin};
use stm32f4xx_hal::timer::SysDelay;

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

// Создаем структуру для более удобного управления сегментами
// 7-сегментного дисплея
struct SevenSegmentDisplay<A, B, C, D, E, F, G, DP> {
    a: A,
    b: B,
    c: C,
    d: D,
    e: E,
    f: F,
    g: G,
    dp: DP,
}

impl<A, B, C, D, E, F, G, DP> SevenSegmentDisplay<A, B, C, D, E, F, G, DP>
where
    A: OutputPin,
    B: OutputPin,
    C: OutputPin,
    D: OutputPin,
    E: OutputPin,
    F: OutputPin,
    G: OutputPin,
    DP: OutputPin,
{
    // Тушит все сегменты, устанавливая логический уровень каждого в 0
    fn clear(&mut self) {
        self.a.set_low().ok();
        self.b.set_low().ok();
        self.c.set_low().ok();
        self.d.set_low().ok();
        self.e.set_low().ok();
        self.f.set_low().ok();
        self.g.set_low().ok();
        self.dp.set_low().ok();
    }

    // Отображает указанную цифру на 7-сегментном дисплее, задавая логический уровень
    // соответствующих определённой цифре сегментов в 1.
    fn show(&mut self, digit: i32) {
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
            _ => ()
        }
    }
}

#[entry]
fn main() -> ! {
    if let (Some(dp), Some(cp)) = (
        pac::Peripherals::take(),
        cortex_m::Peripherals::take(),
    ) {
        let mut rcc = dp.RCC.constrain();
        let mut delay = cp.SYST.delay(&rcc.clocks);

        // Инициализация GPIO-портов
        let gpioa = dp.GPIOA.split(&mut rcc);
        let gpiob = dp.GPIOB.split(&mut rcc);
        let gpioc = dp.GPIOC.split(&mut rcc);

        // Инициализация сегментов 7-сгментного дисплея
        let seg_a = gpioa.pa10.into_push_pull_output();
        let seg_b = gpiob.pb3.into_push_pull_output();
        let seg_c = gpiob.pb5.into_push_pull_output();
        let seg_d = gpiob.pb4.into_push_pull_output();
        let seg_e = gpiob.pb10.into_push_pull_output();
        let seg_f = gpioa.pa8.into_push_pull_output();
        let seg_g = gpioa.pa9.into_push_pull_output();
        let decimal_point = gpioc.pc7.into_push_pull_output();

        // Инициализация кнопки на Breadboard
        let mut button = gpioc.pc0.into_pull_up_input();

        // Создаем 7-сегментный дисплей
        let mut display = SevenSegmentDisplay {
            a: seg_a,
            b: seg_b,
            c: seg_c,
            d: seg_d,
            e: seg_e,
            f: seg_f,
            g: seg_g,
            dp: decimal_point,
        };

        // Текущая цифра, которая будет отображена по умолчанию
        let mut current_digit = 0;
        display.show(current_digit);

        // Указываем изначальное и текущее состояние кнопки, логический уровень
        // которой по умолчанию установлен в 1.
        // Эти переменные необходимы для debouncing'а
        let mut last_state = true;
        let mut current_state;

        loop {
            // Получаем текущее состояние кнопки, подавляя дребезг контактов
            current_state = debounce(&mut button, last_state, &mut delay);

            // Убеждаемся, что кнопка нажата
            if !current_state && last_state {
                // Увеличиваем цифру на 1
                current_digit += 1;
                if current_digit > 9 {
                    current_digit = 0;
                }
                // Отображаем цифру на дисплее
                display.show(current_digit);
            }

            // Меняем последнее состояние кнопки на текущее
            last_state = current_state;
        }
    }

    loop {}
}

// Устраняет дребезг контактов кнопки.
// Contact bounce - дребезг контактов (https://en.wikipedia.org/wiki/Switch#Contact_bounce),
// длящийся некоторое время после замыкания электрических контактов.
// После замыкания происходят многократные неконтролируемые замыкания
// и размыкания контактов за счёт упругости материалов и деталей контактной системы.
fn debounce(button: &mut impl InputPin, last_state: bool, delay: &mut SysDelay) -> bool {
    // Получить текущее состояние кнопки
    let current_state = button.is_high().unwrap();

    // Если текущее состояние отличается от предыдущего (кнопка нажата)
    if current_state != last_state {
        // Сделать задержку на несколько миллисекунд
        delay.delay(6.millis());
        // Вернуть новое состояние кнопки
        return button.is_high().unwrap();
    }

    // Возвращаем текущее состояние кнопки, которое никак не изменилось (кнопка не была нажата)
    current_state
}