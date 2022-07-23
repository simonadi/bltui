use bluez_generated::OrgBluezAdapter1Properties;
use dbus::Path;
use serde::{Deserialize, Serialize};
use std::fmt::{self, Display, Formatter};

use crate::Modalias;
use crate::{AddressType, BluetoothError, MacAddress};

/// Opaque identifier for a Bluetooth adapter on the system.
#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct AdapterId {
    #[serde(with = "crate::serde_path")]
    pub(crate) object_path: Path<'static>,
}

impl AdapterId {
    pub(crate) fn new(object_path: &str) -> Self {
        Self {
            object_path: object_path.to_owned().into(),
        }
    }
}

impl From<AdapterId> for Path<'static> {
    fn from(id: AdapterId) -> Self {
        id.object_path
    }
}

impl Display for AdapterId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            self.object_path
                .to_string()
                .strip_prefix("/org/bluez/")
                .ok_or(fmt::Error)?
        )
    }
}

/// Information about a Bluetooth adapter on the system.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdapterInfo {
    /// An opaque identifier for the adapter. This can be used to perform operations on it.
    pub id: AdapterId,
    /// The MAC address of the adapter.
    pub mac_address: MacAddress,
    /// The type of MAC address the adapter uses.
    pub address_type: AddressType,
    /// The Bluetooth system hostname.
    pub name: String,
    /// The Bluetooth friendly name. This defaults to the system hostname.
    pub alias: String,
    /// Information about the Bluetooth adapter, mostly useful for debug purposes.
    pub modalias: Modalias,
    /// Whether the adapter is currently turned on.
    pub powered: bool,
    /// Whether the adapter is currently discovering devices.
    pub discovering: bool,
}

impl AdapterInfo {
    pub(crate) fn from_properties(
        id: AdapterId,
        adapter_properties: OrgBluezAdapter1Properties,
    ) -> Result<AdapterInfo, BluetoothError> {
        let mac_address = adapter_properties
            .address()
            .ok_or(BluetoothError::RequiredPropertyMissing("Address"))?
            .parse()?;
        let address_type = adapter_properties
            .address_type()
            .ok_or(BluetoothError::RequiredPropertyMissing("AddressType"))?
            .parse()?;
        let modalias = adapter_properties
            .modalias()
            .ok_or(BluetoothError::RequiredPropertyMissing("Modalias"))?
            .parse()?;

        Ok(AdapterInfo {
            id,
            mac_address,
            address_type,
            name: adapter_properties
                .name()
                .ok_or(BluetoothError::RequiredPropertyMissing("Name"))?
                .to_owned(),
            alias: adapter_properties
                .alias()
                .ok_or(BluetoothError::RequiredPropertyMissing("Alias"))?
                .to_owned(),
            modalias,
            powered: adapter_properties
                .powered()
                .ok_or(BluetoothError::RequiredPropertyMissing("Powered"))?,
            discovering: adapter_properties
                .discovering()
                .ok_or(BluetoothError::RequiredPropertyMissing("Discovering"))?,
        })
    }
}

#[cfg(test)]
mod tests {
    use dbus::arg::{PropMap, Variant};
    use std::collections::HashMap;

    use super::*;

    #[test]
    fn adapter_info_minimal() {
        let id = AdapterId::new("/org/bluez/hci0");
        let mut adapter_properties: PropMap = HashMap::new();
        adapter_properties.insert(
            "Address".to_string(),
            Variant(Box::new("00:11:22:33:44:55".to_string())),
        );
        adapter_properties.insert(
            "AddressType".to_string(),
            Variant(Box::new("public".to_string())),
        );
        adapter_properties.insert("Name".to_string(), Variant(Box::new("name".to_string())));
        adapter_properties.insert("Alias".to_string(), Variant(Box::new("alias".to_string())));
        adapter_properties.insert(
            "Modalias".to_string(),
            Variant(Box::new("usb:v1234p5678d90AB".to_string())),
        );
        adapter_properties.insert("Powered".to_string(), Variant(Box::new(false)));
        adapter_properties.insert("Discovering".to_string(), Variant(Box::new(false)));

        let adapter = AdapterInfo::from_properties(
            id.clone(),
            OrgBluezAdapter1Properties(&adapter_properties),
        )
        .unwrap();
        assert_eq!(
            adapter,
            AdapterInfo {
                id,
                mac_address: "00:11:22:33:44:55".parse().unwrap(),
                address_type: AddressType::Public,
                name: "name".to_string(),
                alias: "alias".to_string(),
                modalias: Modalias {
                    vendor_id: 0x1234,
                    product_id: 0x5678,
                    device_id: 0x90ab
                },
                powered: false,
                discovering: false
            }
        )
    }

    #[test]
    fn to_string() {
        let adapter_id = AdapterId::new("/org/bluez/hci0");
        assert_eq!(adapter_id.to_string(), "hci0");
    }
}
