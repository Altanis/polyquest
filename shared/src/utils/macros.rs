#[macro_export]
macro_rules! fuzzy_compare {
    ($a:expr, $b:expr, $t:expr) => {
        ($a - $b).abs() <= $t
    }
}

#[macro_export]
macro_rules! bool {
    ($a:expr) => {
        match $a {
            "0" | "0.0" | "false" => false,
            _ => true
        }
    };
}

#[macro_export]
macro_rules! lerp {
    ($a:expr, $b:expr, $t:expr) => {
        $a + ($b - $a) * $t
    };
}

#[macro_export]
macro_rules! lerp_angle
{
    ($a:expr, $b:expr, $t:expr) =>
    {
        {
            let mut value = $a + (-(($a - $b + std::f32::consts::PI * 3.0) % (std::f32::consts::TAU) - std::f32::consts::PI)) * $t;

            if (value > std::f32::consts::PI) {
                value -= std::f32::consts::TAU;
            }
    
            if (value < -std::f32::consts::PI) {
                value += std::f32::consts::TAU;
            }
    
            value
        }
    };
}

#[macro_export]
macro_rules! rand {
    ($min:expr, $max:expr) => {
        rand::thread_rng().gen_range($min..=$max)
    };
}

#[macro_export]
macro_rules! to_locale {
    ($n:expr) => {{
        let mut s = $n.to_string();
        let mut pos = s.len() as isize - 3;
        while pos > 0 {
            s.insert(pos as usize, ',');
            pos -= 3;
        }

        s
    }};
}

#[macro_export]
macro_rules! prettify_score {
    ($score:expr) => {
        if $score >= 1e6 {
            format!("{:.1}m", $score / 1e6)
        } else if $score >= 1e3 {
            format!("{:.1}k", $score / 1e3)
        } else {
            format!("{:.3}", $score)
        }
    };
}