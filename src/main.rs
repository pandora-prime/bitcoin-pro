// Bitcoin Pro: Professional bitcoin accounts & assets management
// Written in 2020 by
//     Dr. Maxim Orlovsky <orlovsky@pandoracore.com>
//
// To the extent possible under law, the author(s) have dedicated all
// copyright and related and neighboring rights to this software to
// the public domain worldwide. This software is distributed without
// any warranty.
//
// You should have received a copy of the AGPL License
// along with this software.
// If not, see <https://www.gnu.org/licenses/agpl-3.0-standalone.html>.

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
