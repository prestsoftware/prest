use std::collections::HashSet;
use crate::model::{self,Penalty,Model,Instance,PreorderParams,DistanceScore};
use crate::precomputed::{self,Precomputed};
use std::result;
use std::fmt;
use std::convert::From;
use std::io::{Read,Write};
use crate::common::{Subject,ChoiceRow};
use crate::codec::{self,Encode,Decode,Packed};
use std::iter::FromIterator;
use rayon::prelude::*;
use num_rational::Ratio;
use num_traits::identities::Zero;

pub type Result<T> = result::Result<T, EstimationError>;

#[derive(Debug)]
pub enum EstimationError {
    InstanceError(model::InstanceError),
    PreorderError(precomputed::Error),
}

impl Encode for EstimationError {
    fn encode<W : Write>(&self, f : &mut W) -> codec::Result<()> {
        match self {
            &EstimationError::InstanceError(ref e) => (0u8, e).encode(f),
            &EstimationError::PreorderError(ref e) => (1u8, e).encode(f),
        }
    }
}

impl fmt::Display for EstimationError {
    fn fmt(&self, f : &mut fmt::Formatter) -> fmt::Result {
        match self {
            &EstimationError::InstanceError(ref e) => e.fmt(f),
            &EstimationError::PreorderError(ref e) => e.fmt(f),
        }
    }
}

impl From<model::InstanceError> for EstimationError {
    fn from(e : model::InstanceError) -> EstimationError {
        EstimationError::InstanceError(e)
    }
}

impl From<precomputed::Error> for EstimationError {
    fn from(e : precomputed::Error) -> EstimationError {
        EstimationError::PreorderError(e)
    }
}

#[derive(Debug, Clone)]
pub struct Request {
    subjects : Vec<Packed<Subject>>,
    models : Vec<model::Model>,
    disable_parallelism : bool,
    disregard_deferrals : bool,
    distance_score : DistanceScore,
}

impl Decode for Request {
    fn decode<R : Read>(f : &mut R) -> codec::Result<Request> {
        Ok(Request {
            subjects: Decode::decode(f)?,
            models: Decode::decode(f)?,
            disable_parallelism: Decode::decode(f)?,
            disregard_deferrals: Decode::decode(f)?,
            distance_score: Decode::decode(f)?,
        })
    }
}

// fields public for testing
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct InstanceInfo {
    pub model : Model,
    pub penalty : Penalty,
    pub instance : Vec<u8>,
}

impl InstanceInfo {
    pub fn from(model : Model, penalty : Penalty, inst : &Instance) -> Self {
        InstanceInfo {
            model,
            penalty,
            instance: codec::encode_to_memory(inst).unwrap(),
        }
    }
}

impl Encode for InstanceInfo {
    fn encode<W : Write>(&self, f : &mut W) -> codec::Result<()> {
        self.model.encode(f)?;
        self.penalty.encode(f)?;
        self.instance.encode(f)
    }
}

pub struct Response {
    pub subject_name : String,
    pub score : Penalty,
    pub best_instances : Vec<InstanceInfo>,
}

impl Encode for Response {
    fn encode<W : Write>(&self, f : &mut W) -> codec::Result<()> {
        (&self.subject_name, self.score.clone(), &self.best_instances).encode(f)
    }
}

struct BestInstances {
    lowest_penalty : Option<Penalty>,
    instances : HashSet<InstanceInfo>,
}

impl BestInstances {
    fn new() -> Self {
        BestInstances {
            lowest_penalty: None,
            instances: HashSet::new(),
        }
    }

    fn upper_bound_for(&self, model : Model) -> Option<Ratio<u32>> {
        self.instances.iter().filter_map(|inst|
            if inst.model == model {
                Some(inst.penalty.upper_bound)
            } else {
                None
            }
        ).max()
    }

