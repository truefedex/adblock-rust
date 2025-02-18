use criterion::*;

use serde::{Deserialize, Serialize};

use adblock::utils::rules_from_lists;
use adblock::blocker::{Blocker, BlockerOptions};
use adblock::request::Request;
use adblock::url_parser::parse_url;
use adblock::engine::Engine;

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Clone)]
struct TestRequest {
    frameUrl: String,
    url: String,
    cpt: String
}

fn load_requests() -> Vec<TestRequest> {
    let requests_str = adblock::utils::read_file_lines("data/requests.json");
    let reqs: Vec<TestRequest> = requests_str.into_iter().map(|r| serde_json::from_str(&r)).filter_map(Result::ok).collect();
    reqs
}

fn get_blocker(rules: &Vec<String>) -> Blocker {
    let (network_filters, _) = adblock::lists::parse_filters(rules, false, Default::default());

    let blocker_options = BlockerOptions {
        enable_optimizations: true,
    };

    Blocker::new(network_filters, &blocker_options)
}

fn bench_rule_matching(engine: &Engine, requests: &Vec<TestRequest>) -> (u32, u32) {
    let mut matches = 0;
    let mut passes = 0;
    requests
        .iter()
        .for_each(|r| {
            let res = engine.check_network_urls(&r.url, &r.frameUrl, &r.cpt);
            if res.matched {
                matches += 1;
            } else {
                passes += 1;
            }
        });
  // println!("Got {} matches, {} passes, {} errors", matches, passes, errors);
  (matches, passes)
}

fn bench_matching_only(blocker: &Blocker, requests: &Vec<Request>) -> (u32, u32) {
    let mut matches = 0;
    let mut passes = 0;
    requests
        .iter()
        .for_each(|parsed| {
            let check =  blocker.check(&parsed);
            if check.matched {
                matches += 1;
            } else {
                passes += 1;
            }
        });
  // println!("Got {} matches, {} passes", matches, passes);
  (matches, passes)
}

fn bench_rule_matching_browserlike(blocker: &Engine, requests: &Vec<(String, String, String, String, Option<bool>)>) -> (u32, u32) {
    let mut matches = 0;
    let mut passes = 0;
    requests
        .iter()
        .for_each(|(url, hostname, source_hostname, request_type, third_party)| {
            let check = blocker.check_network_urls_with_hostnames(&url, &hostname, &source_hostname, &request_type, *third_party);
            if check.matched {
                matches += 1;
            } else {
                passes += 1;
            }
        });
  // println!("Got {} matches, {} passes", matches, passes);
  (matches, passes)
}

fn rule_match(c: &mut Criterion) {
    let mut group = c.benchmark_group("rule-match");

    let requests = load_requests();
    let elep_req = requests.clone();
    let el_req = requests.clone();
    let slim_req = requests.clone();
    let requests_len = requests.len() as u64;

    group.throughput(Throughput::Elements(requests_len));
    group.sample_size(10);

    group.bench_function(
        "el+ep",
        move |b| {
            let rules = rules_from_lists(&vec![
                "data/easylist.to/easylist/easylist.txt".to_owned(),
                "data/easylist.to/easylist/easyprivacy.txt".to_owned()
            ]);
            let engine = Engine::from_rules(&rules, Default::default());
            b.iter(|| bench_rule_matching(&engine, &elep_req))
        },
    );
    group.bench_function(
        "easylist",
        move |b| {
            let rules = rules_from_lists(&vec![
                "data/easylist.to/easylist/easylist.txt".to_owned(),
            ]);
            let engine = Engine::from_rules(&rules, Default::default());
            b.iter(|| bench_rule_matching(&engine, &el_req))
        },
    );
    group.bench_function(
        "slimlist",
        move |b| {
            let rules = rules_from_lists(&vec![
                "data/slim-list.txt".to_owned()
            ]);
            let engine = Engine::from_rules(&rules, Default::default());
            b.iter(|| bench_rule_matching(&engine, &slim_req))
        },
    );

    group.finish();
}

