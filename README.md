# Prest

Revealed-preference analysis and preference estimation from choice data: https://prestsoftware.com

Reach us at gerasimou AT outlook DOT com  and  ziman AT functor DOT sk.

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
* Python 3.10+

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

The most comprehensive test is the [integration
test](https://github.com/prestsoftware/prest/blob/master/gui/test/integration_test.py),
which runs the whole pipeline including the Rust core on the [example
datasets](https://github.com/prestsoftware/prest/tree/master/docs/src/_static/examples).
It is invoked in the course of the above commands; `make test` uses only the small
example datasets, while `make fulltest` uses all of them.

### Packaging

We build stand-alone binaries using PyInstaller. These build scripts are not
published at the moment.

## License

Prest consists of two separate programs: GUI (GNU GPL) and core (BSD-3-Clause).
The full license texts can be found in the corresponding subdirectories of the source code.

## Citation

If you use Prest in your academic work, please cite it as follows:

Georgios Gerasimou and Matúš Tejiščák (2018) Prest: Open-Source Software for Computational Revealed Preference Analysis, _Journal of Open Source Software_, 3(30), 1015, [doi:10.21105.joss.01015](https://doi.org/10.21105/joss.01015).

## Declarations

Prest does not collect any data entered by its users.

The latest version of Prest will always be available online for free.
