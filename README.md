# yei CLI

The command-line tool for adding [yei](https://github.com/aodihis/yei) components to your Yew project.

## Installation

```sh
cargo install yei
```

## Commands

### `yei init`

Sets up yei in the current project. Run this once before adding components.

```sh
yei init
```

Prompts for:
- **Component output directory** (default: `src/components`)
- **CSS entry file** (default: `src/style.css`)

Then it:
- Downloads `yei.css` from the registry and places it next to your CSS entry file
- Adds `@import "./yei.css";` to your CSS entry file
- Creates the component output directory with an empty `mod.rs`
- Installs the `icons` component (required by most components)
- Creates `yei.json` with your configuration

### `yei add <component>`

Copies one or more components into your project.

```sh
yei add button
yei add button badge alert          # multiple at once
yei add button@0.1.0                # pin to a specific version
```

- Resolves and installs dependencies automatically
- Rewrites `use crate::components::` imports to match your project's module path
- Appends `pub mod {name};` to your `mod.rs`
- Prints any required `Cargo.toml` additions

### `yei list`

Lists all available components in the registry.

```sh
yei list
```

### `yei versions`

Lists available registry versions.

```sh
yei versions
```

### `yei upgrade`

Upgrades all installed components to their latest versions.

```sh
yei upgrade
```

## Configuration

`yei.json` is created by `yei init` and lives at the project root:

```json
{
  "registry": "https://yei-api.aodihis.com",
  "version": "latest",
  "output_path": "src/components",
  "module_path": ""
}
```

| Field | Description |
|---|---|
| `registry` | Registry API base URL |
| `version` | Default version to install (`latest` or a tag like `0.1.0`) |
| `output_path` | Where component `.rs` files are written |
| `module_path` | Override the Rust module path (derived from `output_path` if empty) |

`yei.lock` tracks the installed version of each component and is committed to version control.

## Theming

`yei.css` ships default design tokens as CSS custom properties. Override any token in your own CSS after the import:

```css
@import "tailwindcss";
@import "./yei.css";

:root {
  --primary: oklch(0.5 0.2 250);
  --radius: 0.375rem;
}
```
