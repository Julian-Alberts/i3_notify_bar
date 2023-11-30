use log::*;
use std::any::Any;
use std::sync::mpsc::Receiver;
use std::{
    io::{BufRead, Read, Stdout, Write},
    time::SystemTime,
};

use crate::{
    components::prelude::*,
    protocol::{ClickEvent, Header},
};

use super::component_manager_messenger::{
    ComponentManagerMassenger, ComponentManagerMassengerQueue, Message,
};
use super::ManageComponents;

pub struct ComponentManager {
    layers: Vec<Vec<Box<dyn Component>>>,
    event_reader: Receiver<ClickEvent>,
    out_writer: Stdout,
    last_update: SystemTime,
    next_instance_id: u32,
    global_event_listener: fn(&mut dyn ManageComponents, &ClickEvent),
    component_manager_messenger: ComponentManagerMassenger,
}

impl ComponentManager {
    pub fn update(&mut self) {
        let dt = match self.last_update.elapsed() {
            Ok(elapsed) => elapsed.as_secs_f64(),
            Err(_) => {
                error!("Do not mess with time!");
                0.0
            }
        };
        self.last_update = SystemTime::now();

        self.handle_events();
        self.handle_messenges();
        self.update_components(dt);
        if let Err(err) = self.build_json() {
            error!("Could not write bytes: {:#?}", err);
            return;
        }
        if self.out_writer.flush().is_err() {
            error!("Error while flushing buffer");
        }
    }

    fn handle_events(&mut self) {
        let event_reader = &mut self.event_reader;
        let events = event_reader.try_iter().collect::<Vec<ClickEvent>>();
        let cmm = &mut self.component_manager_messenger;
        let layer = self.layers.last_mut().unwrap();
        let global_event_listener = &self.global_event_listener;

        events.iter().for_each(|event| {
            debug!("Event detected: {:?}", event);
            (global_event_listener)(cmm, event);
            let element_id = event.get_instance();
            let comp = layer.iter_mut().find(|comp| {
                let mut blocks = Vec::new();
                comp.collect_base_components(&mut blocks);
                trace!("Blocks: {:?}", blocks);
                blocks
                    .iter()
                    .any(|b| b.get_properties().instance == element_id)
            });

            if let Some(comp) = comp {
                comp.event(cmm, event);
            }
        });
    }

    fn update_components(&mut self, dt: f64) {
        self.get_layer_mut().iter_mut().for_each(|c| c.update(dt));
    }

    fn build_json(&mut self) -> std::io::Result<()> {
        let write = &mut self.out_writer.lock();
        write.write_all(&[b'['])?;
        self.get_layer()
            .iter()
            .fold(Vec::new(), |mut blocks, c| {
                c.collect_base_components(&mut blocks);
                blocks
            })
            .iter()
            .enumerate()
            .try_for_each(|(index, block)| {
                if index != 0 {
                    write.write_all(&[b','])?;
                }
                block.serialize_cache(write)
            })?;
        write.write_all(&[b']', b',', 10])
    }

    pub fn get_component_mut<'a, T: Component>(&'a mut self, name: &str) -> Option<&'a mut T> {
        self.get_layer_mut().iter_mut().find_map(|c| {
            if c.name() == Some(name) {
                let c: &mut dyn Any = c;
                c.downcast_mut::<T>()
            } else {
                None
            }
        })
    }

    fn get_layer(&self) -> &Vec<Box<dyn Component>> {
        self.layers.last().unwrap()
    }

    fn get_layer_mut(&mut self) -> &mut Vec<Box<dyn Component>> {
        self.layers.last_mut().unwrap()
    }

    fn get_layer_by_id(&self, layer: usize) -> &Vec<Box<dyn Component>> {
        &self.layers[layer]
    }

    fn get_layer_by_id_mut(&mut self, layer: usize) -> &mut Vec<Box<dyn Component>> {
        &mut self.layers[layer]
    }

    pub fn set_global_event_listener(&mut self, cb: fn(&mut dyn ManageComponents, &ClickEvent)) {
        self.global_event_listener = cb;
    }

    fn handle_messenges(&mut self) {
        self.component_manager_messenger
            .take_queue()
            .into_iter()
            .for_each(|message| match message {
                Message::AddComponent(component) => {
                    self.add_component(component);
                }
                Message::RemoveByName(component) => {
                    self.remove_by_name(component.as_str());
                }
                Message::NewLayer => {
                    self.new_layer();
                }
                Message::PopLayer => {
                    self.pop_layer();
                }
            });
    }
}

