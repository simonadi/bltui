use btleplug::api::CentralEvent;
use btleplug::api::{Central, Manager as _, Peripheral as _, ScanFilter};
use btleplug::platform::{Adapter, Manager, Peripheral, PeripheralId};
use futures::Stream;
use log::info;
use std::pin::Pin;

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
        let periph = self.adapter.peripheral(periph_id).await;

        match periph {
            Ok(per) => {
                if per.is_connected().await.unwrap() {
                    Ok(())
                } else {
                    per.connect().await
                }
            }
            Err(err) => Err(err),
        }
    }

    pub async fn disconnect(&self, periph_id: &PeripheralId) -> Result<(), btleplug::Error> {
        let periph = self.adapter.peripheral(periph_id).await;

        match periph {
            Ok(per) => {
                if !per.is_connected().await.unwrap() {
                    Ok(())
                } else {
                    per.disconnect().await
                }
            }
            Err(err) => Err(err),
        }
    }

    pub async fn events(&self) -> Pin<Box<dyn Stream<Item = CentralEvent> + std::marker::Send>> {
        self.adapter.events().await.unwrap()
    }

    pub async fn get_peripheral(&self, periph_id: &PeripheralId) -> Peripheral {
        self.adapter.peripheral(periph_id).await.unwrap()
    }
}
