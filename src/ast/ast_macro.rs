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

#[derive(Debug)]
pub enum Expr {
    Assign {
        pub name: Token,
        pub value: Box<Expr>,
    },
    Binary(Binary),
    Grouping(Grouping),
    Literal(Literal),
    Unary(Unary),
    Variable(Variable),
    Empty,
}

#[derive(Debug)]
pub struct Assign {
    pub name: Token,
    pub value: Box<Expr>,
}

#[derive(Debug)]
pub struct Binary {
    pub left: Box<Expr>,
    pub operator: Token,
    pub right: Box<Expr>,
}

#[derive(Debug)]
pub struct Grouping {
    pub expression: Box<Expr>,
}

#[derive(Debug)]
pub struct Literal {
    pub value: Token,
}

#[derive(Debug)]
pub struct Unary {
    pub operator: Token,
    pub right: Box<Expr>,
}

#[derive(Debug)]
pub struct Variable {
    pub name: Token,
}
