Datasets
========

.. _general-datasets:

General Datasets
----------------

Here, the choice set of interest `X=\{x_1,\ldots,x_m\}` consists of finitely many general/unstructured alternatives, and 
the available data is a collection of **menus** of such alternatives and the **observed choices** at these menus. 
Formally, a menu is a set `A\subseteq X`, and the observed choice(s) at this menu is (are) captured by the set `C(A)`, where `\emptyset\subseteq C(A)\subseteq A`. 

A **general dataset** 

.. math::
	\mathcal{D}=\left\{\big(A_i,C(A_i)\bigr)\right\}_{i=1}^k

is a collection of `k` observations, with each of them a pair of a menu and the alternative(s) chosen from it (if any). 
In particular, if `C(A)` contains more than one alternative for some menu `A` in `\mathcal{D}`, 
it is understood that the decision maker has chosen (or may be thought of as having chosen)
any or all these alternatives at `A`, possibly over different instances where `A` was presented in `\mathcal{D}` (see also :ref:`merging <merging-tip>`). 
If `C(A)=\emptyset`, then it is understood that the decision maker has chosen the **no-choice/outside option**, hence
to **avoid** or **defer** choice at menu `A`.


It is also possible that the data available to the analyst features a **default/status quo option**, reflecting situations where the decision 
maker was initially endowed with some alternative `s\in A` before asked to choose from menu `A`.

A **general dataset with default/status quo alternatives** 

.. math::
	\mathcal{D}=\left\{\big((A_i,s_i),C(A_i,s_i)\bigr)\right\}_{i=1}^k

is a collection of `k` observations, with each of them a pair comprising a **decision problem** and the alternative(s) that was/were observed to be chosen at this decision problem. 
In each decision problem `(A_i,s_i)`, `A_i` is a menu and `s_i\in A_i` the default/status quo alternative at that menu, 
while `\emptyset\neq C(A_i,s_i)\subseteq A_i` is required to hold for all `i\leq k` in such datasets.

.. _dataset-examples:

.. tip::
     To be analyzable by Prest |version|, a general dataset must be a .csv file.

     An  `example general dataset </_static/examples/general-no-defaults.csv>`_.

     An `example general dataset with default/status quo alternatives </_static/examples/general-defaults.csv>`_.
	 
     An `example hybrid general dataset containing both types of observations </_static/examples/general-hybrid.csv>`_.
    
     To import such a dataset into Prest, select *"Workspace -> Import general dataset"* and select the target file from the relevant directory.
     
     The new pop-up window features four column headers under *"Columns"*: **Subject**, **Menu**, **Default** and **Choice**. 
     Select the appropriate column name in your .csv file from the drop-down menu to match the corresponding column header. 
     If your dataset does not feature default alternatives, select *"None"* for the **Default** header.
	 
     To view the imported dataset in Prest, double-click on it in the workspace area.


.. _budgetary-datasets:
	 
Budgetary Datasets
------------------

Here, consumer behavior with respect to `n` commodities is analyzed when data is available on 
the **prices** of these commodities, captured by a vector `p\in\mathbb{R}^n_{+}`, and also on consumer **demand** at these prices, 
captured by a vector/**consumption bundle** `x\in\mathbb{R}^n_+`.
 
A **budgetary dataset**  

.. math::
	\mathcal{D}=\left\{(p^i,x^i)\right\}_{i=1}^k

is a collection of `k` observations, with each of them a pair `(p^i,x^i)` comprising the consumption bundle `x^i` that was observed to be chosen when prices were `p^i`.


.. tip::
     To be analyzable by Prest |version|, a budgetary dataset must be a .csv file.

     An `example budgetary dataset </_static/examples/budgetary.csv>`_.
     
     To import such a dataset, go to *"Workspace -> Import budgetary dataset"* and select the target file from the relevant directory.
     
     **Budgetary datasets with** `n` **goods must have the following structure:** 
	 
     * Column 1: subject ID
     * Column 2: price of good 1
     * Column `n+1`: price of good `n`
     * Column `n+2`: demand of good 1
     * Column  `2n+1`: demand of good `n`

     To view the imported dataset, double-click on it in the workspace area. **An extra column with the total expenditure associated with each observation is automatically added.**
