.. _estimation:

Preference Estimation
=====================


In **general datasets** Prest estimates which deterministic model/heuristic in its 
toolkit is the best match for each agent. 

It does so by identifying "how far" each model/heuristic is from explaining 
that person's choices. 

In this process, Prest also recovers the agent's preferences *conditional* on this best-matching model/heuristic.

A model or heuristic's proximity to an agent's data is captured by its **distance score**.

This is the number of observations that need to be removed from an agent's data 
in order for the remaining observations to be fully compatible with the model in question. 

Prest also provides information about the compatible *instances* of every model that is optimal in 
the sense of having a minimum distance score. [#score]_

.. note::
     Since the release of Prest 1.1.0 users can also visualize this preference-estimation output and save it in .png format. 
     This service builds on `GraphViz <https://graphviz.org>`_ and requires GraphViz to be installed and its 'dot' binary file
     to be available in Prest's root directory. Users can then view and save the directed preference graph that corresponds to some instance
     of a model used in estimation by right-clicking and viewing the relevant model-estimation dataset in the workspace, locating the subject, model
     and instance of interest from the relevant list, and clicking on the question mark icon that appears next to the code label of that instance.

These estimation features allow users to test for the proximity of choice behaviour 
not only with the textbook model of rational choice/utility maximization but also with several 
models/heuristics that explain well-documented behavioural phenomena 
such as **context-dependent choices**, **cyclic choices**, **status-quo biased choices**, **choice deferral** and **choice overload**.


.. note::  
     By default, Prest's core program is designed to utilise all available computing power by simultaneously engaging all CPU cores.
     To change that, check the *"Disable parallelism"* box at the bottom of the *"Model estimation"* window.

.. note::  
     If your dataset includes observations where the deferral/outside option was chosen by some agent(s) and you wish to ignore
     these observations, you can do so by checking the *"Disregard deferrals"* box at the bottom of the *"Model estimation"* window.

.. note::  
     If your dataset includes observations where multiple items were chosen by some agent(s) in some menu(s), you can do model estimation 
     either by selecting either the *"Houtman-Maks"* (default) or the *"Houtman-Maks-Jaccard*" method from the *"Distance score"* scroll-down 
     menu on the bottom left of the *"Model estimation"* window. The two methods coincide when at most one alternative is chosen at 
     every menu in the dataset. When this is not the case, the second method is less punitive in its computation of the distance score 
     by accounting for the (Jaccard [#jaccard]_) (dis)similarity between the agent's actual choices at a menu and what would have been the model-optimal choices
     at that menu.

.. rubric::   Footnotes

.. [#score]   The idea of a model's *distance score* as described above was introduced by 
              :cite:authors:`houtman-maks85` (:cite:year:`houtman-maks85`), who 
              applied it to the model of rational choice / utility maximization using budgetary datasets.
              The extension of this idea to other models using general datasets was made in 
              :cite:authors:`CCGT22` (:cite:year:`CCGT22`).

.. [#jaccard] The Jaccard metric identifies the dissimilarity between two finite sets
              by the elements they have in common relative to their total number of 
              unique elements (see :cite:author:`levandowsky-winter71`, :cite:year:`levandowsky-winter71`).  