fn rule_match_parsed_el(c: &mut Criterion) {
    let mut group = c.benchmark_group("rule-match-parsed");

    let rules = rules_from_lists(&vec![
        String::from("data/easylist.to/easylist/easylist.txt"),
    ]);
    let requests = load_requests();
    let requests_parsed: Vec<_> = requests.into_iter().map(|r| { Request::from_urls(&r.url, &r.frameUrl, &r.cpt) }).filter_map(Result::ok).collect();
    let requests_len = requests_parsed.len() as u64;
    let blocker = get_blocker(&rules);

    group.throughput(Throughput::Elements(requests_len));
    group.sample_size(10);

    group.bench_function(
        "easylist",
        move |b| {
            b.iter(|| bench_matching_only(&blocker, &requests_parsed))
        },
    );

    group.finish();
}

fn rule_match_parsed_elep_slimlist(c: &mut Criterion) {
    let mut group = c.benchmark_group("rule-match-parsed");

    let full_rules = rules_from_lists(&vec![
        String::from("data/easylist.to/easylist/easylist.txt"),
        String::from("data/easylist.to/easylist/easyprivacy.txt")
    ]);
    let blocker = get_blocker(&full_rules);

    let requests = load_requests();
    let requests_parsed: Vec<_> = requests.into_iter().map(|r| { Request::from_urls(&r.url, &r.frameUrl, &r.cpt) }).filter_map(Result::ok).collect();
    let requests_len = requests_parsed.len() as u64;

    let slim_rules = rules_from_lists(&vec![
        String::from("data/slim-list.txt"),
    ]);
    let slim_blocker = get_blocker(&slim_rules);

    let requests_copy = load_requests();
    let requests_parsed_copy: Vec<_> = requests_copy.into_iter().map(|r| { Request::from_urls(&r.url, &r.frameUrl, &r.cpt) }).filter_map(Result::ok).collect();

    group.throughput(Throughput::Elements(requests_len));
    group.sample_size(10);

    group.bench_function(
        "el+ep",
        move |b| {
            b.iter(|| bench_matching_only(&blocker, &requests_parsed))
        },
    );
    group.bench_function(
        "slimlist",
        move |b| {
            b.iter(|| bench_matching_only(&slim_blocker, &requests_parsed_copy))
        },
    );

    group.finish();
}

fn serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("blocker-serialization");

    group.sample_size(20);

    group.bench_function(
        "el+ep",
        move |b| {
            let full_rules = rules_from_lists(&vec![
                String::from("data/easylist.to/easylist/easylist.txt"),
                String::from("data/easylist.to/easylist/easyprivacy.txt")
            ]);

            let engine = Engine::from_rules(&full_rules, Default::default());
            b.iter(|| assert!(engine.serialize_raw().unwrap().len() > 0))
        },
    );
    group.bench_function(
        "el",
        move |b| {
            let full_rules = rules_from_lists(&vec![
                String::from("data/easylist.to/easylist/easylist.txt"),
            ]);

            let engine = Engine::from_rules(&full_rules, Default::default());
            b.iter(|| assert!(engine.serialize_raw().unwrap().len() > 0))
        }
    );
    group.bench_function(
        "slimlist",
        move |b| {
            let full_rules = rules_from_lists(&vec![
                String::from("data/slim-list.txt"),
            ]);

            let engine = Engine::from_rules(&full_rules, Default::default());
            b.iter(|| assert!(engine.serialize_raw().unwrap().len() > 0))
        }
    );

    group.finish();
}

