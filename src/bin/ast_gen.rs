// bin/ast_gen.rs

use std::fs::File;
use std::io::{self, Write};
use std::ops::Deref;

pub fn main() {
    let output_dir = "src/ast";

    let expr_nodes = [
        "Assign   : Token name, Box<Expr> value",
        "Binary   : Box<Expr> left, Token operator, Box<Expr> right",
        "Grouping : Box<Expr> expression",
        "Literal  : Token value",
        "Unary    : Token operator, Box<Expr> right",
        "Call     : Box<Expr> callee, Token paren, Vec<Expr> arguments",
        "Variable : Token name",
    ];
    let expr_atoms = ["scanner::token::Token"];
    if let Err(e) = define_ast(output_dir, "Expr", &expr_atoms, &expr_nodes) {
        println!("{:?}", e);
    }

    let stmt_nodes = [
        // declaration
        "Var        : Token name, Expr initializer",
        // statement
        "Block      : Vec<Stmt> statements",
        "If         : u32 line_number, Expr condition, Box<Stmt> then_block, Box<Stmt> else_block",
        "While      : u32 line_number, Expr condition, Box<Stmt> body",
        "Function   : Token name, Vec<Token> params, Box<Stmt> body",
        "Expression : Expr expression",
        // function stand-in (remove later)
        "Print      : Expr expression",
    ];
    let stmt_atoms = ["ast::expr::Expr", "scanner::token::Token"];
    if let Err(e) = define_ast(output_dir, "Stmt", &stmt_atoms, &stmt_nodes) {
        println!("{:?}", e);
    }
}

fn define_ast(dir: &str, base: &str, atoms: &[&str], nodes: &[&str]) -> io::Result<()> {
    let path = format!("{}/{}.rs", dir, base.to_lowercase());
    let mut w = AstWriter::from(File::create(&path)?);
    let config = &AstConfig::new(base, atoms, nodes);
    let AstConfig { atoms, .. } = config;

    // Write header
    w.print_header(&path, &atoms)?;

    // Write base enum
    w.define_base(config)?;

    // Write base impl
    w.define_impl(config)?;

    // Write visitor trait
    w.define_visitor(config)
}

struct AstWriter<T: Write> {
    writer: T,
}

impl<T: Write> AstWriter<T> {
    fn from(writer: T) -> Self {
        Self { writer: writer }
    }

    // Level 1 (root blocks)

    fn print_header(&mut self, path: &str, atoms: &[AstAtom]) -> io::Result<()> {
        let cmt_path: String = path.chars().skip(4).collect();
        self.println(format!("// {}", cmt_path))?;
        self.println("")?;
        for atom in atoms.iter() {
            self.println(format!("use {};", atom.path))?;
        }
        self.println("")
    }

    fn define_base(&mut self, config: &AstConfig) -> io::Result<()> {
        let AstConfig { base, nodes, .. } = config;
        self.println("#[derive(Debug, Clone)]")?;
        self.println(format!("pub enum {} {{", base))?;
        for ref node in nodes.iter() {
            self.define_node(node)?;
        }
        self.println(format!("    Empty,"))?;
        self.println("}")?;
        self.println("")
    }

    fn define_impl(&mut self, config: &AstConfig) -> io::Result<()> {
        let AstConfig { base, nodes, .. } = config;
        self.println(format!("impl {} {{", base))?;
        self.define_accept(base, nodes)?;
        for node in nodes.iter() {
            self.define_node_constructors(base, node)?;
        }
        self.println("}")?;
        self.println("")
    }

    fn define_visitor(&mut self, config: &AstConfig) -> io::Result<()> {
        let AstConfig { base, nodes, .. } = config;
        self.println(format!("pub trait {}Visitor<R> {{", base))?;
        for ref node in nodes.iter() {
            self.define_node_visitor(node)?;
        }
        self.define_empty_visitor(base)?;
        self.println("}")?;
        self.println("")
    }

    // Level 2 (struct)

    fn define_node(&mut self, node: &AstNode) -> io::Result<()> {
        let AstNode { name, fields } = node;
        self.println(format!("    {} {{", name))?;
        for field in fields.iter() {
            let AstField {
                field_name,
                field_type,
            } = field;
            self.println(format!("        {}: {},", field_name, field_type))?;
        }
        self.println("    },")?;
        self.println("")
    }

    fn define_accept(&mut self, base: &str, nodes: &[AstNode]) -> io::Result<()> {
        self.println(format!(
            "    pub fn accept<R, T:{}Visitor<R>>(&self, visitor: &mut T) -> R {{",
            base
        ))?;
        self.println("        match self {")?;
        for ref node in nodes.iter() {
            self.define_node_accept(base, node)?;
        }
        self.define_empty_accept(base)?;
        self.println("        }")?;
        self.println("    }")
    }

