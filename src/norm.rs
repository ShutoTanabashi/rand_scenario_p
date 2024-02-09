//! 正規分布に従う乱数生成プログラム
// use super::ScenarioError;

extern crate serde;
use serde::{Serialize, Deserialize};
use std::path::Path;
use std::fs;
use std::io::Write;
use std::str::FromStr;
extern crate toml;

extern crate process_param;
use process_param::{Process, ProcessSimulator};
use process_param::norm::{Scenario, Parameter};


/// Seed値の型
pub type Seed = u64;

/// シナリオから生成した乱数を格納
///
/// # 引数
/// * `scenario` - 乱数生成に利用したシナリオ
/// * `seed` - 乱数生成に利用したシード値
/// * `random_variables` - 生成された乱数列
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RandomScenario {
    scenario: Scenario,
    seed: Seed,
    random_variables: Vec<Vec<<Parameter as Process>::Observation>>
}

type RandValToml = Vec<Vec<f64>>;

// toml::to_string()用
#[derive(Serialize, Deserialize)]
struct StrRandValToml {
    random_variables: RandValToml
}

// TOML形式のRandomScenarioを読み取り・書き込みするための構造体
// プログラム内で利用する乱数(RandomScenarioScenario)とは若干形式が異なるため別で定義
#[derive(Debug, Serialize, Deserialize)]
struct RandomScenarioToml {
    scenario: toml::value::Table,
    seed: String, // u64からだと整数型に変換できない可能性があるため文字列として記述
    random_variables: RandValToml,
}

extern crate rand;
use rand::RngCore;
extern crate rand_mt;
use rand_mt::Mt64;
extern crate rayon;
use rayon::prelude::*;
impl RandomScenario {
    /// 乱数列を取得
    pub fn rand_vars(&self) -> &Vec<Vec<<Parameter as Process>::Observation>> {
        &self.random_variables
    }

    /// seedを取得
    pub fn get_seed(&self) -> Seed {
        self.seed
    }

    /// 最初のパラメータを取得
    ///
    /// サンプル自体が従うパラメータを取得する．
    ///
    /// # 返り値
    /// * `param_0` - 最初の状態における正規分布のパラメータ
    pub fn get_init_param(&self) -> Parameter {
        let (mu, sigma2) = self.scenario.param_in_control();
        Parameter::new(mu, sigma2).unwrap()
    }

    /// サンプル平均の従う最初のパラメータを取得
    ///
    /// 正規分布には再生性があり，サンプルの平均値も正規分布に従う．
    /// 乱数生成の最初の状態において，サンプル平均が従う正規分布のパラメータを取得する．
    ///
    /// # 返り値
    /// * `param_barx0` - 最初の状態でサンプル平均が従う正規分布のパラメータ
    pub fn get_sm_init_param(&self) -> Parameter {
        let (mu, sigma2) = self.scenario.param_samplemean();
        Parameter::new(mu, sigma2).unwrap()
    }


    /// Scenarioから乱数列を生成
    ///
    /// # 引数
    /// * `scenario` - 乱数生成に用いるシナリオ
    /// 
    /// # 使用例
    /// ```
    /// extern crate process_param;
    /// use process_param::norm::Scenario;
    /// # use rand_scenario::norm::RandomScenario;
    /// let path = std::path::Path::new("test/test_scenario.toml");
    /// let scenario = Scenario::from_toml(&path).unwrap();
    /// let randoms = RandomScenario::from_scenario(&scenario);
    /// println!("{:?}", randoms);
    /// ```
    pub fn from_scenario(scenario: &Scenario) -> Result<Self, process_param::ScenarioError> {
        let seed = rand::thread_rng().next_u64();
        Self::from_scenario_seed(scenario, seed)
    }

    /// Seedを指定してScenarioから乱数列を生成
    ///
    /// # 引数
    /// * `scenario` - 乱数生成に用いるシナリオ
    /// * `seed` - 乱数生成に用いるseed値
    /// 
    /// # 使用例
    /// ```
    /// extern crate process_param;
    /// use process_param::norm::Scenario;
    /// # use rand_scenario::norm::RandomScenario;
    /// let path = std::path::Path::new("test/test_scenario.toml");
    /// let scenario = Scenario::from_toml(&path).unwrap();
    /// let randoms = RandomScenario::from_scenario_seed(&scenario, 42).unwrap();
    /// println!("{:?}", randoms);
    /// ```
    pub fn from_scenario_seed(scenario: &Scenario, seed: Seed) -> Result<Self, process_param::ScenarioError> {
        let random_variables = Self::gen_random(&scenario, seed)?;
        Ok(RandomScenario{ scenario: scenario.clone(), seed, random_variables })
    }

