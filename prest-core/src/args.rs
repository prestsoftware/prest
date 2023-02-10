use argparse;

pub struct Args {
    pub fname_precomputed_preorders : Option<String>,
}

pub fn parse() -> Args {
    let mut fname_precomputed_preorders = String::new();

    {
        let mut ap = argparse::ArgumentParser::new();
        ap.set_description("Prest core process");

        ap.refer(&mut fname_precomputed_preorders)
            .add_option(&["--precomputed-preorders"], argparse::Store, "Path to precomputed preorders");

        ap.parse_args_or_exit();
    }

    Args {
        fname_precomputed_preorders:
            if fname_precomputed_preorders.is_empty() {
                None
            } else {
                Some(fname_precomputed_preorders)
            },
    }
}
