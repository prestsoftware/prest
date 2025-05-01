===================
Models & Heuristics
===================

.. note::  
     | The list of models and heuristics that are currently implemented in Prest and are presented here is far from being exhaustive.
     | Some have been studied in the literature extensively. 
     | Others represent more recent developments in the field. 
     | The inclusion/implentation of the latter has been guided by computational feasibility considerations, and by the developers' own research interests and expertise.
     | With the exception of Utility Maximization that appears first, the presentation order does not reflect any preference or priority.

.. note::  
     Unless otherwise stated, the dataset `\mathcal{D}` here is general, defined on an underlying set `X`, and without default/status quo options.

.. _um2:

---------------------------
Utility Maximization [#um]_
---------------------------

There is a weak order `\succsim` on `X` such that for every menu `A` in `\mathcal{D}`

.. math:: C(A) = \{x\in A: x\succsim y\;\; \text{for all $y\in A$}\} 

Prest allows for testing either/both special cases of this model:

* | **Utility Maximization (Strict)**: there are *no* distinct alternatives `x` and `y` such that `x\sim y`.
* | **Utility Maximization (Non-Strict)**: there are *some* distinct alternatives `x` and `y` such that `x\sim y`.

|

.. tip::  
     When analysing other models that generalize Utility Maximization, Prest only considers instances of the more general 
     models that do not overlap with those covered by the above two variants of Utility Maximization.
     It is therefore recommended that Utility Maximization always be included in all model-estimation tasks.

.. tip::  
     When "Utility Maximization - Swaps" is selected in the *Model estimation* window, Prest computes the "Swaps" index 
     of :cite:authors:`apesteguia-ballester15` :cite:yearpar:`apesteguia-ballester15`. 
     *Note:* this is only possible for datasets with nonempty and single choices at every menu.

.. _umoo2:

-----------------------------------------------------
Utility Maximization with an Outside Option [#umoo]_
-----------------------------------------------------

There is a weak order `\succsim` on `X` and 
some "acceptability-threshold" alternative `x^*\in X` such that for every menu `A` in `\mathcal{D}`

.. math::
   \begin{array}{lllll}
      C(A) & = & \{x\in A: x\succsim y\;\; \text{for all $y\in A$}\} & \Longleftrightarrow & z\succ x^* \text{ for some } z\in A \\
      C(A) & = & \emptyset & \Longleftrightarrow & x^*\succsim z \text{ for all } z\in A
   \end{array}

Prest allows for testing either/both special cases of this model:

* | **Utility Maximization with an Outside Option (Strict)**: there are *no* distinct alternatives `x` and `y` such that `x\sim y`.
* | **Utility Maximization with an Outside Option (Non-Strict)**: there are *some* distinct alternatives `x` and `y` such that `x\sim y`.

.. _uc2:

------------------------------------------------------
Undominated Choice with Incomplete Preferences [#uc]_
------------------------------------------------------

There is a strict partial order `\succ` on `X` such that for every menu `A` in `\mathcal{D}`

.. math::
	C(A) = \{x\in A: y\not\succ x\;\; \text{for all $y\in A$}\}

Prest allows for testing the following general and special versions of this model:

* | **Undominated Choice with Incomplete Preferences (Strict)**: the relation `\succ` is not the asymmetric part of a preorder `\succsim` on `X`.
* | **Undominated Choice with Incomplete Preferences (Non-Strict)**: the relation `\succ` is the asymmetric part of a preorder `\succsim` on `X` and there are *some* distinct alternatives `x` and `y` such that `x\sim y`.

.. note::  
     If a dataset is explained by *non-strict* undominated choice under some preorder `\succsim`
     with asymmetric and symmetric parts `\succ` and `\sim` where `x\sim y` is true for 
     distinct alternatives `x` and `y`, then it is also explained by
     *strict* undominated choice under strict partial order `\succ` where `x\nsucc y\nsucc x`
     for all such `x` and `y`. The converse is not true in general. 
     
.. _bew2:

---------------------------------------------------------------------------------
Status-Quo-Biased Undominated Choice with Incomplete Preferences (Bewley) [#bew]_
---------------------------------------------------------------------------------

A general dataset with default/status quo alternatives `\mathcal{D}` is explained by this model if
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

.. _rsm2:

------------------------------
Rational Shortlisting [#rsm]_ 
------------------------------

| [experimental implementation] 
| There are two strict partial orders `\succ_1`, `\succ_2` on `X` such that for every menu `A` in `\mathcal{D}`

.. math::
   \begin{array}{llll}
      |C(A)| & = & 1 \\ 
      C(A) & = & M_{\succ_1}\Big(M_{\succ_2}(A)\Bigr) & 
   \end{array}

where

.. math::
	M_{\succ_i}(A) := \{x\in A: y\not\succ_i x\;\; \text{for all}\;\; y\in A\} 

is the set of undominated alternatives in `A` according to `\succ_i` and `|C(A)|=1` means that `C(A)` contains exactly one alternative.

   
.. tip::   
     Prest currently supports only a **Pass/Fail** test for this model, with distance score output "0" and ">0", respectively.

.. _dom2:

----------------------------------------------------
Dominant Choice with Incomplete Preferences [#dom]_
----------------------------------------------------

There is an incomplete preorder `\succsim` on `X` such that for every menu `A` in `\mathcal{D}`

.. math:: C(A) = \{x\in A: x\succsim y\;\; \text{for all $y\in A$}\} 

In particular, `C(A) = \emptyset` `\Longleftrightarrow` for all `x\in A` there is `y_x\in A` such that `x\not\succsim y_x`.

Prest allows for testing either/both special cases of this model:

* | **Dominant Choice with Incomplete Preferences (Strict)**: there are *no* distinct alternatives `x` and `y` such that `x\sim y`.
* | **Dominant Choice with Incomplete Preferences (Non-Strict)**: there are *some* distinct alternatives `x` and `y` such that `x\sim y`.

.. _pdcfc2:

-----------------------------------------------------
Partially Dominant Choice with Incomplete Preferences
-----------------------------------------------------

Forced-Choice version [#pdcfc]_
................................

There is a strict partial order `\succ` on `X` such that for every menu `A` in `\mathcal{D}`

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

.. _pdca2:

Free-Choice version [#pdca]_
............................

There is a strict partial order `\succ` on `X` such that for every menu `A` in `\mathcal{D}` with at least two alternatives

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
     In its distance-score computation of the free-choice version of this model, Prest penalizes deferral/choice of the outside option at singleton menus. 
     Although this is not a formal requirement of the model, its predictions at non-singleton menus are compatible with the assumption that all alternatives are desirable,
     and hence that active choices be made at all singletons.

.. _over2:

--------------------------------------------------
Overload-Constrained Utility Maximization [#over]_
--------------------------------------------------

There is a weak order `\succsim` on `X` and an integer `n` such that for every menu `A` in `\mathcal{D}`

.. math::
   \begin{array}{lllll}
      C(A) & = & \{x\in A: x\succsim y\;\; \text{for all $y\in A$}\} & \Longleftrightarrow & |A|\leq n \\
      C(A) & = & \emptyset & \Longleftrightarrow & |A|>n
   \end{array}

Prest allows for testing either/both special cases of this model:

* | **Overload-Constrained Utility Maximization (Strict)**: there are *no* distinct alternatives `x` and `y` such that `x\sim y`.
* | **Overload-Constrained Utility Maximization (Non-Strict)**: there are *some* distinct alternatives `x` and `y` such that `x\sim y`.

|

.. rubric::   Footnotes

.. [#um]  This is the textbook economic model of rational choice. Its revealed-preference implications have been studied extensively. 
          Some important references include 
          :cite:authors:`samuelson38` (:cite:year:`samuelson38`), :cite:authors:`houthakker50` (:cite:year:`houthakker50`),  
          :cite:authors:`uzawa56` (:cite:year:`uzawa56`), :cite:authors:`arrow59` (:cite:year:`arrow59`),  
          :cite:authors:`richter66` (:cite:year:`richter66`) and the monograph by :cite:authors:`chambers-echenique16` (:cite:year:`chambers-echenique16`).
          
.. [#umoo] The model was formulated and analysed in this way in :cite:author:`gerasimou18` (:cite:year:`gerasimou18`, Section 3).
          If choice of the deferral/outside option at menu `A` is not captured as `C(A)=\emptyset` in the dataset but, instead, 
          as `C(A)=o` for some alternative `o` that is feasible in *every* menu, then this model can be tested in Prest via 
          the Utility Maximization model presented above. Encoding choice of the deferral/outside option as `C(A)=\emptyset` 
          gives the user more flexibility because they can also test the dataset against some of the other models below 
          where the `C(A)=o` way of encoding that choice is no longer equivalent to `C(A)=\emptyset`.   

.. [#uc] See, among others, :cite:authors:`schwartz76` (:cite:year:`schwartz76`), :cite:authors:`bossert-sprumont-suzumura05` (:cite:year:`bossert-sprumont-suzumura05`),   
         :cite:authors:`eliaz-ok06` (:cite:year:`eliaz-ok06`) and :cite:authors:`ribeiro-riella17` (:cite:year:`ribeiro-riella17`).

.. [#bew] See :cite:authors:`bewley02` (:cite:year:`bewley02`), :cite:authors:`mandler04` (:cite:year:`mandler04`), 
          :cite:authors:`masatlioglu-ok05` (:cite:year:`masatlioglu-ok05`) and, for this specific formulation, 
          :cite:authors:`gerasimou16a` (:cite:year:`gerasimou16a`).

.. [#rsm] See :cite:authors:`manzini-mariotti07` (:cite:year:`manzini-mariotti07`), :cite:authors:`dutta-horan15` (:cite:year:`dutta-horan15`),  
          :cite:authors:`declippel-rozen21` (:cite:year:`declippel-rozen21`) and :cite:authors:`declippel-rozen24` (:cite:year:`declippel-rozen24`).

.. [#dom] See :cite:authors:`gerasimou18` (:cite:year:`gerasimou18`, Section 2).

.. [#pdcfc] See :cite:authors:`gerasimou16b` (:cite:year:`gerasimou16b`) and :cite:authors:`qin17` (:cite:year:`qin17`).

.. [#pdca] See :cite:authors:`gerasimou16a` (:cite:year:`gerasimou16a`).

.. [#over] See :cite:authors:`gerasimou18` (:cite:year:`gerasimou18`, Section 4).