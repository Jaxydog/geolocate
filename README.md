# Geolocate

A CLI tool, library, and data generator for IP-to-country conversion.

## Usage

This application can be run by simply executing the binary directly.

```sh
./geolocate-cli --help
# -- or with Cargo --
cargo run --release --bin geolocate-cli -- --help
```

In order to download a local copy of the ISO-3166 database, you can first run the `geolocate-data` command.

```sh
./geolocate-data ./data/countries.json
# -- or with Cargo --
cargo run --release --bin geolocate-data ./data/countries.json
```

## License

Geolocate is free software: you can redistribute it and/or modify it under the terms of the GNU Affero General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

Geolocate is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU Affero General Public License for more details.

You should have received a copy of the GNU Affero General Public License along with Geolocate (located within [LICENSE](./LICENSE)). If not, see <https://www.gnu.org/licenses/>.
