# Bitcoin Pro

Professional bitcoin accounts & smart contract management by 
[Pandora Cora AG](https://pandoracore.com), Switzerland & Dr. Maxim Orlovsky.

Application is founded on [LNP/BP Core Library](https://github.com/LNP-BP/rust-lnpbp)
and allows [RGB smart contracts](https://rgb-org.github.com) management.

The application **is not a bitcoin wallet**: it does not require access to 
private keys or creates any signatures. All operations are saved & exported in 
form of PSBTs (partially-signed bitcoin transactions) and for their completion
must be signed and published to bitcoin network outside of the application.

Bitcoin Pro is written exclusively in Rust language with GTK framework and 
natively compiles/works on Linux, MacOS (GTK is not supported on Big Sur yet; 
but previous versions should work fine) and (probably) Windows 10.

NB: This is an ultra-early alpha version; use it at your own risk!

## Features

* Extended public key management with advanced convertor and derivation 
  functionality
* Creation of arbitrary complex descriptors for UTXOs
* View on bitcoin UTXOs, transactions [Partially implemented]
* Creation and management of RGB fungible assets (RGB-20 standard) [WIP]
  - Secondary issuance
  - Renomination
  - Burn & replacement
* Creation and management of RGB collectibles/non-fungible token contracts 
  (RGB-21 standard) [Planned]
* Identity management with RGB-22 schema [Planned]
* Audit logs with RGB-23 schema [Planned]
* Monitoring new bitcoin transactions under certain descriptors [Planned]
* Bitcoin transaction and blockchain explorer [Planned]
* PSBT composer/editor [Planned]
* Custom RGB schema and state transition editor [Planned]

## Installation

Install rust language and run

```constole
$ sudo apt update
$ sudo apt install -y cargo libssl-dev libzmq3-dev pkg-config g++ cmake libgtk-3-dev libsqlite3-dev
$ cargo install bitcoin-pro --version 0.1.0-beta.1
$ bitcoin-pro
```

## Using

### Main interface

![Main window](https://github.com/pandoracore/bitcoin-pro/raw/v0.1.0-beta.1/doc/ui/main_app.png)

### Extended public key management

![Extended public key management](https://github.com/pandoracore/bitcoin-pro/raw/v0.1.0-beta.1/doc/ui/xpub_dlg.png)

### Output descriptors

![Output descriptors](https://github.com/pandoracore/bitcoin-pro/raw/v0.1.0-beta.1/doc/ui/descriptors.png)

### RGB-20 assets

![Asset creation](https://github.com/pandoracore/bitcoin-pro/raw/v0.1.0-beta.1/doc/ui/asset_creation.png)

## License

The application is dually-licensed under AGPL v0.3 for non-commercial use and
under commercial license with enterprise support by Pandora Core for commercial
usage.

For non-commercial use this program is free software: you can redistribute it 
and/or modify it under the terms of the GNU Affero General Public License as 
published by the Free Software Foundation, version 3.

This program is distributed in the hope that it will be useful, but WITHOUT ANY 
WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A 
PARTICULAR PURPOSE. See the GNU Affero General Public License for more details.

You should have received a copy of the GNU Affero General Public License along 
with this program. If not, see <https://www.gnu.org/licenses/>