    fn add_instance(&mut self, model : Model, this_penalty : Penalty, instance : Instance) {
        if let Some(ref mut lowest_penalty) = self.lowest_penalty {
            if this_penalty.upper_bound < lowest_penalty.lower_bound {
                // yay! we're strictly better
                self.instances.clear();
                self.instances.insert(InstanceInfo::from(model, this_penalty.clone(), &instance));
                *lowest_penalty = this_penalty;
            } else if this_penalty.lower_bound > lowest_penalty.upper_bound {
                // we're strictly worse, forget this instance
            } else {
                // we're neither better nor worse, we have to keep this
                lowest_penalty.merge_min(&this_penalty);
                self.instances.insert(InstanceInfo::from(model, this_penalty, &instance));

                // lowest_penalty.merge_min() may decrease the upper bound of lowest_penalty
                // this may make some instances in best_instances obsolete --
                // their lower bounds may become higher than the upper bound of lowest_penalty
                // we don't care for now; we'll filter them out at the end of this function
                //
                // if this turns out to be too inefficient, we can:
                // 1. keep the instances in a reverse (greatest-first) heap by lower bound
                // 2. keep them in a binary search tree, again by lower bound
                // both would allow us efficient pruning of obsolete instances
            }
        } else {
            // no instances yet
            self.instances.clear();  // not necessary but let's do it anyway
            self.instances.insert(InstanceInfo::from(model, this_penalty.clone(), &instance));
            self.lowest_penalty = Some(this_penalty);
        }
    }

    fn finish(self) -> Option<(Vec<InstanceInfo>, Penalty)> {  
        match self.lowest_penalty { Some(lowest_penalty) => {
            Some((
                Vec::from_iter(
                    self.instances.into_iter().filter(
                        // pick only those instances that overlap with the best estimate
                        |i| i.penalty.lower_bound <= lowest_penalty.upper_bound
                    )
                ),
                lowest_penalty,
            ))
        } _ => {
            // no instances
            None
        }}
    }

    fn combine(self, other : BestInstances) -> BestInstances {
        match (self, other) {
            (BestInstances{
                lowest_penalty: Some(self_penalty),
                instances: self_instances,
             },
             BestInstances{
                lowest_penalty: Some(other_penalty),
                instances: other_instances,
             },
            ) => {
                if self_penalty.upper_bound < other_penalty.lower_bound {
                    // self is strictly better
                    BestInstances{
                        lowest_penalty: Some(self_penalty),
                        instances: self_instances,
                    }
                } else if self_penalty.lower_bound > other_penalty.upper_bound {
                    // other is strictly better
                    BestInstances{
                        lowest_penalty: Some(other_penalty),
                        instances: other_instances,
                    }
                } else {
                    // neither is strictly better
                    let mut penalty = self_penalty;
                    penalty.merge_min(&other_penalty);

                    // usable instances from self
                    let mut instances = HashSet::from_iter(
                        self_instances.into_iter().filter(
                            |i| i.penalty.lower_bound <= penalty.upper_bound
                        )
                    );

                    // usable instances from other
                    instances.extend(
                        other_instances.into_iter().filter(
                            |i| i.penalty.lower_bound <= penalty.upper_bound
                        )
                    );

                    BestInstances{
                        lowest_penalty: Some(penalty),
                        instances,
                    }
                }
            }

            (BestInstances{ lowest_penalty: None, .. }, other) => other,
            (xself, BestInstances{ lowest_penalty: None, .. }) => xself,
        }
    }
}

fn evaluate_model(
    precomputed : &Precomputed,
    distance_score : DistanceScore,
    model : Model,
    alt_count : u32,
    choices : &[ChoiceRow],
) -> Result<BestInstances> {
    let mut model_instances = BestInstances::new();

    model::traverse_all(precomputed, model, alt_count, choices, &mut |inst| {
        model_instances.add_instance(
            model,
            inst.penalty(distance_score, choices),
            inst,
        )
    })?;

    Ok(model_instances)
}

