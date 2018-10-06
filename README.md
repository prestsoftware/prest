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

Prest core is open-source under the [3-Clause BSD
license](https://github.com/prestsoftware/prest/blob/master/core/LICENSE).

Prest GUI is open-source under the [GNU GPL](https://github.com/prestsoftware/prest/blob/master/gui/LICENSE),
due to the [license of PyQt5](https://www.riverbankcomputing.com/commercial/license-faq).

## Declarations

Prest does not collect any data entered by its users.

The latest version of Prest will always be available online for free.
