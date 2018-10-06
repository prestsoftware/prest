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

## Building

Dependencies:
* Rust + Cargo
* Python 3 (we use 3.6 and 3.7) + pip

```bash
$ pip install --user -r gui/requirements.txt
$ make run
```

We build stand-alone binaries using PyInstaller. These build scripts are not
published at the moment.

## License

Prest consists of two separate programs: GUI (GNU GPL) and core (BSD-3-Clause).
The full license texts can be found in the corresponding subdirectories of the source code.

## Declarations

Prest does not collect any data entered by its users.

The latest version of Prest will always be available online for free.
