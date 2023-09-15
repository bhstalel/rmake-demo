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
- `$()`: Holds a variable name, if not found RMake will check your `env`
- `$(shell cmd)`: Runs a shell command

## Usage

```sh
git clone git@github.com:bhstalel/rmake-demo.git
cd rmake-demo
cargo run -- --help
```

## Arguments

```sh
cargo run -- --help
cargo run -- <target>
```

* Specify custom target (If no target is specified, first target will be run):

```sh
cargo run -- <target>
```

* Specify custom directory that contains `RMakefile.yml`:

```sh
cargo run -- main -C examples/

2023-09-15T03:06:13.323256Z  INFO Setting build directory ..
2023-09-15T03:06:13.334186Z  INFO Running: gcc -Iinclude -c main.c
2023-09-15T03:06:13.345977Z  INFO Running: gcc -Iinclude -c hello.c
2023-09-15T03:06:13.359252Z  INFO Running: gcc hello.o -shared -o hello.so
2023-09-15T03:06:13.369782Z  INFO Running: gcc main.o -l:hello.so -L/home/talel/Documents/SelfWork/rust/rmake-demo/examples
 -o main
```

## Logging

By default `INFO` level is activated, to manipulate the level using one of:

* INFO
* DEBUG
* ERROR
* WARN
* TRACE (Not used in RMake)

set the env variable `LOGL` to whatever level you want, example:

```sh
LOGL=DEBUG cargo run -- main -C examples/
```

## Limitations

Limitations are same as [TODO](#todo)

## TODO

- [ ] Add more `Makefile` core functions like `shell` (done), `wildcard`, ...
- [X] Complete variable expansion
- [X] Complete running shell commands
- [X] Handle variable expansion recursively
- [ ] Handle file depends
- [X] Make `target` argument with default value, if default run first target
- [ ] Add more special characters handling
- [ ] Add `@` as first character of the command to ignore printing the command
- [X] Add feature to test if variable is an env var if not declared in the Yaml file
- [ ] Add feature to declare a function.

Feel free to add what you want.