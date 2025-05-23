.. note::

     **Links to two example general datasets that can be analysed for choice consistency as described later on this page:**
     
     `1. Without default/status quo alternatives </_static/examples/general-no-defaults.csv>`_
     
     `2. With default/status quo alternatives </_static/examples/general-defaults.csv>`_ 


|

Deterministic Consistency Criteria for General Datasets
=======================================================


For datasets where each menu appears once, Prest can compute, view and 
export the total number of violations of each of the axioms/criteria 
of deterministic choice consistency listed below, 
for every subject in the dataset.

Much of the terminology and notation that follows is introduced and explained in the 
:ref:`Datasets <general-datasets>` and :ref:`Revealed Preference Relations <revealed>` sections.

.. _contraction-consistency:


Contraction Consistency
-----------------------

For any alternative `x` in `X` and menu `A` in `\mathcal{D}`

.. math:: 
	x \in A \subset B \text{ and } x\in C(B)\;\; \Longrightarrow\;\; x\in C(A)

	
.. note::
     Prest reports two Contraction-Consistency counts for general datasets: 
     **Contraction Consistency (pairs)** and **Contraction Consistency (all)**.
	 
     **Contraction Consistency (pairs)** is the number of *pairs of menus* that are implicated 
     in a Contraction Consistency violation.
     
     **Contraction Consistency (all)** is the total number of Contraction Consistency violations.
	 
     For example, the data `C(\{x,y,z\})=\{x,z\}` and `C(\{x,z\})=\emptyset` 
     is associated with a Contraction Consistency (pairs) count of 1 and a 
     Contraction Consistency (all) count of 2, the latter involving alternatives `x` and `y`.


.. _weak-axiom-of-revealed-preference-warp:


Weak Axiom of Revealed Preference - WARP
----------------------------------------

For any two distinct alternatives `x,y` in `X`

.. math:: 
	x\succ^R y\;\; \Longrightarrow\;\; y\not\succsim^R x

	
.. note::
     Prest reports two WARP counts for general datasets: **WARP (pairs)** and **WARP (all)**.
	 
     **WARP (pairs)** is the number of *pairs of menus* that are implicated in a WARP violation.
     
     **WARP (all)** is the total number of WARP violations.
	 
     For example, the data `C(\{x,y,z\})=\{x,y\}` and `C(\{x,z\})=\{z\}` 
     is associated with a WARP (pairs) count of 1 and a WARP (all) count of 2, 
     the latter involving alternatives `x,z` and `y,z`, respectively.


.. _congruence:

Congruence / Strong Axiom of Revealed Preference
------------------------------------------------

For any two distinct alternatives `x,y` in `X`

.. math::
	x\succ^R y\;\; \Longrightarrow\;\; y\not\succsim^{\widehat{R}} x

.. note::
     In Prest, Congruence violations of length 2 coincide with the WARP (all) count.

	
.. _strict-choice-consistency:

Strict Choice Consistency
-------------------------

For any two distinct alternatives `x,y` in `X`

.. math::
	x \succ^{\widehat{R}} y\;\; \Longrightarrow\;\; y\not\succsim^R x


.. _binary-choice-transitivity:

Binary Choice Transitivity
--------------------------

For any sequence of distinct alternatives `x_1,\ldots,x_k` in `X`

.. math::
    x_1\succsim^B x_2,\; \ldots, x_{k-1}\succsim^B x_k\;\;\;\; \text{and}\;\;\; 
    (x_1,x_k)\in\mathcal{D}\;\; \Longrightarrow\;\; x_1\succsim^B x_k


.. _binary-choice-consistency:

Binary Choice Consistency
-------------------------

For any two distinct alternatives `x,y` in `X`

.. math::
    x\succsim^{\widehat{B}} y\;\; \Longrightarrow\;\; y\not\succ^B x


.. _stric-binary-choice-consistency:

Strict Binary Choice Consistency
--------------------------------

For any two distinct alternatives `x,y` in `X`

