# Rust make-like Build System | Yaml-based

This project is for teaching purposes in an Embedded Linux Make/Cmake course to show how `Make` build system works.

## RMakefile

The input file is a Yaml file, that has the following rules:

* Attributes with no attributes are parsed as variables, like:

```yaml
CC: gcc
CFLAGS: -Iinclude -Werror -O2
```

* Attributes with sub attributes are parsed as `targets`, like:

```yaml
main:
    dep: main.c
    cmd: $(CC) $< -o $@
```

### Targets

* Targets `MUST` contain: `cmd`.
* `dep` is optional

You can specify one or multiple commands, like follows:

```yaml
main:
    dep: main.c
    cmd: |
        echo Compiling main.c
        $(CC) $< -o $@
```

## Variable expansion

Variables in the same format as in `Makefile` will be expanded from the global variables.

Supported variables for this demo are:

- `$@` : The same target name
- `$^` : Full dependencies list
- `$<` : First element of the dependencies list
- `$()`: Holds a variable name

## Usage

```sh
git clone git@github.com:bhstalel/rmake-demo.git
cd rmake-demo
cargo run
```

* Example:

Change the absolute path in `examples/RMakefile.yml` first. This should be set as `$(shell pwd)` but it is not implemented.

## Limitations

Limitations are same as [TODO](#todo)

## TODO

- [ ] Complete variable expansion
- [ ] Complete running shell commands
- [ ] Handle variable expansion recursively
- [ ] Handle file depends
- [ ] Make `target` argument with default value, if default run first target
- [ ] Add more special characters handling
- [ ] Add `@` as first character of the command to ignore printing the command
- [ ] Add feature to test if variable is an env var if not declared in the Yaml file
- [ ] Add feature to declare a function.

Feel free to add what you want.