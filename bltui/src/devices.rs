use std::{
    collections::HashMap,
    fmt::{Display, Formatter},
};

use btleplug::platform::PeripheralId;
use std::hash::Hash;

#[derive(PartialEq, Eq, Hash, Clone, Debug, Copy)]
pub struct MacAddress {
    address: [u8; 6],
}

impl MacAddress {
    pub fn from_dbus_path(object_path: zvariant::ObjectPath<'_>) -> MacAddress {
        let address = object_path
            .as_str()
            .split('/')
            .last()
            .unwrap()
            .split('_')
            .skip(1)
            .map(|s| u8::from_str_radix(s, 16).unwrap())
            .collect::<Vec<u8>>()
            .try_into()
            .expect("MAC address is not 6 bytes long");

        MacAddress { address }
    }

    pub fn from_peripheral_id(peripheral_id: &PeripheralId) -> MacAddress {
        let address = peripheral_id
            .to_string()
            .split('/')
            .last()
            .unwrap()
            .to_string()
            .split('_')
            .skip(1)
            .map(|s| u8::from_str_radix(s, 16).unwrap()) // Convert to u8
            .collect::<Vec<u8>>()
            .try_into()
            .expect("MAC address is not 6 bytes long");

        MacAddress { address }
    }
}

impl std::fmt::Display for MacAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
            self.address[0],
            self.address[1],
            self.address[2],
            self.address[3],
            self.address[4],
            self.address[5]
        )
    }
}

#[derive(Clone, Debug)]
pub struct Device {
    pub name: Option<String>,
    pub peripheral_id: PeripheralId,
    pub address: MacAddress,
    pub connected: bool,
    pub paired: bool,
    pub rssi: Option<i16>,
    pub tx_power: Option<i16>,
}

impl Display for Device {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name.clone().unwrap_or("Unknown".to_string()))
    }
}

impl Hash for Device {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.address.hash(state);
    }
}

impl Device {
    pub fn new(
        peripheral_id: PeripheralId,
        name: Option<String>,
        connected: bool,
        rssi: Option<i16>,
        tx_power: Option<i16>,
    ) -> Device {
        Device {
            name,
            peripheral_id: peripheral_id.clone(),
            address: MacAddress::from_peripheral_id(&peripheral_id),
            connected,
            paired: false,
            rssi,
            tx_power,
        }
    }
}

#[derive(Clone)]
pub struct Devices {
    devices: Vec<Device>,
    mac_address_index_map: HashMap<MacAddress, usize>,
}

impl Default for Devices {
    fn default() -> Self {
        Self::new()
    }
}

impl Devices {
    pub fn new() -> Devices {
        Devices {
            devices: Vec::new(),
            mac_address_index_map: HashMap::new(),
        }
    }

    pub fn add(&mut self, device: Device) {
        let mac_address = device.address;
        let index = self.devices.len();
        self.devices.push(device);
        self.mac_address_index_map.insert(mac_address, index);
    }

    pub fn update(&mut self, device: Device) {
        let mac_address = &device.address;
        if let Some(index) = self.mac_address_index_map.get(mac_address) {
            self.devices[*index] = device;
        } else {
            self.add(device);
        }
    }

    pub fn get_by_mac_address(&self, mac_address: &MacAddress) -> Option<Device> {
        self.mac_address_index_map
            .get(mac_address)
            .map(|index| self.devices[*index].clone())
    }

    pub fn get_index_by_mac_address(&self, mac_address: &MacAddress) -> Option<usize> {
        self.mac_address_index_map.get(mac_address).copied()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Device> {
        self.devices.iter()
    }
}
