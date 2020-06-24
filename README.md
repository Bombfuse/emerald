![Emerald](./banner_large.png)

[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](https://opensource.org/licenses/MIT)
[![Crates.io](https://img.shields.io/crates/v/emerald.svg)](https://crates.io/crates/emerald)
[![Build Status](https://travis-ci.com/Bombfuse/emerald.svg?branch=master)](https://travis-ci.com/Bombfuse/emerald)

## IN DEVELOPMENT
[x] Sprites

[] Aseprite

[] Logging

[] Tilemaps

[] Fonts

[] Audio

[] WASM/Android


## Lite

A fully featured 2D engine with minimal dependencies.

## Simple, Powerful API

A simple API giving you direct access to physics, audio, and graphics.

Given a handle to the emerald engine, you directly control physics, audio, and the game worlds.

### Asset Loading
```
let my_sprite = emd.loader()
    .sprite("./my_assets/my_sprite.png").unwrap();

let my_font = emd.loader()
    .font("./my_assets/my_font.ttf").unwrap();
```

### Physics

```
    // You decide when physics steps!
    // This makes it very easy to "pause" the game without altering your physics data.

    emd.world().physics().step();
```

### ECS

Emerald uses the [Entity Component System](https://en.wikipedia.org/wiki/Entity_component_system) paradigm for creating, managing, and updating game entities.

Emerald uses [Legion](https://github.com/TomGillen/legion) under the hood for extremely fast entity iteration, and a remarkably clean query API.

```
query example here
```

## [Aseprite](https://www.aseprite.org/)

Emerald has built in aseprite loading and rendering. Simply load in the texture and animation file, then tell it which animations to play.

```
let mut aseprite = emd.loader()
    .aseprite("./assets/my_texture.png", "./assets/my_animation.json").unwrap();

aseprite.play("some_aseprite_animation");

emd.world()
    .insert((), Some((aseprite, Position::zero())));
```


## Portable

Built on top of [miniquad](https://github.com/not-fl3/miniquad) and other cross platform libraries, Emerald is able to run almost anywhere.

* Windows
* MacOS
* Linux
* Android
* Web via WASM


## Demos
* Links
* To
* Hosted
* WASM demos
* with source code