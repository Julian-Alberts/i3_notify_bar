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

pub struct ComponentManager {
    layers: Vec<Vec<Box<dyn Component>>>,
    event_reader: Receiver<ClickEvent>,
    out_writer: Stdout,
    last_update: SystemTime,
    next_instance_id: u128,
    global_event_listener: &'static dyn Fn(&mut Self, &ClickEvent)
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
        self.update_components(dt);
        let blocks = self.build_json();

        debug!("{:#?}", String::from_utf8(blocks.clone()));

        if let Err(_) = self.out_writer.write_all(&blocks) {
            error!("Could not write bytes: {:#?}", blocks);
            return;
        }
        if let Err(_) = self.out_writer.flush() {
            error!("Error while flushing buffer");
        }
    }

    fn handle_events(&mut self) {
        let events = self.event_reader.try_iter().collect::<Vec<ClickEvent>>();

        events.iter().for_each(|event| {
            (self.global_event_listener)(self, event);
            let element_id = event.get_id();
            let comp = self.get_layer_mut().iter_mut().find(|comp| {
                let mut blocks = Vec::new();
                comp.collect_base_components(&mut blocks);
                blocks.iter().any(|b| b.get_id() == element_id)
            });

            if let Some(comp) = comp {
                comp.event(event);
            }
        });
    }

    fn update_components(&mut self, dt: f64) {
        self.get_layer_mut().iter_mut().for_each(|c| c.update(dt));
    }

    fn build_json(&mut self) -> Vec<u8> {
        let mut blocks = self
            .get_layer_mut()
            .iter_mut()
            .fold(Vec::new(), |mut blocks, c| {
                c.collect_base_components_mut(&mut blocks);
                blocks
            })
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
        blocks
    }

    pub fn add_component(&mut self, mut comp: Box<dyn Component>) {
        let mut base_components = Vec::new();
        comp.collect_base_components_mut(&mut base_components);

        base_components.iter_mut().for_each(|component| {
            component.get_properties_mut().instance = Some(self.next_instance_id.to_string());
            self.next_instance_id += 1;
        });

        self.get_layer_mut().push(comp);
    }

    pub fn get_component_mut<'a, T: Component>(&'a mut self, name: &str) -> Option<&'a mut T> {
        self.get_layer_mut().iter_mut().find_map(|c| {
            if c.name() == name {
                let c: &mut dyn Any = c;
                c.downcast_mut::<T>()
            } else {
                None
            }
        })
    }

    pub fn remove_by_name(&mut self, name: &str) {
        let index = match self.get_layer().iter().position(|c| c.name() == name) {
            Some(s) => s,
            None => return,
        };

        self.get_layer_mut().remove(index);
    }

    fn get_layer(&self) -> &Vec<Box<dyn Component>> {
        self.layers.last().unwrap()
    }

    fn get_layer_mut(&mut self) -> &mut Vec<Box<dyn Component>> {
        self.layers.last_mut().unwrap()
    }

    pub fn new_layer(&mut self) {
        self.layers.push(Vec::new())
    }

    pub fn pop_layer(&mut self) {
        if self.layers.len() > 1 {
            self.layers.pop();
        }
    }

    pub fn set_global_event_listener(&mut self, cb: &'static dyn Fn(&mut Self, &ClickEvent)) {
        self.global_event_listener = cb;
    }
}

fn read_events(
    reader: &mut dyn BufRead,
    event_number: &mut usize,
    tx: &std::sync::mpsc::Sender<ClickEvent>,
) {
    let mut event = String::new();

    if let Err(_) = reader.read_line(&mut event) {
        error!("Could not read event from stdin");
        return;
    }

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
    if let Err(_) = tx.send(click_event) {
        debug!("No event rx found");
        return;
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
        if let Err(_) = out_writer.write_all(&header_buffer) {
            debug!("Could not write header");
            panic!("Could not write header")
        }

        if let Err(_) = out_writer.write_all(&[10, b'[']) {
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
            global_event_listener: &default_listener
        }
    }
}

fn default_listener(_: &mut ComponentManager, _: &ClickEvent) {}