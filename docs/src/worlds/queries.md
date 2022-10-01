# Queries

### What is a query?
A query is something that operates on the entities of the World.
Entites are just bundles of data, and we use queries to access that data.


Let's write a query that moves the player!

```rust
let is_jog_pressed = emd.input().is_key_pressed(KeyCode::D);

// A query that moves our player to the right if they're in jogging state
for (entity_id, (transform, aseprite, player_data)) in world.query::<(&mut Transform, &mut Aseprite, &mut MyPlayerData)>().iter() {
    player.is_jogging = is_jog_pressed;

    if player.is_jogging {
        transform.translation.x += 10.0;
    }
}
```

