#![no_std]
#![no_main]

mod gauss;
mod utils;

use cortex_m_rt::entry;
use panic_halt as _;
use stm32f1xx_hal::{adc, pac, prelude::*, timer::{Channel, Tim3NoRemap, Tim4NoRemap}};

use crate::utils::data_limit;

#[entry]
fn main() -> ! {
    let steer_pwm_duty_center: u16 = 750 - 5; // 舵机 PWM 中值
    let steer_pwm_duty_max = steer_pwm_duty_center + 60; // 舵机 PWM 最大值
    let steer_pwm_duty_min = steer_pwm_duty_center - 60; // 舵机 PWM 最小值


    // 拿到硬件外设抽象层句柄
    let dp = pac::Peripherals::take().unwrap();
    let cp = cortex_m::Peripherals::take().unwrap();
    // 拿到FLASH控制器句柄
    let mut flash = dp.FLASH.constrain();
    // 拿到RCC时钟句柄
    let mut rcc = dp.RCC.constrain();
    let mut afio = dp.AFIO.constrain(&mut rcc);
    

    // 拿到延时定时器句柄
    let mut delay = cp.SYST.delay(&rcc.clocks);
    // GPIO 开启
    let mut gpioa = dp.GPIOA.split(&mut rcc);
    let mut gpiob = dp.GPIOB.split(&mut rcc);
    let mut gpioc = dp.GPIOC.split(&mut rcc);



    // Release PB3 from JTAG to use it as GPIO
    afio.mapr.disable_jtag(gpioa.pa15, gpiob.pb3, gpiob.pb4);

    // 将 PA4 和 PC5 配置为模拟输入
    let mut adc1 = adc::Adc::new(dp.ADC1, &mut rcc);
    let mut adc1_ch4_pa4 = gpioa.pa4.into_analog(&mut gpioa.crl);
    let mut adc1_ch15_pc5 = gpioc.pc5.into_analog(&mut gpioc.crl);

    // 配置四个拨码开关为推挽输入
    let switch1 = gpioa.pa8.into_pull_up_input(&mut gpioa.crh);
    let switch2 = gpioc.pc9.into_pull_up_input(&mut gpioc.crh);
    let switch3 = gpioc.pc8.into_pull_up_input(&mut gpioc.crh);
    let switch4 = gpioc.pc7.into_pull_up_input(&mut gpioc.crl);

    // 配置四个 LED 灯的引脚为推挽输出
    let mut led1_pa11 = gpioa.pa11.into_push_pull_output(&mut gpioa.crh);
    let mut led3_pc12 = gpioc.pc12.into_push_pull_output(&mut gpioc.crh);
    // let mut led4_pb3 = gpiob.pb3.into_push_pull_output(&mut gpiob.crl);

    // 配置使能开关为推挽输入
    let enable_switch_pa12 = gpioa.pa12.into_pull_up_input(&mut gpioa.crh);
    let mut enable_out_pc13 = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);

    // 将 PB0 配置为 TIM3_CH3 复用推挽输出
    let tim3_ch3 = gpiob.pb0.into_alternate_push_pull(&mut gpiob.crl);
    // 配置 TIM3 为 PWM 输出，频率 1kHz
    let mut steer_pwm = 
    dp.TIM3.pwm_hz::<Tim3NoRemap, _, _>(
        tim3_ch3,
        &mut afio.mapr,
        1.kHz(),
        &mut rcc
    );
    // 设置占空比为 舵机 PWM 中值
    steer_pwm.set_duty(Channel::C1, steer_pwm_duty_center);
    // 使能舵机 PWM 输出
    steer_pwm.enable(Channel::C1);

    // 配置左右轮控制流输出
    let mut left_wheel_pb8 = gpiob.pb8.into_push_pull_output(&mut gpiob.crh);
    let mut right_wheel_pb9 = gpiob.pb9.into_push_pull_output(&mut gpiob.crh);
    let tim4_ch1 = gpiob.pb6.into_alternate_push_pull(&mut gpiob.crl);
    let tim4_ch2 = gpiob.pb7.into_alternate_push_pull(&mut gpiob.crl);
    let mut wheels_pwm = 
    dp.TIM4.pwm_hz::<Tim4NoRemap, _, _>(
        (tim4_ch1, tim4_ch2),
        &mut afio.mapr,
        1.kHz(),
        &mut rcc
    );
    wheels_pwm.set_duty(Channel::C2, 0);
    wheels_pwm.set_duty(Channel::C1, 0);
    wheels_pwm.enable(Channel::C2);
    wheels_pwm.enable(Channel::C1);

    let mut adc_value_right_buffer = [0i32; 16];
    let mut adc_value_left_buffer = [0i32; 16];

    loop {
        // 读取 ADC 值
        let adc_value_right_v: i32 = adc1.read(&mut adc1_ch4_pa4).unwrap();
        let adc_value_left_v: i32 = adc1.read(&mut adc1_ch15_pc5).unwrap();

        // ADC 值入缓冲区
        for i in (1..16).rev() {
            adc_value_right_buffer[i] = adc_value_right_buffer[i - 1];
            adc_value_left_buffer[i] = adc_value_left_buffer[i - 1];
        }
        adc_value_right_buffer[0] = adc_value_right_v;
        adc_value_left_buffer[0] = adc_value_left_v;

        // 计算高斯滤波后的值
        let filtered_right = gauss::gauss_filter(&adc_value_right_buffer, 16);
        let filtered_left = gauss::gauss_filter(&adc_value_left_buffer, 16);

        let diff = (filtered_right - filtered_left) as i32;
        let steer_pwm_diff = 
        data_limit(diff as f64 * 0.3, steer_pwm_duty_min as f64, steer_pwm_duty_max as f64) as i32;
        let steer_pwm_duty = steer_pwm_duty_center as i32 - steer_pwm_diff; 

        // 设置舵机 PWM 占空比
        steer_pwm.set_duty(Channel::C1, steer_pwm_duty as u16);

        let mut wheel_speed_left = 0;
        let mut wheel_speed_right = 0;
        // 根据使能开关状态控制输出
        if enable_switch_pa12.is_high() {
            wheel_speed_left = 800;
            wheel_speed_right = 800;
            enable_out_pc13.set_high();
        } else {
            enable_out_pc13.set_low();
        }

        let wheel_left_pwm;
        let wheel_right_pwm;
        if wheel_speed_left >= 0 {
            wheel_left_pwm = wheel_speed_left as u16;
            left_wheel_pb8.set_low();
        } else {
            wheel_left_pwm = (4800 + wheel_speed_left) as u16;
            left_wheel_pb8.set_high();
        }
        if wheel_speed_right >= 0 {
            wheel_right_pwm = wheel_speed_right as u16;
            right_wheel_pb9.set_low();
        } else {
            wheel_right_pwm = (4800 + wheel_speed_right) as u16;
            right_wheel_pb9.set_high();
        }
        // 设置左右轮 PWM 占空比
        wheels_pwm.set_duty(Channel::C2, (wheel_left_pwm / 4800) * wheels_pwm.get_max_duty());
        wheels_pwm.set_duty(Channel::C1, (wheel_right_pwm / 4800) * wheels_pwm.get_max_duty());

        // 通过拨码开关调整板子上 LED 灯的亮灭


        // 延时 5 毫秒
        delay.delay_ms(5_u32);
    }
}
