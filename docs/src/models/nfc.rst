Non-Forced-Choice Models (feasible outside option)
==================================================

Utility Maximization with an Outside Option
-------------------------------------------

[:cite:authors:`gerasimou18`, :cite:year:`gerasimou18`]

Strict
......

A general choice dataset `\mathcal{D}` on a set of alternatives `X` is explained by
**(strict) utility maximization with an outside option** if
there is a strict linear order `\succ` on `X` and an alternative `x^*\in X` such
that for every menu `A` in `\mathcal{D}`

.. math::
    C(A) = \left\{
        \begin{array}{ll}
	        \mathcal{B}_{\succ}(A), & \text{if $x\succ x^*$ for $\{x\}= \mathcal{B}_\succ(A)$}\\
	    &\\
	    \emptyset, & \text{otherwise}\\
        \end{array}
    \right.

where 

.. math::
    \mathcal{B}_{\succ}(A):=\Big\{x\in A: x\succ y\; \text{for all $y\in A\setminus\{x\}$}\Bigr\}
	
is the strictly most preferred alternative in `A` according to `\succ`.



Non-strict
..........

A general choice dataset `\mathcal{D}` on a set of alternatives `X` is explained by
**(non-strict) utility maximization with an outside option** if
there is a weak order `\succsim` on `X` and an alternative `x^*\in X` such
that for every menu `A` in `\mathcal{D}`

.. math::
    C(A) = \left\{
        \begin{array}{ll}
	        \mathcal{B}_{\succsim}(A), & \text{if $x\succ x^*$ for all $x\in \mathcal{B}_\succsim(A)$}\\
	    &\\
	    \emptyset, & \text{otherwise}\\
        \end{array}
    \right.
	
.. centered:: and

.. math::
    x\sim y\;\; \text{for distinct}\; x,y\; \text{in}\; X
	
where 

.. math::
    \mathcal{B}_{\succsim}(A):=\{x\in A: x\succsim y\; \text{for all $y\in A$}\}
	
is the set of weakly most preferred alternatives in `A` according to `\succsim`.

|

Overload-Constrained Utility Maximization
-----------------------------------------

[:cite:authors:`gerasimou18`, :cite:year:`gerasimou18`]

Strict
......

A general choice dataset `\mathcal{D}` on a set of alternatives `X` is explained by
**(strict) overload-constrained utility maximization** if there is a strict linear order
`\succ` on `X` and an integer `n` such that for every menu `A` in `\mathcal{D}`

.. math:: 
	C(A) = &
	\left\{
	\begin{array}{ll}
	\mathcal{B}_{\succ}(A), & \text{if $|A|\leq n$}\\
	&\\
	\emptyset, &  \text{otherwise}
	\end{array}
	\right.

where 

.. math::
    \mathcal{B}_{\succ}(A):=\Big\{x\in A: x\succ y\; \text{for all $y\in A\setminus\{x\}$}\Bigr\}
	
is the strictly most preferred alternative in `A` according to `\succ`.
	
Non-strict
..........

A general choice dataset `\mathcal{D}` on a set of alternatives `X` is explained by
**(non-strict) overload-constrained utility maximization** if there is a weak order
`\succsim` on `X` and an integer `n` such that for every menu `A` in `\mathcal{D}`

.. math:: 
	C(A) = &
	\left\{
	\begin{array}{ll}
	\mathcal{B}_{\succsim}(A), & \text{if $|A|\leq n$}\\
	&\\
	\emptyset, &  \text{otherwise}
	\end{array}
	\right.

.. centered:: and

.. math::
    x\sim y\;\; \text{for distinct}\; x,y\; \text{in}\; X
	
where 

.. math::
    \mathcal{B}_{\succsim}(A):=\{x\in A: x\succsim y\; \text{for all $y\in A$}\}
	
is the set of weakly most preferred alternatives in `A` according to `\succsim`.

|

Incomplete-Preference Maximization: Maximally Dominant Choice
-------------------------------------------------------------

[:cite:authors:`gerasimou18`, :cite:year:`gerasimou18`]

Strict
......

A general choice dataset on a set of alternatives `X` is explained by
**(strict) maximally dominant choice** if there is a strict partial order
`\succ` on `X` such that for every menu `A` in `\mathcal{D}`

.. math::
    C(A) = \left\{
        \begin{array}{ll}
	        \mathcal{B}_{\succ}(A), & \text{if $\mathcal{B}_\succ(A)\neq\emptyset$}\\
	    &\\
	    \emptyset, & \text{otherwise}\\
        \end{array}
    \right.

where 

.. math::
    \mathcal{B}_{\succ}(A):=\Big\{x\in A: x\succ y\; \text{for all $y\in A\setminus\{x\}$}\Bigr\}
	
is the (possibly non-existing) strictly most preferred alternative in `A` according to `\succ`.


Non-strict
..........

A general choice dataset `\mathcal{D}` on a set of alternatives `X` is explained by
**(non-strict) maximally dominant choice** if there is an incomplete preorder
`\succsim` on `X` such that for every menu `A` in `\mathcal{D}`

.. math::
	C(A) =
	\left\{
        \begin{array}{ll}
	    \mathcal{B}_{\succsim}(A), & \text{if $\mathcal{B}_{\succsim}(A)\neq\emptyset$}\\
	    &\\
	    \emptyset, & \text{otherwise}\\
        \end{array}
    \right.

.. centered:: and

.. math::
    x\sim y\;\; \text{for distinct}\; x,y\; \text{in}\; X	

where 

.. math::
    \mathcal{B}_{\succsim}(A):=\{x\in A: x\succsim y\; \text{for all $y\in A$}\}
	
is the (possibly empty) set of the weakly most preferred alternatives in `A` according to `\succsim`.

|

Incomplete-Preference Maximization: Partially Dominant Choice (non-forced)
--------------------------------------------------------------------------

[:cite:authors:`gerasimou18`, :cite:year:`gerasimou16a`]

A general choice dataset `\mathcal{D}` on a set of alternatives `X` is explained by 
**partially dominant choice (non-forced)** if there exists a strict partial order `\succ` on `X`
such that for every menu `A` in `\mathcal{D}` with at least two alternatives


.. math::
   	\begin{array}{llc}
	C(A)=\emptyset & \Longleftrightarrow & x\nsucc y\;\; \text{and}\;\; y\nsucc x\;\;	\text{for all}\;\; x,y\in A\\
	& &\\
	C(A)\neq\emptyset & \Longleftrightarrow &  
	C(A)=
	\left\{
	\begin{array}{lll}
	& & \hspace{-12pt} z\nsucc x\qquad \text{for all}\;\; z\in A\\
	x\in A: & & \;\;\;\;\;\;\text{and}\\
	& & \hspace{-12pt} x\succ y\qquad \text{for some}\;\; y\in A
	\end{array}
	\right\}
	\end{array}
	
.. note::
     In its distance-score computation of this model, Prest penalizes deferral/choice of the outside option at singleton menus. 
     Although this is not a formal requirement of the model, its predictions at non-singleton menus are compatible with the assumption that all alternatives are desirable,
     and hence that active choices be made at all singletons.