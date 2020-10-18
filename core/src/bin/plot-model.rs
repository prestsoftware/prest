extern crate prest;
extern crate base64;

use std::env;
use prest::{model,codec,preorder,alt};
use alt::Alt;
use model::Instance;
use preorder::Preorder;

fn fmt_digraph(p : &Preorder, alt_names : &[String], unattr : &[Alt]) -> String {
    let mut result = String::from("digraph G {\n");

    for &i in unattr {
        result += &format!(
            "  \"{}\" [color=\"gray\", fontcolor=\"gray\"];\n",
            alt_names[i.index() as usize]
        );
    }

    for &(i, j) in &p.simple_digraph() {
        result += &format!(
            "  \"{}\" -> \"{}\"{};\n",
            alt_names[i.index() as usize],
            alt_names[j.index() as usize],
            if unattr.contains(&i) || unattr.contains(&j) {
                " [color=\"gray\"]"
            } else {
                ""
            }
        );
    }

    result += "}\n";

    result
}

fn main() {
    // parse the cmdline
    let (model_str, alt_names) = {
        let mut args = env::args();
        let _exe_name = args.next().unwrap();
        let model_str = args.next().unwrap();
        let alt_names : Vec<_> = args.collect();
        (model_str, alt_names)
    };

    println!("model str: {}", model_str);
    println!("alternatives: {:?}", alt_names);

    let bytes = base64::decode(&model_str).unwrap();
    println!("bytes: {:?}", bytes);

    let instance : model::Instance = codec::decode_from_memory(&bytes).unwrap();
    println!("instance: {:?}", instance);

    println!("model: {:?}", instance.determine_model());

    match instance {
        Instance::PreorderMaximization(ref p) => {
            assert_eq!(p.size, alt_names.len() as u32, "preorder size ({}) does not match the number of alternatives ({})", p.size, alt_names.len());
            println!("{}", fmt_digraph(p, &alt_names, &[]));
        }

        Instance::Unattractiveness{ref p, ref mask} => {
            assert_eq!(p.size, alt_names.len() as u32, "preorder size ({}) does not match the number of alternatives ({})", p.size, alt_names.len());
            let mut attr = Vec::new();
            let mut unattr = Vec::new();
            let mut unattr_idx = Vec::new();
            for i in Alt::all(p.size) {
                if mask.view().contains(i) {
                    attr.push(&alt_names[i.index() as usize]);
                } else {
                    unattr.push(&alt_names[i.index() as usize]);
                    unattr_idx.push(i);
                }
            }
            println!("attractive: {:?}", attr);
            println!("unattractive: {:?}", unattr);
            println!("{}", fmt_digraph(p, &alt_names, &unattr_idx));
        }

        Instance::UndominatedChoice(ref p) => {
            assert_eq!(p.size, alt_names.len() as u32, "preorder size ({}) does not match the number of alternatives ({})", p.size, alt_names.len());
            println!("{}", fmt_digraph(p, &alt_names, &[]));
        }

        Instance::PartiallyDominantChoice{ref p, fc:_} => {
            assert_eq!(p.size, alt_names.len() as u32, "preorder size ({}) does not match the number of alternatives ({})", p.size, alt_names.len());
            assert!(p.is_strict());  // if this does not hold, something's gone very wrong
            // see the corresponding match branch in Instance::choice() for explanation

            println!("{}", fmt_digraph(p, &alt_names, &[]));
        }

        Instance::HybridDomination(ref p) => {
            assert_eq!(p.size, alt_names.len() as u32, "preorder size ({}) does not match the number of alternatives ({})", p.size, alt_names.len());
            assert!(!p.is_total());

            println!("{}", fmt_digraph(p, &alt_names, &[]));
        }

        Instance::StatusQuoUndominatedChoice(ref p) => {
            assert_eq!(p.size, alt_names.len() as u32, "preorder size ({}) does not match the number of alternatives ({})", p.size, alt_names.len());
            println!("{}", fmt_digraph(p, &alt_names, &[]));
        }

        Instance::Overload{ref p, limit:_} => {
            assert_eq!(p.size, alt_names.len() as u32, "preorder size ({}) does not match the number of alternatives ({})", p.size, alt_names.len());
            println!("{}", fmt_digraph(p, &alt_names, &[]));
        }

        Instance::TopTwo(ref p) => {
            assert_eq!(p.size, alt_names.len() as u32, "preorder size ({}) does not match the number of alternatives ({})", p.size, alt_names.len());

            let order : Vec<&str> = p.as_linear_order().into_iter().map(
                |Alt(i)| alt_names[i as usize].as_ref()
            ).collect();

            println!("elements in descending order of preference:");
            println!("{:?}", order);
        }

        Instance::SequentiallyRationalizableChoice(ref p, ref q) => {
            assert_eq!(p.size, alt_names.len() as u32, "preorder size ({}) does not match the number of alternatives ({})", p.size, alt_names.len());
            assert_eq!(q.size, alt_names.len() as u32, "preorder size ({}) does not match the number of alternatives ({})", q.size, alt_names.len());

            println!("P = {}", fmt_digraph(p, &alt_names, &[]));
            println!("Q = {}", fmt_digraph(q, &alt_names, &[]));
        }
    }
}
