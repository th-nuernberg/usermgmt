# Issues

All kinds of bug reports, feature requests or suggestion to improve code are written as issues
on this Github repository.

## TODOs in code base

In this code base there are lines with the prefix "TODO:".
These lines are notes about places which should be improved.

Via command on linux systems

```sh
grep -nr "TODO:" src/*
```

you can see all the places.

# How to Test

Before submitting run

```sh
cargo test
```

## Snapshot testing

Some tests use a crate called [Insta](https://insta.rs/docs/cli/) to perform
snapshot testing. This makes auditing and writing test with complex expected output 
a lot easier.

If you write an unit test which uses this tool or you change the code in a way which changes the snapshot
then you should use the cargo insta plugin to review changed/created snapshots.

You can install this cargo plugin via

```sh
cargo install cargo-insta
```

