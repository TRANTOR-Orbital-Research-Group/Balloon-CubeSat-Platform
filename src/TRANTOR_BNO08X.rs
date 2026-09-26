use bno080::wrapper::BNO080;
use bno080::interface::I2cInterface;
use rp235x_hal::{I2C, gpio, pac::I2C1};
use embedded_hal_bus::i2c::RefCellDevice;
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

    pub fn get_output_pos_string(&self) -> (heapless::String<512>, heapless::String<512>)
    {
        let mut output1: String<512> = String::new();
        let mut output2: String<512> = String::new();

        write!(output1, 
            "\n\rAngular Position, Real: {}
            \n\rAngular Position, i: {}
            "
                , &self.angle_pos_real, &self.angle_pos_i)
                .expect("Creating BNO08X position output message 1");

        write!(output2, 
            "\n\rAngular Position, j: {}
            \n\rAngular Position, k: {}
            "
                , &self.angle_pos_j, &self.angle_pos_k)
                .expect("Creating BNO08X position output message 2");

        return (output1, output2);
    }

    pub fn get_output_velocity_string(&self) -> heapless::String<512>
    {
        let mut output: String<512> = String::new();

        write!(output, 
            "\n\rAngular velocity, i: {}
            \n\rAngular velocity, j: {}
            \n\rAngular velocity, k: {}
            "
                , &self.angle_velocity_i, &self.angle_velocity_j, &self.angle_velocity_k)
                .expect("Creating BNO08X velocity output message");

        return output;
    }

    pub fn get_output_accel_string(&self) -> heapless::String<512>
    {
        let mut output: String<512> = String::new();

        write!(output, 
            "
            \n\rLinear acceleration, x: {}
            \n\rLinear acceleration, y: {}
            \n\rLinear acceleration, z: {}
            "
                , &self.linear_acceleration_x, &self.linear_acceleration_y, &self.linear_acceleration_z)
                .expect("Creating BNO08X output message");

        return output;
    }
}

/*
The type parameters should be of the form rp235x_hal::gpio::bank0::GpioXX
*/
pub struct TRANTORBNO08X<'a, I2CGpioPin1: rp235x_hal::gpio::PinId, I2CGpioPin2: rp235x_hal::gpio::PinId>
{
    bno: BNO080<I2cInterface
                <RefCellDevice<'a, I2C<I2C1,
                    (rp235x_hal::gpio::Pin<I2CGpioPin1, rp235x_hal::gpio::FunctionI2c, rp235x_hal::gpio::PullUp>,
                     rp235x_hal::gpio::Pin<I2CGpioPin2, rp235x_hal::gpio::FunctionI2c, rp235x_hal::gpio::PullUp>)
                    >
                >>
            >,
    pub recent_data: BNO08XData
}

impl<I2CGpioPin1: rp235x_hal::gpio::PinId, I2CGpioPin2: rp235x_hal::gpio::PinId> 
    TRANTORBNO08X<'_, I2CGpioPin1, I2CGpioPin2>
{
    pub fn new<DelayNs: embedded_hal::delay::DelayNs>(i2c: RefCellDevice<I2C<I2C1, (
        gpio::Pin<I2CGpioPin1, gpio::FunctionI2c, gpio::PullUp>, 
        gpio::Pin<I2CGpioPin2, gpio::FunctionI2c, gpio::PullUp>
        )>>, report_interval_millis: u16, mut timer: DelayNs) -> TRANTORBNO08X<I2CGpioPin1, I2CGpioPin2>
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

        self.recent_data.angle_pos_i = self.bno.rotation_quaternion()
            .expect("Get rotation_quaternion")[0];
        self.recent_data.angle_pos_j = self.bno.rotation_quaternion()
            .expect("Get rotation_quaternion")[1];
        self.recent_data.angle_pos_k = self.bno.rotation_quaternion()
            .expect("Get rotation_quaternion")[2];
        self.recent_data.angle_pos_real= self.bno.rotation_quaternion()
            .expect("Get rotation_quaternion")[3];

        self.recent_data.angle_velocity_i = self.bno.gyro()
            .expect("Get gyro")[0];
        self.recent_data.angle_velocity_j = self.bno.gyro()
            .expect("Get gyro")[1];
        self.recent_data.angle_velocity_k = self.bno.gyro()
            .expect("Get gyro")[2];

        self.recent_data.linear_acceleration_x = self.bno.linear_accel()
            .expect("Get linear_acceleration")[0];
        self.recent_data.linear_acceleration_y = self.bno.linear_accel()
            .expect("Get linear_acceleration")[1];
        self.recent_data.linear_acceleration_z = self.bno.linear_accel()
            .expect("Get linear_acceleration")[2];
    }
}