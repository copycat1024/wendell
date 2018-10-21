macro_rules! end_parenthesize {
    ($x:expr) => {{
        let x: String = $x.to_string();
        format!("{})", x)
    }};
    ($x:expr, $($y:expr),+) => {{
        let x: String = $x.to_string();
        format!("{} {}", x, end_parenthesize!($($y),+))
    }};
}

macro_rules! parenthesize {
    ($x:expr) => {{
        let x: String = $x.to_string();
        format!("({})", x)
    }};
    ($x:expr, $($y:expr),+) => {{
        let x: String = $x.to_string();
        format!("({} {}", x, end_parenthesize!($($y),+))
    }};
}
