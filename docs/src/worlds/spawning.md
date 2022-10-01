# Spawning

If worlds are a space for entities to act, then lets spawn some entities!

Let's start by spawning our player.

Our player will need a Transform (its location/stretch/rotation data).
Let's give it a visual component (ex. ColorRect) and a custom player
component as well.

```rust
use emerald::{World, Transform, ColorRect};

// Lets spawn our player, visualized by a rectangle!
struct MyPlayerData { pub is_jogging: bool }

let mut world = World::new();
world.spawn((
    Transform::default(), 
    ColorRect::default(), 
    MyPlayerData { is_jogging: false }
));
```

Later on, we'll query this player and mutate its data!