    // 乱数生成コア
    fn gen_random(scenario: &Scenario, seed: Seed) -> Result<Vec<Vec<<Parameter as Process>::Observation>>, process_param::ScenarioError> {
        let mut rng = Mt64::new(seed);
        let dec_param = scenario.decomplession()?;
        let n = match usize::try_from(scenario.n()){
            Ok(val) => val,
            Err(_) => return Err(process_param::ScenarioError{
                message: "Sample size n doesn't convert to usize.".to_string()
            }),
        };
        Ok(dec_param.iter()
                    .map(|parameter| Parameter::rand_with_n(parameter, &mut rng, n))
                    .collect())
    }

    /// Scenarioから複数の乱数列を生成
    /// 
    /// # 引数
    /// * `scenario`- 乱数生成に用いるシナリオ
    /// * `num` - 生成する乱数列の個数
    /// 
    /// # 使用例
    /// ```
    /// extern crate process_param;
    /// use process_param::norm::Scenario;
    /// # use rand_scenario::norm::RandomScenario;
    /// let path = std::path::Path::new("test/test_scenario.toml");
    /// let scenario = Scenario::from_toml(&path).unwrap();
    /// let randoms = RandomScenario::from_scenario_multiple(&scenario, 4).unwrap();
    /// println!("{:?}", randoms);
    /// ```
    pub fn from_scenario_multiple(scenario: &Scenario, num: usize) -> Result<Vec<Self>, process_param::ScenarioError> {
        let mut seeds = Vec::with_capacity(num);
        let mut rng_for_seed = rand::thread_rng(); 
        for _i in 0..num {
            seeds.push(rng_for_seed.next_u64());
        }
        seeds.par_iter()
             .map(|seed| Self::from_scenario_seed(scenario, *seed))
             .collect()
    }


    /// TOMLファイルからRandomScenarioを作成
    /// 
    /// RandomScenario::to_tomlにより生成されたTOMLファイルを読み込む．
    /// 
    /// # 引数
    /// * `path` - 読み込むTOMLファイルのパス
    /// 
    /// # 使用例
    /// ```
    /// extern crate process_param;
    /// use process_param::norm::Scenario;
    /// # use rand_scenario::norm::RandomScenario;
    /// let path_scenario = std::path::Path::new("test/test_scenario.toml");
    /// let path_toml = std::path::Path::new("test/randoms_from_test_scenario.toml");
    /// let scenario = Scenario::from_toml(&path_scenario).unwrap();
    /// let randoms = RandomScenario::from_scenario(&scenario).unwrap();
    /// // TOMLファイルに保存
    /// randoms.to_toml(&path_toml).unwrap();
    /// // TOMLファイルから読み出し
    /// let rs_read = RandomScenario::from_toml(&path_toml).unwrap();
    /// assert_eq!(rs_read, randoms);
    /// ```
    pub fn from_toml<P: AsRef<Path>>(path: &P) -> Result<Self, Box<dyn std::error::Error>> {
        let file_str = fs::read_to_string(path)?;
        Self::parse_toml_str(&file_str)
    }


    /// Scenarioから管理図が管理外れ状態を検出するまで乱数を生成
    ///
    /// 管理図には$ \bar{X} $管理図とs管理図の併用を想定．
    /// 最初の変化点以前で管理外れ状態を検出した場合には乱数列を再生成する．
    ///
    /// # 引数
    /// * `scenario` - 乱数生成に用いるシナリオ
    /// 
    /// # 使用例
    /// ```
    /// extern crate process_param;
    /// use process_param::norm::Scenario;
    /// # use rand_scenario::norm::RandomScenario;
    /// let path = std::path::Path::new("test/test_scenario.toml");
    /// let scenario = Scenario::from_toml(&path).unwrap();
    /// let randoms = RandomScenario::from_scenario_controlchart(&scenario);
    /// println!("{:?}", randoms);
    /// ```
    pub fn from_scenario_controlchart(scenario: &Scenario) -> Result<Self, process_param::ScenarioError> {
        let seed = rand::thread_rng().next_u64();
        Self::from_scenario_seed_controlchart(scenario, seed)
    }


