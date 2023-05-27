# test rkyv

Please place some `archive.zip` at the root of the repo to test it out (the bigger the `.zip` the better).
Then, if you have `Just` installed, run the alias:

```sh
just t
```

otherwise just run:

```sh
cargo t -- --test-threads 1 --nocapture
```
