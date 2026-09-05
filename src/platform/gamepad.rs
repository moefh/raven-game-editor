use std::collections::HashMap;

#[derive(Debug)]
pub struct AxisMapping {
    pub min: u32,
    pub max: u32,
}

impl AxisMapping {
    pub fn new(min: u32, max: u32) -> Self {
        AxisMapping {
            min,
            max,
        }
    }
}

#[derive(Debug)]
pub struct Mapping {
    pub buttons: HashMap<u32, u32>,       // buttons[raw] = GAMEPAD_xxx
    pub axes: HashMap<u32, AxisMapping>,  // axes[raw].min = GAMEPAD_xxx, axes[raw].max = GAMEPAD_xxx
}

impl Mapping {
    pub fn new() -> Self {
        Self::with_mapping(HashMap::new(), HashMap::new())
    }

    pub fn with_mapping(buttons: HashMap<u32, u32>, axes: HashMap<u32, AxisMapping>) -> Self {
        Mapping {
            buttons,
            axes,
        }
    }

    pub fn clear(&mut self) {
        self.buttons.clear();
        self.axes.clear();
    }

    pub fn is_button_set(&self, button: u32) -> bool {
        self.buttons.values().find(|&b| *b == button).is_some() ||
            self.axes.values().find(|a| a.min == button || a.max == button).is_some()
    }
}

#[derive(Debug)]
pub enum RawEvent {
    Button { code: u32 },
    Axis { code: u32, val: f32 },
}

impl RawEvent {
    pub fn add_to_mapping(&self, button: u32, mapping: &mut Mapping) -> bool {
        match self {
            RawEvent::Button { code } => {
                mapping.buttons.insert(*code, button);
                true
            }
            RawEvent::Axis { code, val } => {
                let is_min = if *val <= -0.9 {
                    true
                } else if *val >= 0.9 {
                    false
                } else {
                    return false;
                };
                let axis = mapping.axes.entry(*code).or_insert_with(|| AxisMapping::new(0, 0));
                if is_min {
                    axis.min = button;
                } else {
                    axis.max = button;
                }
                true
            }
        }
    }
}

#[allow(unused)]
pub struct Gamepad {
    pub id: String,
    pub cur: u32,
    pub old: u32,
}

impl Gamepad {
    #[allow(unused)]
    pub fn new(id: String) -> Self {
        Gamepad {
            id,
            cur: 0,
            old: 0,
        }
    }

    #[allow(unused)]
    pub fn held(&self, buttons: u32) -> bool {
        (self.cur & buttons) != 0
    }

    #[allow(unused)]
    pub fn pressed(&self, buttons: u32) -> bool {
        (self.old & buttons) == 0 && (self.cur & buttons) != 0
    }
}
