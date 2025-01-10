========
Datasets
========

.. _general-datasets:

----------------
General Datasets
----------------

Such datasets consist of a finite collection of **menus** and the **choices observed** at these menus. 

They can be further distinguished between datasets with or without default/status quo options.

General datasets without default/status quo options
===================================================

Given a choice set of interest that is denoted `X=\{x_1,\ldots,x_m\}`, a menu is a set `A\subseteq X`, 
and the observed choice(s) at this menu is captured by the set `C(A)`, where `\emptyset\subseteq C(A)\subseteq A`. 

A **general dataset without default/status quo options** 

.. math::
	\mathcal{D}=\left\{\big(A_i,C(A_i)\bigr)\right\}_{i=1}^k

is a collection of `k` observations, each of them a pair that comprises a menu and the alternative(s) chosen from it, if any. 

If `C(A)` contains more than one alternative, it is understood that the subject has chosen (or may be thought of as having chosen)
any or all these alternatives at `A`, possibly over different instances where `A` was presented in `\mathcal{D}` 
(see also :ref:`merging <merging-tip>`). 

If `C(A)=\emptyset`, it is understood that the agent has opted for the **deferral outside option**, i.e.
to **avoid** or **delay** making an active choice at menu `A`.

General datasets with default/status quo options
================================================

Such datasets reflect situations where the agent under study was known to be initially endowed 
with some alternative `s\in A` before being observed to choose from menu `A`.

Formally, a **general dataset with default/status quo alternatives** 

.. math::
	\mathcal{D}=\left\{\big((A_i,s_i),C(A_i,s_i)\bigr)\right\}_{i=1}^k

is a collection of `k` observations, with each of them a pair comprising a **decision problem** and the alternative(s) 
that was/were observed to be chosen at this decision problem. 
In each decision problem `(A_i,s_i)`, `A_i` is a menu and `s_i\in A_i` the default/status quo alternative at that menu, 
while `\emptyset\neq C(A_i,s_i)\subseteq A_i` is required to hold for all `i\leq k` in such datasets.

.. _dataset-examples:

.. tip::
     To be analyzable by Prest, a general dataset must be a .csv file.

     An  `example general dataset </_static/examples/general-no-defaults.csv>`_.

     An `example general dataset with default/status quo alternatives </_static/examples/general-defaults.csv>`_.
	 
     An `example hybrid general dataset containing both types of observations </_static/examples/general-hybrid.csv>`_.
    
     To import such a dataset into Prest, select *"Workspace -> Import general dataset"* and select the target file from the relevant directory.
     
     The new pop-up window features four column headers under *"Columns"*: **Subject**, **Menu**, **Default** and **Choice**. 
     Select the appropriate column name in your .csv file from the drop-down menu to match the corresponding column header. 
     If your dataset does not feature default alternatives, select *"None"* for the **Default** header.
	 
     To view the imported dataset in Prest, double-click on it in the workspace area.



.. _budgetary-datasets:

------------------	 
Budgetary Datasets
------------------

In such datasets the analyst has observed consumer choices over bundles of `n` commodities and   
the prices of these commodities. 

**Prices** are captured by a vector `p\in\mathbb{R}^n_{+}`. 

A consumer's **demand** at these prices is captured by the **consumption bundle** `x(p)\in\mathbb{R}^n_+`.
 
A **budgetary dataset**  

.. math::
	\mathcal{D}=\left\{(p^i,x^i)\right\}_{i=1}^k

is a collection of `k` observations, each of them a pair `(p^i,x^i)` comprising the consumption bundle `x^i` that was observed to be chosen when prices were `p^i`.

.. tip::
     To be analyzable by Prest, a budgetary dataset must be a .csv file.

     An `example budgetary dataset </_static/examples/budgetary.csv>`_.
     
     To import such a dataset, go to *"Workspace -> Import budgetary dataset"* and select the target file from the relevant directory.
     
     **Budgetary datasets with** `n` **goods must have the following structure:** 
	 
     * Column 1: subject ID
     * Column 2: price of good 1
     * Column `n+1`: price of good `n`
     * Column `n+2`: demand of good 1
     * Column  `2n+1`: demand of good `n`

     To view the imported dataset, double-click on it in the workspace area. **An extra column with the total expenditure associated with each observation is added automatically.**


