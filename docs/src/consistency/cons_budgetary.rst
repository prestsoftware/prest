Consistency Criteria for Budgetary Datasets
===========================================

For every subject whose choices are in the dataset, Prest can find and count the 
total number of violations (resp. score) for each of the axioms/criteria (resp. choice consistency index) listed below.

.. note::
     Much of the terminology and notation that follows is introduced and explained in the 
     :ref:`Datasets <general-datasets>` and :ref:`Revealed Preference Relations <revealed>` sections.


Weak Axiom of Revealed Preference - WARP
----------------------------------------

**Strict:**

.. math::
    x^i\succsim^R x^j\;\; \Longrightarrow\;\; x^j\not\succsim^R x^i

**Non-strict:**

.. math::
    x^i\succsim^R x^j\;\; \Longrightarrow\;\; x^j\not\succ^R x^i
	
Strong Axiom of Revealed Preference - SARP
------------------------------------------

.. math::
    x^i\succsim^{\widehat{R}}x^j\;\; \Longrightarrow\;\; x^j\not\succsim^R x^i
	
	
Generalized Axiom of Revealed Preference - GARP
-----------------------------------------------

.. math::
    x^i\succsim^{\widehat{R}}x^j\;\; \Longrightarrow\;\; x^j\not\succ^R x^i

.. note::
	*SARP* implies *WARP strict*.
	
	*GARP* implies *WARP non-strict*.
	
Houtman-Maks index - HM
-----------------------

This corresponds to the smallest number of observations that need to be removed from a given subject's data
in order for the remaining choices to satisfy GARP, SARP, WARP (strict) or WARP (non-strict). 

Prest computes each of these four HM indices for budgetary data by finding both 
their **upper and lower bounds**. 

If the two bounds coincide, Prest reports the exact HM index for the axiom in question. If they differ, Prest reports
the range `m\leq HM \leq n` of possible values.


.. _budgetary-consistency-tip:

.. tip::
     **To use the consistency-analysis feature:** right-click on the dataset of interest [e.g. "DatasetX.csv"] in the workspace and select *"Analysis -> Consistency analysis"*.

     **To view the consistency-analysis output:** right-click on the Prest-generated dataset [e.g. "DatasetX.csv (consistency)"] in the workspace and then click on "View".

     **To export the consistency-analysis output (in .xslx or .csv format):** right-click on the Prest-generated dataset [e.g. "DatasetX.csv (consistency)"] 
     in the workspace, click on "Export", and then select one of the following options:
     
     **An example of a budgetary dataset that can be analysed in this way can be found** `here </_static/examples/budgetary.csv>`_. 
