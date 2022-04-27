.. image:: _static/images/prest-logo.png

Introduction
------------

Prest is an open-source and user-friendly Windows & macOS desktop application. 

It can be used to analyze choice datasets created by economists, psychologists and marketing researchers. 

Its key novelties pertain to general datasets where choice alternatives are discrete. 

Prest allows for estimating non-parametrically the decision maker's preferences from such general datasets.

It does so by finding out how "close" the observed choices are to being explainable by **rational choice or some model of bounded-rational choice**.

In this way, Prest recovers both the individual's **decision rule** and their **preferences conditional on that rule**.

Declarations
------------

*Prest is open-source software and its latest version will always be available online for free.*


*Prest does not collect any data entered by its users.*


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

Documentation
-------------

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


Prest Developers
----------------

`Georgios Gerasimou <https:georgiosgerasimou.com/>`_ (project coordinator) 

`Matúš Tejiščák <https://ziman.functor.sk/>`_ (lead programmer)

If you use Prest in your work, please cite it as follows: 

Georgios Gerasimou and Matúš Tejiščák (2018) "Prest: Open-Source Software for Computational Revealed Preference Analysis", 
*Journal of Open Source Software*, 3(30), 1015, `doi:10.21105.joss.01015 <https://doi.org/10.21105/joss.01015>`_.