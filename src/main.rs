#![no_std]
#![no_main]

use cortex_m_rt::entry;
use panic_halt as _;
use stm32f1xx_hal::{pac, prelude::*, timer::{Channel, Tim2NoRemap}};

#[entry]
fn main() -> ! {
    // 拿到硬件外设抽象层句柄
    let dp = pac::Peripherals::take().unwrap();
    let cp = cortex_m::Peripherals::take().unwrap();
    let mut afio = dp.AFIO.constrain();
    // 拿到FLASH控制器句柄
    let mut flash = dp.FLASH.constrain();
    // 拿到RCC时钟句柄
    let rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr.freeze(&mut flash.acr);
    // 拿到延时定时器句柄
    let mut delay = cp.SYST.delay(&clocks);
    // GPIO 开启
    let mut gpioa = dp.GPIOA.split();
    let mut gpioc = dp.GPIOC.split();
    // 将 PC13 配置为推挽输出
    let mut led = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);

    // 将 PA0 配置为 TIM2_CH1 复用推挽输出
    let tim2_c1 = gpioa.pa0.into_alternate_push_pull(&mut gpioa.crl);
    // 配置 TIM2 为 PWM 输出，频率 1kHz
    let mut pwm2 = 
    dp.TIM2.pwm_hz::<Tim2NoRemap, _, _>(
        tim2_c1,
        &mut afio.mapr,
        1.kHz(),
        &clocks
    );
    // 设置占空比为 50%
    let max_duty = pwm2.get_max_duty();
    pwm2.set_duty(Channel::C1, max_duty / 2);
    // 使能 PWM 输出
    pwm2.enable(Channel::C1);

    loop {
        // 翻转 LED 电平
        led.toggle();
        // 延时 500 毫秒
        delay.delay_ms(500_u32);
    }
}
