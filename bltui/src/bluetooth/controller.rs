use crate::Error;
use btleplug::{
    api::{Central, CentralEvent, Manager as _, Peripheral as _, ScanFilter},
    platform::{Adapter, Manager, PeripheralId},
};
use futures::{executor, Stream, StreamExt};
use log::info;
use std::pin::Pin;

use crate::devices::Device;

#[derive(Clone, Debug)]
pub enum AdapterEvent {
    Discovered(Device),
    Updated(Device),
    Connecting(Device),
    Disconnecting(Device),
    Connected(Device),
    Disconnected(Device),
    FailedToConnect(Device),
    FailedToDisconnect(Device),
    // TODO : add service/manufacturer data events
}

// fn wait_for<R>(future: &dyn Future<Output = R>) -> R {
//     let (tx, rx) = bounded(1);
//     todo!()
// }

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
    /// Returns a bool indicating if the scan was started or stopped
    pub async fn trigger_scan(&mut self) -> Result<bool, btleplug::Error> {
        self.scanning = !self.scanning;
        if !self.scanning {
            info!("Stopping the scan");
            self.adapter.stop_scan().await?;
            Ok(false)
        } else {
            info!("Starting the scan");
            self.adapter.start_scan(ScanFilter::default()).await?;
            Ok(true)
        }
    }

    pub async fn connect(&self, device: &Device) -> Result<(), Error> {
        let periph = self.adapter.peripheral(&device.peripheral_id).await?;

        if !periph.is_connected().await? {
            periph.connect().await?;
        }

        Ok(())
    }

    pub async fn disconnect(&self, device: &Device) -> Result<(), btleplug::Error> {
        let periph = self.adapter.peripheral(&device.peripheral_id).await?;

        if periph.is_connected().await? {
            periph.disconnect().await?;
        }

        Ok(())
    }

    async fn convert_event(&self, event: CentralEvent) -> AdapterEvent {
        match event {
            CentralEvent::DeviceDiscovered(periph_id) => {
                let device = self.get_device(&periph_id).await;
                AdapterEvent::Discovered(device)
            }
            _ => todo!(),
        }
    }

    pub async fn events(
        &self,
    ) -> Result<Pin<Box<dyn Stream<Item = AdapterEvent> + std::marker::Send>>, btleplug::Error>
    {
        let controller = self.clone();
        Ok(self
            .adapter
            .events()
            .await?
            .map(move |event| {
                let controller = controller.clone();
                info!("Gonna block");
                executor::block_on(controller.convert_event(event))
                // todo!()
            })
            .boxed())
    }

    pub async fn get_device(&self, periph_id: &PeripheralId) -> Device {
        let periph = self.adapter.peripheral(periph_id).await.unwrap();
        let properties = periph.properties().await.unwrap().unwrap();

        Device::new(
            periph_id.clone(),
            properties.local_name,
            periph.is_connected().await.unwrap(),
            properties.rssi,
            properties.tx_power_level,
        )
    }
}
