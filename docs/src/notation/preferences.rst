Preference Relations
====================

Let the set of choice alternatives being analysed be denoted by `X`.  


Weak Preferences
----------------

A binary relation `\succsim` on `X` is a **weak preference relation** if it satisfies

Reflexivity
...........
*For all* `x\in X`, `x\succsim x`.


The relation of **strict preference** that is derived from `\succsim` is defined by

.. math::
	x\succ y\;\; \text{if}\;\; x\succsim y\;\; \text{and}\;\; y\not\succsim x

The relation of **indifference** that is derived from `\succsim` is defined by

.. math::
	x\sim y\;\; \text{if}\;\; x\succsim y\;\; \text{and}\;\; y\succsim x

	
Additional properties that a weak preference relation may have are:

Completeness
............

*For all* `x,y\in X`, *either* `x\succsim y` *or* `y\succsim x`.



Transitivity
............

*For all* `x,y,z\in X`, `x\succsim y\succsim z` *implies* `x\succsim z`.


Preorder
........

`\succsim` *is reflexive and transitive*.


Weak Order
..........

`\succsim` *is complete and transitive*.


Incomplete Preorder
...................

`\succsim` *is reflexive and transitive and there exist* `x,y\in X` *such that* `x\not\succsim y` *and* `y\not\succsim x`.



Strict Preferences
------------------

If it is assumed that no two distinct alternatives are related by indifference, then a **strict preference relation** `\succ` on `X` is taken as primitive. 
Such a relation `\succ` satisfies:

Asymmetry
.........

*For all* `x,y\in X`, `x\succ y` *implies* `y\not\succ x`.

Additional properties that a strict preference relation `\succ` may have are:

Totality
........

*For all distinct* `x,y\in X`, either `x\succ y` or `y\succ x`.


Transitivity
............

*For all* `x,y,z\in X`, `x\succ y\succ z` *implies* `x\succ z`.


Strict Linear Order
...................

`\succ` *is asymmetric, total and transitive*.

Strict Partial Order
....................

`\succ` *is asymmetric and transitive and there exist distinct* `x,y\in X` *such that* `x\not\succ y` *and* `y\not\succ x`.
