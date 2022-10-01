# Input

There are a few different ways to get input in Emerald.

## Keyboard
Access key states directly
```rust
if emd.input().is_key_just_pressed(KeyCode::Space) {
    // player jump
}
```

## Buttons
Access the buttons on a gamepad directly.
```rust
if emd.input().is_button_just_pressed(Button::South) {
    // player jump
}
```

## Joysticks
Access the joysticks of a gamepad directly.
```rust
let joystick_value = emd.input().joystick(Joystick::Left);
// Now we can use this value to move the player in the joysticks direction
```

## Actions
Access the states of action bindings.
```rust
emd.input().add_action_binding_button("jump".to_string(), Button::South);
emd.input().add_action_binding_key("jump".to_string(), KeyCode::Space);
if emd.input().is_action_just_pressed("jump".to_string()) {
    // player jump
}
```
