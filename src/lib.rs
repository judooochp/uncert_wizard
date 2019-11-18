use std::process;
use std::f64;

#[derive (Clone)]
pub struct Measurement {                    // A measurement is incomplete without a statement of uncertainty
    pub value:          f64,                // The measurement
    pub unit:           String,             // The unit of measure
    pub resolution:     f64,                // exact resolution in context of chosen unit of measure
    pub uncert:         Uncert,             // The statement of uncertainty expressed in like units as the measurement
}

#[derive (Clone)]
pub struct Uncert {
    pub estimate:       Option<f64>,        // Estimate after being figured
    pub unit:           Option<String>,     // Really just the SI prefix. Formatted to hold the Unit of Measure later
    pub comps:          Vec<Component>,     // Uncertainty components
}

#[derive (Clone)]
pub struct Component {
    pub evaluation:     Evaluation,         // Type A/B Evaluation
    pub source:         Source,             // Main contributor of this component
    pub distribution:   Distribution,       // Distribution Type
    pub sensitivity:    u32,                // Sensitivity Coefficient
    pub description:    String,
    pub ref_str:        Option<String>,
    pub ref_url:        Option<String>,
}

#[derive (Clone)]
pub enum Source {
    Method,         // Method
    Standard,       // Standard
    UnitUnderTest,  // Unit Under Test
    Environment,    // Environment
}

#[derive (Clone)]
pub enum Evaluation {
    A (f64),   // Type A evaluation and encoded values
    B (f64),   // Type B evaluation and value
}

#[derive (Clone)]
pub enum Distribution {
    Normal1,        // Normal, k=1
    Normal2,        // Normal, k=2
    Normal3,        // Normal, k=3
    UShaped,        // U-shaped, k=sqrt(2)
    Rectangular,    // Rectangular, k=sqrt(3)
    Quadratic,      // Quadratic, k=sqrt(5)
    Triangular,     // Triangular, k=sqrt(6)
}

impl Measurement {
//  Intaking a new measurement file, assigning uncertainty if there is none
    pub fn new(file_line: String) -> Measurement {
        //  Get the file. For now hard-coded to "1,2A.ucrt" from main()
        let f_str_vec: Vec<Vec<String>> = get_measurement(file_line);
        let mut wrk_comps = Uncert::new();
        let mut meas = Measurement { value: 0.0, resolution: 0.1, unit: String::new(), uncert: Uncert { estimate: Some(0.0), unit: Some(String::from("")), comps: Vec::new() } };
        //  Parse the file line by line
        for vector in f_str_vec {
            match vector[0].chars().next().unwrap() {
                '#' | '?'               => continue,    //  Skip comment lines in the source file for now
                '0'..='9' | '-' | '~'   => wrk_comps.comps.push(prs_comp(vector)),//  Uncertainty Component lines
                '!'                     => meas = get_meas(vector),     //  First used line; less common than other lines
                _                       => process::exit(2),            //  Your file is bad and you should feel bad. Read the file rules.
            }
        }
        
        let wrk_est = wrk_comps.rss();
        let meas_unit = meas.unit.clone();
        let u64_val_str_unit = get_uncert_unit(wrk_est, meas.unit).unwrap();
        let uncert = Uncert { 
            estimate:   Some(u64_val_str_unit.0 as f64),
            unit:       Some(u64_val_str_unit.1),
            comps:      vec![]  //  Original data remains intact. No components needed anymore.
        };

        Measurement {
            value:      meas.value,
            resolution: meas.resolution,
            unit:       meas_unit,
            uncert:     uncert,
        }
    }

//  
    pub fn value_string(self) -> String {
        let rounded_val = self.value;
        let mut res = self.resolution;
        let rounded_val = f64::trunc(rounded_val / res * 10.0) * res / 10.0;
        let rounded_val = f64::round(rounded_val / res) * res;

        let val = rounded_val.to_string();
        let chars: Vec<char> = val.chars().collect();
        let mut iter = chars.iter();
        let mut val_str = String::from("");
        let mut decimal_flag = false;
        while iter.len() > 0 {
            let next = iter.next().unwrap();
            if decimal_flag == true {
                if res < 1.0 {
                    res = res * 10.0;
                    val_str.push(*next);
                } else {
                    break;
                }
            } else {
                val_str.push(*next);
            }
            if next == &'.' {
                decimal_flag = true;
            }
        }
        while res < 1.0 {
            val_str.push('0');
            res = res * 10.0;
        }

        String::from(val_str)
    }

