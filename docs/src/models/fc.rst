Forced-Choice Models (non-feasible outside option)
==================================================

Utility Maximization
--------------------

Strict
......

A general choice dataset `\mathcal{D}` on a set of alternatives `X` is explained by
**(strict) utility maximization** if there is a strict linear
order `\succ` on `X` such that for every menu `A` in `\mathcal{D}`

.. math::
	C(A) = \Big\{x\in A: x\succ y\;\; \text{for all $y\in A\setminus\{x\}$}\Big\} \text{.}


Non-strict
..........

A general choice dataset `\mathcal{D}` on a set of alternatives `X` is explained by
**(non-strict) utility maximization** if there is a weak order
`\succsim` on `X` such that for every menu `A` in `\mathcal{D}`

.. math::
    C(A) = \{x \in A: x\succsim y\;\; \text{for all $y\in A$}\}

.. centered:: and

.. math::
    x\sim y\;\; \text{for distinct}\; x,y\; \text{in}\; X.


.. tip::  
     When analysing other models that generalize utility maximization/rational choice, 
     Prest |version| only considers instances of the more general models that do not overlap with those covered by the above two variants of utility maximization.
     It is therefore recommended that both of them always be included in all model-estimation tasks.
     	

|
  
Incomplete-Preference Maximization: Undominated Choice
------------------------------------------------------

Strict
......

A general choice dataset on a set of alternatives `X` is explained by
**(strict) undominated choice** if there is a strict
partial order `\succ` on `X` such that for every menu `A` in `\mathcal{D}`

.. math::
	C(A) = \{x\in A: y\not\succ x\;\; \text{for all $y\in A$}\} \text{.}

	
	
Non-strict
..........

A general choice dataset on a set of alternatives `X` is explained by
**(non-strict) undominated choice** if there is an incomplete preorder `\succsim` on `X` such
that for every menu `A` in `\mathcal{D}`

.. math::
    C(A) = \{x\in A: y\not\succ x\;\; \text{for all $y\in A$}\}

.. centered:: and

.. math::
    x\sim y\;\; \text{for distinct}\; x,y\; \text{in}\; X

|

Sequentially Rationalizable Choice
----------------------------------

A general choice dataset `\mathcal{D}` on a set of alternatives `X` is explained by 
**sequentially rationalizable choice** if there exist 
two strict partial orders `\succ_1`, `\succ_2` on `X` such that for every menu 
`A` in `\mathcal{D}`

.. math::
    |C(A)| = 1\;\;\;\;\; \text{and}\;\;\;\;\; C(A) = M_{\succ_1}\Big(M_{\succ_2}(A)\Bigr)

where, for any `A\subseteq X`,

.. math::
	M_{\succ_i}(A) := \{x\in A: y\not\succ_i x\;\; \text{for all}\;\; y\in A\}.

   
.. tip::   
     Prest |version| supports only a **Pass/Fail** test for this model, with the corresponding output being "0" and ">0", respectively.
	
|

Top-Two Choice
--------------

A general choice dataset `\mathcal{D}` on a set of alternatives `X` is explained by 
**top-two choice** if there exists a strict linear order `\succ` on `X`
such that for every menu `A` in `\mathcal{D}`

.. math::
   |C(A)| = 2\;\;\;\;\; \text{and}\;\;\;\;\; C(A)=\{x,y\}\;\; \Longleftrightarrow\;\; x,y\succ z\;\; \text{for all}\;\; z\in A\setminus\{x,y\}

|
   
Incomplete-Preference Maximization: Partially Dominant Choice (forced)
----------------------------------------------------------------------

A general choice dataset `\mathcal{D}` on a set of alternatives `X` is explained by 
**partially dominant choice (forced)** if there exists a strict partial order `\succ` on `X`
such that for every menu `A` in `\mathcal{D}`


.. math::
   	\begin{array}{llc}
	C(A)=A & \Longleftrightarrow & x\nsucc y\;\; \text{and}\;\; y\nsucc x\;\;	\text{for all}\;\; x,y\in A\\
	& &\\
	C(A)\subset A & \Longleftrightarrow &  
	C(A)=
	\left\{
	\begin{array}{lll}
	& & \hspace{-12pt} z\nsucc x\qquad \text{for all}\;\; z\in A\\
	x\in A: & & \;\;\;\;\;\;\text{and}\\
	& & \hspace{-12pt} x\succ y\qquad \text{for some}\;\; y\in A
	\end{array}
	\right\}
	\end{array}