    /// Seedを指定してScenarioから管理図が管理外れ状態を検出するまで乱数を生成
    ///
    /// 管理図には$ \bar{X} $管理図とs管理図の併用を想定．
    /// 最初の変化点以前で管理外れ状態を検出した場合には乱数列を再生成する．
    ///
    /// # 引数
    /// * `scenario` - 乱数生成に用いるシナリオ
    /// * `seed` - 乱数生成に用いるseed値
    ///
    /// # 使用例
    /// ```
    /// extern crate process_param;
    /// use process_param::norm::Scenario;
    /// # use rand_scenario::norm::RandomScenario;
    /// let path = std::path::Path::new("test/test_scenario.toml");
    /// let scenario = Scenario::from_toml(&path).unwrap();
    /// let randoms = RandomScenario::from_scenario_seed_controlchart(&scenario, 42).unwrap();
    /// println!("{:?}", randoms);
    /// ```
    pub fn from_scenario_seed_controlchart(scenario: &Scenario, seed: Seed) -> Result<Self, process_param::ScenarioError> {
        let random_variables = Self::gen_random_controlchart(&scenario, seed)?;
        Ok(RandomScenario{ scenario: scenario.clone(), seed, random_variables })
    }
 
 
    // 管理図が管理外れ状態を検出するまで乱数を生成
    fn gen_random_controlchart(scenario: &Scenario, seed: Seed) -> Result<Vec<Vec<<Parameter as Process>::Observation>>, process_param::ScenarioError> {
        let mut rng = Mt64::new(seed);
        let (inctrl_param ,dec_param, last_cp) = scenario.decomp_exclude_last()?;
        let n = scenario.n_as_usize()?;
        let mut randoms: Vec<Vec<<Parameter as Process>::Observation>>;
 
        // 管理状態の乱数列
        loop {
            randoms = inctrl_param.iter()
                                  .map(|parameter| Parameter::rand_with_n(parameter, &mut rng, n))
                                  .collect::<Vec<Vec<<Parameter as Process>::Observation>>>();
            let params_dec_inctrl = match <Parameter as process_param::Mle>::mle_all(&randoms) {
                Err(e) => return Err(process_param::ScenarioError{
                    message: format!("Random number generation fails: {e}")
                }),
                Ok(pd) => pd,
            };
            if scenario.in_control_all(&params_dec_inctrl) {
                // 管理状態ならば現在のrandomsを利用
                break;
            }
        }

        // 最後の変化点前までの乱数生成
        let mut randoms_dec = dec_param.iter()
                                       .map(|parameter| Parameter::rand_with_n(parameter, &mut rng, n))
                                       .collect::<Vec<Vec<<Parameter as Process>::Observation>>>();
        let params_dec = match <Parameter as process_param::Mle>::mle_all(&randoms_dec) {
            Err(e) => return Err(process_param::ScenarioError{
                message: format!("Random number generation fails: {e}")
            }),
            Ok(pd) => pd,
        };
        match scenario.index_out_of_control(&params_dec) {
            None => randoms.append(&mut randoms_dec),
            Some(i) =>  {
                    // 管理外れ状態を検出した時点までの乱数を返す
                    randoms.append(&mut randoms_dec[..=i].to_vec());
                    return Ok(randoms)
                },
        };

        // 最後の変化点の情報に基づいて，管理外れ状態を検出するまで乱数を生成
        let mut ind_outctrl = 0;
        loop {
            ind_outctrl = ind_outctrl + 1;
            let param_ind = match last_cp.get_param(ind_outctrl) {
                Ok(p) => p,
                Err(e) => return Err(process_param::ScenarioError{
                    message: format!("Parameters are out of range before control chart alart.: {e}")
                }),
            };
            let rand_ind = param_ind.rand_with_n(&mut rng, n);
            let mle_ind = match <Parameter as process_param::Mle>::mle(&rand_ind) {
                Err(e) => return Err(process_param::ScenarioError{
                    message: format!("Random number generation fails: {e}")
                }),
                Ok(pd) => pd,
            };
            randoms.push(rand_ind);
            if scenario.out_of_control(&mle_ind) {
                // 管理外れ状態
                break;
            }
        }
        
        Ok(randoms)
    }


    /// Scenarioから管理図を併用した場合の複数の乱数列を生成
    /// 
    /// # 引数
    /// * `scenario`- 乱数生成に用いるシナリオ
    /// * `num` - 生成する乱数列の個数
    /// 
    /// # 使用例
    /// ```
    /// extern crate process_param;
    /// use process_param::norm::Scenario;
    /// # use rand_scenario::norm::RandomScenario;
    /// let path = std::path::Path::new("test/test_scenario.toml");
    /// let scenario = Scenario::from_toml(&path).unwrap();
    /// let randoms = RandomScenario::from_scenario_controlchart_multiple(&scenario, 4).unwrap();
    /// println!("{:?}", randoms);
    /// ```
    pub fn from_scenario_controlchart_multiple(scenario: &Scenario, num: usize) -> Result<Vec<Self>, process_param::ScenarioError> {
        let mut seeds = Vec::with_capacity(num);
        let mut rng_for_seed = rand::thread_rng(); 
        for _i in 0..num {
            seeds.push(rng_for_seed.next_u64());
        }
        seeds.par_iter()
             .map(|seed| Self::from_scenario_seed_controlchart(scenario, *seed))
             .collect()
    }


