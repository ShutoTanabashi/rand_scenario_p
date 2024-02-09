//! 変化点検出プログラムに向け乱数生成プログラム
//!
//! 変化点とパラメータが記述されたシナリオからそれに従う乱数を生成する．
//! 
//! # cargo run による実行
//! *version 0.3.4の内容です．それ以降では変更の恐れがあります．*  
//! シェルから以下のようにコマンドを実行すると，正規分布に従う乱数をCSVで生成します．
//! 
//! > cargo　run　----release　シナリオを描いたtomlファイル　計算結果の出力先ディレクトリ　生成するファイル数
//! <!-- 注意：Markdownの表記上`----release`となっていますが，正しくはハイフン2個つまり`--release`です．あと，`--release`はコンパイルの最適化のためのコマンドで必須ではないため，`--release`無しでも動きます． -->
//! 
//! 例えば，次のように実行すれば，`.\test\test_scenario.toml`に記載されたシナリオにしたがって，`.\rands`ディレクトリに乱数列を記載した1000個のCSVファイルと乱数生成に使用したシード値をメモしたテキストファイルが出力されます．
//! 
//! > cargo run ----release .\test\test_scenario.toml .\rands 1000
//! <!-- 注意：前述の理由から`----release`となっていますが，正しくは `cargo run --release .\test\test_scenario.toml .\rands 1000` です．-->
//! 
//! 出力先のディレクトリですが，既存のディレクトリを指定した場合はエラーになります．
//! 既存のディレクトリを移動or削除してから再度実行してください．  
//! csvファイルですが，1行で同時点でのn個のサンプルを表し，それが時系列の進行に合わせて列方向にT回分並んでいます．  
//! 
//! ## 乱数生成法について
//! 正規乱数の生成は，Box-Mullar法で行っています（詳しくは[`process_param`]クレートを参照）．
//! またBox-Mullar法に必要な一様乱数は[Mersenne-Twister法](http://www.math.sci.hiroshima-u.ac.jp/m-mat/MT/mt.html)にて生成してます．
//! Mersenne-Twister法の部分は[`rand_mt`]クレートを利用しています．
//!
//! ## 管理図との併用
//! 変化点検出法での利用を想定し，管理図を併用（正規分布に対しては$ \bar{X}  - s $管理図）した場合の乱数も生成できます．
//! `src/main.rs`の`main`関数内部に
//!
//! > `gen_norm_rand_csv`  
//! 
//! という関数を利用している部分があると思うので，これを  
//! 
//! > `gen_norm_rand_controlchart_csv`  
//! 
//! に変更してください．
//! 引数等は変更しなくても動くはずです．

pub mod norm;

use std;
use std::fmt;
use std::fs::File;
use std::io::Write;

/// シナリオに関するエラー
#[derive(Debug, Clone)]
pub struct ScenarioError {
    pub message: String,
}

impl fmt::Display for ScenarioError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ScenarioError {
    fn description(&self) -> &str {
        &self.message
    }
}

use std::path::{Path,PathBuf};
use std::fs::create_dir;
extern crate rayon;
use rayon::prelude::*;
extern crate serde;
use serde::Serialize;
extern crate process_param;
/// 生成した乱数列を指定した個数分csvファイルで出力
///
/// # 引数
/// * `path_scenario` - 乱数生成のシナリオが記述されたTOMLファイルのパス
/// * `dir_out`- 出力するディレクトリ名
/// * `num` - 出力するファイルの個数
/// 
/// # 注意
/// 出力ファイルは「シナリオ名_番号.csv」となります．  
/// また，各乱数生成に用いたseed値は「seed.txt」に記録します．
/// 
/// # 使用例
/// ```
/// # use rand_scenario::gen_norm_rand_csv;
/// # use std::path::Path;
/// # use std::fs::remove_dir_all;
/// let path_scenario = Path::new("test/test_scenario.toml");
/// let dir_out = Path::new("test/gen_norm_rand_csv");
/// # remove_dir_all(dir_out.clone()).ok();
/// gen_norm_rand_csv(&path_scenario, &dir_out, 10).unwrap();
/// ```
pub fn gen_norm_rand_csv<P: AsRef<Path>>(path_scenario: &P, dir_out: &P, num: usize) -> Result<(), Box<dyn std::error::Error>> {
    let scenario = process_param::norm::Scenario::from_toml(path_scenario)?;
    // ファイルパスの準備
    let filename = path_scenario.as_ref().file_stem().unwrap().to_str().unwrap();
    if let Err(e) = create_dir(dir_out) {
        panic!("{:?}: {}", dir_out.as_ref(), e)
    }
    let dir_out_ref = dir_out.as_ref();
    let csvs: Vec<PathBuf> = (1..num+1).collect::<Vec<usize>>()
                                       .par_iter()
                                       .map(|i| dir_out_ref.join(Path::new(&format!("{}_{}.csv",filename, i))))
                                       .collect();

    // seed値の記録用
    let mut wtr = csv::Writer::from_path(
                      dir_out.as_ref().join(Path::new("seed.txt"))
                  )?;
    #[derive(Serialize)]
    struct SeedRecord {
        file: String,
        seed: norm::Seed,
    }

    let randoms = norm::RandomScenario::from_scenario_multiple(&scenario, num)?;
    for (r, fb) in randoms.iter().zip(csvs.iter()) {
        r.to_csv(fb)?;
        wtr.serialize( SeedRecord {file: fb.to_str().unwrap().to_string(), seed: r.get_seed()})?;
    }
    wtr.flush()?;
    Ok(())
}


