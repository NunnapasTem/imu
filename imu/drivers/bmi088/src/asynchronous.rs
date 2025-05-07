use crate::{Bmi088, BmiI2CError, ImuError, Vector3};
use crate::registers::{AccelRange, AccelRegisters, Constants, GyroRange, GyroRegisters};
use i2cdev::linux::{LinuxI2CDevice, LinuxI2CError};
use i2cdev::core::I2CDevice; 
// use tokio::sync::Mutex; // might have to use lateer
use tokio::task;
use std::sync::{Arc, Mutex};
use log::{debug, error, warn};
use async_trait::async_trait;
use std::thread;
use std::time::Duration;

pub struct AsyncBmiI2CError(LinuxI2CError);

impl From<LinuxI2CError> for AsyncBmiI2CError {
    fn from(err: LinuxI2CError) -> Self {
        AsyncBmiI2CError(err)
    }
}

impl From<AsyncBmiI2CError> for ImuError {
    fn from(err: AsyncBmiI2CError) -> Self {
        ImuError::DeviceError(err.0.to_string())
    }
}

pub struct AsyncBmi088 {
    // Use Mutex because LinuxI2CDevice is not thread safe
    accel_i2c: Arc<Mutex<LinuxI2CDevice>>,
    gyro_i2c: Arc<Mutex<LinuxI2CDevice>>,
    accel_range: AccelRange,
    gyro_range: GyroRange,
}

impl AsyncBmi088 {

    pub async fn new(i2c_path: &str) -> Result<Self, ImuError> {
        debug!("Initializing Async Bmi088...");
        let accel_i2c = Arc::new(Mutex::new(
            LinuxI2CDevice::new(i2c_path, Constants::AccelI2cAddr as u16).map_err(AsyncBmiI2CError)?,));
        let gyro_i2c = Arc::new(Mutex::new(
            LinuxI2CDevice::new(i2c_path, Constants::GyroI2cAddr as u16).map_err(AsyncBmiI2CError)?,));
    
        // Clone to retain ownership after blocking thread
        let accel_i2c_cloned = accel_i2c.clone();
        let gyro_i2c_cloned = gyro_i2c.clone();

        
        task::spawn_blocking(move || {
            // Access inner value
            let mut accel = accel_i2c_cloned.lock().unwrap();
            let mut gyro = gyro_i2c_cloned.lock().unwrap();

            // Verify Accel Chip ID
            let chip_id = accel
                .smbus_read_byte_data(AccelRegisters::ChipId as u8)
                .map_err(AsyncBmiI2CError)?;
            if chip_id != Constants::AccelChipIdValue as u8 {
                return Err(ImuError::DeviceError("Invalid chip ID".to_string()));
            }

            // Soft reset
            accel
                .smbus_write_byte_data(
                    AccelRegisters::SoftReset as u8, 
                    Constants::SoftResetCmd as u8,
                )
                .map_err(AsyncBmiI2CError)?;
            thread::sleep (Duration::from_millis(50));
            // Enable the accelerometer and disables the temp sensor 
            accel
                .smbus_write_byte_data(AccelRegisters::PowerCtrl as u8, 0x04)
                .map_err(AsyncBmiI2CError)?;
            // Set the over-sampling-ratio of 4
            accel
                .smbus_write_byte_data(AccelRegisters::AccConf as u8, 0x80)
                .map_err(AsyncBmiI2CError)?;
            // Configure accelerometer
            accel
                .smbus_write_byte_data(AccelRegisters::AccRange as u8, AccelRange::G3 as u8)
                .map_err(AsyncBmiI2CError)?;

            // Configure gyroscope
            gyro
                .smbus_write_byte_data(GyroRegisters::PowerMode as u8, 0x00)
                .map_err(AsyncBmiI2CError)?;
            gyro
                .smbus_write_byte_data(GyroRegisters::Range as u8, GyroRange::Dps2000 as u8)
                .map_err(AsyncBmiI2CError)?;
            gyro
                .smbus_write_byte_data(GyroRegisters::Bandwidth as u8, 0x07)
                .map_err(AsyncBmiI2CError)?;

            Ok(())
        })
        .await
        .map_err(|e| ImuError::DeviceError(format!("Task join failed: {}", e)))??; 

        Ok(AsyncBmi088 {
            accel_i2c,
            gyro_i2c,
            accel_range: AccelRange::G3,
            gyro_range: GyroRange::Dps2000,
        })
    }

    // pub async fn read_raw_accelerometer(&mut self) -> Result<Vector3, ImuError> {


    //     Ok(())
    // }


}