use btleplug::api::CentralEvent;
use btleplug::api::{Central, Manager as _, Peripheral as _, ScanFilter};
use btleplug::platform::{Adapter, Manager, Peripheral, PeripheralId};
use futures::Stream;
use log::info;
use std::pin::Pin;

use crate::devices::Device;

#[derive(Clone)]
pub struct BluetoothController {
    adapter: Adapter,
    pub scanning: bool,
}

impl BluetoothController {
    pub async fn from_first_adapter() -> BluetoothController {
        let manager = Manager::new().await.unwrap();
        let adapters = manager.adapters().await.unwrap();
        let adapter = adapters.into_iter().next().unwrap();

        BluetoothController {
            adapter,
            scanning: false,
        }
    }

    pub async fn trigger_scan(&mut self) -> Result<(), btleplug::Error> {
        self.scanning = !self.scanning;
        if !self.scanning {
            info!("Stopping the scan");
            self.adapter.stop_scan().await
        } else {
            info!("Starting the scan");
            self.adapter.start_scan(ScanFilter::default()).await
        }
    }

    pub async fn connect(&self, periph_id: &PeripheralId) -> Result<(), btleplug::Error> {
        let periph = self.adapter.peripheral(periph_id).await?;

        if periph.is_connected().await.unwrap() {
            info!("Device is already connected");
            Ok(())
        } else {
            periph.connect().await
        }
    }

    pub async fn disconnect(&self, periph_id: &PeripheralId) -> Result<(), btleplug::Error> {
        let periph = self.adapter.peripheral(periph_id).await?;

        if !periph.is_connected().await.unwrap() {
            info!("Device already disconnected");
            Ok(())
        } else {
            periph.disconnect().await
        }
    }

    pub async fn pair(&self, periph_id: &PeripheralId) -> Result<(), btleplug::Error> {
        let periph = self.adapter.peripheral(periph_id).await?;

        if !periph.is_paired().await.unwrap() {
            periph.pair().await
        } else {
            Ok(())
        }
    }

    // pub async fn trigger_trust(&self, periph_id: &PeripheralId) -> Result<(), btleplug::Error> {
    //     let periph = self.adapter.peripheral(periph_id).await?;
    //     periph
    //         .set_trusted(!periph.is_trusted().await.unwrap())
    //         .await
    // }

    pub async fn events(&self) -> Pin<Box<dyn Stream<Item = CentralEvent> + std::marker::Send>> {
        self.adapter.events().await.unwrap()
    }

    pub async fn get_peripheral(&self, periph_id: &PeripheralId) -> Peripheral {
        self.adapter.peripheral(periph_id).await.unwrap()
    }

    pub async fn get_device(&self, periph_id: &PeripheralId) -> Device {
        let periph = self.adapter.peripheral(periph_id).await.unwrap();
        let properties = periph.properties().await.unwrap().unwrap();

        Device {
            periph_id: periph.id(),
            address: periph.address().to_string(),
            name: if let Some(name) = properties.local_name {
                name
            } else {
                String::from("Unknown")
            },
            connected: periph.is_connected().await.unwrap(),
            paired: periph.is_paired().await.unwrap(),
            // trusted: periph.is_trusted().await.unwrap(),
            rssi: properties.rssi,
            tx_power: properties.tx_power_level,
        }
    }
}
