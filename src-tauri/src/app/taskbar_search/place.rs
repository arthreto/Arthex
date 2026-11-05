/// The elements of the taskbar that decide where the button goes, as UI Automation names them.
pub const AUTOMATION_IDS: [&str; 4] = [
    "StartButton",
    "SearchButton",
    "TaskViewButton",
    "WidgetsButton",
];
pub const CLASS_NAMES: [&str; 2] = ["Taskbar.TaskListButtonAutomationPeer", "TrayNotifyWnd"];

/// The search box must be this many buttons wide. The narrower forms of the Windows search have
/// no free room, and the button would hide what they show.
const MIN_BOX_WIDTH: i32 = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Bounds {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

impl Bounds {
    fn width(self) -> i32 {
        self.right - self.left
    }

    fn height(self) -> i32 {
        self.bottom - self.top
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    SearchBox,
    /// Start, task view and the apps: the group the free room comes after.
    Group,
    /// Widgets and the notification area: what the free room ends at.
    Limit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Item {
    pub kind: Kind,
    pub bounds: Bounds,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Placement {
    pub x: i32,
    pub y: i32,
    pub size: u16,
}

pub fn kind_of(id: &str, class: &str) -> Option<Kind> {
    match (id, class) {
        ("SearchButton", _) => Some(Kind::SearchBox),
        ("StartButton" | "TaskViewButton", _) | (_, "Taskbar.TaskListButtonAutomationPeer") => {
            Some(Kind::Group)
        }
        ("WidgetsButton", _) | (_, "TrayNotifyWnd") => Some(Kind::Limit),
        _ => None,
    }
}

/// Inside the search box when it has room; otherwise, as when the Windows search is hidden, in
/// the free room after the apps. `None` when there is no room at all: it never covers anything.
pub fn placement(taskbar: Bounds, items: &[Item]) -> Option<Placement> {
    let shown = || {
        items
            .iter()
            .filter(|item| item.bounds.width() > 0 && item.bounds.height() > 0)
    };
    let search_box = shown().find(|item| item.kind == Kind::SearchBox);
    if let Some(placement) = search_box.and_then(|item| inside_search_box(item.bounds)) {
        return Some(placement);
    }
    let group_end = shown()
        .filter(|item| item.kind != Kind::Limit)
        .map(|item| item.bounds.right)
        .max()
        .unwrap_or(taskbar.left);
    let limit = shown()
        .filter(|item| item.kind == Kind::Limit && item.bounds.left >= group_end)
        .map(|item| item.bounds.left)
        .min()
        .unwrap_or(taskbar.right);
    let size = taskbar.height() * 8 / 15;
    let x = group_end + size / 4;
    if size <= 0 || x + size > limit {
        return None;
    }
    Some(Placement {
        x,
        y: taskbar.top + (taskbar.height() - size) / 2,
        size: u16::try_from(size).ok()?,
    })
}

/// At its right end, far enough from the edge to clear its rounded corners.
fn inside_search_box(search_box: Bounds) -> Option<Placement> {
    let height = search_box.height();
    let inset = height / 10;
    let size = height - 2 * inset;
    if size <= 0 || search_box.width() < size * MIN_BOX_WIDTH {
        return None;
    }
    Some(Placement {
        x: search_box.right - height / 5 - size,
        y: search_box.top + inset,
        size: u16::try_from(size).ok()?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const TASKBAR: Bounds = Bounds {
        left: 0,
        top: 1020,
        right: 1920,
        bottom: 1080,
    };

    fn item(kind: Kind, left: i32, right: i32) -> Item {
        Item {
            kind,
            bounds: Bounds {
                left,
                right,
                ..TASKBAR
            },
        }
    }

    fn apps() -> Vec<Item> {
        vec![
            item(Kind::Group, 0, 69),
            item(Kind::Group, 349, 404),
            item(Kind::Group, 404, 459),
            item(Kind::Group, 569, 624),
            item(Kind::Limit, 1409, 1599),
            item(Kind::Limit, 1614, 1920),
        ]
    }

    fn search_box(left: i32, right: i32) -> Item {
        Item {
            kind: Kind::SearchBox,
            bounds: Bounds {
                left,
                top: 1030,
                right,
                bottom: 1070,
            },
        }
    }

    #[test]
    fn sits_inside_a_wide_search_box_clear_of_its_rounded_end() {
        let mut items = apps();
        items.push(search_box(72, 347));
        assert_eq!(
            placement(TASKBAR, &items),
            Some(Placement {
                x: 307,
                y: 1034,
                size: 32
            })
        );
    }

    #[test]
    fn follows_the_apps_when_the_windows_search_is_hidden() {
        assert_eq!(
            placement(TASKBAR, &apps()),
            Some(Placement {
                x: 632,
                y: 1034,
                size: 32
            })
        );
    }

    #[test]
    fn follows_the_apps_when_the_windows_search_is_only_a_button() {
        let mut items = apps();
        items.push(item(Kind::SearchBox, 625, 680));
        assert_eq!(placement(TASKBAR, &items).map(|place| place.x), Some(688));
    }

    #[test]
    fn ignores_elements_the_taskbar_does_not_show() {
        let mut items = apps();
        items.push(search_box(0, 0));
        items.push(item(Kind::Group, 1000, 1000));
        assert_eq!(placement(TASKBAR, &items).map(|place| place.x), Some(632));
    }

    #[test]
    fn ignores_widgets_left_of_a_centred_group() {
        let items = [
            item(Kind::Limit, 0, 190),
            item(Kind::Group, 700, 1100),
            item(Kind::Limit, 1614, 1920),
        ];
        assert_eq!(placement(TASKBAR, &items).map(|place| place.x), Some(1108));
    }

    #[test]
    fn never_covers_anything_when_the_taskbar_is_full() {
        let items = [item(Kind::Group, 0, 1590), item(Kind::Limit, 1614, 1920)];
        assert_eq!(placement(TASKBAR, &items), None);
    }

    #[test]
    fn recognises_what_it_asked_for() {
        for id in AUTOMATION_IDS {
            assert!(kind_of(id, "").is_some(), "{id}");
        }
        for class in CLASS_NAMES {
            assert!(kind_of("", class).is_some(), "{class}");
        }
        assert_eq!(kind_of("Other", "Other"), None);
    }
}
