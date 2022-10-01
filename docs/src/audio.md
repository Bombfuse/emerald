# Audio
Emerald supports .ogg and .wav audio files.


## Mixers
Mixers are basically sound containers, we'll play sounds using these.

```rust
let sound = emd.loader().sound("my_sound.ogg").unwrap();
emd.audio().mixer("sfx").unwrap().play(sound).unwrap();
```