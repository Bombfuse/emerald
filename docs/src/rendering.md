
## Rendering
Simple world rendering, with the option of more custom
rendering via the graphics api.
```rust
emd.graphics().begin();
emd.graphics().draw_world(&mut self.world);
emd.graphics().render();
```

It's also possible to draw visual components directly in relation to the
screen.

```rust
let sprite = emd.loader().sprite("my_sprite.png").unwrap();
emd.graphics().draw_sprite(&sprite, &Transform::default()).unwrap();
```