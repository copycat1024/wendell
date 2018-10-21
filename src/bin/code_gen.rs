// bin/code_gen.rs

use std::fs::File;
use std::io::{self, Write};
use std::ops::Deref;

pub fn main() {
    let output_dir = "src/ast";

    let expr_nodes = [
        "Assign   : Token name, Expr value",
        "Binary   : Expr left, Token operator, Expr right",
        "Grouping : Expr expression",
        "Literal  : Token value",
        "Unary    : Token operator, Expr right",
        "Variable : Token name",
    ];
    let expr_atoms = ["scanner::token::Token"];
    if let Err(e) = define_ast(output_dir, "Expr", &expr_atoms, &expr_nodes) {
        println!("{:?}", e);
    }

    let stmt_nodes = [
        "Expression : Expr expression",
        "Print      : Expr expression",
        "Var        : Token name, Expr initializer",
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
    let AstConfig { base, nodes, atoms } = config;

    // Write header
    w.print_header(&path, &atoms)?;

    // Write base enum
    w.define_base(config)?;

    // Write AST structs
    for ref node in nodes.iter() {
        w.define_node(base, node)?;
    }

    // Write visitor trait
    w.define_visitor(config)

    // Write pretty printer
//    w.define_printer(config)
}

struct AstWriter<T: Write> {
    writer: T,
}

impl<T: Write> AstWriter<T> {
    fn from(writer: T) -> Self {
        Self { writer: writer }
    }

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
        self.println("#[derive(Debug)]")?;
        self.println(format!("pub enum {} {{", base))?;
        for ref node in nodes.iter() {
            let AstNode { name, .. } = node;
            self.println(format!("    {}({}),", name, name))?;
        }
        self.println(format!("    Empty,"))?;
        self.println("}")?;
        self.println("")
    }

    fn define_node(&mut self, base: &str, node: &AstNode) -> io::Result<()> {
        let AstNode { name, fields } = node;
        self.println("#[derive(Debug)]")?;
        self.println(format!("pub struct {} {{", name))?;
        for field in fields.iter() {
            let AstField {
                field_name,
                field_type,
            } = field;
            let field_type = if field_type != base {
                field_type.clone()
            } else {
                format!("Box<{}>", base)
            };
            self.println(format!("    pub {}: {},", field_name, field_type))?;
        }
        self.println("}")?;
        self.println("")
    }

    fn define_visitor(&mut self, config: &AstConfig) -> io::Result<()> {
        let AstConfig { base, nodes, .. } = config;
        self.println(format!("pub trait {}Visitor<T> {{", base))?;
        self.define_visitor_method(base)?;
        for ref node in nodes.iter() {
            let AstNode { name, .. } = node;
            self.define_visitor_method(name)?;
        }
        self.println("}")?;
        self.println("")
    }

    fn define_visitor_method(&mut self, name: &str) -> io::Result<()> {
        self.println(format!(
            "    fn visit_{}(&mut self, n: &{}) -> T;",
            name.to_lowercase(),
            name
        ))
    }

    fn define_printer(&mut self, config: &AstConfig) -> io::Result<()> {
        let AstConfig { base, nodes, .. } = config;
        self.println(format!("pub struct {}PrettyPrinter;", base))?;
        self.println(format!(
            "impl {}Visitor<String> for {}PrettyPrinter {{",
            base, base
        ))?;
        self.define_printer_base(base, nodes)?;
        for node in nodes.iter() {
            self.define_printer_node(node, config)?;
        }
        self.println("}")
    }

    fn define_printer_base(&mut self, base: &str, nodes: &[AstNode]) -> io::Result<()> {
        self.define_printer_method(base)?;
        self.println("        match n {")?;
        for node in nodes.iter() {
            let AstNode { name, .. } = node;
            self.define_printer_base_match(base, name)?;
        }
        self.println(format!(r#"            {}::Empty => "empty".into(),"#, base))?;
        self.println("        }")?;
        self.println("    }")?;
        self.println("")
    }

    fn define_printer_base_match(&mut self, base: &str, name: &str) -> io::Result<()> {
        self.println(format!(
            "            {}::{}(n) => self.visit_{}(n),",
            base,
            name,
            name.to_lowercase()
        ))
    }

    fn define_printer_node(&mut self, node: &AstNode, config: &AstConfig) -> io::Result<()> {
        let AstNode { name, fields } = node;

        self.define_printer_method(name)?;
        self.println("        parenthesize!(")?;
        self.println(format!(r#"            "{}","#, name.to_lowercase()))?;
        for field in fields.iter().take(fields.len() - 1) {
            self.define_printer_node_field(field, config, true)?;
        }
        let mut iter = fields.iter().skip(fields.len() - 1);
        self.define_printer_node_field(iter.next().unwrap(), config, false)?;
        self.println("        )")?;
        self.println("    }")?;
        self.println("")
    }

    fn define_printer_node_field(
        &mut self,
        field: &AstField,
        config: &AstConfig,
        comma: bool,
    ) -> io::Result<()> {
        let AstConfig { base, nodes, .. } = config;
        let AstField {
            field_name,
            field_type,
        } = field;

        let is_node_type = |x: &str| nodes.into_iter().any(|v| &(v.name) == &x);
        let item_for_base = || {
            format!(
                "            self.visit_{}(&n.{}){}",
                field_type.to_lowercase(),
                field_name,
                if comma { "," } else { "" }
            )
        };
        let item_for_node = || {
            format!(
                "            self.visit_{}(n.{}){}",
                field_type.to_lowercase(),
                field_name,
                if comma { "," } else { "" }
            )
        };
        let item_for_value = || {
            format!(
                "            n.{}{}",
                field_name,
                if comma { "," } else { "" }
            )
        };

        let item = match field_type {
            x if x == base => item_for_base(),
            x if is_node_type(x) => item_for_node(),
            _ => item_for_value(),
        };
        self.println(item)
    }

    fn define_printer_method(&mut self, name: &str) -> io::Result<()> {
        self.println(format!(
            "    fn visit_{}(&mut self, n: &{}) -> String {{",
            name.to_lowercase(),
            name
        ))
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