    pub fn printout(self) {
        let val = self.clone();
        let sig_fig_2 = get_2_sig_fig(self.uncert.estimate.unwrap(), self.uncert.unit.unwrap());
        let est = sig_fig_2.0.unwrap();
        let unt = self.unit;
        let mut unc_unt: String = sig_fig_2.1.unwrap();
        unc_unt.push_str(&unt);
        println!(
            "{} {} ±{} {}", 
            val.value_string(), 
            unt, 
            est, 
            unc_unt,
        );
    }
}

pub fn get_2_sig_fig(int: f64, unit: String) -> (Option<String>, Option<String>) {
    let mut int_str = (f64::round(int) as u64).to_string();
    let mut unit_2 = unit;
    match int_str.len() {
        4 => {
            int_str.insert(1,'.');
            int_str.pop();
            int_str.pop();
            unit_2 = match_magnitude(match_prefix(&unit_2.chars().next().unwrap()) + 3).to_string();
        },
        3 => {
            int_str.insert(0,'0');
            int_str.insert(1,'.');
            int_str.pop();
            unit_2 = match_magnitude(match_prefix(&unit_2.chars().next().unwrap()) + 3).to_string();
        },
        _ => {},
    }

    (Some(int_str), Some(unit_2))
}

pub fn match_prefix(prefix: &char) -> i64 {
    match prefix {
        'Y' => 24,  'Z' => 21,  'E' => 18,  'P' => 15,  'T' => 12,  'G' => 9,   'M' => 6,   'k' => 3,
        'h' => 2,   'd' => -1,  'c' => -2,  'm' => -3,  'µ' => -6,  'n' => -9,  'p' => -12, 'f' => -15,
        'a' => -18, 'z' => -21, 'y' => -24, _   => process::exit(6),
    }
}

pub fn match_magnitude(mag: i64) -> char {
    match mag {
        24  => 'Y', 21  => 'Z', 18  => 'E', 15  => 'P', 12  => 'T', 9   => 'G', 6   => 'M', 
        3   => 'k', 2   => 'h', 0 => ' ', -1  => 'd', -2  => 'c', -3  => 'm', -6  => 'µ', -9  => 'n', 
        -12 => 'p', -15 => 'f', -18 => 'a', -21 => 'z', -24 => 'y', _   => process::exit(7),
    }
}

pub fn get_uncert_unit(value: f64, meas_unit: String) -> Option<(u64, String)> {
    // take the f64 from the uncertainty, say... 0.00032
    // convert it by 3 place values to the relevant unit, say ... µ, since 
    let unit: Vec<char> = meas_unit.to_string().chars().collect();
    let prefix: Option<i64>;
    if unit.len() < 2 {
        prefix = None;
    } else {
        let mut pre = unit.iter();
        prefix = Some(match_prefix(pre.next().unwrap()));
    }
    
    let mut fig = value;
    let mut mag: i64 = 0;

    while fig > 9905.0 {
        fig = fig / 1000.0;
        mag = mag + 3;
    }

    while fig < 10.0 {
        fig = fig * 1000.0;
        mag = mag - 3;
    }

    
    let diff: i64 = if prefix.is_some() { 
        match_prefix(unit.first().unwrap()) + mag 
    } else {
        mag
    };

    Some((f64::round(fig) as u64,match_magnitude(diff).to_string()))
}