    // Level 3 (one-liner)

    fn define_node_visitor(&mut self, node: &AstNode) -> io::Result<()> {
        let AstNode { name, fields } = node;

        let mut field_string = String::new();
        for field in fields.iter() {
            let AstField {
                field_name,
                field_type,
            } = field;
            field_string += format!(", {}: &{}", field_name, field_type).as_str();
        }

        self.println(format!(
            "    fn visit_{}(&mut self{}) -> R;",
            name.to_lowercase(),
            field_string
        ))
    }

    fn define_empty_visitor(&mut self, base: &str) -> io::Result<()> {
        self.println(format!(
            "    fn visit_empty_{}(&mut self) -> R;",
            base.to_lowercase()
        ))
    }

    fn define_node_accept(&mut self, base: &str, node: &AstNode) -> io::Result<()> {
        let AstNode { name, fields } = node;
        let field_names: Vec<&str> = fields.iter().map(|ref f| f.field_name.as_str()).collect();

        self.println(format!("        {}::{} {{", base, name))?;
        for field in field_names.iter() {
            self.println(format!("            ref {},", field))?;
        }
        self.println(format!(
            "        }} => visitor.visit_{}(",
            name.to_lowercase()
        ))?;
        for field in field_names.iter() {
            self.println(format!("            {},", field))?;
        }
        self.println("        ),")
    }

    fn define_empty_accept(&mut self, base: &str) -> io::Result<()> {
        self.println(format!(
            "            {}::Empty => visitor.visit_empty_{}(),",
            base,
            base.to_lowercase()
        ))
    }

    fn define_node_constructors(&mut self, base: &str, node: &AstNode) -> io::Result<()> {
        let AstNode { name, fields } = node;

        self.println("")?;
        self.println(format!("    pub fn new_{}(", name.to_lowercase()))?;
        for field in fields.iter() {
            let AstField {
                field_name,
                field_type,
            } = field;
            self.println(format!("        {}: {},", field_name, field_type))?;
        }
        self.println("    ) -> Self {")?;

        self.println(format!("        {}::{} {{", base, name))?;
        for field in fields.iter() {
            let AstField { field_name, .. } = field;
            self.println(format!("        {}: {},", field_name, field_name))?;
        }
        self.println("        }")?;
        self.println("    }")
    }
}

trait PrintLine<S: Deref<Target = str>> {
    fn println(&mut self, text: S) -> io::Result<()>;
}

impl<'a, T: Write, S: Deref<Target = str>> PrintLine<S> for AstWriter<T> {
    fn println(&mut self, text: S) -> io::Result<()> {
        self.writer.write(text.as_bytes())?;
        self.writer.write(b"\n")?;
        Ok(())
    }
}

#[derive(Debug)]
struct AstConfig {
    pub base: String,
    pub nodes: Vec<AstNode>,
    pub atoms: Vec<AstAtom>,
}

impl AstConfig {
    fn new(base: &str, atoms_input: &[&str], nodes_input: &[&str]) -> Self {
        let node_output = nodes_input.iter().map(|node| AstNode::new(node)).collect();
        let atom_output = atoms_input.iter().map(|atom| AstAtom::new(atom)).collect();
        Self {
            base: base.into(),
            nodes: node_output,
            atoms: atom_output,
        }
    }
}

#[derive(Debug)]
struct AstAtom {
    pub path: String,
}

impl AstAtom {
    fn new(atom: &str) -> Self {
        Self {
            path: String::from(atom),
        }
    }
}

#[derive(Debug)]
struct AstNode {
    pub name: String,
    pub fields: Vec<AstField>,
}

impl AstNode {
    fn new(node: &str) -> Self {
        let mut iter = node.split(':');
        let name = get_next(&mut iter);
        let field_string = get_next(&mut iter);
        let field_output = field_string
            .split(", ")
            .map(|field| AstField::new(field))
            .collect();
        Self {
            name: name,
            fields: field_output,
        }
    }
}

#[derive(Debug)]
struct AstField {
    pub field_name: String,
    pub field_type: String,
}

impl AstField {
    fn new(field: &str) -> Self {
        let mut iter = field.split(' ');
        let field_type = get_next(&mut iter);
        let field_name = get_next(&mut iter);
        Self {
            field_name: field_name,
            field_type: field_type,
        }
    }
}

fn get_next(i: &mut Iterator<Item = &str>) -> String {
    i.next().unwrap().trim().to_string()
}