impl ManageComponents for ComponentManager {
    fn new_layer(&mut self) {
        self.layers.push(Vec::new())
    }

    fn pop_layer(&mut self) {
        if self.layers.len() > 1 {
            self.layers.pop();
        }
    }

    fn add_component(&mut self, mut comp: Box<dyn Component>) {
        let mut base_components = Vec::new();
        comp.collect_base_components_mut(&mut base_components);

        base_components.iter_mut().for_each(|component| {
            component.get_properties_mut().instance = Some(self.next_instance_id);
            self.next_instance_id += 1;
        });

        self.get_layer_mut().push(comp);
    }

    fn add_component_at(&mut self, mut comp: Box<dyn Component>, pos: isize) {
        let mut base_components = Vec::new();
        comp.collect_base_components_mut(&mut base_components);

        base_components.iter_mut().for_each(|component| {
            component.get_properties_mut().instance = Some(self.next_instance_id);
            self.next_instance_id += 1;
        });

        let pos = if pos < 0 {
            (self.get_layer().len() as isize + pos) as usize
        } else {
            pos as usize
        };

        self.get_layer_mut().splice(pos..pos, [comp]);
    }

    fn add_component_at_on_layer(
        &mut self,
        mut comp: Box<dyn Component>,
        pos: isize,
        layer: usize,
    ) {
        let mut base_components = Vec::new();
        comp.collect_base_components_mut(&mut base_components);

        base_components.iter_mut().for_each(|component| {
            component.get_properties_mut().instance = Some(self.next_instance_id);
            self.next_instance_id += 1;
        });

        let pos = if pos < 0 {
            (self.get_layer_by_id(layer).len() as isize + pos) as usize
        } else {
            pos as usize
        };

        self.get_layer_by_id_mut(layer).splice(pos..pos, [comp]);
    }

    fn remove_by_name(&mut self, name: &str) {
        self.layers.iter_mut().for_each(|layer| {
            let index = layer.iter().position(|c| c.name() == Some(name));
            if let Some(index) = index {
                layer.remove(index);
            }
        });
    }
}

fn read_events(
    reader: &mut dyn BufRead,
    event_number: &mut usize,
    tx: &std::sync::mpsc::Sender<ClickEvent>,
) {
    let mut event = String::new();

    if reader.read_line(&mut event).is_err() {
        error!("Could not read event from stdin");
        return;
    }

    trace!("Raw event data: {}", event);

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

    let click_event = match serde_json::from_str::<ClickEvent>(event) {
        Ok(ev) => ev,
        Err(e) => {
            error!("Invalid click event {}", e.to_string());
            return;
        }
    };
    if tx.send(click_event).is_err() {
        debug!("No event rx found");
    }
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
        let header_buffer = match serde_json::to_vec(&header) {
            Ok(hb) => hb,
            Err(_) => {
                debug!("Could not convert header to json {:#?}", header);
                panic!("Could not convert header to json {:#?}", header)
            }
        };
        if out_writer.write_all(&header_buffer).is_err() {
            debug!("Could not write header");
            panic!("Could not write header")
        }

        if out_writer.write_all(&[10, b'[']).is_err() {
            debug!("Could not write json start");
            panic!("Could not write json start")
        }

        let (tx, rx) = std::sync::mpsc::channel();

        std::thread::spawn(move || {
            let stdin = std::io::stdin();
            let mut last_event_id = 0;
            let tx = tx;

            loop {
                read_events(&mut stdin.lock(), &mut last_event_id, &tx);
            }
        });

        ComponentManager {
            layers: vec![Vec::new()],
            event_reader: rx,
            out_writer,
            last_update: SystemTime::now(),
            next_instance_id: 1,
            global_event_listener: default_listener,
            component_manager_messenger: Default::default(),
        }
    }
}

fn default_listener(_: &mut dyn ManageComponents, _: &ClickEvent) {}