impl Uncert {
    pub fn new() -> Uncert {
        Uncert { 
            estimate: Some(0.0), 
            unit: Some(String::new()), 
            comps: Vec::new(),
        }
    }

    pub fn rss(&self) -> f64 {
//  Combine uncertainty components, return as k=2
        let val = self.clone();
        let mut wrk_sum = 0.0;
        for value in val.comps.iter() {
            let distr = value.get_divisor();
            let eval = value.get_estimate();
            let wrk_ucrt = (eval * value.sensitivity as f64) / distr;
            let wrk_ucrt = wrk_ucrt * wrk_ucrt;
            wrk_sum = wrk_sum + wrk_ucrt;
        }
        f64::sqrt(wrk_sum) * 2.0
    }
}

impl Component {
    pub fn get_estimate(&self) -> f64 {
        match self.evaluation {
            Evaluation::A(thisf64)      => thisf64,
            Evaluation::B(thisf64)      => thisf64,
        }
    }

    pub fn get_divisor(&self) -> f64 {
        match self.distribution {
            Distribution::Normal1       => 1.0,
            Distribution::Normal2       => 2.0,
            Distribution::Normal3       => 3.0,
            Distribution::UShaped       => f64::sqrt(2.0),
            Distribution::Rectangular   => f64::sqrt(3.0),
            Distribution::Quadratic     => f64::sqrt(5.0),
            Distribution::Triangular    => f64::sqrt(6.0),
        }
    }
}

//  Read file, output CSV as 2D vector
pub fn get_measurement(file_line: String) -> Vec<Vec<String>> {
    let mut lines: Vec<Vec<String>> = Vec::new();
    for line in file_line.lines() {
        let f_lines: Vec<String> = line.split(",").map(|s| s.to_string()).collect();
        lines.push(f_lines);
    }
    lines
}

pub fn get_meas(f_str_vec: Vec<String>) -> Measurement {
    let mut wrk_vec: Vec<String> = f_str_vec.clone();
    wrk_vec.drain(0..1);
    
    let mut wrk_vec = wrk_vec.iter();

    let val = wrk_vec.next().unwrap().parse().unwrap();
    let res = wrk_vec.next().unwrap().parse().unwrap();
    let wrk_unit = wrk_vec.next().unwrap().to_string();
    let wrk_est = wrk_vec.next();
    let mut wrk_uncert = Uncert::new();
    if wrk_est.is_some() {
        wrk_uncert.estimate = Some(wrk_est.unwrap().parse::<f64>().unwrap());
        wrk_uncert.unit = wrk_vec.next().map(String::from);
    }
    
    Measurement {
        value: val,
        resolution: res,
        uncert: wrk_uncert,
        unit: wrk_unit,
    }
}

pub fn prs_comp(comp: Vec<String>) -> Component {
    let mut hld = comp.iter();
    let hld_eval = hld.next().unwrap();
    let eval = if hld_eval.starts_with("~") { 
        Evaluation::A(std_dev_from_line(String::from(hld_eval)))
    } else {
        Evaluation::B(hld_eval.parse().unwrap())
    };

    let src_ref = hld.next().unwrap().as_ref();
    let src = match src_ref {
        "s"     => Source::Standard,
        "u"     => Source::UnitUnderTest,
        "m"     => Source::Method,
        "e"     => Source::Environment,
        _       => {
                    println!("Source Variable was: {}", src_ref);
                    println!("There has been an error with the Source parsing"); 
                    process::exit(1);
        },
    };
    
    let dist = match hld.next().unwrap().as_ref() {
        "n1"    => Distribution::Normal1,
        "n2"    => Distribution::Normal2,
        "n3"    => Distribution::Normal3,
        "u"     => Distribution::UShaped,
        "r"     => Distribution::Rectangular,
        "q"     => Distribution::Quadratic,
        "t"     => Distribution::Triangular,
        _       => {
                    println!("There has been an error with the Distribution parsing");
                    process::exit(4);
        },
    };
    
    let sens: u32   = hld.next().unwrap().parse().unwrap();
    let descr       = hld.next().unwrap().to_string();
    let ref_str     = hld.next().map(String::from);
    let url_str     = hld.next().map(String::from);

    Component {
        evaluation: eval,
        source: src,
        distribution: dist,
        sensitivity: sens,
        description: descr,
        ref_str: ref_str,
        ref_url: url_str,
    }
}

