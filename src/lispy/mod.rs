use super::log;

mod parser;
mod eval;

pub fn eval_str(s: &str) {
    log!(
        "\"{}\"\nevaled gives us: {:?}",
        s,
        eval::eval_from_str(s)
      );
}

pub fn parse_script() {
    //let input = "{ + 2 }";
    let input = "{ + 2 {* 3 4} }";
    eval_str(&input);
}