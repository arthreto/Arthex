use std::mem::{ManuallyDrop, transmute_copy};

use windows::{
    Win32::{
        Foundation::{HWND, RECT},
        System::{
            Com::{CLSCTX_INPROC_SERVER, CoCreateInstance},
            Variant::{VARIANT, VARIANT_0, VARIANT_0_0, VARIANT_0_0_0, VT_BSTR},
        },
        UI::Accessibility::{
            CUIAutomation, IUIAutomation, IUIAutomationCacheRequest, IUIAutomationCondition,
            TreeScope_Descendants, UIA_AutomationIdPropertyId, UIA_BoundingRectanglePropertyId,
            UIA_ClassNamePropertyId, UIA_PROPERTY_ID,
        },
    },
    core::{BSTR, Result},
};

use super::place::{AUTOMATION_IDS, Bounds, CLASS_NAMES, Item, kind_of};

pub struct Locator {
    automation: IUIAutomation,
    watched: IUIAutomationCondition,
    /// Brings every property back in one call: Explorer is only asked once per reading.
    properties: IUIAutomationCacheRequest,
}

impl Locator {
    /// Must be created and used on a thread where COM is initialised.
    pub fn new() -> Option<Self> {
        Self::create()
            .inspect_err(|error| tracing::warn!(%error, "UI Automation is unavailable"))
            .ok()
    }

    fn create() -> Result<Self> {
        // SAFETY: COM calls on a thread where COM is initialised.
        unsafe {
            let automation: IUIAutomation =
                CoCreateInstance(&CUIAutomation, None, CLSCTX_INPROC_SERVER)?;
            let mut watched = automation.CreateFalseCondition()?;
            let ids = AUTOMATION_IDS.map(|id| (UIA_AutomationIdPropertyId, id));
            let classes = CLASS_NAMES.map(|class| (UIA_ClassNamePropertyId, class));
            for (property, value) in ids.into_iter().chain(classes) {
                let condition = text_condition(&automation, property, value)?;
                watched = automation.CreateOrCondition(&watched, &condition)?;
            }
            let properties = automation.CreateCacheRequest()?;
            properties.AddProperty(UIA_BoundingRectanglePropertyId)?;
            properties.AddProperty(UIA_AutomationIdPropertyId)?;
            properties.AddProperty(UIA_ClassNamePropertyId)?;
            Ok(Self {
                automation,
                watched,
                properties,
            })
        }
    }

    /// The taskbar and what is on it, in physical pixels.
    pub fn read(&self, taskbar: isize) -> Option<(Bounds, Vec<Item>)> {
        // SAFETY: COM calls on objects this thread created; a stale window handle only fails.
        unsafe {
            let root = self
                .automation
                .ElementFromHandle(HWND(taskbar as *mut _))
                .ok()?;
            let bounds = bounds_of(root.CurrentBoundingRectangle().ok()?);
            let found = root
                .FindAllBuildCache(TreeScope_Descendants, &self.watched, &self.properties)
                .ok()?;
            let mut items = Vec::new();
            for index in 0..found.Length().ok()? {
                let element = found.GetElement(index).ok()?;
                let id = element.CachedAutomationId().ok()?.to_string();
                let class = element.CachedClassName().ok()?.to_string();
                if let Some(kind) = kind_of(&id, &class) {
                    let bounds = bounds_of(element.CachedBoundingRectangle().ok()?);
                    items.push(Item { kind, bounds });
                }
            }
            Some((bounds, items))
        }
    }
}

fn bounds_of(rectangle: RECT) -> Bounds {
    Bounds {
        left: rectangle.left,
        top: rectangle.top,
        right: rectangle.right,
        bottom: rectangle.bottom,
    }
}

/// # Safety
/// COM must be initialised on this thread.
unsafe fn text_condition(
    automation: &IUIAutomation,
    property: UIA_PROPERTY_ID,
    text: &str,
) -> Result<IUIAutomationCondition> {
    let owned = BSTR::from(text);
    // SAFETY: the variant only borrows the string, which `owned` frees after the call; the
    // condition keeps its own copy.
    unsafe {
        let value = VARIANT {
            Anonymous: VARIANT_0 {
                Anonymous: ManuallyDrop::new(VARIANT_0_0 {
                    vt: VT_BSTR,
                    Anonymous: VARIANT_0_0_0 {
                        bstrVal: transmute_copy(&owned),
                    },
                    ..Default::default()
                }),
            },
        };
        automation.CreatePropertyCondition(property, &value)
    }
}
