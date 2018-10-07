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

1. Run all tests using `make longtest` in the root directory.

1. Update the documentation, the surrounding code, examples elsewhere, whatever
   is affected by your contribution

1. Use appropriate code formatting for both Rust and/or Python.

Feel free to submit pull requests for review if you're unsure how to e.g. fix
the tests.  We'll do our best to help you out.

## License

Prest consists of two separate programs: the GUI and the core.
While the core is licensed under the BSD 3-Clause license, the GUI has to be
licensed under the GPL, due to the PyQt5 license.

You should be comfortable to contribute your code under these terms.

## Credits

Adapted from [the contribution guidelines for Idris](https://github.com/idris-lang/Idris-dev/).
