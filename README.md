# scp_containment_unit

![CI](https://github.com/Celeo/scp_containment_unit/workflows/CI/badge.svg?branch=master)

A Discord bot.

## Design

- Guild admins should be able to declare a "breach" for a user
- That user is granted a configured role
- All of the user's roles with some configured prefix are removed
- An admin can revoke the "breach" which resets the actions taken prior
  - The added role is removed, and the previously-removed roles are restored

## Installing

1. Clone the repo and build the binary

## Using

1. Copy 'scp_config.json.example' to 'scp_config.json' and populate
1. Run the bot (does not take any arguments)

## Developing

### Building

### Requirements

- Git
- A recent version of [Rust](https://www.rust-lang.org/tools/install)

### Steps

```sh
git clone https://github.com/Celeo/scp_containment_unit
cd scp_containment_unit
cargo build
```

## License

Licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

## Contributing

Note: preceding my normal "contributions welcome" message is a disclaimer that this bot is specifically made for a Discord guild I'm in. If you want to contribute a bit, you're welcome to, but the intent of this bot won't really grow.

Please feel free to contribute. Please open an issue first (or comment on an existing one) so that I know that you want to add/change something.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license,
shall be dual licensed as above, without any additional terms or conditions.
