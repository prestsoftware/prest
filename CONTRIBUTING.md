# Contributing to Prest

We welcome bug reports and pull requests!

However, we cannot do it all ourselves, and want to make it as easy as possible
to contribute improvements.  Here are a few guidelines that we would like
contributors to follow so that their (and our) effort and time is put to best
use.

## Issue Reporting

Before you report an issue, or wish to add functionality please try and check
to see if there are existing
[issues](https://github.com/prestsoftware/prest/issues) and [pull
requests](https://github.com/prestsoftware/prest/pulls).

### Template

Try as closely as possible to describe the issue, ideally adhering to the following structure.

> \# Steps to Reproduce
>
> \# Expected Behavior
> 
> \# Observed Behavior

Where applicable, attach the relevant dataset (or a workspace file). In order
to fix the issue you're observing, we have to _reproduce_ it on our development
machines so any data or advice how to do it is very helpful.

### Feature Requests

You are welcome to file feature requests. Since the development
team consists of volunteers working on Prest in their free time, the
fastest way to have features included in Prest is submitting pull requests.

## Contributing to Prest source

To make contribution easier for everyone involved, please:

1. Discuss your change and the overall approach with us before starting
   substantial work via e-mail at contact AT prestsoftware DOT com.  We do not
   want you to waste your time or duplicate somebody's work.  We will be happy
   to answer questions about Prest's internals.

1. Include type annotations in all code.

1. Run all tests using `make longtest` in the root directory. During
   development, you can use `make test` to run just the quick subset.

1. Update the documentation, the surrounding code, examples elsewhere, whatever
   is affected by your contribution

1. Use appropriate code formatting for both Rust and/or Python.

Feel free to submit pull requests for review if you're unsure how to e.g. fix
the tests.  We'll do our best to help you out.

## Design

We hope that the following will help you hack on the code.

### Functional approach to the GUI code

Some aspects of the code lean towards the functional way rather than the
Pythonic way of doing things.

* Classes (especially datasets) are used as data containers with attached
  functionality rather than encapsulated entities. We access (and modify) their
  properties directly, instead of doing it the OOP way.

* We simulate ADTs using MyPy unions and named tuples. While we could use
  inheritance to achieve the same goal, lightweight tuples perform better and
  use much less memory.

* Codecs for various data types are modelled after Haskell typeclass instances
  / Rust traits; they are defined compositionally for each new type (mostly
  named tuples).

* We make sure that once a codec is constructed, it is as fast as possible.
  That's why we save sub-codecs in locals, which will in turn be stored
  directly in the closure of the codec being constructed. (We absolutely avoid
  attribute lookups every time `encode()` and `decode()` are called.)

* We put a lot of emphasis on typechecking -- if you use the `Makefile`, you
  can't run a program that hasn't been typechecked. Although MyPy types are not
  very precise, they are still a huge boost in reliability.

* We include mainly only integration tests; local sanity is ensured by
  typechecking with MyPy.

## License

Prest consists of two separate programs: the GUI and the core.
While the core is licensed under the BSD 3-Clause license, the GUI has to be
licensed under the GPL, due to the PyQt5 license.

You should be comfortable to contribute your code under these terms.

## Credits

Adapted from [the contribution guidelines for Idris](https://github.com/idris-lang/Idris-dev/).
