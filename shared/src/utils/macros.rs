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
macro_rules! constrain {
    ($min:expr, $value:expr, $max:expr) => {
        if $value < $min {
            $min
        } else if $value > $max {
            $max
        } else {
            $value
        }
    };
}