/// 生成した乱数列を指定した個数分tomlファイルで出力
///
/// # 引数
/// * `path_scenario` - 乱数生成のシナリオが記述されたTOMLファイルのパス
/// * `dir_out`- 出力するディレクトリ名
/// * `num` - 出力するファイルの個数
/// 
/// # 注意
/// 出力ファイルは「シナリオ名_番号.toml」となります．  
/// 
/// # 使用例
/// ```
/// # use rand_scenario::gen_norm_rand_toml;
/// # use std::path::Path;
/// # use std::fs::remove_dir_all;
/// let path_scenario = Path::new("test/test_scenario.toml");
/// let dir_out = Path::new("test/gen_norm_rand_toml");
/// # remove_dir_all(dir_out.clone()).ok();
/// gen_norm_rand_toml(&path_scenario, &dir_out, 10).unwrap();
/// ```
pub fn gen_norm_rand_toml<P: AsRef<Path>>(path_scenario: &P, dir_out: &P, num: usize) -> Result<(), Box<dyn std::error::Error>> {
    let scenario = process_param::norm::Scenario::from_toml(path_scenario)?;
    // ファイルパスの準備
    let filename = path_scenario.as_ref().file_stem().unwrap().to_str().unwrap();
    if let Err(e) = create_dir(dir_out) {
        panic!("{:?}: {}", dir_out.as_ref(), e)
    }
    let dir_out_ref = dir_out.as_ref();
    let csvs: Vec<PathBuf> = (1..num+1).collect::<Vec<usize>>()
                                       .par_iter()
                                       .map(|i| dir_out_ref.join(Path::new(&format!("{}_{}.toml",filename, i))))
                                       .collect();

    let randoms = norm::RandomScenario::from_scenario_multiple(&scenario, num)?;
    for (r, fb) in randoms.iter().zip(csvs.iter()) {
        r.to_toml(fb)?;
    }
    Ok(())
}


/// 管理図を併用して生成した乱数列を指定した個数分csvファイルで出力
///
/// # 引数
/// * `path_scenario` - 乱数生成のシナリオが記述されたTOMLファイルのパス
/// * `dir_out`- 出力するディレクトリ名
/// * `num` - 出力するファイルの個数
/// 
/// # 注意
/// 出力ファイルは「シナリオ名_番号.csv」となります．  
/// また，各乱数生成に用いたseed値は「seed.txt」，管理図の管理限界は「controlLimit.txt」に記録します．
/// 
/// # 使用例
/// ```
/// # use rand_scenario::gen_norm_rand_controlchart_csv;
/// # use std::path::Path;
/// # use std::fs::remove_dir_all;
/// let path_scenario = Path::new("test/test_scenario.toml");
/// let dir_out = Path::new("test/gen_norm_rand_controlchart_csv");
/// # remove_dir_all(dir_out.clone()).ok();
/// gen_norm_rand_controlchart_csv(&path_scenario, &dir_out, 10).unwrap();
/// ```
pub fn gen_norm_rand_controlchart_csv<P: AsRef<Path>>(path_scenario: &P, dir_out: &P, num: usize) -> Result<(), Box<dyn std::error::Error>> {
    let scenario = process_param::norm::Scenario::from_toml(path_scenario)?;
    // ファイルパスの準備
    let filename = path_scenario.as_ref().file_stem().unwrap().to_str().unwrap();
    if let Err(e) = create_dir(dir_out) {
        panic!("{:?}: {}", dir_out.as_ref(), e)
    }
    let dir_out_ref = dir_out.as_ref();
    let csvs: Vec<PathBuf> = (1..num+1).collect::<Vec<usize>>()
                                       .par_iter()
                                       .map(|i| dir_out_ref.join(Path::new(&format!("{}_{}.csv",filename, i))))
                                       .collect();

    // seed値の記録用
    let mut wtr_seed = csv::Writer::from_path(
                      dir_out.as_ref().join(Path::new("seed.txt"))
                  )?;
    #[derive(Serialize)]
    struct SeedRecord {
        file: String,
        seed: norm::Seed,
    }

    let randoms = norm::RandomScenario::from_scenario_controlchart_multiple(&scenario, num)?;
    for (r, fb) in randoms.iter().zip(csvs.iter()) {
        r.to_csv(fb)?;
        wtr_seed.serialize( SeedRecord {file: fb.to_str().unwrap().to_string(), seed: r.get_seed()})?;
    }
    wtr_seed.flush()?;

    wtr_norm_control_limit(dir_out, &scenario)?;

    Ok(())
}