    /// TOMLファイルから管理図を併用した場合のRandomScenarioを作成
    /// 
    /// RandomScenario::to_tomlにより生成されたTOMLファイルを読み込む．
    /// 
    /// # 引数
    /// * `path` - 読み込むTOMLファイルのパス
    /// 
    /// # 使用例
    /// ```
    /// extern crate process_param;
    /// use process_param::norm::Scenario;
    /// # use rand_scenario::norm::RandomScenario;
    /// let path_scenario = std::path::Path::new("test/test_scenario.toml");
    /// let path_csv = std::path::Path::new("test/randoms_from_test_scenario_controlchart.csv");
    /// let scenario = Scenario::from_toml(&path_scenario).unwrap();
    /// let randoms = RandomScenario::from_scenario_controlchart(&scenario).unwrap();
    /// // TOMLファイルに保存
    /// randoms.to_csv(&path_csv).unwrap();
    /// ```
    pub fn from_toml_controlchart<P: AsRef<Path>>(path: &P) -> Result<Self, Box<dyn std::error::Error>> {
        let file_str = fs::read_to_string(path)?;
        Self::parse_toml_str(&file_str)
    }


    /// TOML形式の文字列からRandScenario読み取り
    pub fn parse_toml_str(toml_str: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let file_toml: RandomScenarioToml = toml::from_str(&toml_str)?;
        println!("{:?}", file_toml);
        let seed = Seed::from_str(&file_toml.seed)?;
        let scenario_toml = toml::to_string(&file_toml.scenario)?;
        let scenario = Scenario::parse_toml_str(&scenario_toml)?;

        Ok(RandomScenario {scenario, seed, random_variables: file_toml.random_variables})
    }


    /// 乱数列をCSVとして出力
    /// 
    /// # 引数
    /// * `path` - 出力ファイルパス
    /// 
    /// # 使用例
    /// ```
    /// extern crate process_param;
    /// use process_param::norm::Scenario;
    /// # use rand_scenario::norm::RandomScenario;
    /// let path_scenario = std::path::Path::new("test/test_scenario.toml");
    /// let path_csv = std::path::Path::new("test/randoms_from_test_scenario.csv");
    /// let scenario = Scenario::from_toml(&path_scenario).unwrap();
    /// let randoms = RandomScenario::from_scenario(&scenario).unwrap();
    /// println!("{:?}", randoms);
    /// randoms.to_csv(&path_csv).unwrap();
    /// ```
    /// 
    /// # 注意: 出力されるCSVファイルの見方
    ///
    /// 行方向（横）に同一時点でのn個のサンプルが並ぶ．
    /// 列方向（縦）は，時系列の昇順に並んでいる．
    pub fn to_csv<P: AsRef<Path>>(&self, path: &P) -> Result<(), Box<dyn std::error::Error>> {
        let mut wtr = csv::Writer::from_path(path)?;
        for rnds in self.rand_vars() {
            wtr.serialize(rnds)?;
        }
        wtr.flush()?;
        Ok(())
    }


    fn rands_to_toml_string(&self) -> String {
        let srvt= StrRandValToml{ random_variables: self.rand_vars().clone() };
        toml::to_string(&srvt).unwrap()
    }


    /// TOML形式の文字列に変換
    pub fn to_toml_string(&self) -> String {
        let scenario = self.scenario.to_toml_string();
        let rands = self.rands_to_toml_string();
        format!("seed = \"{}\"\n{}\n\n[scenario]\n{}", self.get_seed(), rands, scenario)
    }


    /// 乱数列をtomlとして出力
    /// 
    /// # 引数
    /// * `path` - 出力ファイルパス
    /// 
    /// # 使用例
    /// ```
     /// extern crate process_param;
    /// use process_param::norm::Scenario;
    /// # use rand_scenario::norm::RandomScenario;
    /// let path_scenario = std::path::Path::new("test/test_scenario.toml");
    /// let path_toml = std::path::Path::new("test/randoms_from_test_scenario.toml");
    /// let scenario = Scenario::from_toml(&path_scenario).unwrap();
    /// let randoms = RandomScenario::from_scenario(&scenario).unwrap();
    /// println!("{:?}", randoms);
    /// randoms.to_toml(&path_toml).unwrap();
    /// ```
    pub fn to_toml<P: AsRef<Path>>(&self, path: &P) -> Result<(), Box<dyn std::error::Error>> {
        let mut wtr = fs::File::create(path)?;
        let str_self = self.to_toml_string();
        write!(wtr, "{}", str_self)?;
        wtr.flush()?;
        Ok(())
    }
}
