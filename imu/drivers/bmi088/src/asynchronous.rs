use crate::{Bmi088, BmiI2CError, ImuError, Vector3};
use crate::registers::{AccelRange, AccelRegisters, Constants, GyroRange, GyroRegisters};
use i2cdev::linux::{LinuxI2CDevice, LinuxI2CError};
use i2cdev::core::I2CDevice; 
use tokio::sync::Mutex; // might have to use lateer
use std::sync::Arc;
use log::{debug, error, warn};
use async_trait::async_trait;

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
    accel_i2c: Mutex<LinuxI2CDevice>,
    gyro_i2c: Mutex<LinuxI2CDevice>,
    accel_range: AccelRange,
    gyro_range: GyroRange,
}

impl AsyncBmi088 {
    pub async fn new(i2c_path: &str) -> Result<Self, ImuError> {
        debug!("Initializing Async Bmi088...");
        let accel_i2c = 
            LinuxI2CDevice::new(i2c_path, Constants::AccelI2cAddr as u16).map_err(AsyncBmiI2CError)?;
        let gyro_i2c =
            LinuxI2CDevice::new(i2c_path, Constants::GyroI2cAddr as u16).map_err(AsyncBmiI2CError)?;
    
        // TODO: Verify ID using read_accel_register

        Ok(AsyncBmi088 {
            accel_i2c: Mutex::new(accel_i2c),
            gyro_i2c: Mutex::new(gyro_i2c),
            accel_range: AccelRange::G3,
            gyro_range: GyroRange::Dps2000,
        })

        
    
    }

    async fn read_accel_register(&self, reg: u8) -> Result<u8, ImuError> {
        let mut accel = self.accel_i2c.lock().await;
        let mut buffer = [0u8; 1];

        // Spawn sync tasks to side thread
        // TODO: Fix imuerror stuff
        tokio::task::spawn_blocking (move || {
            accel.write(&[reg])?;
            accel.read(&mut buffer)?;
            Ok(buffer[0])
        }).await?
    }

    
}