.. image:: _static/images/prest-logo.png
  :align: center  
  :width: 80%
  :target: ../build/html/index.html

|
|

Introduction
------------

Prest is a free and open-source desktop application for choice-based 
preference estimation.

|

Downloads
---------

* | `Prest VERSION for Windows <_static/prest-win-VERSION.exe>`_
  | No installation required: run by double-clicking the :code:`.exe` file.
  | If Windows blocks Prest, right-click on the file and in *"Properties -> General"* select *"Unblock"*.

* | `Prest VERSION for macOS <_static/prest-osx-VERSION.zip>`_ 
  | No installation required: run by double-clicking the :code:`.command` file. Select *"Open anyway"* if prompted. 
  | If the *"Open anyway"* button is not available, close the dialog window and double-click the :code:`.command` file again.
  
* | `Prest VERSION for GNU/Linux <_static/prest-linux-VERSION.zip>`_

* The `Prest source code <https://github.com/prestsoftware/prest>`_, written
  in `Rust <https://www.rust-lang.org/>`_ (core program) and `Python
  <https://www.python.org/>`_ (graphical user interface).

Previous downloadable versions are available in :ref:`the archive <history>`.

|

Recently Added Features
-----------------------

* | Since VERSION: new suite of tests for datasets where the same menu is presented more than once.
  | (support for possibly *random choice data*)
  
* | Since VERSION: new measure of model proximity for datasets with multiple choices per menu. 
  | (support for possibly *multi-valued choice functions/correspondences*)
  
* | Since v1.1.0: visualization of preference-estimation output using `GraphViz <https://graphviz.org>`_ .
  | (the GraphViz binary file must be placed in the same directory as Prest)

|

Documentation
-------------

The pages linked below (also in the navigation menu on the left) contain information about Prest's features, 
define the terms used in its graphical user interface, and explain relevant background concepts.

.. tip:: 
     Text boxes with the  **Tip** label provide essential information about Prest's features.

.. note::
     Text boxes with the **Note** label provide supplementary information.

.. toctree::
   :maxdepth: 2

   notation/index
   workspace/index
   consistency/index
   estimation/index
   models/index
   simulations/index
   references
   history/index
   copyright/index
   acknowledgements/index   

|

Citation
--------

`Georgios Gerasimou <https:georgiosgerasimou.com/>`_ and `Matúš Tejiščák <https://ziman.functor.sk/>`_ (2018) "Prest: Open-Source Software for Computational Revealed Preference Analysis", 
*Journal of Open Source Software*, 3(30), 1015, `doi:10.21105.joss.01015 <https://doi.org/10.21105/joss.01015>`_.

|

Declarations
------------

* | *Prest is open-source software and its latest version will always be available online for free.*


* | *Prest does not collect any data entered by its users.*