.. math::
    x\succ^{\widehat{B}} y\;\; \Longrightarrow\;\; y\not\succsim^B x


.. _general-consistency-note:

.. note::
     The difference between *(Strict) Binary Choice Consistency* and *Binary Choice Transitivity* is that, under the same antecedent, 
     the latter will also count `C(\{x_1,x_k\})=\emptyset` as a violation whereas the former will not.  

.. _general-consistency-tip:

.. tip::
     **To use the deterministic-consistency feature:** right-click on the dataset of interest [e.g. "DatasetX.csv"] in the workspace and select *"Analysis -> Deterministic consistency analysis"*.

     **To view the deterministic-consistency output:** right-click on the Prest-generated dataset ["DatasetX.csv (deterministic consistency)"] in the workspace and then click on "View".

     **To export the deterministic-consistency output (in .xslx or .csv format):** right-click on the Prest-generated dataset ["DatasetX.csv (deterministic consistency)"] 
     in the workspace, click on "Export", and then select one of the following options:

     * **Summary**: lists the total number of violations of each axiom (per subject).
     * **Contraction Consistency violations**: lists the number of Contraction Consistency (pairs) (all) violations.     
     * **WARP violations**: lists the number of WARP (pairs) (all) violations.     
     * **Congruence violations (wide)**: lists the number of Congruence violations, decomposed by cycle length.
     * **Strict general cycles (wide)**: lists the number of Strict Choice Consistency violations, decomposed by cycle length.
     * **Binary intransitivities (wide)**: lists the number of Binary Choice Transitivity violations, decomposed by length of the violating sequence.
     * **Binary cycles (wide)**: lists the number of Binary Choice Consistency violations, decomposed by cycle length.
     * **Strict binary cycles (wide)**: lists the number of Strict Binary Choice Consistency violations, decomposed by cycle length.     
     

Additional Features: Cyclic Tuples
----------------------------------------

.. _menu-tuples:

Cyclic tuples of menus
............................

By right-clicking on the dataset and then selecting *"Analysis -> Cyclic tuples of menus"*, Prest computes and enumerates 
all distinct pairs, triples, quadruples, ..., `n`-tuples of menus that have led to a Congruence violation, and groups them according to the size of `n`.

.. note::
     The number of cyclic *pairs* of menus coincides with the **WARP (pairs)** count.

Following the same steps as above, this output can be viewed within Prest or exported to a .csv or .xslx file.


.. _alternative-tuples:

Cyclic tuples of alternatives
...................................

By right-clicking on the dataset and then selecting *"Analysis -> Cyclic tuples of alternatives"*, Prest computes and enumerates 
all distinct pairs, triples, quadruples, ..., `n`-tuples of alternatives that have led to a Congruence violation, and groups them according to the size of `n`.

Following the same steps as above, this output can be viewed within Prest or exported to a .csv or .xslx file.

.. _merging-tip:

.. tip::	 
     If the same menu `A` appears more than once for the same subject in `\mathcal{D}`, 
     Prest allows for **merging the choices** made at this menu in the different observations.
      
     For example, if the dataset `\mathcal{D}` is such that `A_1=A_5=\{w,x,y\}` and `C(A_1)=\{x\}`, `C(A_5)=\{y\}` for the same subject,  
     then `\mathcal{D}` would be altered after the merging operation so that the menu `A_1=A_5:=A`
     appears only once, and with `C(A)=\{x,y\}` being the subject's new choice at this menu. 
     	 
     **To use this feature:** right-click on the dataset of interest [e.g. "DatasetX.csv"]
     in the workspace and select *"Analysis -> Merge options at the same menu"*. The resulting merged dataset appears in the workspace ["DatasetX.csv (merged)"] and can then be analysed separately 
     for consistency analysis or model estimation after the potential "noisiness" of choice data has been accounted for in this way through multi-valued choice.
     	      
     **An example dataset that illustrates these merging features is available** `here </_static/examples/general-merging.csv>`_. 