pub fn run_one(
    precomputed : &Precomputed, distance_score : DistanceScore,
    subject : &Subject, models : &[Model],
) -> Result<Response> {
    let alt_count = subject.alternatives.len() as u32;

    let mut best_instances = BestInstances::new();
    for &model in models {
        // do SRC only if UM/UC do not rationalise perfectly
        if model == Model::SequentiallyRationalizableChoice {
            continue;
        }

        best_instances = best_instances.combine(
            evaluate_model(precomputed, distance_score, model, alt_count, &subject.choices)?
        );
    }

    // evaluate SRC only if:
    // 1) it was chosen by the user
    if models.contains(&Model::SequentiallyRationalizableChoice)
        // 2) and there's no perfect rationalisation by UC
        && best_instances.upper_bound_for(Model::UndominatedChoice{strict:true}) != Some(Zero::zero())
        // 3) and there's no perfect rationalisation by UM, either
        && best_instances.upper_bound_for(Model::PreorderMaximization(
            PreorderParams{strict: Some(true), total: Some(true)}
        )) != Some(Zero::zero())
        // Note that we don't need to check non-strict models
        // because if non-strict models rationalise perfectly
        // then we have a multi-choice somewhere,
        // which breaks the requirements of SRC (and won't get 0).
    {
        best_instances = best_instances.combine(
            evaluate_model(
                precomputed,
                distance_score,
                Model::SequentiallyRationalizableChoice,
                alt_count,
                &subject.choices
            )?
        );
    }

    let (mut best_instances, score) = best_instances.finish().unwrap();
    best_instances.sort();

    Ok(Response {
        subject_name: subject.name.clone(),
        best_instances,
        score,
    })
}

pub fn run(precomputed : &mut Precomputed, request : &Request) -> Result<Vec<Packed<Response>>> {
    // precompute up to the maximum number of alternatives
    let alt_count = request.subjects.iter().map(
        |subj| subj.unpack().alternatives.len() as u32
    ).max().expect("zero subjects in request");

    // don't precompute if searching only permutations (strict UM)
    if request.models != &[Model::PreorderMaximization(PreorderParams{strict:Some(true),total:Some(true)})] {
        precomputed.precompute(alt_count)?;
    }

    let results : Vec<Result<Response>> = if request.disable_parallelism {
        // run estimation sequentially
        request.subjects.iter().map(
            |subj| run_one(
                precomputed,
                request.distance_score,
                &subj.unpack().drop_deferrals(request.disregard_deferrals),
                &request.models,
            )
        ).collect()
    } else {
        // run estimation in parallel
        let mut results = Vec::new();
        request.subjects.par_iter().map(
            |subj| run_one(
                precomputed,
                request.distance_score,
                &subj.unpack().drop_deferrals(request.disregard_deferrals),
                &request.models
            )
        ).collect_into_vec(&mut results);
        results
    };

    // collect results
    let mut responses = Vec::with_capacity(results.len());
    for result in results.into_iter() {
        responses.push(Packed(result?));
    }

    Ok(responses)
}

#[cfg(test)]
mod test {
    use crate::precomputed::Precomputed;
    use crate::model;
    use crate::preorder;
    use crate::fast_preorder;
    use base64::Engine;
    use base64::prelude::BASE64_STANDARD;
    use crate::model::{Instance,Penalty,DistanceScore};
    use crate::codec;
    use crate::alt_set::AltSet;
    use crate::alt::Alt;
    use crate::common::{ChoiceRow,Subject};
    use std::iter::FromIterator;

    fn testsubj(alt_count : u32, choices : Vec<ChoiceRow>) -> Subject {
        Subject{
            name: String::from("subject"),
            alternatives: (0..alt_count).map(|s| s.to_string()).collect(),
            choices,
        }
    }

    #[test]
    fn top_two() {
        use model::Model;

        let choices = choices![
            [0,1,2,3] -> [0,1],
            [0,1,2] -> [0,1],
            [0,1,3] -> [0,1],
            [0,2,3] -> [0,2],
            [1,2,3] -> [1,2],
            [0,1] -> [0,1],
            [0,2] -> [0,2],
            [0,3] -> [0,3],
            [1,2] -> [1,2],
            [1,3] -> [1,3],
            [2,3] -> [2,3]
        ];

        let subject = testsubj(4, choices);
        let models = [Model::TopTwo];
        let mut precomputed = Precomputed::new(None);
        precomputed.precompute(4).unwrap();
        let response = super::run_one(&precomputed, DistanceScore::HoutmanMaks, &subject, &models).unwrap();

        assert_eq!(response.score, Penalty::exact(0));
        assert_eq!(response.best_instances.len(), 2);
    }

