use super::{AnyComponent, ManageComponents};

#[derive(Default)]
pub struct ComponentManagerMassenger {
    queue: Vec<Message>,
}

impl ManageComponents for ComponentManagerMassenger {
    fn add_component(&mut self, comp: Box<dyn AnyComponent>) {
        self.queue.push(Message::AddComponent(comp));
    }

    fn new_layer(&mut self) {
        self.queue.push(Message::NewLayer);
    }

    fn pop_layer(&mut self) {
        self.queue.push(Message::PopLayer);
    }

    fn remove_by_name(&mut self, name: &str) {
        self.queue.push(Message::RemoveByName(name.to_string()));
    }

    fn add_component_at(&mut self, _: Box<dyn AnyComponent>, _: isize) {
        unimplemented!()
    }

    fn add_component_at_on_layer(&mut self, _: Box<dyn AnyComponent>, _: isize, _: usize) {
        unimplemented!()
    }
}

impl ComponentManagerMassengerQueue for ComponentManagerMassenger {
    fn take_queue(&mut self) -> Vec<Message> {
        std::mem::take(&mut self.queue)
    }
}

pub enum Message {
    AddComponent(Box<dyn AnyComponent>),
    NewLayer,
    PopLayer,
    RemoveByName(String),
}

pub trait ComponentManagerMassengerQueue {
    fn take_queue(&mut self) -> Vec<Message>;
}
