===================
Models & Heuristics
===================

----------------------------
Utility Maximization [#um]_
----------------------------

A general choice dataset `\mathcal{D}` on a set of alternatives `X` is explained by
**strict utility maximization** if there is a strict linear
order `\succ` on `X` such that for every menu `A` in `\mathcal{D}`

.. math::
	C(A) = \Big\{x\in A: x\succ y\;\; \text{for all $y\in A\setminus\{x\}$}\Big\} 

It is explained by
**non-strict utility maximization** if there is a weak order
`\succsim` on `X` such that for every menu `A` in `\mathcal{D}`

.. math::
    C(A) = \{x \in A: x\succsim y\;\; \text{for all $y\in A$}\}

.. centered:: and


.. math::
    x\sim y\;\; \textnormal{for distinct}\; x,y\; \text{in}\; X.
    
|

.. tip::  
     When analysing other models that generalize Utility Maximization, Prest only considers instances of the more general 
     models that do not overlap with those covered by the above two variants of Utility Maximization.
     It is therefore recommended that Utility Maximization always be included in all model-estimation tasks.

.. tip::  
     When "Utility Maximization - Swaps" is selected in the *Model estimation* window, Prest computes the "Swaps" index 
     that is analyzed in :cite:authors:`apesteguia-ballester15` :cite:yearpar:`apesteguia-ballester15`.
     
     *Note:* this is only possible for datasets with nonempty and single choices at every menu.

|

-----------------------------------------------------
Utility Maximization with an Outside Option [#umoo]_
-----------------------------------------------------

A general choice dataset `\mathcal{D}` on a set of alternatives `X` is explained by
**strict utility maximization with an outside option** if
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

It is explained by
**non-strict utility maximization with an outside option** if
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

------------------------------------------------------
Undominated Choice with Incomplete Preferences [#uc]_
------------------------------------------------------

A general choice dataset on a set of alternatives `X` is explained by
**strict undominated choice** if there is a strict
partial order `\succ` on `X` such that for every menu `A` in `\mathcal{D}`

.. math::
	C(A) = \{x\in A: y\not\succ x\;\; \text{for all $y\in A$}\}

It is explained by
**non-strict undominated choice** if there is an incomplete preorder `\succsim` on `X`
whose asymmetric part is `\succ` such that the above holds for every menu `A` in `\mathcal{D}`
**and**

.. math::
    x\sim y\;\; \textnormal{for distinct}\; x,y\; \text{in}\; X

.. note::  
     If a dataset is explained by *non-strict* undominated choice under some preorder `\succsim`
     with asymmetric and symmetric parts `\succ` and `\sim` where `x\sim y` is true for 
     distinct alternatives `x` and `y`, then it is also explained by
     *strict* undominated choice under strict partial order `\succ` where `x\nsucc y\nsucc x`
     for all such `x` and `y`. The converse is not true in general. 
     


|

----------------------------------------------------------------------------------------
Status-Quo-Biased Undominated Choice with Incomplete Preferences (Bewley model) [#bew]_
----------------------------------------------------------------------------------------

A general dataset with default/status quo alternatives `\mathcal{D}` is explained by **status-quo-biased undominated choice (Bewley model)** if 
there exists a strict partial order `\succ` on `X` such that for every decision problem `(A,s)` in `\mathcal{D}`

.. math::
	\begin{array}{llc}
	C(A,s)=\{s\} & \Longleftrightarrow & \text{$x\nsucc s$ for all $x\in A$}\\
	& &\\
	C(A,s)\neq \{s\} &\Longleftrightarrow & C(A,s)=
	\left\{
	\begin{array}{lc}
	& z\nsucc x\; \text{for all $z\in A$}\\
	x\in A:  &\text{and}\\
	& x\succ s
	\end{array}
	\right\}
	\end{array}

|

------------------------------
Rational Shortlisting [#rsm]_ 
------------------------------

(experimental implementation)
-----------------------------

A general choice dataset `\mathcal{D}` on a set of alternatives `X` is explained by   
**rational shortlisting** if there exist 
two strict partial orders `\succ_1`, `\succ_2` on `X` such that for every menu 
`A` in `\mathcal{D}`

.. math::
    |C(A)| = 1\;\;\;\;\; \text{and}\;\;\;\;\; C(A) = M_{\succ_1}\Big(M_{\succ_2}(A)\Bigr)

where, for any `A\subseteq X`,

.. math::
	M_{\succ_i}(A) := \{x\in A: y\not\succ_i x\;\; \text{for all}\;\; y\in A\}.

   
.. tip::   
     Prest currently supports only a **Pass/Fail** test for this model, with distance score output "0" and ">0", respectively.

|

----------------------------------------------------
Dominant Choice with Incomplete Preferences [#dom]_
----------------------------------------------------

A general choice dataset on a set of alternatives `X` is explained by
**strict dominant choice with incomplete preferences** if there is a strict partial order
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

It is explained by
**non-strict dominant choice with incomplete preferences** if there is an incomplete preorder
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

--------------------------------------------------------------------------------
Partially Dominant Choice with Incomplete Preferences (forced-choice) [#pdcfc]_
--------------------------------------------------------------------------------

A general choice dataset `\mathcal{D}` on a set of alternatives `X` is explained by 
**partially dominant choice with incomplete preferences (forced-choice variant)** if 
there exists a strict partial order `\succ` on `X`
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

-----------------------------------------------------------------------------
Partially Dominant Choice with Incomplete Preferences (free-choice) [#pdca]_
-----------------------------------------------------------------------------

A general choice dataset `\mathcal{D}` on a set of alternatives `X` is explained by 
**partially dominant choice with incomplete preferences (free-choice variant)** if there 
exists a strict partial order `\succ` on `X`
such that for every menu `A` in `\mathcal{D}` with at least two alternatives

.. math::
   	\begin{array}{llc}
	C(A)=\emptyset & \Longleftrightarrow & x\nsucc y\;\; \text{and}\;\; y\nsucc x\;\; \text{for all}\;\; x,y\in A\\
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

|

---------------------------------------------------
Overload-Constrained Utility Maximization [#over]_
---------------------------------------------------

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
	
It is explained by **non-strict overload-constrained utility maximization** if there is a weak order
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

.. rubric::   Footnotes

.. [#um]  The revealed-preference implications of this model have been studied extensively. Some important references include 
          :cite:authors:`samuelson38` (:cite:year:`samuelson38`), :cite:authors:`houthakker50` (:cite:year:`houthakker50`),  
          :cite:authors:`uzawa56` (:cite:year:`uzawa56`), :cite:authors:`arrow59` (:cite:year:`arrow59`),  
          :cite:authors:`richter66` (:cite:year:`richter66`) and the monograph by :cite:authors:`chambers-echenique16` (:cite:year:`chambers-echenique16`).
          
.. [#umoo] The model was formulated and analysed in this way in :cite:author:`gerasimou18` (:cite:year:`gerasimou18`, Section 3).
          If choice of the deferral/outside option at menu `A` is not captured as `C(A)=\emptyset` in the dataset but, instead, 
          as `C(A)=o` for some alternative `o` that is feasible in *every* menu, then this model can be tested in Prest via 
          the Utility Maximization model presented above. Encoding choice of the deferral/outside option as `C(A)=\emptyset` 
          gives the user more flexibility because they can also test the dataset against some of the other models below 
          where the `C(A)=o` way of encoding that choice is no longer equivalent to `C(A)=\emptyset`.   

.. [#uc] See, among others, :cite:authors:`schwartz76` (:cite:year:`schwartz76`), :cite:authors:`bossert-sprumont-suzumura05` (:cite:year:`bossert-sprumont-suzumura05`) and  
         :cite:authors:`eliaz-ok06` (:cite:year:`eliaz-ok06`).

.. [#bew] See :cite:authors:`bewley02` (:cite:year:`bewley02`), :cite:authors:`mandler04` (:cite:year:`mandler04`), 
          :cite:authors:`masatlioglu-ok05` (:cite:year:`masatlioglu-ok05`) and, for this specific formulation, 
          :cite:authors:`gerasimou16a` (:cite:year:`gerasimou16a`).

.. [#rsm] See :cite:authors:`manzini-mariotti07` (:cite:year:`manzini-mariotti07`), :cite:authors:`dutta-horan15` (:cite:year:`dutta-horan15`),  
          :cite:authors:`declippel-rozen21` (:cite:year:`declippel-rozen21`) and :cite:authors:`declippel-rozen24` (:cite:year:`declippel-rozen24`).

.. [#dom]       See :cite:authors:`gerasimou18` (:cite:year:`gerasimou18`, Section 2).

.. [#pdcfc]    See :cite:authors:`gerasimou16b` (:cite:year:`gerasimou16b`) and :cite:authors:`qin17` (:cite:year:`qin17`).

.. [#pdca]   See :cite:authors:`gerasimou16a` (:cite:year:`gerasimou16a`).

.. [#over]      See :cite:authors:`gerasimou18` (:cite:year:`gerasimou18`, Section 4).





          


          
