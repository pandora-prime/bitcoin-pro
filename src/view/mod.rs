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

mod app_window;
mod descriptor_dlg;
mod issue_dlg;
mod open_dlg;
mod pubkey_dlg;
mod save_dlg;

pub use app_window::{AppWindow, Error as AppError};
pub use descriptor_dlg::DescriptorDlg;
pub use issue_dlg::IssueDlg;
pub use open_dlg::OpenDlg;
pub use pubkey_dlg::PubkeyDlg;
pub use save_dlg::SaveDlg;
