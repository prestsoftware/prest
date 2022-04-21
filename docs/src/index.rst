.. image:: _static/images/prest-logo.png

Introduction
------------

Prest is a free and user-friendly desktop application for computational revealed preference analysis.
It allows for processing choice datasets that economists, psychologists and consumer or marketing researchers 
often generate through experiments, market studies or surveys.

The key novelties of Prest pertain to general datasets where choice alternatives are discrete.

Prest allows for estimating non-parametrically the decision maker's preferences from such general datasets.

It does so by finding out how "close" the observed choices are to being explainable by rational choice or by some model of bounded-rational choice.

In this way, Prest recovers both the individual's **decision rule** and their **preferences** conditional on this decision rule.

Declarations
------------

Prest is open-source software and its latest version will always be available online for free.


Prest does not collect any data entered by its users.


Downloads
---------

* `Prest VERSION for Windows <_static/prest-win-VERSION.exe>`_ —
  No installation required: run by double-clicking the :code:`.exe` file.

* `Prest VERSION for macOS <_static/prest-osx-VERSION.zip>`_ —
  No installation required: run by double-clicking the :code:`.command` file.
  Select "Open anyway" if prompted. If the "Open anyway" button is not available,
  close the dialog window and double-click the :code:`.command` file again.

* The `Prest source code <https://github.com/prestsoftware/prest>`_, written
  in `Rust <https://www.rust-lang.org/>`_ (core) and `Python
  <https://www.python.org/>`_ (graphical user interface).

* Previous downloadable versions of Prest are available in :ref:`the archive <history>`.


Prest Developers
----------------

`Georgios Gerasimou <https://sites.google.com/site/georgiosgerasimou/>`_ (project coordinator) and `Matúš Tejiščák <https://ziman.functor.sk/>`_ (lead programmer).

If you use Prest in your work, please cite it as follows:

Georgios Gerasimou and Matúš Tejiščák (2018) Prest: Open-Source Software for Computational Revealed Preference Analysis, 
*Journal of Open Source Software*, 3(30), 1015, `doi:10.21105.joss.01015 <https://doi.org/10.21105/joss.01015>`_.


Prest Documentation
-------------------

The pages linked below (and also in the navigation menu on the left) contain information about Prest's features, 
define the terms used in the graphical user interface, and explain relevant background concepts.

.. tip:: 
     Text boxes with the  **Tip** label provide essential information about Prest's features.

.. note::
     Text boxes with the **Note** label provide supplementary information.

.. toctree::
   :maxdepth: 2

   workspace/index
   notation/index
   consistency/index
   models/index
   simulations/index
   references
   history/index   
   acknowledgements/index
   copyright/index
   privacy/index
