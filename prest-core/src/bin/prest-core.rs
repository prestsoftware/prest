extern crate prest;
extern crate prest_core;
extern crate rand;

use rand::SeedableRng;
use rand::rngs::SmallRng;
use prest_core::{rpc,args};
use prest::{precomputed,estimation,consistency,simulation,instviz};
use prest::{experiment_stats,budgetary,integrity};
use precomputed::Precomputed;

fn rpc_loop(args : &args::Args) {
    use rpc::*;

    // core state
    let mut rng : SmallRng = SeedableRng::from_seed([0;32]);
    let mut rpc = IO::from_stdio();
    let mut precomp = Precomputed::new(
        args.fname_precomputed_preorders.as_ref().map(String::as_str)
    );

    loop {
        let request : ActionRequest = rpc.read().unwrap();

        match request {
            ActionRequest::Quit => {
                break;
            }

            ActionRequest::Echo(msg) => {
                rpc.write_result(Ok::<String, bool>(msg)).unwrap();
            }

            ActionRequest::Crash(msg) => {
                panic!("{}", msg);
            }

            ActionRequest::Fail(msg) => {
                rpc.write_result(Err::<bool, String>(msg)).unwrap();
            }

            ActionRequest::Estimation(req) => {
                rpc.write_result(estimation::run(&mut precomp, &req)).unwrap();
            }

            ActionRequest::ConsistencyDeterministic(req) => {
                rpc.write_result(consistency::deterministic::run(&req)).unwrap();
            }

            ActionRequest::ConsistencyStochastic(req) => {
                rpc.write_result(consistency::stochastic::run(&req)).unwrap();
            }

            ActionRequest::TupleIntransMenus(req) => {
                rpc.write_result(consistency::deterministic::tuple_intrans::run_menus(&req)).unwrap();
            }

            ActionRequest::TupleIntransAlts(req) => {
                rpc.write_result(consistency::deterministic::tuple_intrans::run_alts(&req)).unwrap();
            }

            ActionRequest::SetRngSeed(seed) => {
                if seed.len() == 32 {
                    let mut xs = [0;32];
                    for (i, &x) in seed.iter().enumerate() {
                        xs[i] = x;
                    }
                    rng = SeedableRng::from_seed(xs);

                    rpc.write_result(Ok::<String, bool>(String::from("OK"))).unwrap();
                } else {
                    rpc.write_result(Err::<bool, String>(
                        String::from("rng seed must contain exactly 32 numbers")
                    )).unwrap();
                }
            }

            ActionRequest::Simulation(req) => {
                rpc.write_result(simulation::run(&mut rng, req)).unwrap();
            }

            ActionRequest::Summary(req) => {
                rpc.write_result(experiment_stats::run(req)).unwrap();
            }

            ActionRequest::BudgetaryConsistency(req) => {
                let resp = budgetary::consistency::run(Logger::new(&mut rpc), req);
                rpc.write_result(resp).unwrap();
            }

            ActionRequest::IntegrityCheck(req) => {
                rpc.write_result(integrity::run(req)).unwrap();
            }

            ActionRequest::InstViz(req) => {
                rpc.write_result(instviz::run(req)).unwrap();
            }
        }
    }
}

fn main() {
    let args = args::parse();
    rpc_loop(&args);
}
