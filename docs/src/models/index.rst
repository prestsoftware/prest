.. _estimation:

Non-Parametric Preference Estimation
====================================


For **general/non-budgetary datasets**, Prest can estimate non-parametrically which model (or models) in its 
current toolkit is the best match for a given subject in the dataset by identifying *"how far"* each model is from fully explaining 
that person's choices. 

Generalizing the Houtman-Maks [houtman-maks85]_ method in the *model-based* way suggested in [CCGT16]_,  Prest computes the **distance score** 
associated with every user-selected model for every subject in the dataset. This corresponds to the number of observations 
that need to be removed from a subject's data in order for the remaining choices to be fully compatible with the model in question. 
Prest also provides information about the compatible *instances* of every model that is optimal in this sense. 

These model- and preference-estimation features enable users to analyse the available data to test  for the proximity of choice behavior 
not only with utility maximization but also with several more general  models that provide explanations of well-documented behavioral phenomena 
such as *context-dependent choice*, *cyclic choice*, *status quo bias*, *choice deferral* and *choice overload*. 

.. toctree::
    fc
    nfc
    default