    #[test]
    fn undominated_detail() {
        let bytes = BASE64_STANDARD.decode("AgUBAgQPHwE=").unwrap();
        let inst : Instance = codec::decode_from_memory(&bytes).unwrap();

        let rows = choices![
            [0,1] -> [0,1],
            [0,1,3] -> [0,1],
            [0,1,4] -> [0,1],
            [1,2] -> [1,2],
            [1,2,3] -> [1,2],
            [1,2,4] -> [1,2],
            [0,2] -> [0,2],
            [0,2,3] -> [0,2],
            [0,2,4] -> [0,2],

            [0,3] -> [0],
            [1,3] -> [1],
            [2,3] -> [2],
            [0,4] -> [0],
            [1,4] -> [1],
            [2,4] -> [2],

            [3,4] -> [3]
        ];

        for cr in rows {
            assert_eq!(inst.choice(cr.menu.view(), None), cr.choice, "menu: {}", cr.menu);
        }
    }

    #[test]
    fn seqrc() {
        use model::Model;
        use super::InstanceInfo as II;

        let models = [Model::SequentiallyRationalizableChoice];
        let subject = testsubj(4, choices![
                [0,1,2,3] -> [1],
                [0,1,2] -> [1],
                [0,1,3] -> [1],
                [0,2,3] -> [0],
                [1,2,3] -> [2],
                [0,1] -> [1],
                [0,2] -> [0],
                [0,3] -> [0],
                [1,2] -> [2],
                [1,3] -> [1],
                [2,3] -> [2]
        ]);

        let mut precomputed = Precomputed::new(None);
        precomputed.precompute(4).unwrap();
        let response = super::run_one(&precomputed, DistanceScore::HoutmanMaks, &subject, &models).unwrap();

        assert_eq!(response.score, Penalty::exact(0));
        assert_eq!(response.best_instances.len(), 11);

        let model = Model::SequentiallyRationalizableChoice;
        let penalty = Penalty::exact(0);
        assert_eq!(response.best_instances, vec![
            II{ model, penalty: penalty.clone(), instance: vec![7, 4, 1, 2, 5, 9, 4, 7, 6, 4, 14] },
            II{ model, penalty: penalty.clone(), instance: vec![7, 4, 1, 2, 5, 11, 4, 7, 6, 4, 12] },
            II{ model, penalty: penalty.clone(), instance: vec![7, 4, 1, 2, 5, 11, 4, 7, 6, 4, 14] },
            II{ model, penalty: penalty.clone(), instance: vec![7, 4, 1, 2, 5, 11, 4, 15, 6, 4, 12] },
            II{ model, penalty: penalty.clone(), instance: vec![7, 4, 1, 2, 5, 13, 4, 7, 6, 4, 14] },
            II{ model, penalty: penalty.clone(), instance: vec![7, 4, 1, 2, 5, 15, 4, 7, 6, 4, 8] },
            II{ model, penalty: penalty.clone(), instance: vec![7, 4, 1, 2, 5, 15, 4, 7, 6, 4, 12] },
            II{ model, penalty: penalty.clone(), instance: vec![7, 4, 1, 2, 5, 15, 4, 7, 6, 4, 14] },
            II{ model, penalty: penalty.clone(), instance: vec![7, 4, 1, 2, 5, 15, 4, 15, 6, 4, 8] },
            II{ model, penalty: penalty.clone(), instance: vec![7, 4, 1, 2, 5, 15, 4, 15, 6, 4, 12] },
            II{ model, penalty: penalty.clone(), instance: vec![7, 4, 1, 2, 5, 15, 4, 15, 14, 4, 8] }
        ]);
    }

    #[test]
    fn undominated() {
        use model::Model;
        use super::InstanceInfo as II;

        let mut precomputed = Precomputed::new(None);
        precomputed.precompute(5).unwrap();

        let models = [Model::UndominatedChoice{strict: true}];
        let subject = testsubj(5, choices![
                [0,1] -> [0,1],
                [0,1,3] -> [0,1],
                [0,1,4] -> [0,1],
                [1,2] -> [1,2],
                [1,2,3] -> [1,2],
                [1,2,4] -> [1,2],
                [0,2] -> [0,2],
                [0,2,3] -> [0],
                [0,2,4] -> [2],

                [0,3] -> [0],
                [1,3] -> [1],
                [2,3] -> [2],
                [0,4] -> [0],
                [1,4] -> [1],
                [2,4] -> [2],

                [3,4] -> [3]
        ]);

        let response = super::run_one(&precomputed, DistanceScore::HoutmanMaks, &subject, &models).unwrap();
        assert_eq!(response.score, Penalty::exact(2));
        assert_eq!(response.best_instances.len(), 3);

        let m = Model::UndominatedChoice{strict: true};
        assert_eq!(response.best_instances, vec![
            II{ model: m, penalty: Penalty::exact(2), instance: vec![2, 5, 1, 2, 4, 15, 31] },
            II{ model: m, penalty: Penalty::exact(2), instance: vec![2, 5, 1, 2, 5, 15, 31] },
            II{ model: m, penalty: Penalty::exact(2), instance: vec![2, 5, 5, 2, 4, 15, 31] },
        ]);
    }