//  Standard Deviation of a Sample = sqrt(sum(abs(x-avg(x1...xn))^2)/(n-1))
//  From string like "~1.2~1.3~1.1~1.0~1.2~1.1~1.2~1.2~1.3~1.4~1.2"
//  Calculates to 10 decimal digits of precision, rounded up
/************************************************************************************
//IN THE FUTURE this will not have to be updated because future revisions of the code
//will allow for the mathematic-ing of units with their respective SI prefix ignored.
//That will allow for the control of precision so that floating-point error will not
//render a problem, so that the error introduced by rounding to 8 digits (We'll never
//need that many for the uncertainty math) becomes negligible.
************************************************************************************/
pub fn std_dev_from_line(hld_eval: String) -> f64 {
    let mut eval_str: Vec<&str> = hld_eval.split("~").collect();
    eval_str.drain(0..1);           //  First element was empty string due to file parsing
    let eval_f64: Vec<f64> = eval_str.iter().map(|x| x.parse::<f64>().unwrap()).collect();
//  Make sure we have more than 1 value to parse, 'cause 1 - 1 = 0 and divide by 0 is silly
    if eval_f64.len() < 2 {
        process::exit(1);
    }
//  Find Mean
    let n = eval_f64.len() as f64;
    let sum: f64 = eval_f64.iter().sum();
    let mean = sum / n;
//  Squares of Deviations from the Mean
    let eval_f64_dev: Vec<f64> = eval_f64.iter().map(|x| (x - mean) * (x - mean)).collect();
//  Sum of Squares of Deviations from the Mean
    let dev_sum: f64 = eval_f64_dev.iter().sum();
//  Divide by n-1 since sample and Return Standard Deviation of a Sample
    let end: f64 = ((f64::sqrt(dev_sum / (n - 1.0))) * 1e10).ceil() as f64 * 1e-10;
    end
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn std_dev_from_line_test() {
        assert_eq!(std_dev_from_line(String::from("~1.20003~1.20002~1.20001~1.2~1.20001~1.20001~1.20003~1.20004~1.20001~1.19995")),0.0000246982);
        assert_eq!(std_dev_from_line(String::from("~0~0")),0.0);
        assert_eq!(std_dev_from_line(String::from("~1~2~3")),1.0);
        assert_eq!(std_dev_from_line(String::from("~1e-1~2e-1~3e-1")),0.1);
        assert_eq!(std_dev_from_line(String::from("~1e-3~2e-3~3e-3")),1e-3);
    }

    #[test]
    fn get_uncert_unit_test() {
        assert_eq!(get_uncert_unit(0.00032,String::from("A")),Some((320,String::from("µ"))));
        assert_eq!(get_uncert_unit(32.0,String::from("MΩ")),Some((32,String::from("M"))));
        assert_eq!(get_uncert_unit(3200.0,String::from("MΩ")),Some((3200,String::from("M"))));
        assert_eq!(get_uncert_unit(10.0,String::from("Ω")),Some((10,String::from(" "))));
    }

    #[test]
    fn test_value_string() {
        assert_eq!(Measurement::value_string(
            Measurement {
                value: 1.20638, 
                resolution: 0.001, 
                uncert: Uncert::new(), 
                unit: String::from("A") 
            }),
            String::from("1.206")
        );
        assert_eq!(Measurement::value_string(   /////////////THIS ONE FAILS. UNKNOWN CAUSE
            Measurement {
                value: 1.20658, 
                resolution: 0.001, 
                uncert: Uncert::new(), 
                unit: String::from("A") 
            }),
            String::from("1.207")
        );
    }
}
