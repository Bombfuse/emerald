# Worlds

### What is a World?
A world is Emerald's equivalent to a scene tree.
It's a space in which game entities live and act.

A world does not contain a scene tree or hierarchy,
as a flat world architecture lends
itself very well to Rust's borrow checker.

A world (optionally) has physics that the games entities can interact with.