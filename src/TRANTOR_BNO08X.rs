use bno080::wrapper::BNO080;
use bno080::interface::I2cInterface;
use rp235x_hal::{I2C, gpio, pac::I2C1};
use core::fmt::Write;
use heapless::String;

pub struct BNO08XData
{
    pub angle_pos_real: f32,
    pub angle_pos_i: f32,
    pub angle_pos_j: f32,
    pub angle_pos_k: f32,

    pub angle_velocity_i: f32,
    pub angle_velocity_j: f32,
    pub angle_velocity_k: f32,

    pub linear_acceleration_x: f32,
    pub linear_acceleration_y: f32,
    pub linear_acceleration_z: f32,
}

impl BNO08XData
{
    pub fn new_null() -> BNO08XData
    {
        return BNO08XData
        {
            angle_pos_real: 0.0,
            angle_pos_i: 0.0,
            angle_pos_j: 0.0,
            angle_pos_k: 0.0,
            angle_velocity_i: 0.0,
            angle_velocity_j: 0.0,
            angle_velocity_k: 0.0,
            linear_acceleration_x: 0.0,
            linear_acceleration_y: 0.0,
            linear_acceleration_z: 0.0,
        }
    }

    pub fn get_output_string(&self) -> heapless::String<128>
    {
        let mut output: String<128> = String::new();

        write!(output, 
            "\n\rAngular Position, Real: {}
            \n\rAngular Position, i: {}
            \n\rAngular Position, j: {}
            \n\rAngular Position, k: {}\n
            \n\rAngular velocity, i: {}
            \n\rAngular velocity, j: {}
            \n\rAngular velocity, k: {}\n
            \n\rLinear acceleration, x: {}
            \n\rLinear acceleration, y: {}
            \n\rLinear acceleration, z: {}
            "
                , &self.angle_pos_real, &self.angle_pos_i, &self.angle_pos_j, &self.angle_pos_k
                , &self.angle_velocity_i, &self.angle_velocity_j, &self.angle_velocity_k
                , &self.linear_acceleration_x, &self.linear_acceleration_y, &self.linear_acceleration_z)
                .expect("Creating BNO08X output message");

        return output;
    }
}

/*
The type parameters should be of the form rp235x_hal::gpio::bank0::GpioXX
*/
pub struct TRANTORBNO08X<I2CGpioPin1: rp235x_hal::gpio::PinId, I2CGpioPin2: rp235x_hal::gpio::PinId>
{
    bno: BNO080<I2cInterface
                <I2C<I2C1,
                    (rp235x_hal::gpio::Pin<I2CGpioPin1, rp235x_hal::gpio::FunctionI2c, rp235x_hal::gpio::PullUp>,
                     rp235x_hal::gpio::Pin<I2CGpioPin2, rp235x_hal::gpio::FunctionI2c, rp235x_hal::gpio::PullUp>)
                    >
                >
            >,
    recent_data: BNO08XData
}

impl<I2CGpioPin1: rp235x_hal::gpio::PinId, I2CGpioPin2: rp235x_hal::gpio::PinId> 
    TRANTORBNO08X<I2CGpioPin1, I2CGpioPin2>
{
    fn init<DelayNs: embedded_hal::delay::DelayNs>(i2c: I2C<I2C1, (
        gpio::Pin<I2CGpioPin1, gpio::FunctionI2c, gpio::PullUp>, 
        gpio::Pin<I2CGpioPin2, gpio::FunctionI2c, gpio::PullUp>
        )>, report_interval_millis: u16, mut timer: DelayNs) -> TRANTORBNO08X<I2CGpioPin1, I2CGpioPin2>
    {
        let i2c_interface = bno080::interface::I2cInterface::default(i2c);
        let mut new_bno = BNO080::new_with_interface(i2c_interface);

        new_bno.init(&mut timer).expect("Initialize BNO08X");

        new_bno.enable_gyro(report_interval_millis)
            .expect("Enable gyro reporting");

        // new_bno.enable_rotation_vector(report_interval_millis)
        //     .expect("Enable rotation reporting");

        new_bno.enable_linear_accel(report_interval_millis)
            .expect("Enable linear acceleration reporting");

        return TRANTORBNO08X
        {
            bno: new_bno,
            recent_data: BNO08XData::new_null()
        }
    }

    pub fn record_data<DelayNs: embedded_hal::delay::DelayNs>(&mut self, timer: &mut DelayNs, timeout_millis: u8)
    {
        self.bno.handle_all_messages(timer, timeout_millis);
        
        self.recent_data.angle_pos_i;
    }
}