/// 管理図を併用して生成した乱数列を指定した個数分tomlファイルで出力
///
/// # 引数
/// * `path_scenario` - 乱数生成のシナリオが記述されたTOMLファイルのパス
/// * `dir_out`- 出力するディレクトリ名
/// * `num` - 出力するファイルの個数
/// 
/// # 注意
/// 出力ファイルは「シナリオ名_番号.toml」となります．
/// また管理図の管理限界は「controlLimit.txt」に保存されます．
/// 
/// # 使用例
/// ```
/// # use rand_scenario::gen_norm_rand_controlchart_toml;
/// # use std::path::Path;
/// # use std::fs::remove_dir_all;
/// let path_scenario = Path::new("test/test_scenario.toml");
/// let dir_out = Path::new("test/gen_norm_rand_controlchart_toml");
/// # remove_dir_all(dir_out.clone()).ok();
/// gen_norm_rand_controlchart_toml(&path_scenario, &dir_out, 10).unwrap();
/// ```
pub fn gen_norm_rand_controlchart_toml<P: AsRef<Path>>(path_scenario: &P, dir_out: &P, num: usize) -> Result<(), Box<dyn std::error::Error>> {
    let scenario = process_param::norm::Scenario::from_toml(path_scenario)?;
    // ファイルパスの準備
    let filename = path_scenario.as_ref().file_stem().unwrap().to_str().unwrap();
    if let Err(e) = create_dir(dir_out) {
        panic!("{:?}: {}", dir_out.as_ref(), e)
    }
    let dir_out_ref = dir_out.as_ref();
    let csvs: Vec<PathBuf> = (1..num+1).collect::<Vec<usize>>()
                                       .par_iter()
                                       .map(|i| dir_out_ref.join(Path::new(&format!("{}_{}.toml",filename, i))))
                                       .collect();

    let randoms = norm::RandomScenario::from_scenario_controlchart_multiple(&scenario, num)?;
    for (r, fb) in randoms.iter().zip(csvs.iter()) {
        r.to_toml(fb)?;
    }

    wtr_norm_control_limit(dir_out, &scenario)?;

    Ok(())
}


// 正規分布に従うプロセスについて，管理限界の情報を書き出し
fn wtr_norm_control_limit<P: AsRef<Path>>(path_dir: &P, scenario: &process_param::norm::Scenario) -> Result<(), Box<dyn std::error::Error>> {
    let (mu_0, sigma_0_2) = scenario.param_in_control();
    let (lcl_xbar, ucl_xbar) = scenario.control_limit_xbar();
    let (lcl_s, ucl_s) = scenario.control_limit_s();
    let cl_info = format!("μ_0, {mu_0}\nσ_0^2, {sigma_0_2}\n\nbarX control chart\nLCL, {lcl_xbar}\nUCL, {ucl_xbar}\n\ns control chart\nLCL, {lcl_s}\nUCL, {ucl_s}");
    let mut wtr_cl = File::create(
        path_dir.as_ref().join(Path::new("controlLimit.txt"))
        )?;
    wtr_cl.write_all(cl_info.as_bytes())?;
    wtr_cl.flush()?;
    
    Ok(())
}
