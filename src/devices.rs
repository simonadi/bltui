use std::{
    cmp::min,
    collections::{
        hash_map::{self, DefaultHasher},
        HashMap,
    },
    hash::{Hash, Hasher},
};

use btleplug::{
    api::Peripheral as _,
    platform::{Peripheral, PeripheralId},
};
use tui::{
    text::Text,
    widgets::{ListItem, ListState},
};

use crate::bluetooth::get_periph_name;

#[derive(Debug, Clone)]
pub struct Device {
    pub periph_id: PeripheralId,
    pub address: String,
    pub name: String,
    pub connected: bool,
    pub rssi: Option<i16>,
    pub tx_power: Option<i16>,
}

impl Device {
    pub async fn from_periph(periph: &Peripheral) -> Device {
        let properties = periph.properties().await.unwrap().unwrap();
        let address = periph.address().to_string();
        let name = get_periph_name(&periph).await;
        let rssi = properties.rssi;
        let connected = periph.is_connected().await.unwrap();
        let periph_id = periph.id();
        let tx_power = properties.tx_power_level;

        Device {
            periph_id,
            address,
            name,
            connected,
            rssi,
            tx_power,
        }
    }
}

impl Into<Text<'_>> for Device {
    fn into(self) -> Text<'static> {
        Text::from(if self.name == "Unknown" {
            format!("{} ({})", self.name, self.address)
        } else {
            self.name
        })
    }
}

impl PartialEq for Device {
    fn eq(&self, other: &Device) -> bool {
        self.address == other.address
    }
}
impl Eq for Device {}

impl Hash for Device {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.address.hash(state);
    }
}

#[derive(Clone)]
pub struct Devices {
    pub list_state: ListState,
    hash_index_map: HashMap<u64, usize>,
    pub devices: Vec<Device>,
}

impl Devices {
    pub fn new() -> Devices {
        Devices {
            list_state: ListState::default(),
            hash_index_map: HashMap::<u64, usize>::new(),
            devices: Vec::new(),
        }
    }

    pub fn insert(&mut self, device: Device) -> Result<(), ()> {
        let mut hasher = DefaultHasher::default();
        device.hash(&mut hasher);
        let hash = hasher.finish();

        if self.hash_index_map.contains_key(&hash) {
            Err(())
        } else {
            let index = self.devices.len();
            self.devices.push(device);
            self.hash_index_map.insert(hash, index);
            Ok(())
        }
    }

    pub fn insert_or_replace(&mut self, device: Device) {
        let mut hasher = DefaultHasher::default();
        device.hash(&mut hasher);
        let hash = hasher.finish();

        if let hash_map::Entry::Vacant(entry) = self.hash_index_map.entry(hash) {
            let index = self.devices.len();
            self.devices.push(device);
            entry.insert(index);
        } else {
            self.devices[self.hash_index_map[&hash]] = device;
        }
    }

    pub fn move_selector_down(&mut self) {
        let current_index = self.list_state.selected();

        if let Some(index) = current_index {
            self.list_state
                .select(Some(min(index + 1, self.devices.len() - 1)));
        } else {
            if !self.devices.is_empty() {
                self.list_state.select(Some(0));
            }
        }
    }

    pub fn move_selector_up(&mut self) {
        let current_index = self.list_state.selected();

        if let Some(index) = current_index {
            self.list_state.select(Some(index.saturating_sub(1)));
        } else {
            if !self.devices.is_empty() {
                self.list_state.select(Some(0));
            }
        }
    }

    pub fn list_items(&self) -> Vec<ListItem> {
        self.devices
            .clone()
            .into_iter()
            // .filter(|device| !hide_unnamed || device.name != "Unknown")
            .map(ListItem::new)
            .collect()
    }

    pub async fn get_selected_device(&self) -> Option<Device> {
        self.list_state
            .selected()
            .map(|index| self.devices[index].clone())
    }
}
