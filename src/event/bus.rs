use parking_lot::RwLock;
use super::Event;

pub type Listener<E> = Box<dyn Fn(&mut E) + Send + Sync + 'static>;
pub type AnyListener = Box<dyn Fn(&mut dyn Event) + Send + Sync + 'static>;

pub struct EventBus {
    listeners: RwLock<Vec<(String, AnyListener)>>,
}

impl EventBus {
    pub fn new() -> Self {
        Self { listeners: RwLock::new(Vec::new()) }
    }

    pub fn on<E: Event + 'static, F>(&self, name: &str, f: F)
    where
        F: Fn(&mut E) + Send + Sync + 'static,
    {
        let name = name.to_string();
        self.listeners.write().push((name.clone(), Box::new(move |event: &mut dyn Event| {
            if event.name() == std::any::type_name::<E>().rsplit("::").next().unwrap_or("") {
                if let Some(e) = event.as_any_mut().downcast_mut::<E>() {
                    f(e);
                }
            }
        })));
    }

    pub fn on_any<F>(&self, name: &str, f: F)
    where
        F: Fn(&mut dyn Event) + Send + Sync + 'static,
    {
        let name = name.to_string();
        self.listeners.write().push((name, Box::new(f)));
    }

    pub fn fire<E: Event + 'static>(&self, mut event: E) -> E {
        let name = event.name().to_string();
        let listeners = self.listeners.read().clone_to_vec(&name);
        drop(listeners);
        let listeners = self.listeners.read();
        let filtered: Vec<&AnyListener> = listeners.iter()
            .filter(|(n, _)| n == &name)
            .map(|(_, l)| l)
            .collect();
        let filtered: Vec<&AnyListener> = filtered;
        for listener in filtered {
            listener(&mut event as &mut dyn Event);
        }
        event
    }

    pub fn listener_count(&self) -> usize {
        self.listeners.read().len()
    }
}

trait CloneToVec {
    fn clone_to_vec(&self, _name: &str) -> Vec<(String, AnyListener)>;
}

impl CloneToVec for Vec<(String, AnyListener)> {
    fn clone_to_vec(&self, _name: &str) -> Vec<(String, AnyListener)> { Vec::new() }
}

impl Default for EventBus {
    fn default() -> Self { Self::new() }
}
