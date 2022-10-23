use crate::Error;
use btleplug::{
    api::{Central, CentralEvent, Manager as _, Peripheral as _, ScanFilter},
    platform::{Adapter, Manager, PeripheralId},
};
use futures::Stream;
use log::info;
use std::pin::Pin;

use crate::bluetooth::devices::Device;

#[derive(Clone)]
pub struct BluetoothController {
    adapter: Adapter,
    pub scanning: bool,
}

fn get_periph_name(props: Option<String>) -> String {
    if let Some(name) = props {
        name
    } else {
        "Unknown".to_string()
    }
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

    pub async fn from_adapter(id: &str) -> Result<BluetoothController, Error> {
        let manager = Manager::new().await?;
        let adapters = manager.adapters().await?;
        for adapter in adapters.into_iter() {
            let adapter_id = adapter
                .adapter_info()
                .await
                .unwrap()
                .as_str()
                .split(' ')
                .next()
                .unwrap()
                .to_string();
            if adapter_id == id {
                return Ok(BluetoothController {
                    adapter,
                    scanning: false,
                });
            }
        }
        Err(Error::InvalidInput(
            "no adapter was found for the given name".to_string(),
        ))
    }

    /// Trigger the scan. Starting it will also power on the adapter
    /// if it is off
    pub async fn trigger_scan(&mut self) -> Result<(), btleplug::Error> {
        self.scanning = !self.scanning;
        if !self.scanning {
            info!("Stopping the scan");
            self.adapter.stop_scan().await?;
            Ok(())
        } else {
            info!("Starting the scan");
            self.adapter.start_scan(ScanFilter::default()).await?;
            Ok(())
        }
    }

    pub async fn connect(&self, periph_id: &PeripheralId) -> Result<(), btleplug::Error> {
        let periph = self.adapter.peripheral(periph_id).await?;
        let properties = periph.properties().await.unwrap().unwrap();
        let name = get_periph_name(properties.local_name);

        if periph.is_connected().await? {
            info!("Already connected to {}", name);
            Ok(())
        } else {
            info!("Connecting to {}", name);
            periph.connect().await?;
            Ok(())
        }
    }

    pub async fn disconnect(&self, periph_id: &PeripheralId) -> Result<(), btleplug::Error> {
        let periph = self.adapter.peripheral(periph_id).await?;
        let properties = periph.properties().await.unwrap().unwrap();
        let name = get_periph_name(properties.local_name);

        if !periph.is_connected().await? {
            info!("Not connected to {}", name);
            Ok(())
        } else {
            info!("Disconnecting from {}", name);
            periph.disconnect().await?;
            Ok(())
        }
    }

    pub async fn events(
        &self,
    ) -> Result<Pin<Box<dyn Stream<Item = CentralEvent> + std::marker::Send>>, btleplug::Error>
    {
        self.adapter.events().await
    }

    pub async fn get_device(&self, periph_id: &PeripheralId) -> Device {
        let periph = self.adapter.peripheral(periph_id).await.unwrap();
        let properties = periph.properties().await.unwrap().unwrap();

        Device {
            periph_id: periph.id(),
            address: periph.address().to_string(),
            name: get_periph_name(properties.local_name),
            connected: periph.is_connected().await.unwrap(),
            // paired: periph.is_paired().await.unwrap(),
            // trusted: periph.is_trusted().await.unwrap(),
            rssi: properties.rssi,
            tx_power: properties.tx_power_level,
        }
    }
}
