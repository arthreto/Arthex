//! Decides what each key event means for the hotkey. Pure, so it is tested without Windows.

use super::keys::{Hotkey, Key, Mods, classify, key_name};

const ESCAPE: u16 = 0x1B;

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Reaction {
    /// Hide the event from every other application.
    pub swallow: bool,
    /// Press the neutral mask key, so releasing Win or Alt opens neither the Start menu nor a menu bar.
    pub mask: bool,
    pub signal: Option<Signal>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Signal {
    Fire,
    /// Modifiers held while recording, for live feedback.
    Held(Mods),
    /// End of a recording: the keys pressed, or `None` when cancelled with Escape.
    Recorded(Option<Hotkey>),
}

#[derive(Default)]
struct Chord {
    armed: bool,
    tainted: bool,
    fired: bool,
}

pub struct Machine {
    hotkey: Hotkey,
    held: Mods,
    chord: Chord,
    /// Keys whose key-down was hidden: their key-up is hidden too, so no key ever looks stuck.
    swallowed: Vec<u16>,
    /// Modifiers seen together since recording started.
    recording: Option<Mods>,
}

impl Machine {
    pub fn new(hotkey: Hotkey) -> Self {
        Self {
            hotkey,
            held: Mods::default(),
            chord: Chord::default(),
            swallowed: Vec::new(),
            recording: None,
        }
    }

    pub fn set_hotkey(&mut self, hotkey: Hotkey) {
        self.hotkey = hotkey;
        self.chord = Chord::default();
    }

    pub fn record(&mut self, active: bool) {
        self.recording = active.then(Mods::default);
    }

    /// The modifiers Windows says are down, just before a modifier is pressed. The hook may miss
    /// a key-up as the focus moves to the overlay: the chord would then never end, and the next
    /// press would be ignored. A recording hides key presses from Windows, so it keeps its own.
    pub fn sync(&mut self, physical: Mods) {
        if self.recording.is_none() && self.held != physical {
            self.held = physical;
            self.chord = Chord::default();
        }
    }

    pub fn on_key(&mut self, vk: u16, down: bool) -> Reaction {
        let key = classify(vk);
        if let Key::Modifier(mods) = key {
            if down {
                self.held.insert(mods);
            } else {
                self.held.remove(mods);
            }
        }

        let mut reaction = if self.recording.is_some() {
            self.on_recording(&key, down)
        } else if self.hotkey.key.is_some() {
            self.on_combo(&key, down)
        } else {
            self.on_modifiers(&key, down)
        };

        if down {
            if reaction.swallow && !self.swallowed.contains(&vk) {
                self.swallowed.push(vk);
            }
        } else if let Some(index) = self.swallowed.iter().position(|&code| code == vk) {
            self.swallowed.swap_remove(index);
            reaction.swallow = true;
        }

        if self.held.is_empty() {
            self.chord = Chord::default();
        }
        reaction
    }

    /// Modifier-only hotkeys fire on release, and only if no other key was pressed meanwhile:
    /// Win + Shift + S keeps taking screenshots.
    fn on_modifiers(&mut self, key: &Key, down: bool) -> Reaction {
        let target = self.hotkey.mods;
        let chord = &mut self.chord;
        match (key, down) {
            (Key::Other(_), true) if !self.held.is_empty() => chord.tainted = true,
            (Key::Modifier(_), true) => {
                if !target.contains(self.held) {
                    chord.tainted = true;
                } else if self.held == target && !chord.tainted && !chord.armed {
                    chord.armed = true;
                    return Reaction {
                        mask: target.intersects(Mods::WIN.union(Mods::ALT)),
                        ..Reaction::default()
                    };
                }
            }
            (Key::Modifier(_), false) if chord.armed && !chord.tainted && !chord.fired => {
                chord.fired = true;
                return Reaction {
                    signal: Some(Signal::Fire),
                    ..Reaction::default()
                };
            }
            _ => {}
        }
        Reaction::default()
    }

    /// Hotkeys with a key fire on its key-down, which no application then receives.
    fn on_combo(&self, key: &Key, down: bool) -> Reaction {
        match key {
            Key::Other(vk)
                if down && Some(*vk) == self.hotkey.key && self.held == self.hotkey.mods =>
            {
                let first = !self.swallowed.contains(vk);
                Reaction {
                    swallow: true,
                    mask: first && self.held.intersects(Mods::WIN.union(Mods::ALT)),
                    signal: first.then_some(Signal::Fire),
                }
            }
            _ => Reaction::default(),
        }
    }

    /// While recording, every key-down is hidden so Windows shortcuts and the Start menu stay quiet.
    fn on_recording(&mut self, key: &Key, down: bool) -> Reaction {
        let held = self.held;
        let swallow = |signal| Reaction {
            swallow: true,
            mask: false,
            signal,
        };
        match (key, down) {
            (Key::Other(ESCAPE), true) if held.is_empty() => {
                self.recording = None;
                swallow(Some(Signal::Recorded(None)))
            }
            (Key::Other(vk), true) => {
                if key_name(*vk).is_none() {
                    return swallow(None);
                }
                self.recording = None;
                swallow(Some(Signal::Recorded(Some(Hotkey {
                    mods: held,
                    key: Some(*vk),
                }))))
            }
            (Key::Modifier(_), true) => {
                if let Some(peak) = &mut self.recording {
                    peak.insert(held);
                }
                swallow(Some(Signal::Held(held)))
            }
            (Key::Modifier(_), false) => {
                let peak = self.recording.unwrap_or_default();
                if held.is_empty() && !peak.is_empty() {
                    self.recording = None;
                    let hotkey = Hotkey {
                        mods: peak,
                        key: None,
                    };
                    return Reaction {
                        signal: Some(Signal::Recorded(Some(hotkey))),
                        ..Reaction::default()
                    };
                }
                Reaction {
                    signal: Some(Signal::Held(held)),
                    ..Reaction::default()
                }
            }
            (Key::Other(_), false) => Reaction::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SHIFT: u16 = 0xA0;
    const WIN: u16 = 0x5B;
    const CTRL: u16 = 0xA2;
    const ALT: u16 = 0xA4;
    const S: u16 = 0x53;
    const K: u16 = 0x4B;
    const SPACE: u16 = 0x20;

    fn machine(names: &[&str]) -> Machine {
        Machine::new(Hotkey::parse(names).unwrap())
    }

    /// Plays `(vk, down)` events and returns how many times the hotkey fired.
    fn fires(machine: &mut Machine, events: &[(u16, bool)]) -> usize {
        events
            .iter()
            .filter(|(vk, down)| machine.on_key(*vk, *down).signal == Some(Signal::Fire))
            .count()
    }

    #[test]
    fn shift_win_fires_once_on_release_in_any_order() {
        let mut m = machine(&["Shift", "Win"]);
        assert_eq!(
            fires(
                &mut m,
                &[(SHIFT, true), (WIN, true), (SHIFT, false), (WIN, false)]
            ),
            1
        );
        assert_eq!(
            fires(
                &mut m,
                &[(WIN, true), (SHIFT, true), (WIN, false), (SHIFT, false)]
            ),
            1
        );
    }

    #[test]
    fn a_missed_key_up_does_not_swallow_the_next_press() {
        let mut m = machine(&["Shift", "Win"]);
        // Shift is released, but the hook never hears it.
        assert_eq!(
            fires(&mut m, &[(SHIFT, true), (WIN, true), (WIN, false)]),
            1
        );
        m.sync(Mods::default());
        assert_eq!(
            fires(
                &mut m,
                &[(SHIFT, true), (WIN, true), (WIN, false), (SHIFT, false)]
            ),
            1
        );
    }

    #[test]
    fn sync_agrees_with_a_normal_press() {
        let mut m = machine(&["Shift", "Win"]);
        m.sync(Mods::default());
        assert!(m.on_key(SHIFT, true).signal.is_none());
        m.sync(Mods::SHIFT);
        assert!(m.on_key(WIN, true).mask);
        assert_eq!(m.on_key(WIN, false).signal, Some(Signal::Fire));
    }

    #[test]
    fn shift_win_masks_the_start_menu_and_never_swallows() {
        let mut m = machine(&["Shift", "Win"]);
        assert_eq!(m.on_key(SHIFT, true), Reaction::default());
        let armed = m.on_key(WIN, true);
        assert!(armed.mask && !armed.swallow);
        assert!(!m.on_key(SHIFT, false).swallow);
        assert!(!m.on_key(WIN, false).swallow);
    }

    #[test]
    fn other_windows_shortcuts_do_not_fire() {
        let mut m = machine(&["Shift", "Win"]);
        let screenshot = [
            (WIN, true),
            (SHIFT, true),
            (S, true),
            (S, false),
            (SHIFT, false),
            (WIN, false),
        ];
        assert_eq!(fires(&mut m, &screenshot), 0);
        let typing = [
            (SHIFT, true),
            (S, true),
            (S, false),
            (WIN, true),
            (WIN, false),
            (SHIFT, false),
        ];
        assert_eq!(fires(&mut m, &typing), 0);
        let extra = [
            (CTRL, true),
            (SHIFT, true),
            (WIN, true),
            (WIN, false),
            (SHIFT, false),
            (CTRL, false),
        ];
        assert_eq!(fires(&mut m, &extra), 0);
        assert_eq!(fires(&mut m, &[(WIN, true), (WIN, false)]), 0);
        assert_eq!(
            fires(
                &mut m,
                &[(SHIFT, true), (WIN, true), (WIN, false), (SHIFT, false)]
            ),
            1
        );
    }

    #[test]
    fn combo_fires_on_key_down_and_hides_the_key() {
        let mut m = machine(&["Ctrl", "Alt", "Space"]);
        m.on_key(CTRL, true);
        m.on_key(ALT, true);
        let fired = m.on_key(SPACE, true);
        assert_eq!(fired.signal, Some(Signal::Fire));
        assert!(fired.swallow && fired.mask);

        let repeat = m.on_key(SPACE, true);
        assert!(repeat.swallow && repeat.signal.is_none());
        assert!(m.on_key(SPACE, false).swallow);
        assert!(!m.on_key(ALT, false).swallow);
        assert!(!m.on_key(CTRL, false).swallow);
    }

    #[test]
    fn combo_ignores_other_modifiers() {
        let mut m = machine(&["Ctrl", "Space"]);
        let events = [
            (CTRL, true),
            (SHIFT, true),
            (SPACE, true),
            (SPACE, false),
            (SHIFT, false),
            (CTRL, false),
        ];
        assert_eq!(fires(&mut m, &events), 0);
        assert!(!m.on_key(SPACE, true).swallow);
    }

    #[test]
    fn a_new_hotkey_applies_immediately() {
        let mut m = machine(&["Shift", "Win"]);
        m.set_hotkey(Hotkey::parse(&["Ctrl", "Space"]).unwrap());
        assert_eq!(
            fires(
                &mut m,
                &[(SHIFT, true), (WIN, true), (WIN, false), (SHIFT, false)]
            ),
            0
        );
        assert_eq!(fires(&mut m, &[(CTRL, true), (SPACE, true)]), 1);
    }

    #[test]
    fn records_modifiers_only_without_leaking_them() {
        let mut m = machine(&["Ctrl", "Space"]);
        m.record(true);
        assert_eq!(
            m.on_key(SHIFT, true).signal,
            Some(Signal::Held(Mods::SHIFT))
        );
        assert!(m.on_key(WIN, true).swallow);
        assert!(m.on_key(WIN, false).swallow);
        let done = m.on_key(SHIFT, false);
        assert!(done.swallow);
        assert_eq!(done.signal, Some(Signal::Recorded(Some(Hotkey::DEFAULT))));
        assert_eq!(fires(&mut m, &[(CTRL, true), (SPACE, true)]), 1);
    }

    #[test]
    fn records_a_key_combination() {
        let mut m = machine(&["Shift", "Win"]);
        m.record(true);
        m.on_key(CTRL, true);
        let done = m.on_key(K, true);
        let expected = Hotkey {
            mods: Mods::CTRL,
            key: Some(K),
        };
        assert_eq!(done.signal, Some(Signal::Recorded(Some(expected))));
        assert!(m.on_key(K, false).swallow);
        assert!(m.on_key(CTRL, false).swallow);
    }

    #[test]
    fn escape_cancels_and_keys_held_before_recording_are_released_normally() {
        let mut m = machine(&["Shift", "Win"]);
        m.on_key(K, true);
        m.record(true);
        assert!(!m.on_key(K, false).swallow);
        assert_eq!(m.on_key(ESCAPE, true).signal, Some(Signal::Recorded(None)));
        assert!(m.on_key(ESCAPE, false).swallow);
        assert!(!m.on_key(S, true).swallow);
    }
}