fn deserialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("blocker-deserialization");

    group.sample_size(20);

    group.bench_function(
        "el+ep",
        move |b| {
            let full_rules = rules_from_lists(&vec![
                String::from("data/easylist.to/easylist/easylist.txt"),
                String::from("data/easylist.to/easylist/easyprivacy.txt")
            ]);

            let engine = Engine::from_rules(&full_rules, Default::default());
            let serialized = engine.serialize_raw().unwrap();

            b.iter(|| {
                let mut deserialized = Engine::default();
                assert!(deserialized.deserialize(&serialized).is_ok());
            })
        },
    );
    group.bench_function(
        "el",
        move |b| {
            let full_rules = rules_from_lists(&vec![
                String::from("data/easylist.to/easylist/easylist.txt"),
            ]);

            let engine = Engine::from_rules(&full_rules, Default::default());
            let serialized = engine.serialize_raw().unwrap();

            b.iter(|| {
                let mut deserialized = Engine::default();
                assert!(deserialized.deserialize(&serialized).is_ok());
            })
        },
    );
    group.bench_function(
        "slimlist",
        move |b| {
            let full_rules = rules_from_lists(&vec![
                String::from("data/slim-list.txt"),
            ]);

            let engine = Engine::from_rules(&full_rules, Default::default());
            let serialized = engine.serialize_raw().unwrap();

            b.iter(|| {
                let mut deserialized = Engine::default();
                assert!(deserialized.deserialize(&serialized).is_ok());
            })
        }
    );

    group.finish();
}

fn rule_match_browserlike_comparable(c: &mut Criterion) {
    let mut group = c.benchmark_group("rule-match-browserlike");

    let requests = load_requests();
    let requests_len = requests.len() as u64;

    group.throughput(Throughput::Elements(requests_len));
    group.sample_size(20);

    fn requests_parsed(requests: &[TestRequest]) -> Vec<(String, String, String, String, Option<bool>)> {
        requests.iter().map(|r| {
            let url_norm = r.url.to_ascii_lowercase();
            let source_url_norm = r.frameUrl.to_ascii_lowercase();

            let maybe_parsed_url = parse_url(&url_norm);
            if maybe_parsed_url.is_none() {
                return Err("bad url");
            }
            let parsed_url = maybe_parsed_url.unwrap();

            let maybe_parsed_source = parse_url(&source_url_norm);

            if maybe_parsed_source.is_none() {
                Ok((
                    parsed_url.url.to_owned(),
                    parsed_url.hostname().to_owned(),
                    "".to_owned(),
                    r.cpt.clone(),
                    None
                ))
            } else {
                let parsed_source = maybe_parsed_source.unwrap();
                Ok((
                    parsed_url.url.to_owned(),
                    parsed_url.hostname().to_owned(),
                    parsed_source.hostname().to_owned(),
                    r.cpt.clone(),
                    Some(parsed_source.domain() != parsed_url.domain())
                ))
            }
        })
        .filter_map(Result::ok)
        .collect::<Vec<_>>()
    }

    let elep_req = requests_parsed(&requests);
    let el_req = elep_req.clone();
    let slim = elep_req.clone();

    group.bench_function(
        "el+ep",
        move |b| {
            let rules = rules_from_lists(&vec![
                "data/easylist.to/easylist/easylist.txt".to_owned(),
                "data/easylist.to/easylist/easyprivacy.txt".to_owned()
            ]);
            let engine = Engine::from_rules_parametrised(&rules, Default::default(), false, true);
            b.iter(|| bench_rule_matching_browserlike(&engine, &elep_req))
        },
    );
    group.bench_function(
        "el",
        move |b| {
            let rules = rules_from_lists(&vec![
                "data/easylist.to/easylist/easylist.txt".to_owned(),
            ]);
            let engine = Engine::from_rules_parametrised(&rules, Default::default(), false, true);
            b.iter(|| bench_rule_matching_browserlike(&engine, &el_req))
        },
    );
    group.bench_function(
        "slimlist",
        move |b| {
            let rules = rules_from_lists(&vec![
                "data/slim-list.txt".to_owned()
            ]);
            let engine = Engine::from_rules_parametrised(&rules, Default::default(), false, true);
            b.iter(|| bench_rule_matching_browserlike(&engine, &slim))
        },
    );

    group.finish();
}

criterion_group!(
    benches,
    rule_match,
    rule_match_parsed_el,
    rule_match_parsed_elep_slimlist,
    rule_match_browserlike_comparable,
    serialization,
    deserialization
);
criterion_main!(benches);
