# Prest

Preference estimation from choice data: https://prestsoftware.com

Reach us at contact AT prestsoftware DOT com.

## Introduction

Prest is a free and user-friendly desktop application for computational revealed
preference analysis. It allows for processing choice datasets that economists,
psychologists and consumer/marketing researchers often generate through
experiments, market studies or surveys.

## Documentation and downloads

Pre-built Prest binaries can be downloaded from the landing page of the
documentation at https://prestsoftware.com.

## Building and running Prest

Dependencies:
* Rust 1.26 stable + Cargo
* Python 3 (we use 3.6 and 3.7) + pip

Install the dependencies, compile everything and run:

```bash
$ pip install --user -r gui/requirements.txt
$ make run
```

Optionally, work in a virtual environment:

```bash
$ python3 -m venv prest.env
$ source prest.env/bin/activate
# The next line is different from the previous installation command.
$ pip install -r gui/requirements.txt
$ make run
```

The build invoked by the commands above will, among other things, build the
HTML documentation, which is required for the help features of Prest.  It will
also typecheck the code using `mypy`.

### Testing

```bash
$ make test      # quick test during development
$ make fulltest  # includes long-running tests
```

### Packaging

We build stand-alone binaries using PyInstaller. These build scripts are not
published at the moment.

## License

Prest consists of two separate programs: GUI (GNU GPL) and core (BSD-3-Clause).
The full license texts can be found in the corresponding subdirectories of the source code.

## Declarations

Prest does not collect any data entered by its users.

The latest version of Prest will always be available online for free.