    #[test]
    fn jaccard() {
        use model::PreorderParams as PP;
        use model::Model::PreorderMaximization as PM;

        let mut precomputed = Precomputed::new(None);
        precomputed.precompute(5).unwrap();

        let models = [PM(PP{strict:Some(true),total:Some(true)})];
        let subject = testsubj(5, choices![
                [0,1] -> [0],
                [1,2] -> [1],
                [2,3] -> [2],
                [3,4] -> [3],
                [4,0] -> [0]
        ]);

        let response_hm = super::run_one(&precomputed, DistanceScore::HoutmanMaks, &subject, &models).unwrap();
        assert_eq!(response_hm.score, Penalty::exact(0));
        assert_eq!(response_hm.best_instances.len(), 1);

        let response_jm = super::run_one(&precomputed, DistanceScore::Jaccard, &subject, &models).unwrap();
        assert_eq!(response_jm.score, Penalty::exact(0));
        assert_eq!(response_jm.best_instances.len(), 1);
    }

    #[test]
    fn indecisive() {
        use crate::model::PreorderParams as PP;
        use crate::model::Model::PreorderMaximization as PM;
        use crate::alt_set::AltSet;

        let mut precomputed = Precomputed::new(None);
        precomputed.precompute(5).unwrap();

        let models = [PM(PP{ strict: None, total: None })];
        let subject = testsubj(5, choices![
                [0,1,2,3,4] -> [],
                [0,1,2,3] -> [],
                [0,1,2,4] -> [],
                [0,1,3,4] -> [],
                [0,2,3,4] -> [],
                [1,2,3,4] -> [],
                [0,1,2] -> [],
                [0,1,3] -> [],
                [0,2,3] -> [],
                [1,2,3] -> [],
                [0,1,4] -> [],
                [0,2,4] -> [],
                [1,2,4] -> [],
                [0,3,4] -> [],
                [1,3,4] -> [],
                [2,3,4] -> [],
                [3,4] -> [],
                [2,4] -> [],
                [1,4] -> [],
                [0,4] -> [],
                [2,3] -> [],
                [1,3] -> [],
                [0,3] -> [],
                [1,2] -> [],
                [0,2] -> [],
                [0,1] -> [],
                [4] -> [4],
                [3] -> [3],
                [2] -> [2],
                [1] -> [1],
                [0] -> [0]
        ]);

        // we want a 5-element diagonal here
        let p = preorder::Preorder::from_fast_preorder(5,
            fast_preorder::FastPreorder(0x1008040201)  // 8 bits per row
        );

        {
            let mut choice = alts![0,1];
            assert_eq!(choice, alts![0,1]);
            let up0 = p.upset(Alt(0));
            assert_eq!(up0.iter().collect::<AltSet>(), alts![0]);
            choice &= up0;
            assert_eq!(choice, alts![0]);
            let up1 = p.upset(Alt(1));
            assert_eq!(up1.iter().collect::<AltSet>(), alts![1]);
            choice &= up1;
            assert_eq!(choice, alts![]);
        }

        let instance = model::Instance::PreorderMaximization(p);
        assert_eq!(instance.choice(alts![0,1].view(), None), alts![]);

        let response = super::run_one(&precomputed, DistanceScore::HoutmanMaks, &subject, &models).unwrap();
        assert_eq!(response.score, Penalty::exact(0));
        assert_eq!(response.best_instances, vec![super::InstanceInfo{
            model: PM(PP{ strict: None, total: None }),
            penalty: Penalty::exact(0),
            instance: vec![0, 5, 1, 2, 4, 8, 16],
        }]);
    }
}
