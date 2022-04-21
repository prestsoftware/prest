Consistency Criteria for General Datasets
=========================================


For every subject whose choices are in the dataset, Prest |version| can compute, view and export the 
total number of violations for each of the axioms/criteria of choice consistency that are listed below.

**Note:** *much of the terminology and notation that follows is introduced and explained in the* 
:ref:`Datasets <general-datasets>` *and* :ref:`Revealed Preference Relations <revealed>` *sections*.


Weak Axiom of Revealed Preference - WARP
----------------------------------------

For any two distinct alternatives `x,y` in `X`

.. math:: 
	x\succ^R y\;\; \Longrightarrow\;\; y\not\succsim^R x

	
.. note::
     Prest |version| reports two WARP counts for general datasets: **WARP (pairs)** and **WARP (all)**.
	 
     **WARP (pairs)** is the number of *pairs of menus* that are implicated in a WARP violation.
     
     **WARP (all)** is the total number of WARP violations.
	 
     For example, the data `C(\{x,y,z\})=\{x,y\}` and `C(\{x,z\})=\{z\}` 
     is associated with a WARP (pairs) count of 1 and a WARP (all) count of 2, 
     the latter involving alternatives `x,z` and `y,z`, respectively.


Congruence
----------

For any two distinct alternatives `x,y` in `X`

.. math::
	x\succ^R y\;\; \Longrightarrow\;\; y\not\succsim^{\widehat{R}} x

.. note::
     In Prest |version|, Congruence violations of length 2 coincide with the **WARP (all)** count.
	
	
Strict Choice Consistency
-------------------------

For any two distinct alternatives `x,y` in `X`

.. math::
	x \succ^{\widehat{R}} y\;\; \Longrightarrow\;\; y\not\succsim^R x


Strict Binary Choice Consistency
--------------------------------

For any two distinct alternatives `x,y` in `X`

.. math::
    x\succ^{\widehat{B}} y\;\; \Longrightarrow\;\; y\not\succsim^B x
	
	
Binary Choice Consistency
-------------------------

For any two distinct alternatives `x,y` in `X`

.. math::
    x\succsim^{\widehat{B}} y\;\; \Longrightarrow\;\; y\not\succ^B x


.. _general-consistency-tip:

.. tip::
     **To use the consistency-analysis feature:** right-click on the dataset of interest [e.g. "DatasetX.csv"] in the workspace and select *"Analysis -> Consistency analysis"*.

     **To view the consistency-analysis output:** right-click on the Prest-generated dataset ["DatasetX.csv (consistency)"] in the workspace and then click on "View".

     **To export the consistency-analysis output (in .xslx or .csv format):** right-click on the Prest-generated dataset ["DatasetX.csv (consistency)"] 
     in the workspace, click on "Export", and then select one of the following options:

     * **Summary**: lists the total number of violations of each axiom (per subject).
     * **Congruence violations (wide)**: lists the number of Congruence violations, decomposed by cycle length.
     * **Strict general cycles (wide)**: lists the number of Strict Choice Consistency violations, decomposed by cycle length.
     * **Strict binary cycles (wide)**: lists the number of Strict Binary Choice Consistency violations, decomposed by cycle length.
     * **Binary cycles (wide)**: lists the number of Binary Choice Consistency violations, decomposed by cycle length.
     

Additional Features: Inconsistent Tuples
----------------------------------------

.. _menu-tuples:

Inconsistent tuples of menus
............................

By right-clicking on the dataset and then selecting *"Analysis -> Inconsistent tuples of menus"*, Prest computes and enumerates 
all distinct pairs, triples, quadruples, ..., `n`-tuples of menus that have led to a Congruence violation, and groups them according to the size of `n`.

.. note::
     The number of inconsistent *pairs* of menus coincides with the **WARP (pairs)** count.

Following the same steps as above, this output can be viewed within Prest or exported to a .csv or .xslx file.


.. _alternative-tuples:

Inconsistent tuples of alternatives
...................................

By right-clicking on the dataset and then selecting *"Analysis -> Inconsistent tuples of alternatives"*, Prest computes and enumerates 
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
     	 
     **Remark:** *If the merging operation is applied on a non-forced-choice dataset where a subject has chosen an alternative from menu* `A` *in one or more instances and has deferred choice/opted for the outside option
     in at least another, then the merged dataset will feature menu* `A` *appearing twice: one where* `C(A)` *comprises all alternatives in* `A` *that were chosen at least once; and one where* `C(A)=\emptyset`.
     
     **An example of a dataset that may help as an illustration for these merging features is available** `here </_static/examples/general-merging.csv>`_. 
	 
.. note::

     **We provide an** `example general dataset with default alternatives </_static/examples/general-defaults.csv>`_  **and** `an example general dataset without default alternatives </_static/examples/general-no-defaults.csv>`_, **that can be analysed for consistency as described above**.
