use core::fmt;

#[derive(Debug, Default)]
pub struct Tudu {
    completed: bool,
    description: String,
}

impl fmt::Display for Tudu {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.completed {
            write!(f, "✓ {}", self.description)
        } else {
            write!(f, "☐ {}", self.description)
        }
    }
}

impl Tudu {
    pub fn new(description: String) -> Self {
        Tudu {
            completed: false,
            description,
        }
    }
}
