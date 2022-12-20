

## Specs

### Fetching an Entity that doesn't exist will crash

This will compile fine, but fetching an entity that doesn't exist will crash,
and you'll likely need to look at the backtrace to figure out what happened.

### Multiple fetches

Multiple fetches are unsafe:


```rust
let map_im = ecs.fetch::<Map>();
let map = &mut ecs.fetch_mut::<Map>();
```


and can cause errors like:

```
thread 'main' panicked at 'Tried to fetch data of type "alloc::boxed::Box<dyn shred::world::Resource>", but it was already borrowed.', /home/bbarker/.cargo/registry/src/github.com-1ecc6299db9ec823/shred-0.13.0/src/cell.rs:299:33
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

Workaround: use scopes to destruct the first `Fetch`:

```rust
let ix = {
    let map = ecs.fetch::<Map>();
    map.pos_idx(pos.from())
};
let map = &mut ecs.fetch_mut::<Map>();
```


### Fetching a resource mutably and immutably simultaneously

Similar to the above; unlike the rust borrow checker, specs can't do this check at compile time.
So it does it at runtime.