# test rkyv

## why ?

I was curious to see if [rkyv](https://crates.io/crates/rkyv/0.7.42) can produce sort of a zip annotated with metadata.

![image](https://github.com/Roms1383/test-rkyv/assets/21016014/053f1c96-8066-4cbe-8bc6-ac8390378f14)

## going further

Currently the metadata and internal `.zip` are serialized / deserialized manually, allowing:
- ✅ zero-copy deserialization
- ✅ preserve internal zip

Experiments with current `rkyv 0.7.42` helpers:

- ❌ if `Vec<u8>` is defined in struct, then it's somehow loaded, slowing everything down (even when removing `validation`).
- ❌ using `#[With(Raw)]` allows for expected efficient loading, but it serialize/deserialize the internal `.zip` as a pointer (pointing to nothing).

A potential idiomatic solution would be to implement a custom `#[With(Blob)]` to handle this specific use-case.

## usage

Please place some `archive.zip` at the root of the repo to test it out (the bigger the `.zip` the better).
Then, if you have `Just` installed, run the alias:

```sh
just t
```

otherwise just run:

```sh
cargo t -- --test-threads 1 --nocapture
```
