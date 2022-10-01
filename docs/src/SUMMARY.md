# Introduction

Welcome to the Emerald Engine! Emerald is a free, open-source, 2D game engine that focuses on portability and usability.

## World
### What is a World?
A world is Emerald's equivalent to a scene tree.
It's a space in which game entities live and act.

A world does not contain a scene tree or hierarchy,
a flat world architecture lends itself very well to Rust's borrow checker.

A world (optionally) has physics that the games entities can interact with.
```rust
// Lets spawn our player, visualized by a rectangle!
struct MyPlayerData { pub is_jogging: bool }

let mut world = World::new();
world.spawn((Transform::default(), ColorRect::default(), MyPlayerData { is_jogging: false }));
```

## Queries
### What is a query?
A query is something that operates on the entities of the World.
```rust
// A query that moves our player to the right if they're in jogging state
for (entity_id, (transform, aseprite, player_data)) in world.query::<(&mut Transform, &mut Aseprite, &mut MyPlayerData)>().iter() {
    if player.is_jogging {
        transform.translation.x += 10.0;
    }
}
```

## Aseprite
[Aseprite](https://www.aseprite.org/) is an open-source pixel art animation tool.
Many people use Aseprite and providing easy access to
rendering aseprites in the engine increases usability.

## Rendering
Simple world rendering, with the option of more custom
rendering via the graphics api.
```rust
emd.graphics().begin();
emd.graphics().draw_world(&mut self.world);
emd.graphics().render();
```

## Portability Table
| Platform      | Graphics | Audio     | Gamepad Input     | User Data Writing (save files)     |
| :---        |    :----:   |     :----:   |          ---: |          ---: |
| ![Windows](../../assets/windows.svg)     | &#x2611;       | &#x2611;    | &#x2611;    | &#x2611;    |
| ![Linux](../../assets/linux.svg)     | &#x2611;       | &#x2611;    | &#x2611;    | &#x2611;    |
| ![MacOS](../../assets/apple.svg)     | &#x2611;       | &#x2611;    | &#x2611;    |&#x2611;    |
| ![Raspberry Pi (Debian)](../../assets/raspberrypi.svg)     | &#x2611;       | &#x2611;    | &#x2611;    | &#x2611;    |
| ![Web Browser](../../assets/webassembly.svg)     | &#x2611;       | &#x2611;    | &#x2612;    | &#x2612;    |
| ![Android](../../assets/android.svg)     | &#x2611;       | &#x2611;    | &#x2612;    | &#x2612;    |
| iOS     | &#x2611;       | &#x2611;    | &#x2612;    | &#x2612;    |

