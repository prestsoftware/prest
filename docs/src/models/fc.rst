Forced-Choice Models (no outside option)
========================================

Utility Maximization / Rational Choice
--------------------------------------

[:cite:authors:`samuelson38`, :cite:year:`samuelson38`; :cite:authors:`houthakker50`, :cite:year:`houthakker50`; :cite:authors:`uzawa56`, :cite:year:`uzawa56`; :cite:authors:`arrow59`, :cite:year:`arrow59`; :cite:authors:`richter66`, :cite:year:`richter66`; :cite:authors:`chambers-echenique16`, :cite:year:`chambers-echenique16`]

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
    
|

.. tip::  
     When analysing other models that generalize utility maximization/rational choice, 
     Prest only considers instances of the more general models that do not overlap with those covered by the above two variants of utility maximization.
     It is therefore recommended that utility maximization/rational choice always be included in all model-estimation tasks.

.. tip::  
     When "Utility Maximization - Swaps" is selected, Prest computes the "Swaps" index 
     that is analyzed in :cite:authors:`apesteguia-ballester15` :cite:yearpar:`apesteguia-ballester15`.
     
     *Note:* this is only possible for forced- and single-valued choice datasets.

  
Incomplete-Preference Maximization: Undominated Choice
------------------------------------------------------

[:cite:authors:`schwartz76`, :cite:year:`schwartz76`; :cite:authors:`bossert-sprumont-suzumura05`, :cite:year:`bossert-sprumont-suzumura05`; :cite:authors:`eliaz-ok06`, :cite:year:`eliaz-ok06`]


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

Incomplete-Preference Maximization: Partially Dominant Choice (forced)
----------------------------------------------------------------------

[:cite:authors:`gerasimou16b`, :cite:year:`gerasimou16b`; :cite:authors:`qin17`, :cite:year:`qin17`]

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

|

Top-Two Choice
-------------- 

[:cite:authors:`eliaz-richter-rubinstein11`, :cite:year:`eliaz-richter-rubinstein11`]

A general choice dataset `\mathcal{D}` on a set of alternatives `X` is explained by 
**top-two choice** if there exists a strict linear order `\succ` on `X`
such that for every menu `A` in `\mathcal{D}`

.. math::
    |C(A)| = 2\;\;\;\;\; \text{and}\;\;\;\;\; C(A)=\{x,y\}\;\; \Longleftrightarrow\;\; x,y\succ z\;\; \text{for all}\;\; z\in A\setminus\{x,y\}

|


Sequentially Rationalizable Choice
----------------------------------

[:cite:authors:`manzini-mariotti07`, :cite:year:`manzini-mariotti07`; :cite:authors:`dutta-horan15`, :cite:year:`dutta-horan15`; :cite:authors:`declippel-rozen16`, :cite:year:`declippel-rozen16`]

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
     Prest currently supports only a **Pass/Fail** test for this model, with the output being "0" and ">0", respectively.
	