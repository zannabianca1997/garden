use simulation::time::TimeDelta;

fn main() {
    println!("1 second: {}", TimeDelta::SECOND);
    println!("1 year 10 days: {}", TimeDelta::YEAR + TimeDelta::DAY * 10);
    println!(
        "1 year 10 seconds:  {}",
        TimeDelta::YEAR + TimeDelta::SECOND * 10
    );
    println!(
        "1 year 10 seconds (:#):  {:#}",
        TimeDelta::YEAR + TimeDelta::SECOND * 10
    );
    println!(
        "32649823 seconds (:.2):  {:.2}",
        TimeDelta::SECOND * 32649823
    );
    println!("32649823 seconds (:#):  {:#}", TimeDelta::SECOND * 32649823);
}
