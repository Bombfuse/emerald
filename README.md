![Emerald](./banner_large.png)
[![Crates.io](https://img.shields.io/crates/v/emerald.svg)](https://crates.io/crates/emerald)
[![Build Status](https://travis-ci.com/Bombfuse/emerald.svg?branch=master)](https://travis-ci.com/Bombfuse/emerald)
[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](https://opensource.org/licenses/MIT)

# The Cross Platform Engine

Emerald is designed to be as lightweight as possible, while remaining a fully-featured and cross-platform game engine.

The Api is simple and powerful, giving you direct access to physics, audio, graphics, game worlds, and asset loading.

## Supported Platforms
<div>
    <img alt="OpenGL" src="./assets/opengl.svg" width=32>
    <img alt="MacOS" src="./assets/apple.svg" width=32>
    <img alt="Linux" src="./assets/linux.svg" width=32>
    <img alt="Windows" src="./assets/windows.svg" width=32>
    <img alt="RaspberryPi" src="./assets/raspberrypi.svg" width=32>
    <img alt="HTML5" src="./assets/webassembly.svg" width=32>
</div>


--- Work in progress ---
<div>
    <img alt="Android" src="./assets/android.svg" width=32>
</div>
--------------------------



## Asset Loading
```rust
let my_sprite = emd.loader()
    .sprite("./assets/my_sprite.png")
    .unwrap();

let my_audio = emd.loader()
    .sound("./assets/my_sound.wav")
    .unwrap();
```


## Physics

### Creating Bodies
```rust
    let entity = emd.world().spawn((Position::new(0.0, 0.0)));

    let body_handle = emd.world().physics().build_body(
        entity,
        RigidBodyDesc::dynamic()
    );

    emd.world().physics().build_collider(
        body_handle,
        ColliderDesc::cuboid(6.0, 6.0)
    );
```

Physics bodies are tied directly to entities, this is so that bodies can be cleaned up automatically when entities are despawned.

### Physics Stepping

```rust

    emd.world()
        .physics()
        .step();
```

You decide when physics steps!
This makes it very easy to "pause" the game without needing to alter any data.

## Graphics

The default method to draw the game is to draw all of the entities in the current world. However, you can write your own `draw` function if you need to do more!

```rust
fn draw(&mut self, mut emd: Emerald) {
    emd.graphics().begin();

    emd.graphics().draw_world();

    emd.graphics().render();
}
```

## ECS

Emerald uses the [Entity Component System](https://en.wikipedia.org/wiki/Entity_component_system) paradigm for creating, managing, and updating game entities.

Emerald uses [Hecs](https://github.com/Ralith/hecs) under the hood for extremely fast entity iteration, and a remarkably clean query Api.

More detailed features can be found in the [Hecs documentation](https://docs.rs/hecs/).

```rust
for (id, (sprite, mut position)) in emd.world().query::<(&Sprite, &mut Position)>().iter() {
    position.x += 10.0;
}
```

## [Aseprite](https://www.aseprite.org/)

Emerald has built in aseprite loading and rendering. Simply load in the texture and animation file, then tell it which animations to play.

```rust
let mut aseprite = emd.loader()
    .aseprite("./assets/my_texture.png", "./assets/my_animation.json").unwrap();

aseprite.play("some_aseprite_animation");

emd.world().inner()
    .push((aseprite, Position::zero()));
```

Export settings
![Preferred export settings](./assets/aseprite_settings.png)



## [WASM](https://webassembly.org/)

### Build

`cargo build --target wasm32-unknown-unknown`

### Asset Loading

In order to keep a clean, simple API, and avoid network requests for assets. Emerald takes the approach of packing all necessary assets into the WASM binary.

This same method can be used to pack all assets into the game binary regardless of which platform you target.

Use the `pack_bytes` function to load data into the engine.

```rust
fn initialize(&mut self, mut emd: Emerald) {
    /// Pack all game files into WASM binary with path references
    /// so that the regular file loading Api is supported.
    #[cfg(target_arch = "wasm32")]
    {
        emd.loader()
            .pack_bytes(
                "./assets/bunny.png",
                include_bytes!("../assets/bunny.png").to_vec()
            );
    }

    /// We can now load texture/sprites via the normal Api,
    /// regardless of which platform we're targeting.
    let sprite = emd.loader()
        .sprite("./assets/bunny.png").unwrap();
    
    let mut position = Position::new(0.0, 0.0);

    self.count = 1000;
    emd.world().inner().extend(
        (0..1000).map(|_| {
            position.x += 6.0;
            position.y += 1.0;
            let mut s = sprite.clone();
            (position.clone(), s, Vel { x: 5.0, y: 3.0 })
        })
    );
}
```

## Demos
* Links
* To
* Hosted
* WASM demos
* with source code