.. image:: _static/images/prest-logo.png

Introduction
------------

Prest is an open-source desktop application for rational and, especially, behavioural revealed-preference analysis.

It can be used to analyze choice datasets created by experiments or surveys in economics or psychology. 

| Prest helps the analyst understand the surveyed decision makers' possible choice heuristics and preferences. 
| It does so using information about their observable decisions only.  

|

Recently Added Features
-----------------------

* | Since v2.0.0: new suite of stochastic-consistency tests for datasets with menu repetitions.
  | (support for *stochastic choice functions*)
  
* | Since v2.0.0: new measure of model proximity for datasets with multiple choices per menu. 
  | (support for *choice correspondences*)
  
* | Since v1.1.0: visualization of preference-estimation output using `GraphViz <https://graphviz.org>`_ .
  | (the GraphViz binary file must be copied to the same directory as Prest)

|

Downloads
---------

* `Prest VERSION for Windows <_static/prest-win-VERSION.exe>`_ —
  No installation required: run by double-clicking the :code:`.exe` file.

* `Prest VERSION for macOS <_static/prest-osx-VERSION.zip>`_ —
  No installation required: run by double-clicking the :code:`.command` file.
  Select "Open anyway" if prompted. If the "Open anyway" button is not available,
  close the dialog window and double-click the :code:`.command` file again.
  
* Prest VERSION for Linux — Follow these `instructions <https://github.com/prestsoftware/prest?tab=readme-ov-file#building-and-running-prest>`_ to build Prest from source code & run it on any Linux distro.

* The `Prest source code <https://github.com/prestsoftware/prest>`_, written
  in `Rust <https://www.rust-lang.org/>`_ (core) and `Python
  <https://www.python.org/>`_ (graphical user interface).

Previous downloadable versions of Prest are available in :ref:`the archive <history>`.

|

Declarations
------------

*Prest is open-source software and its latest version will always be available online for free.*


*Prest does not collect any data entered by its users.*

|

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
   estimation/index
   models/index
   simulations/index
   references
   history/index
   copyright/index
   acknowledgements/index   

|

Prest Developers
----------------

`Georgios Gerasimou <https:georgiosgerasimou.com/>`_ & `Matúš Tejiščák <https://ziman.functor.sk/>`_  

If you use Prest in your work, please cite it: 

Georgios Gerasimou and Matúš Tejiščák (2018) "Prest: Open-Source Software for Computational Revealed Preference Analysis", 
*Journal of Open Source Software*, 3(30), 1015, `doi:10.21105.joss.01015 <https://doi.org/10.21105/joss.01015>`_.
