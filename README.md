# extrude-licenses

This is **a extruder(≈ a formatter with a user template)** for licenses of a Rust and Node.js project.

- Cargo.toml|package.json + template.xxx -> your-project-licenses.xxx
- This is a command-line tool; `extrude-licenses`
- Additional requirements:
    - [`cargo-license`][] command if you want generate from `Cargo.toml` source.
    - [`license-checker`][] command if you want generate from `package.json` source.

[`cargo-license`]:https://github.com/onur/cargo-license
[`license-checker`]:https://github.com/davglass/license-checker

## Usage

1. Install, `cargo install extrude-licenses`.
2. Generate licenses JSON file, a source licenses JSON file, Generate license list JSON file with [`cargo-license`][] or [`license-checker`][] -> `licenses.json`.
   - Eg. `cargo install cargo-license` -> `cargo-license -j > licenses.json`
   - Eg. `npx license-checker --json --relativeLicensePath --summay -out licenses.json`
3. Make a template file for extruding.
   - Eg. [examples/template-1.csv](examples/template-1.csv)
   - Eg. [examples/template-2.md](examples/template-2.md)
   - Eg. [examples/template-3.html](examples/template-3.html)
4. Run, `extrude-licenses -t template.xxx -i licenses.json` and the other options.
   - `-t`, `--template-file`: A template file. (must)
   - `-i`, `--input-file`: An input source JSON file. (must)

### Command-line arguments

+ Required, need a value parameter
    + `-t`, `--template-file`: A template file.
    + `-i`, `--input-file`: An input source JSON file.
+ Optional, need a value parameter
    + `-o`, `--output-file`: Output to the specific file. (The default output is STDOUT.)
    + `-h`, `--header-lines`: A number of the header lines in the template file. (The default is 0.)
    + `-f`, `--footer-lines`: A number of the footer lines in the template file. (The default is 0.)
    + `--match-name`: A regex pattern fintering for a name. (The default will match to any patterns.)
    + `--match-license`: A regex pattern fintering for a license. (The default will match to any patterns.)
+ Optional, flag only
    + `--escape-authors`: if set => `It's Me <its_me@example.com>` -> `It's Me &ltemail@example.com&gt`. (The default is not set.)
    + `--match-name-invert`: If set => Invert a `--match-name` result. (The default is not set.)
    + `--match-licsense-invert`: If set => Invert a `--match-license` result. (The default is not set.)

Note: `--match-name-invert` and `--match-license-invert` are workround of `(?!not-included-pattern)`. Because, regex is not supported it in Rust for now.

### Example results

Note: To use `cargo run -- -t ... -i ...` if you want to try to run in this project repos instead of installing.

1. A simple CSV like template (A body only): `extrude-licenses -t examples/template-1.csv -i examples/result-of-license-checker`

```csv
anymatch,1.3.2,ISC,Elan Shanker
arr-diff,2.0.0,MIT,Jon Schlinkert
arr-diff,4.0.0,MIT,Jon Schlinkert
arr-flatten,1.1.0,MIT,Jon Schlinkert
arr-union,3.1.0,MIT,Jon Schlinkert
```

2. Name filtering: `extrude-licenses -t examples/template-1.csv -i examples/result-of-license-checker --match-name diff`

```csv
arr-diff,4.0.0,MIT,Jon Schlinkert
arr-diff,2.0.0,MIT,Jon Schlinkert
```

1. License filtering: `extrude-licenses -t examples/template-1.csv -i examples/result-of-cargo-license.json --match-license BSL`

```csv
ryu,1.0.5,Apache-2.0 OR BSL-1.0,David Tolnay <dtolnay@gmail.com>
```

4. A markdown table template (Header=2-line2 + Body): `extrude-licenses -t examples/template-2.md -h 2 -i examples/result-of-cargo-license.json --match-name serde`

```md
| Package      | Version | License           | Authors                                     |
| ------------ | ------- | ----------------- | ------------------------------------------- |
| serde        | 1.0.116 | Apache-2.0 OR MIT | Erick Tryzelaar <erick.tryzelaar@gmail.com> | David Tolnay <dtolnay@gmail.com> |
| serde_derive | 1.0.116 | Apache-2.0 OR MIT | Erick Tryzelaar <erick.tryzelaar@gmail.com> | David Tolnay <dtolnay@gmail.com> |
| serde_json   | 1.0.58  | Apache-2.0 OR MIT | Erick Tryzelaar <erick.tryzelaar@gmail.com> | David Tolnay <dtolnay@gmail.com> |
```

1. A html table template (Header=7lines + Body + Footer=2lines): `extrude-licenses -t examples/template-3.html -h 7 -f 2 -i examples/result-of-cargo-license.json --match-name serde`

```html
<table>
  <tbody>
    <tr>
      <th>Package</th>
      <th>License</th>
      <th>Authors</th>
    </tr>
    <tr>
      <a href="https://crates.io/crates/serde"><td>serde 1.0.116</td></a>
      <td>Apache-2.0 OR MIT</td>
      <td>Erick Tryzelaar <erick.tryzelaar@gmail.com>|David Tolnay <dtolnay@gmail.com></td>
    </tr>
    <tr>
      <a href="https://crates.io/crates/serde_derive"><td>serde_derive 1.0.116</td></a>
      <td>Apache-2.0 OR MIT</td>
      <td>Erick Tryzelaar <erick.tryzelaar@gmail.com>|David Tolnay <dtolnay@gmail.com></td>
    </tr>
    <tr>
      <a href="https://crates.io/crates/serde_json"><td>serde_json 1.0.58</td></a>
      <td>Apache-2.0 OR MIT</td>
      <td>Erick Tryzelaar <erick.tryzelaar@gmail.com>|David Tolnay <dtolnay@gmail.com></td>
    </tr>
  </tbody>
</table>
```

### And more...

For more example, .tsx (React/typescript) template:

```typescript
import React, { Component } from "react";
export default class LicensesNode extends Component { render = () => (<ul>
<a href="https://crates.io/crates/{name}"><li>{name} {version} ({license}) -- {authors}</li></a>
</ul>)};
```

and you can use any text patterns for a template file. The supported template patterns are:

+ `{name}`
+ `{version}`
+ `{authors}`
+ `{repository}`
+ `{license}`
+ `{license_file}`
+ `{description}`

## License

- [MIT](LICENSE)

## Author

- USAGI.NETWORK / Usagi Ito <https://usagi.network/>
