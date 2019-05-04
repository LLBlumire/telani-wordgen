#![feature(const_fn, decl_macro, proc_macro_hygiene)]

use rand::seq::SliceRandom;
use rand::thread_rng;
use rand::Rng;
use std::collections::HashMap;
use std::hash::Hash;
use rocket::get;
use rocket::routes;
use lazy_static::lazy_static;

static CON: [(char, f64); 13] = [
    ('l', 8.0),
    ('t', 7.0),
    ('n', 6.5),
    ('m', 6.0),
    ('b', 5.0),
    ('p', 5.0),
    ('d', 4.5),
    ('g', 4.0),
    ('j', 3.0),
    ('k', 2.0),
    ('f', 1.0),
    ('s', 1.0),
    ('x', 0.5),
];

static VOW: [(char, f64); 5] = [('a', 9.5), ('e', 9.0), ('i', 6.75), ('u', 6.5), ('o', 6.25)];

static SYLLABLE_DROPOFF: f64 = 0.4;

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
enum SyllableType {
    Open,
    Closed,
}
use SyllableType::*;
const SYL_TYPE: [(SyllableType, f64); 2] = [(Closed, 0.3), (Open, 0.7)];

fn dist<'a, I, T>(i: I) -> HashMap<T, f64> 
where I: IntoIterator<Item=&'a (T, f64)> + 'a,
    T: Hash + Clone + Eq + 'a
{
    i.into_iter().cloned().collect()
}

lazy_static! {
    static ref CON_DIST: HashMap<char, f64> = {
        dist(&CON)
    };
    static ref VOW_DIST: HashMap<char, f64> = {
        dist(&VOW)
    };
    static ref SYL_TYPE_DIST: HashMap<SyllableType, f64> = {
        dist(&SYL_TYPE)
    };
}

fn list<'a, I, T>(i: I) -> Vec<T>
where I: IntoIterator<Item=&'a (T, f64)> + 'a,
T: Clone + 'a
{
    i.into_iter().map(|(n, _)| n).cloned().collect()
}

lazy_static! {
    static ref CON_LIST: Vec<char> = {
        list(&CON)
    };
    static ref VOW_LIST: Vec<char> = {
        list(&VOW)
    };
    static ref SYL_TYPE_LIST: Vec<SyllableType> = {
        list(&SYL_TYPE)
    };
}

struct Gen<'a, T> {
    list: &'a Vec<T>,
    dist: &'a HashMap<T, f64>
}
impl<'a, T: Eq + Hash> Gen<'a, T> {
    fn gen<R: Rng>(&self, r: &mut R) -> &T {
        self.list.choose_weighted(r, |n| self.dist[n]).unwrap().clone()
    }
}

lazy_static! {
    static ref GEN_CON: Gen<'static, char> = Gen {
        list: CON_LIST.deref(),
        dist: CON_DIST.deref()
    };
    static ref GEN_VOW: Gen<'static, char> = Gen {
        list: VOW_LIST.deref(),
        dist: VOW_DIST.deref()
    };
    static ref GEN_SYL_TYPE: Gen<'static, SyllableType> = Gen {
        list: SYL_TYPE_LIST.deref(),
        dist: SYL_TYPE_DIST.deref()
    };
}

fn gen_syl<R: Rng>(r: &mut R) -> String {
    let mut string_buf = String::with_capacity(3);
    match GEN_SYL_TYPE.gen(r) {
        Open => {
            string_buf.push(*GEN_CON.gen(r));
            string_buf.push(*GEN_VOW.gen(r));
        }
        Closed => {
            string_buf.push(*GEN_CON.gen(r));
            string_buf.push(*GEN_VOW.gen(r));
            string_buf.push(*GEN_CON.gen(r));
        }
    }
    string_buf
}

fn gen_word<R: Rng>(r: &mut R) -> String {
    let mut word_buf = String::with_capacity(6);
    for n in 1.. {
        word_buf.push_str(&gen_syl(r));
        if !r.gen_bool((1.0 - SYLLABLE_DROPOFF).powi(n)) {
            println!("Exit");
            return word_buf;
        }
    }
    unreachable!()
}

#[get("/<count>")]
fn get_words(count: usize) -> String {
    let mut str_buf = String::with_capacity(count * 10);
    for _ in 0..count {
        str_buf.push_str(&gen_word(&mut thread_rng()));
        str_buf.push('\n');
    }
    str_buf
}

fn main() {
    rocket::ignite().mount("/", routes![get_words]).launch();
}
