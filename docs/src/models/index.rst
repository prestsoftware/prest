.. _estimation:

Preference Estimation
=====================


For **general/non-budgetary datasets**, Prest can estimate non-parametrically which model(s) in its 
toolkit is the best match for a given subject in the dataset by identifying "how far" each model is from fully explaining 
that person's choices. 

Generalizing the :cite:authors:`houtman-maks85` :cite:yearpar:`houtman-maks85` method 
in the *model-based* way that was first suggested in :cite:authors:`CCGT22` :cite:yearpar:`CCGT22`,  
Prest computes the **distance score** 
associated with every user-selected model for every subject in the dataset. 

This score corresponds to the number of observations 
that need to be removed from a subject's data in order for the remaining observations to be fully compatible with the model in question. 

Prest also provides information about the compatible *instances* of every model that is optimal in this sense. 

These model- and preference-estimation features allow you to analyse the available data to test for the proximity of choice behaviour 
not only with utility maximization but also with several models of general choice that explain well-documented behavioural phenomena 
such as **context-dependent choices**, **cyclic choices**, **status-quo biased choices**, **choice deferral** and **choice overload**.

.. toctree::
    fc
    nfc
    default
