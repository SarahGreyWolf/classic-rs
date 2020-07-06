pub trait Event {
    fn set_cancelled(&mut self, cancelled: bool);
    fn is_cancelled(&self) -> bool;
}

enum Events {
    /// Dispatched when a user interacts in some way
    /// [false] for right click, [true] for left click
    UserInteract(bool)
}

pub struct UserInteract {
    is_cancelled: bool,
    hand: bool,
}

impl UserInteract {
    pub fn get_hand(&self) -> bool {
        self.hand
    }
}

impl Event for UserInteract {
    fn set_cancelled(&mut self, cancelled: bool) {
        self.is_cancelled = cancelled;
    }

    fn is_cancelled(&self) -> bool {
        self.is_cancelled
    }
}