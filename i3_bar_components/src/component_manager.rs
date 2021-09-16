use log::*;
use std::any::Any;
use std::sync::mpsc::Receiver;
use std::{
    io::{BufRead, Read, Stdout, Write},
    sync::mpsc::Sender,
    time::SystemTime,
};

use crate::{
    components::prelude::*,
    protocol::{ClickEvent, Header},
};

pub struct ComponentManager {
    components: Vec<Box<dyn Component>>,
    event_reader: Receiver<ClickEvent>,
    out_writer: Stdout,
    render_last_block_count: usize,
    message_tx: Sender<Message>,
    message_rx: Receiver<Message>,
    last_update: SystemTime,
    next_instance_id: u128,
}

impl ComponentManager {
    pub fn update(&mut self) {
        let dt = self.last_update.elapsed().unwrap().as_secs_f64();
        self.last_update = SystemTime::now();
        let events = self.event_reader.try_iter().collect::<Vec<ClickEvent>>();

        events.iter().for_each(|event| {
            let element_id = event.get_id();
            let comp = self.components.iter_mut().find(|comp| {
                let mut blocks = Vec::new();
                comp.collect_base_components(&mut blocks);
                blocks.iter().any(|b| b.get_id() == element_id)
            });

            if let Some(comp) = comp {
                comp.event(event);
            }
        });

        self.components.iter_mut().for_each(|c| c.update(dt));

        let messages = self
            .message_rx
            .try_recv()
            .into_iter()
            .collect::<Vec<Message>>();

        messages.iter().for_each(|m| {
            let id = m.get_id();
            match m.get_message_type() {
                MessageType::Remove => {
                    let index =
                        self.components
                            .iter()
                            .enumerate()
                            .find_map(|(index, component)| {
                                if component.get_id() == id {
                                    Some(index)
                                } else {
                                    None
                                }
                            });

                    if let Some(i) = index {
                        self.components.remove(i);
                    }
                }
            }
        });

        drop(messages);

        let mut blocks = self
            .components
            .iter_mut()
            .fold(
                Vec::with_capacity(self.render_last_block_count),
                |mut blocks, c| {
                    c.collect_base_components_mut(&mut blocks);
                    blocks
                },
            )
            .iter_mut()
            .enumerate()
            .fold(vec![b'['], |mut blocks, (index, block)| {
                if index != 0 {
                    blocks.push(b',');
                }
                blocks.extend(block.serialize_cache());
                blocks
            });
        blocks.push(b']');
        blocks.push(b',');
        blocks.push(10);

        self.render_last_block_count = blocks.len();
        self.out_writer.write_all(&blocks).unwrap();
        self.out_writer.flush().unwrap();
    }

    pub fn add_component(&mut self, mut comp: Box<dyn Component>) {
        let mut base_components = Vec::new();
        comp.collect_base_components_mut(&mut base_components);

        base_components.iter_mut().for_each(|component| {
            component
                .get_block_mut()
                .set_instance(self.next_instance_id.to_string());
            self.next_instance_id += 1;
        });

        let comp_ref = comp.as_mut();
        comp_ref.add_component_manager_messenger(ComponentManagerMessenger::new(
            String::from(comp_ref.get_id()),
            self.message_tx.clone(),
        ));
        self.components.push(comp);
    }

    pub fn get_component_mut<'a, T: Component>(&'a mut self, name: &str) -> Option<&'a mut T> {
        self.components.iter_mut().find_map(|c| {
            if c.name() == name {
                let c: &mut dyn Any = c;
                c.downcast_mut::<T>()
            } else {
                None
            }
        })
    }

    pub fn remove_by_name(&mut self, name: &str) {
        let index = match self.components.iter().position(|c| c.name() == name) {
            Some(s) => s,
            None => return,
        };

        self.components.remove(index);
    }
}

fn read_events(
    reader: &mut dyn BufRead,
    event_number: &mut usize,
    tx: &std::sync::mpsc::Sender<ClickEvent>,
) {
    let mut event = String::new();

    reader.read_line(&mut event).unwrap();
    debug!(r#"Received Event "{}""#, event);

    let mut new_event_number = *event_number + 1;

    let event = match event_number {
        0 => {
            std::mem::swap(event_number, &mut new_event_number);
            return;
        }
        1 => &event,
        _ => &event[1..],
    };

    std::mem::swap(event_number, &mut new_event_number);

    let click_event = serde_json::from_str::<ClickEvent>(event).unwrap();
    tx.send(click_event).unwrap();
}

pub struct ComponentManagerBuilder {
    stdin: Option<Box<dyn Read>>,
    stdout: Option<Box<dyn Write>>,
    click_events: bool,
}

impl Default for ComponentManagerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl ComponentManagerBuilder {
    pub fn new() -> Self {
        Self {
            stdin: None,
            stdout: None,
            click_events: false,
        }
    }

    pub fn set_click_events(&mut self, click_events: bool) {
        self.click_events = click_events;
    }

    pub fn with_click_events(mut self, click_events: bool) -> Self {
        self.set_click_events(click_events);
        self
    }

    pub fn set_stdin(&mut self, stdin: Box<dyn Read>) {
        self.stdin = Some(stdin);
    }

    pub fn with_stdin(mut self, stdin: Box<dyn Read>) -> Self {
        self.set_stdin(stdin);
        self
    }

    pub fn set_stdout(&mut self, stdout: Box<dyn Write>) {
        self.stdout = Some(stdout);
    }

    pub fn with_stdout(mut self, stdout: Box<dyn Write>) -> Self {
        self.set_stdout(stdout);
        self
    }

    pub fn build(self) -> ComponentManager {
        let mut out_writer = std::io::stdout();

        let header = Header::new().with_click_events(self.click_events);

        out_writer
            .write_all(&serde_json::to_vec(&header).unwrap())
            .unwrap();
        out_writer.write_all(&[10, b'[']).unwrap();

        let (tx, rx) = std::sync::mpsc::channel();

        std::thread::spawn(move || {
            let stdin = std::io::stdin();
            let mut last_event_id = 0;
            let tx = tx;

            loop {
                read_events(&mut stdin.lock(), &mut last_event_id, &tx);
            }
        });

        let (message_tx, message_rx) = std::sync::mpsc::channel();

        ComponentManager {
            components: Vec::new(),
            event_reader: rx,
            out_writer,
            render_last_block_count: 0,
            message_rx,
            message_tx,
            last_update: SystemTime::now(),
            next_instance_id: 1,
        }
    }
}

pub struct ComponentManagerMessenger {
    id: String,
    tx: Sender<Message>,
}

impl ComponentManagerMessenger {
    fn new(id: String, tx: Sender<Message>) -> Self {
        Self { id, tx }
    }

    pub fn remove(&self) {
        self.tx
            .send(Message {
                id: self.id.clone(),
                message_type: MessageType::Remove,
            })
            .unwrap();
    }
}

pub struct Message {
    id: String,
    message_type: MessageType,
}

impl Message {
    pub fn get_id(&self) -> &str {
        &self.id
    }

    pub fn get_message_type(&self) -> &MessageType {
        &self.message_type
    }
}

pub enum MessageType {
    Remove,
}
