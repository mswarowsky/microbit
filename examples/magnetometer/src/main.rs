#![no_std]
#![no_main]

// use defmt_rtt as _;
// use panic_halt as _;
use panic_rtt_target as _;
use rtt_target::rprintln;
// use panic_probe as _;

use cortex_m_rt::entry;

use microbit::hal::{prelude::*, Timer};
use micromath::F32Ext;

#[cfg(feature = "v1")]
use microbit::{
    hal::twi,
    pac::{twi0::frequency::FREQUENCY_A, TWI0},
};
#[cfg(feature = "v2")]
use microbit::{
    hal::twim,
    pac::{twim0::frequency::FREQUENCY_A, TWIM0},
};

use lsm303agr::{
    interface::I2cInterface, mode::MagOneShot, AccelMode, AccelOutputDataRate, Lsm303agr,
};

const PI : f32 = 3.14159265359;

#[entry]
fn main() -> ! {
    let board = microbit::Board::take().unwrap();
    let mut timer = Timer::new(board.TIMER0);
    let mut x_damped : f32 = 0.1;

    rtt_target::rtt_init_print!();

    rprintln!("Start");
    rprintln!("asin 1 {}", arc2degree((1.0).asin()));

    #[cfg(feature = "v1")]
    let i2c = { twi::Twi::new(board.TWI0, board.i2c.into(), FREQUENCY_A::K100) };

    #[cfg(feature = "v2")]
    let i2c = { twim::Twim::new(board.TWIM0, board.i2c_internal.into(), FREQUENCY_A::K100) };

    let mut sensor = Lsm303agr::new_with_i2c(i2c);
    match sensor.accelerometer_id() {
        Ok(0x33u8) => {}
        _ => (),
        // _ => defmt::panic!("accelerometer not found"),
    }
    sensor.init().unwrap();
    sensor.set_accel_odr(AccelOutputDataRate::Hz50).unwrap();

    rprintln!("normal mode");
    sensor.set_accel_mode(AccelMode::Normal).unwrap();
    timer.delay_ms(1000_u32);
    get_data(&mut sensor, &mut x_damped);

    rprintln!("low power mode");
    sensor.set_accel_mode(AccelMode::LowPower).unwrap();
    timer.delay_ms(1000_u32);
    get_data(&mut sensor, &mut x_damped);

    rprintln!("high resolution mode");
    sensor.set_accel_mode(AccelMode::HighResolution).unwrap();
    timer.delay_ms(1000_u32);
    get_data(&mut sensor, &mut x_damped);

    loop {
        timer.delay_ms(500_u32);
        get_data(&mut sensor, &mut x_damped);
    }
}

#[cfg(feature = "v1")]
type Sensor = Lsm303agr<I2cInterface<twi::Twi<TWI0>>, MagOneShot>;

#[cfg(feature = "v2")]
type Sensor = Lsm303agr<I2cInterface<twim::Twim<TWIM0>>, MagOneShot>;


fn get_data(sensor: &mut Sensor, old_x : &mut f32) {
    loop {
        if sensor.accel_status().unwrap().xyz_new_data {
            let data = sensor.accel_data().unwrap();
            let sensivity : f32 = 980.0;
            let x = (data.x as f32) / sensivity;
            // let mut y = data.y as f32 / sensivity;
            *old_x = *old_x * 0.9 + x.abs() * 0.1;
            *old_x = limit(*old_x, 1.0);
            rprintln!("x {} y {} z {} wx {}", data.x, data.y, data.z, arc2degree(old_x.asin()));
            return;
        }
    }
}


fn arc2degree(arc : f32) -> f32 {
    return (arc/PI)*180.0
}

fn limit(a : f32, limit: f32) -> f32 {
    if a > limit {
        return limit;
    } else {
        return a;
    }
}