<div align="center">

# Starship

**A turn-based space shooter for PrimitiveOS.**

Read the next volley, move between three lanes, and destroy the flagship.

![Starship showing incoming fire in the left lane](docs/images/starship.png)

</div>

## Play

Each action takes one turn. Use **Move left** or **Move right** to change lanes. **Fire** hits the target in your current lane. A lane marked **Incoming fire** is struck after your action. Use **Shield** when you need to stay in that lane. You have two shields per wave and three hull points for the mission.

Clear the patrol, crossfire, and vanguard waves, then defeat the flagship. Targets take one or more hits, shown by the marks under each ship. The final hit clears the wave before its volley lands. Missions and best scores save on the Kindle, and the game plays offline.

![Starship after blocking a volley in dark mode](docs/images/starship-playing.png)

## Get it

Open **Store** in Primitive Launcher, refresh the catalog, and search for **Starship**. The GitHub release also includes a verified ARMHF package. [Installation guide](INSTALL.md)

## Build and preview

Place the PrimitiveOS Tools and Launcher checkouts beside this repository as `doannt-kindle-tools` and `doannt-kindle-launcher`. The desktop preview uses the Launcher fonts.

```sh
cargo test --locked --workspace
cargo run --locked -p starship-desktop -- build/previews
cargo zigbuild --locked --release -p starship-kindle --target armv7-unknown-linux-musleabihf
sh scripts/stage.sh armhf
```

## License

Starship code is [PolyForm Noncommercial 1.0.0](LICENSE). Documentation, screenshots, and original artwork are [CC BY-NC-SA 4.0](LICENSE-DOCS). The release ZIP includes both license files.
