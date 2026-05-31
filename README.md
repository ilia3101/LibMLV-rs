# LibMLV-rs
libMLV in rust

Slightly lost about best way to design the api in a way that isn't 100% dependent on std... really severely overengineered a bunch of stuff ac ouple years ago when I initially wrote this code, now I'm fixing it.

publishing to crates.io (note for myself)
```
cargo publish --allow-dirty
```
Use allow dirty to ignore cargo.lock, I do not see a reason to include it yet, as the actual library has no dependencies, only the less critical examples do.
