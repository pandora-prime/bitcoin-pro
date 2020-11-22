use glade::View;
use gtk::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

static UI: &'static str = include_str!("../../ui/main.glade");

#[derive(Debug, Display, Error, From)]
#[display(doc_comments)]
pub enum Error {
    /// Glade error: {0}
    #[from]
    GladeError(glade::Error),
}

pub struct AppWindow {
    window: gtk::ApplicationWindow,
}

impl View for AppWindow {
    fn load_glade() -> Result<Rc<RefCell<Self>>, glade::Error> {
        let builder = gtk::Builder::from_string(UI);

        Ok(Rc::new(RefCell::new(Self {
            window: glade_load!(builder, "appWindow")?,
        })))
    }
}

impl AppWindow {
    pub fn new() -> Result<Rc<RefCell<Self>>, Error> {
        let me = Self::load_glade()?;
        Ok(me)
    }

    pub fn run(&mut self) {
        self.update();
        self.window.show_all();
        gtk::main();
    }

    pub fn update(&mut self) {}
}
