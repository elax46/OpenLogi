//! Device-list construction and selection helpers for [`super::AppState`].

use openlogi_core::device::{BatteryInfo, DeviceInventory, DeviceKind};

use crate::asset::{AssetResolver, ResolvedAsset};
use crate::hardware::DpiTarget;

/// One paired device with everything the UI needs to switch to it in O(1):
/// the config key (for bindings/DPI persistence), a display name, the
/// resolved asset (PNG + metadata, or `None` for the synthetic fallback),
/// and the routing target for HID++ DPI writes.
///
/// The `kind` / `slot` / `online` / `battery` fields mirror the source
/// [`PairedDevice`](openlogi_core::device::PairedDevice) so the header
/// carousel can render straight from the device list — the list is the single
/// source of truth for "which devices exist", keeping carousel order aligned
/// with [`super::AppState::current_device`].
#[derive(Debug, Clone)]
pub struct DeviceRecord {
    pub config_key: String,
    pub display_name: String,
    pub asset: Option<ResolvedAsset>,
    pub dpi_target: Option<DpiTarget>,
    pub kind: DeviceKind,
    pub slot: u8,
    pub online: bool,
    pub battery: Option<BatteryInfo>,
}

pub(super) fn build_device_list(
    inventories: &[DeviceInventory],
    cache: &AssetResolver,
) -> Vec<DeviceRecord> {
    let mut list = Vec::new();
    for inv in inventories {
        let receiver_uid = inv.receiver.unique_id.clone();
        for paired in &inv.paired {
            let Some(model) = paired.model_info.as_ref() else {
                continue;
            };
            let config_key = model.config_key();
            let asset = cache.resolve(model);
            let display_name = asset
                .as_ref()
                .map(|a| a.display_name.clone())
                .or_else(|| paired.codename.clone())
                .unwrap_or_else(|| format!("Slot {}", paired.slot));
            let dpi_target = receiver_uid.as_ref().map(|uid| DpiTarget {
                receiver_uid: uid.clone(),
                slot: paired.slot,
            });
            list.push(DeviceRecord {
                config_key,
                display_name,
                asset,
                dpi_target,
                kind: paired.kind,
                slot: paired.slot,
                online: paired.online,
                battery: paired.battery.clone(),
            });
        }
    }
    list
}

pub(super) fn pick_initial_device(list: &[DeviceRecord], saved: Option<&str>) -> usize {
    saved
        .and_then(|key| list.iter().position(|r| r.config_key == key))
        .unwrap_or(0)
}
