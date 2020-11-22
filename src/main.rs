#[macro_use]
extern crate amplify_derive;
#[macro_use]
extern crate glade;

mod view;

fn main() -> Result<(), view::AppError> {
    gtk::init().expect("GTK initialization error");

    let app = view::AppWindow::new()?;
    let mut app = app.borrow_mut();
    app.update();
    app.run();

    Ok(())
}
