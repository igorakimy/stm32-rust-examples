#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_stm32::{
    Config,
    gpio::{Input, Level, Output, Pull, Speed},
};
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_time::{Duration, Timer};
use panic_halt as _;

// Определяем тип светодиода для удобства использования
type LedType = Mutex<ThreadModeRawMutex, Option<Output<'static>>>;
// Статически инициализируем светодиоды
static BLUE_LED: LedType = Mutex::new(None);
static RED_LED: LedType = Mutex::new(None);
static YELLOW_LED: LedType = Mutex::new(None);

// Определяем тип кнопки для удобства использования
type ButtonType = Mutex<ThreadModeRawMutex, Option<Input<'static>>>;
// Статически инициализируем кнопки
static BUTTON1: ButtonType = Mutex::new(None);
static BUTTON2: ButtonType = Mutex::new(None);

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    // Получаем периферию платы
    let p = embassy_stm32::init(Config::default());

    // Инициализируем пины для светодиодов
    let blue_led = Output::new(p.PA10, Level::Low, Speed::Low);
    let red_led = Output::new(p.PB5, Level::Low, Speed::Low);
    let yellow_led = Output::new(p.PA8, Level::Low, Speed::Low);

    // Инициализируем пины для кнопок
    let button_1 = Input::new(p.PB9, Pull::Down);
    let button_2 = Input::new(p.PB8, Pull::Up);

    // Записываем значения инициализированных пинов
    // в статические переменные, используя блокировки мьютекса
    {
        *(BLUE_LED.lock().await) = Some(blue_led);
        *(RED_LED.lock().await) = Some(red_led);
        *(YELLOW_LED.lock().await) = Some(yellow_led);
        *(BUTTON1.lock().await) = Some(button_1);
        *(BUTTON2.lock().await) = Some(button_2);
    }

    // Помещаем задачи в executor, передавая ссылки на глобальные пины
    spawner.spawn(toggle_led_on_press_button(
        &BLUE_LED, &BUTTON1,
        Duration::from_millis(2_000)
    ).unwrap());
    spawner.spawn(toggle_led_on_press_button(
        &RED_LED, &BUTTON2,
        Duration::from_millis(500),
    ).unwrap());
    spawner.spawn(toggle_led_on_press_two_buttons(
        &YELLOW_LED, &BUTTON1, &BUTTON2,
        Duration::from_millis(1_000),
    ).unwrap());
}

// Функция переключает светодиод с указанным временным интервалом,
// когда нажата одна кнопка.
#[embassy_executor::task(pool_size = 2)]
async fn toggle_led_on_press_button(
    led: &'static LedType,
    button: &'static ButtonType,
    delay: Duration,
) {
    loop {
        {
            let mut led_unlocked = led.lock().await;
            let btn_unlocked = button.lock().await;
            if let (Some(led_pin_ref), Some(btn_pin_ref)) =
                (led_unlocked.as_mut(), btn_unlocked.as_ref())
            {
                // Проверяем логический уровень на кнопке
                if btn_pin_ref.is_high() {
                    // Переключаем логический уровень на светодиоде
                    led_pin_ref.toggle()
                }
            }
        }
        // Приостановить задачу на указанный интервал
        Timer::after(delay).await;
    }
}

// Функция переключает светодиод с указанным временным интервалом,
// когда нажаты две кнопки.
#[embassy_executor::task]
async fn toggle_led_on_press_two_buttons(
    led: &'static LedType,
    btn1: &'static ButtonType,
    btn2: &'static ButtonType,
    delay: Duration,
) {
    loop {
        {
            let mut led_unlocked = led.lock().await;
            let btn1_unlocked = btn1.lock().await;
            let btn2_unlocked = btn2.lock().await;
            if let (Some(led_pin_ref), Some(btn1_pin_ref), Some(btn2_pin_ref)) = (
                led_unlocked.as_mut(),
                btn1_unlocked.as_ref(),
                btn2_unlocked.as_ref(),
            ) {
                if btn1_pin_ref.is_low() && btn2_pin_ref.is_low() {
                    led_pin_ref.toggle()
                }
            }
        }
        Timer::after(delay).await;
    }
}
