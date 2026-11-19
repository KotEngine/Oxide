pub trait Nameable: Send + Sync {
    fn name(&self) -> &str;
    fn set_name(&mut self, name: String);
    fn has_custom_name(&self) -> bool { !self.name().is_empty() }
}
