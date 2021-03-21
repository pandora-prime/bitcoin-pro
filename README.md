# Bitcoin Pro

Professional bitcoin accounts & smart contract management by 
[Pandora Core AG](https://pandoracore.com), Switzerland & Dr. Maxim Orlovsky.

Application is founded on [LNP/BP Core Library](https://github.com/LNP-BP/rust-lnpbp)
and allows [RGB smart contracts](https://rgb-org.github.io) management.

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
* Creation and management of RGB fungible assets (RGB-20 standard)
  - Asset issuance
  - Secondary issuance [WIP]
  - Renomination [WIP]
  - Burn & replacement [WIP]
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
$ rustup default nightly
$ cargo install bitcoin-pro
$ bitcoin-pro
```

### Installation on Debian

If you try to build bitcoin-pro on current Debian Stable using instructions above you might encounter the following errors:  
`Requested 'gtk+-3.0 >= 3.24.9' but version of GTK+ is 3.24.5`  
or  
`Requested 'glib-2.0 >= 2.64' but version of GLib is 2.58.3`

This is because bitcoin-pro requirement for those packages version is superior to the version currently shipped with Debian Stable. One way to solve this is to modify the `/etc/apt/sources.list` file by appending the following lines:
```
# Testing repository - main, contrib and non-free branches
deb http://deb.debian.org/debian testing main non-free contrib
```

To prevent unwanted package upgrades to testing version, you must modify `/etc/apt/preferences` or `/etc/apt/preferences.d/preferences` (or create one if none of those files exist):
```
Package: *
Pin: release a=stable
Pin-Priority: 700

Package: *
Pin: release a=testing
Pin-Priority: 650
```

Then you can install testing version of the packages:  
`sudo apt install -t testing glib-2.0 gtk+-3.0`

...and resume the installation process.

Be warned that adding unstable packages to your system can, well, make your system less stable, so be careful with that. 

## Using

### Main interface

![Main window](https://github.com/pandoracore/bitcoin-pro/raw/v0.1.0-beta.1/doc/ui/main_app.png)

### Extended public key management

![Extended public key management](https://github.com/pandoracore/bitcoin-pro/raw/v0.1.0-beta.1/doc/ui/xpub_dlg.png)

### Output descriptors

![Output descriptors](https://github.com/pandoracore/bitcoin-pro/raw/v0.1.0-beta.1/doc/ui/descriptors.png)

### RGB-20 assets

![Asset creation](https://github.com/pandoracore/bitcoin-pro/raw/v0.1.0-beta.1/doc/ui/asset_creation.png)
