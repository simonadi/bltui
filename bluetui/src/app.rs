use crate::{bluetooth::devices::Devices, ui::widgets::popup::YesNoPopup};

pub struct AppNew {
    pub devices: Devices,
    pub popup: Option<YesNoPopup>,
}

impl AppNew {
    pub fn new() -> AppNew {
        AppNew {
            devices: Devices::new(),
            popup: None,
        }
    }
}
