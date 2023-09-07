# Rusted Battalions

2D strategy game similar to Advance Wars, but running directly in the browser!

Written entirely in Rust, using a custom game engine and [wgpu](https://crates.io/crates/wgpu) for rendering.


## Installation

Because of copyright, this repository does not contain the spritesheet images which are needed to run the game.

You will need to (legally) acquire the spritesheet images and then place them into the `dist/sprites` folder.

You will also need to [install Rust](https://rustup.rs/), and you will need to [install yarn](https://yarnpkg.com/).

If you are on Windows you will need to install the [Visual Studio build tools](https://visualstudio.microsoft.com/thank-you-downloading-visual-studio/?sku=BuildTools&rel=16) (make sure to enable the "C++ build tools" option).

Then run `yarn install` to install the dependencies.


## Building

Run `yarn build` to build a production-ready optimized binary (which will be placed inside of `dist/js`).

Run `yarn watch` to run in development mode, it will automatically re-compile and reload the page when you edit any files.
