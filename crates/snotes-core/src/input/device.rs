//! Input device abstraction and capability detection

use serde::{Deserialize, Serialize};

/// Represents a connected input device
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputDevice {
    /// Unique device identifier (udev path or HID path)
    pub id: String,
    /// Human-readable device name
    pub name: String,
    /// USB Vendor ID
    pub vendor_id: u16,
    /// USB Product ID
    pub product_id: u16,
    /// Type of input device
    pub device_type: DeviceType,
    /// Device capabilities
    pub capabilities: DeviceCapabilities,
}

/// Type of input device
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DeviceType {
    /// Dedicated graphics tablet (e.g., Wacom Intuos)
    Tablet,
    /// Pen display / tablet monitor (e.g., Wacom Cintiq)
    PenDisplay,
    /// Touchscreen with stylus support
    Touchscreen,
    /// Generic stylus device
    Stylus,
    /// Mouse (limited support)
    Mouse,
    /// Unknown device type
    Unknown,
}

/// Capabilities of an input device
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceCapabilities {
    /// Device supports pressure sensitivity
    pub has_pressure: bool,
    /// Device supports tilt detection
    pub has_tilt: bool,
    /// Device supports rotation detection
    pub has_rotation: bool,
    /// Device has buttons (barrel buttons, etc.)
    pub has_buttons: bool,
    /// Raw pressure range (min, max) from device
    pub pressure_range: (u32, u32),
    /// Tilt range in degrees (min, max)
    pub tilt_range: (f32, f32),
    /// Maximum resolution in LPI (lines per inch)
    pub max_resolution: (u32, u32),
}

impl DeviceCapabilities {
    /// Create capabilities for a basic mouse (no pressure/tilt)
    pub fn mouse() -> Self {
        Self {
            has_pressure: false,
            has_tilt: false,
            has_rotation: false,
            has_buttons: true,
            pressure_range: (0, 1),
            tilt_range: (0.0, 0.0),
            max_resolution: (0, 0),
        }
    }

    /// Create default tablet capabilities
    pub fn tablet_default() -> Self {
        Self {
            has_pressure: true,
            has_tilt: true,
            has_rotation: false,
            has_buttons: true,
            pressure_range: (0, 8192),
            tilt_range: (-60.0, 60.0),
            max_resolution: (5080, 5080),
        }
    }
}

/// Known tablet vendors for automatic configuration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabletVendor {
    Wacom,
    Huion,
    XPPen,
    Gaomon,
    Samsung,
    Microsoft,
    Apple,
    Unknown,
}

impl TabletVendor {
    /// Identify vendor from USB Vendor ID
    pub fn from_vendor_id(vendor_id: u16) -> Self {
        match vendor_id {
            0x056a => TabletVendor::Wacom,
            0x256c => TabletVendor::Huion,
            0x28bd => TabletVendor::XPPen,
            0x256d => TabletVendor::Gaomon,
            0x04e8 => TabletVendor::Samsung,
            0x045e => TabletVendor::Microsoft,
            0x05ac => TabletVendor::Apple,
            _ => TabletVendor::Unknown,
        }
    }

    /// Get display name for the vendor
    pub fn display_name(&self) -> &'static str {
        match self {
            TabletVendor::Wacom => "Wacom",
            TabletVendor::Huion => "Huion",
            TabletVendor::XPPen => "XP-Pen",
            TabletVendor::Gaomon => "Gaomon",
            TabletVendor::Samsung => "Samsung",
            TabletVendor::Microsoft => "Microsoft",
            TabletVendor::Apple => "Apple",
            TabletVendor::Unknown => "Unknown",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vendor_detection() {
        assert_eq!(TabletVendor::from_vendor_id(0x056a), TabletVendor::Wacom);
        assert_eq!(TabletVendor::from_vendor_id(0x256c), TabletVendor::Huion);
        assert_eq!(TabletVendor::from_vendor_id(0x28bd), TabletVendor::XPPen);
        assert_eq!(TabletVendor::from_vendor_id(0xFFFF), TabletVendor::Unknown);
    }
}
