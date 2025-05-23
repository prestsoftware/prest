.. note::
     
     `Link </_static/examples/general-stochastic-consistency.csv>`_
     **to an example general dataset that can be analysed for stochastic choice consistency as described later on this page.**

|

Stochastic Consistency Criteria for General Datasets
====================================================

For general datasets where the same menus appear more than once, 
Prest can compute, view and export the total number of violations 
of each of the axioms/criteria of stochastic choice consistency listed below,
for every subject in the dataset.

In the statements that follow, for a given menu `A` in general dataset `\mathcal{D}`, 
and any alternative `a` in that menu, 
`Pr(a,A)`  denotes the **choice frequency** or **choice probability** of 
`a` at the different presentations of menu `A` for some agent in `\mathcal{D}`.   

.. _weak-stochastic-transitivity:

Weak Stochastic Transitivity
----------------------------
*For all* `x,y,z\in X`, `Pr(x,\{x,y\})\geq \frac{1}{2}` and
`Pr(y,\{y,z\})\geq \frac{1}{2}` *implies* `Pr(x,\{x,z\})\geq \frac{1}{2}`.

.. _moderate-stochastic-transitivity:

Moderate Stochastic Transitivity
--------------------------------
*For all* `x,y,z\in X`, `Pr(x,\{x,y\})\geq \frac{1}{2}` and
`Pr(y,\{y,z\})\geq \frac{1}{2}` *implies* `Pr(x,\{x,z\})\geq \min\{Pr(x,\{x,y\}),Pr(y,\{y,z\})\}`.

.. _strong-stochastic-transitivity:

Strong Stochastic Transitivity
------------------------------
*For all* `x,y,z\in X`, `Pr(x,\{x,y\})\geq \frac{1}{2}` and
`Pr(y,\{y,z\})\geq \frac{1}{2}` *implies* `Pr(x,\{x,z\})\geq \max\{Pr(x,\{x,y\}),Pr(y,\{y,z\})\}`.

.. _regularity-def:

Regularity
----------
*For all menus* `A`, `B` in `\mathcal{D}` such that `A\supset B`, and for all `x` in `A`, 
`Pr(x,A)\leq Pr(x,B\}`.


.. _stochastic-consistency-tip:

.. tip::
     **To use the stochastic-consistency feature:** right-click on the dataset of interest [e.g. "DatasetX.csv"] in the workspace and select *"Analysis -> Stochastic consistency analysis"*.

     **To view the stochastic-consistency output:** right-click on the Prest-generated dataset ["DatasetX.csv (stochastic consistency)"] in the workspace and then click on "View".

     **To export the stochastic-consistency output (in .xslx or .csv format):** right-click on the Prest-generated dataset ["DatasetX.csv (stochastic consistency)"] 
     in the workspace, and click on "Export" and "Summary...".
