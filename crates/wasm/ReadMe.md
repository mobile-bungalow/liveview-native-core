### Live View Native Wasm

In order to maintain protocol parity with vanilla live view the core library can compile to a node wasm library. This library is monkey patched into the vanilla live view
allowing for parity tests between the two. This requires several capital F Funky modifications to our source code which never make it into native production, including
modifying tokio and mio with shims that allow for targets, a feat which can only be justified for this kind of parity check.

## usage

the `build.sh` targets should generate npm packages which can be used to override the libraries in (phoenix live view)[https://github.com/phoenixframework/phoenix_live_view].
`test.sh` will pull the most recent release of the `phoenix_live_view` repo monkeypatch our mocked dependency in to both the unit and e2e tests.

```bash
.scripts/build.sh nodejs
.scripts/build.sh web
.scripts/test.sh
```