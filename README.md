<div align="center">

### Focus Passion

![ReplayManagerIcon](https://github.com/sxlphuric/replay-manager/blob/main/assets/icon_256.png?raw=true)

**A fast and simple online task manager**

*Built with rocket-rs for high performance*

![Version](https://img.shields.io/badge/version-0.1.0-yellow)
![Commit Activity](https://img.shields.io/github/commit-activity/m/sxlphuric/focus-passion?style=flat)
![CI](https://img.shields.io/github/actions/workflow/status/sxlphuric/focus-passion/rust.yml?label=CI&style=flat)
![Stars](https://img.shields.io/github/stars/sxlphuric/focus-passion?style=flat)

[Installation](#Deploying) • [Roadmap](#Roadmap) • [Structure](#Structure)

</div>

---

## Overview

Focus Passion is an online task manager that is open-source, easy to use and fast. It focuses on providing a reliable interface and a robust server.

### Key Features

- 🍃 **Zen** - Simple interface that does not overwhelm you
- 💠 **Simple** - Easy to use and understand
- :iphone: **Responsive** - Fast and simple HTML interface with Tera

## Screenshots

<div align="center">
</div>

---

## Dependencies

- [MongoDB](https://mongodb.com) 8.0
- [Rust](https://rustup.rs/) 1.94.1 or later
- [nanoid](https://crates.io/crates/nanoid) 0.4.0
- [rocket-rs](https://rocket.rs) 0.5.1
- [rocket_dyn_templates](https://crates.io/crates/rocket_dyn_templates) 0.2.0

## Deploying

The only current ways I know of how to deploy is Railway. You can also self-host.

#### Support
[![Linux](https://img.shields.io/badge/Linux-FCC624?logo=linux&logoColor=black&style=for-the-badge)](#) [![macOS](https://img.shields.io/badge/macOS-000000?logo=apple&logoColor=F0F0F0&style=for-the-badge)](#) [![Windows](https://custom-icon-badges.demolab.com/badge/Windows-0078D6?logo=windows11&logoColor=white&style=for-the-badge)](#)

### 1. Install dependencies

Install all the dependencies listed above.
These commands will also install `git` on your machine.

On **Windows**:

`winget install MongoDB.Server Git.Git`

*or*

`choco install mongodb git`

if you prefer [Chocolatey](https://chocolatey.org/install).


On **MacOS**:
> **Note:**
> I'm assuming you already have Homebrew installed. If not, please [install it](https://brew.sh).

```fish
brew tap mongodb/brew # Add mongodb tap

brew update # update package lists

brew install mongodb-community@8.0 # install mongodb
```

On **Linux**:

Use your package manager to install `mongodb` and `git`.

- On **Arch Linux and derivatives**: Use `yay` or your preferred AUR helper to install MongoDB (package `mongodb`)

- On **other distros**: Go look at the official [**documentation**](https://www.mongodb.com/docs/v8.0/administration/install-on-linux/) to install MongoDB.

### 2. Cloning the repository

Clone the repository to your local machine. This can be done with

```fish
# Clone the repository
git clone https://github.com/sxlphuric/focus-passion.git

# Go into the repository's folder
cd focus-passion
```

### 3. Environment

Create a `.env` file by copying `.env.example`.

Then, link or copy the `.env` file to `/mongodb`.

### 4. Running

Run the MongoDB database with `docker-compose`:

`docker-compose up -f mongodb/compose.yml -d`

Run the package with `cargo`:

`cargo run --release`

## Structure
```
/
.github/workflows
|_ rust.yml - github workflow for Cargo
examples
|_ ... - rocket-rs examples
mongodb
|_ compose.yml - compose file to start mongodb
src
|_ app.rs - the rocket app
templates - html templates
|_ index.html.tera - main index page
static
|_ PatrickHand-Regular.ttf - font
|_ style.css - CSS styling
|_ uuid.js - js compiled
|_ uuid.ts - uuid generator
|_ script.js - empty js script
Plan.txt - Project plan
Procfile - Railway settings
Rocket.toml - Rocket config
codespace_install_rust.sh - Script to quickly install rust in a codespace
rust-toolchain.toml - File describing rust toolchain
tsconfig.json - typescript config
README.md - this file
LICENSE - license (gplv3)
Cargo.lock - idk
Cargo.toml - Cargo dependencies
```

## Roadmap
- [x] Set up a MongoDB database
- [ ] Make a good looking frontend
- [ ] Fully functional frontend
- [ ] Finish the Rocket API
  - [x] Adding habits
  - [ ] Removing habits
  - [ ] Querying habits
  - [ ] Querying habits with filter
  - [ ] Modifying habits that are queried via task ID
- [x] Deploy online with Railway
### TODO
- Rocket API
- HTML frontend
