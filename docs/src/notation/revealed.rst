Revealed Preference Relations
=============================

.. _revealed:

Revealed Preference in General Datasets
---------------------------------------

Consider a general dataset `\mathcal{D}=\left\{\big(A_i,C(A_i)\bigr)\right\}_{i=1}^k` and distinct choice alternatives `x,y` in `X`.

The following definitions and notation will be used for the 
different ways in which `x` may be **revealed preferred** to `y` in `\mathcal{D}`:

`x\succsim^R y` if there exists a menu `A_i` in `\mathcal{D}` such that `x\in C(A_i)` and `y\in A_i`.

`x\succ^R y`, if there exists a menu `A_i` in `\mathcal{D}` such that `x\in C(A_i)` and `y\in A_i\setminus C(A_i)` [i.e. `y\in A_i` and `y\not\in C(A_i)`].

`x\succsim^{\widehat{R}} y`, if there exists a sequence of menus 
`A_{(1)},\ldots, A_{(n)}` in `\mathcal{D}` and a sequence of alternatives `x_{(1)},\ldots,x_{(n)}` in `X` such that
`x=x_{(1)}`, `y=x_{(n)}` and `x_{(i)}\succsim^R x_{(i+1)}` for all `i=1,\ldots,n-1`.


`x\succ^{\widehat{R}} y`, if there exists a sequence of menus 
`A_{(1)},\ldots, A_{(n)}` in `\mathcal{D}` and a sequence of alternatives `x_{(1)},\ldots,x_{(n)}` in `X` such that
`x=x_{(1)}`, `y=x_{(n)}` and `x_{(i)}\succ^R x_{(i+1)}` for all `i=1,\ldots,n-1`.

`x\succsim^B y` if `x\in C(\{x,y\})`.

`x\succ^B y` if `\{x\}=C(\{x,y\})`.

`x\succsim^{\widehat{B}}y` if there is a sequence of alternatives `x_{(1)},\ldots,x_{(n)}\in X` such that `x=x_{(1)}`, `y=x_{(n)}`
and `x_{(i)}\succsim^B x_{(i+1)}` for all `i=1,\ldots,n-1`.


`x\succ^{\widehat{B}}y` if there is a sequence of alternatives `x_{(1)},\ldots,x_{(n)}\in X` such that `x=x_{(1)}`, `y=x_{(n)}`
and `x_{(i)}\succ^B x_{(i+1)}` for all `i=1,\ldots,n-1`.

Revealed Preference in Budgetary Datasets
-----------------------------------------

Consider a budgetary dataset `\mathcal{D}=\left\{(p^i,x^i)\right\}_{i=1}^k` 
and consumption bundles `x^i,x^j` in `\mathbb{R}^n_+` such that `i,j\leq k`.

The following definitions and notation will be used for the 
different ways in which `x^i` may be **revealed preferred** to `x^j` in `\mathcal{D}`:

`x^i\succsim^R x^j` if `p^ix^i\geq p^ix^j`.

`x^i\succ^R x^j` if `p^ix^i>p^ix^j`.

`x^i\succsim^{\widehat{R}} x^j` if there exist observations `(p^l,x^l),\ldots,(p^{l+n},x^{l+n})` in `\mathcal{D}` such that
`x^i=x^l`, `x^j=x^{l+n}` and `p^lx^l\geq p^lx^{l+1}`, `\ldots`, `p^{l+n-1}x^{l+n-1}\geq p^{l+n-1}x^{l+n}`.
