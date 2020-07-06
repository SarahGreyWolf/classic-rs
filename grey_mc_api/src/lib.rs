pub mod event;

pub trait Plugin {
    fn on_enable();
    fn on_disable();
}