extern crate rand_scenario;
use std::path::Path;
use std::str::FromStr;
use std::env;
use rand_scenario::gen_norm_rand_csv;
fn main() {
    println!("Generate random variables with scenario.");
    // 引数の確認
    let args: Vec<String> = env::args().collect();
    if args.len() != 4 {
        panic!("Error: Need just 3 argments\n\tFor example...\n\tcargo run scenario.toml outdir number_of_files(such as 10)");
    }
    let path_scenario = Path::new(&args[1]);
    let dir_out = Path::new(&args[2]);
    let num = usize::from_str(&args[3]).expect("Third argument is the number of file to be generated. Therefore, a numberis required.");

    // ファイル生成
    match gen_norm_rand_csv(&path_scenario, &dir_out, num) {
            Ok(_) => println!("Number of {} files generated at {}.", num, &args[2]),
            Err(err) => panic!("{:?}", err),
    }
}
