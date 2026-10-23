use std::iter::successors;

use tree_sitter::Node;

use super::*;
use crate::*;

/// A node carrying the source needed to judge its shape.
#[derive(Clone, Copy)]
pub(super) struct Scan<'src> {
    node: Node<'src>,
    src: &'src str,
}

/// The mark of a boundary DTO: a derive that lets a wire format, a driver, a
/// query macro, a framework, or a CLI parser fill the struct field by field.
const DTO_DERIVES: &[&str] = &[
    "Serialize",
    "Deserialize",
    "FromRow",
    "QueryRow",
    "IntoParams",
    "ToSchema",
    "FromRef",
    "Parser",
    "Args",
    "Subcommand",
];

const TURBOFISHABLE: &[&str] = &[
    "collect",
    "parse",
    "into",
    "try_into",
    "sum",
    "product",
    "unwrap_or_default",
];

const EXEMPT_FUNCTIONS: &[&str] = &["main"];

/// An attribute whose macro requires a free function. A UI `#[component]`
/// expands into a props struct plus a builder, and it reads the `fn` at module
/// scope to do it; `#[proc_macro*]` must sit in the crate root. Nothing else
/// earns a home-less function.
const FRAMEWORK_ATTRIBUTES: &[&str] = &["component", "server", "island"];

impl<'src> Scan<'src> {
    pub(super) fn new(node: Node<'src>, src: &'src str) -> Self {
        Self { node, src }
    }

    fn node(self) -> Node<'src> {
        self.node
    }

    fn src(self) -> &'src str {
        self.src
    }

    pub(super) fn report(self, stem: &str) -> Vec<Item> {
        Crowd::read(self, stem)
            .violations()
            .into_iter()
            .chain(self.items())
            .collect()
    }

    /// Every rule but crowding, which judges a module body rather than one
    /// declaration and so reads the whole scan at once. Crowding alone follows
    /// declarations into test code: a fixture type is still a type, but a test
    /// function is not a free function.
    fn items(self) -> Vec<Item> {
        self.item()
            .into_iter()
            .chain(self.descend().flat_map(Scan::nested))
            .collect()
    }

    /// Test code still reads fields through accessors, and a helper beside the
    /// tests still needs a home.
    fn nested(self) -> Vec<Item> {
        match (self.is_test(), self.kind_of()) {
            (false, _) => self.items(),
            (true, "mod_item") => self.suite(),
            (true, _) => self.accesses(),
        }
    }

    fn suite(self) -> Vec<Item> {
        self.item()
            .filter(|item| item.is(Kind::Access) || item.is(Kind::Function))
            .into_iter()
            .chain(
                self.descend()
                    .flat_map(|child| if child.is_test() { child.accesses() } else { child.suite() }),
            )
            .collect()
    }

    fn accesses(self) -> Vec<Item> {
        self.item()
            .filter(|item| item.is(Kind::Access))
            .into_iter()
            .chain(self.descend().flat_map(Scan::accesses))
            .collect()
    }

    fn item(self) -> Option<Item> {
        self.kind()
            .filter(|kind| !kind.declares_a_type())
            .filter(|kind| !self.allows(kind.rule()))
            .map(|kind| Item::new(kind, self.describe(kind), self.node()))
    }

    pub(super) fn decls(self) -> Vec<Decl> {
        self.decl()
            .into_iter()
            .chain(self.descend().flat_map(Scan::decls))
            .collect()
    }

    fn decl(self) -> Option<Decl> {
        let kind = self.kind()?;
        let scope = Scope::holding(self.node())?;
        kind.declares_a_type().then(|| {
            Decl::new(Item::new(kind, self.describe(kind), self.node()), scope, self.allows(ONE_TYPE_PER_FILE))
        })
    }

    pub(super) fn impls(self) -> Implementors {
        let mut found = Implementors::new();
        for (scope, name, implementor) in self.pairs() {
            found.entry((scope, name)).or_default().push(implementor);
        }
        found
    }

    fn pairs(self) -> Vec<(Scope, String, String)> {
        self.implements()
            .into_iter()
            .chain(self.descend().flat_map(Scan::pairs))
            .collect()
    }

    fn implements(self) -> Option<(Scope, String, String)> {
        let scope = Scope::holding(self.node()).filter(|_| self.kind_of() == "impl_item")?;
        let name = self.field("trait").and_then(Scan::base_name)?;
        let implementor = self.field("type").and_then(Scan::base_name)?;
        Some((scope, name, implementor))
    }

    /// `super::Foo<T>` and `&Foo` both name `Foo`.
    fn base_name(self) -> Option<String> {
        match self.kind_of() {
            "type_identifier" => Some(self.text().to_string()),
            "generic_type" | "reference_type" => self.field("type").and_then(Scan::base_name),
            "scoped_type_identifier" => self.field("name").and_then(Scan::base_name),
            _ => None,
        }
    }

    fn exposed(self) -> bool {
        self.children().any(|child| child.kind_of() == "visibility_modifier")
    }

    fn kind(self) -> Option<Kind> {
        match self.node().kind() {
            "struct_item" | "enum_item" | "union_item" => Some(Kind::Type),
            "trait_item" => Some(Kind::Trait),
            "function_item" if self.free() => Some(Kind::Function),
            "field_declaration" if self.published() => Some(Kind::Field),
            "closure_expression" if self.forwarding() => Some(Kind::Closure),
            "let_declaration" if self.hinted() => Some(Kind::Binding),
            "type_parameter" if self.constrains_a_type() => Some(Kind::Bound),
            "field_expression" if self.reads_a_field() => Some(Kind::Access),
            "identifier" if self.reads_a_field_in_a_macro() => Some(Kind::Access),
            _ => None,
        }
    }

    /// A function is free when nothing owns it: its home is a file or a module,
    /// not an `impl` or a `trait`.
    fn free(self) -> bool {
        self.named("name")
            .filter(|name| !EXEMPT_FUNCTIONS.contains(name))
            .filter(|_| !self.owned_by_a_macro())
            .is_some_and(|_| self.at_module_scope())
    }

    fn at_module_scope(self) -> bool {
        Scope::of(self.node()).is_some()
    }

    fn owned_by_a_macro(self) -> bool {
        self.attributes()
            .filter_map(|text| text.trim_start_matches("#[").split(['(', ']']).next())
            .map(str::trim)
            .any(|name| name.starts_with("proc_macro") || FRAMEWORK_ATTRIBUTES.contains(&name))
    }

    /// A boundary DTO is the one struct allowed exposed fields. A directive
    /// over the struct exempts every field in it, so a wide DTO costs one line.
    fn published(self) -> bool {
        self.exposed() && !self.owner().is_some_and(Scan::exempts_its_fields)
    }

    fn owner(self) -> Option<Self> {
        self.ancestors()
            .find(|ancestor| matches!(ancestor.kind_of(), "struct_item" | "union_item"))
    }

    fn exempts_its_fields(self) -> bool {
        self.is_a_dto() || self.allows(PUB_FIELD)
    }

    fn is_a_dto(self) -> bool {
        self.attributes()
            .any(|text| DTO_DERIVES.iter().any(|derive| text.contains(derive)))
    }

    /// `|d| d.magnitude()` names a method and then hides it. The path says the
    /// same thing and composes.
    fn forwarding(self) -> bool {
        self.sole_parameter()
            .zip(self.receiver())
            .is_some_and(|(parameter, receiver)| parameter == receiver)
    }

    fn sole_parameter(self) -> Option<&'src str> {
        self.field("parameters")
            .filter(|parameters| parameters.node().named_child_count() == 1)
            .filter(|parameters| !parameters.text().contains(','))
            .and_then(|parameters| parameters.child(0))
            .filter(|parameter| parameter.kind_of() == "identifier")
            .map(Scan::text)
    }

    fn method(self) -> Option<&'src str> {
        self.call().and_then(|function| function.field("field")).map(Scan::text)
    }

    fn receiver(self) -> Option<&'src str> {
        self.call()
            .and_then(|function| function.field("value"))
            .filter(|value| value.kind_of() == "identifier")
            .map(Scan::text)
    }

    fn call(self) -> Option<Self> {
        self.field("body")
            .filter(|body| body.kind_of() == "call_expression")
            .filter(Scan::takes_no_arguments)
            .and_then(|body| body.field("function"))
            .filter(|function| function.kind_of() == "field_expression")
    }

    fn takes_no_arguments(&self) -> bool {
        self.field("arguments")
            .is_some_and(|arguments| arguments.node().named_child_count() == 0)
    }

    /// A `let` annotation only competes with a turbofish when the call it binds
    /// could carry the type itself.
    fn hinted(self) -> bool {
        self.field("type").is_some() && self.binds_a_turbofishable_call()
    }

    fn binds_a_turbofishable_call(self) -> bool {
        self.field("value")
            .filter(|value| value.kind_of() == "call_expression")
            .and_then(|value| value.field("function"))
            .filter(|function| function.kind_of() == "field_expression")
            .filter(|function| function.field("type_arguments").is_none())
            .and_then(|function| function.field("field"))
            .map(Scan::text)
            .is_some_and(|method| TURBOFISHABLE.contains(&method))
    }

    fn constrains_a_type(self) -> bool {
        self.field("bounds").is_some()
            && self
                .field("name")
                .is_some_and(|name| name.kind_of() == "type_identifier")
    }

    /// `self.j` outside the accessor that returns it. A method call is not a
    /// read, and neither is a tuple index.
    fn reads_a_field(self) -> bool {
        self.field("field")
            .is_some_and(|field| field.kind_of() == "field_identifier")
            && !self.is_called()
            && !self.is_an_accessor_body()
    }

    fn is_called(self) -> bool {
        self.parent()
            .map(|parent| match parent.kind_of() {
                "generic_function" => parent.parent().unwrap_or(parent),
                _ => parent,
            })
            .filter(|call| call.kind_of() == "call_expression")
            .and_then(|call| call.field("function"))
            .is_some_and(|function| {
                function.node() == self.node()
                    || function
                        .field("function")
                        .is_some_and(|inner| inner.node() == self.node())
            })
    }

    /// `fn j(&self) -> usize { self.j }`, `&mut self.j`, or one argument-free
    /// method on it: `self.names.as_slice()`.
    fn is_an_accessor_body(self) -> bool {
        self.field("value").is_some_and(|value| value.kind_of() == "self")
            && self
                .wrapper()
                .parent()
                .filter(|block| block.kind_of() == "block" && block.node().named_child_count() == 1)
                .and_then(Scan::parent)
                .filter(|function| function.kind_of() == "function_item")
                .and_then(|function| function.field("parameters"))
                .is_some_and(|parameters| {
                    parameters
                        .children()
                        .all(|parameter| parameter.kind_of() == "self_parameter")
                })
    }

    fn wrapper(self) -> Self {
        let parent = self.parent();
        let method = parent
            .filter(|parent| parent.kind_of() == "field_expression")
            .and_then(Scan::parent)
            .filter(|call| call.kind_of() == "call_expression")
            .filter(Scan::takes_no_arguments);
        let wrapped = method.unwrap_or(self);
        wrapped
            .parent()
            .filter(|parent| matches!(parent.kind_of(), "reference_expression" | "unary_expression"))
            .unwrap_or(wrapped)
    }

    /// A macro's token tree is not parsed, so `format!("{}", self.j)` is read
    /// token by token: a name after a `.` that no `(`, `!`, or `::` follows.
    fn reads_a_field_in_a_macro(self) -> bool {
        self.parent().is_some_and(|parent| parent.kind_of() == "token_tree")
            && self.node().prev_sibling().is_some_and(|dot| dot.kind() == ".")
            && self
                .node()
                .prev_sibling()
                .and_then(|dot| dot.prev_sibling())
                .is_some_and(|receiver| matches!(receiver.kind(), "identifier" | "self" | "token_tree"))
            && !self.node().next_sibling().is_some_and(|next| {
                matches!(next.kind(), "!" | "::")
                    || (next.kind() == "token_tree" && self.src()[next.byte_range()].starts_with('('))
            })
    }

    fn parent(self) -> Option<Self> {
        self.node().parent().map(|node| self.at(node))
    }

    fn describe(self, kind: Kind) -> String {
        match kind {
            Kind::Access if self.kind_of() == "identifier" => format!("_.{}", self.text()),
            Kind::Access => format!(
                "{}.{}",
                self.field("value")
                    .map_or("_", Scan::text)
                    .rsplit(['.', '\n'])
                    .next()
                    .unwrap_or("_")
                    .trim(),
                self.named("field").unwrap_or("_")
            ),
            Kind::Closure => format!(
                "{}| {}.{}",
                self.sole_parameter().unwrap_or("x"),
                self.sole_parameter().unwrap_or("x"),
                self.method().unwrap_or("method")
            ),
            Kind::Binding => self.field("pattern").map_or("_", Scan::text).trim().to_string(),
            Kind::Bound => self.named("name").unwrap_or("T").to_string(),
            _ => self.named("name").unwrap_or("_").to_string(),
        }
    }

    fn descend(self) -> impl Iterator<Item = Self> {
        self.children()
    }

    fn is_test(self) -> bool {
        matches!(self.kind_of(), "mod_item" | "function_item") && self.attributes().any(Block::is_test_attribute)
    }

    fn allows(self, rule: &str) -> bool {
        self.preceding()
            .take_while(Scan::is_trivia)
            .filter(Scan::is_comment)
            .map(Scan::text)
            .chain(self.overhead())
            .any(|text| Directive::grants(text, rule))
    }

    fn overhead(self) -> impl Iterator<Item = &'src str> {
        self.src()[..self.node().start_byte() - self.node().column()]
            .lines()
            .rev()
            .map(str::trim)
            .take_while(|line| line.starts_with("//"))
    }

    fn attributes(self) -> impl Iterator<Item = &'src str> {
        self.children()
            .filter(|child| child.kind_of() == "attribute_item")
            .chain(self.preceding().take_while(Scan::is_trivia))
            .filter(|node| node.kind_of() == "attribute_item")
            .map(Scan::text)
    }

    fn children(self) -> impl Iterator<Item = Self> {
        (0..self.node().named_child_count()).filter_map(move |index| self.child(index))
    }

    fn child(self, index: usize) -> Option<Self> {
        self.node().named_child(index).map(move |node| self.at(node))
    }

    fn ancestors(self) -> impl Iterator<Item = Self> {
        successors(self.node().parent(), Node::parent).map(move |node| self.at(node))
    }

    fn preceding(self) -> impl Iterator<Item = Self> {
        successors(self.node().prev_named_sibling(), Node::prev_named_sibling).map(move |node| self.at(node))
    }

    fn field(self, name: &str) -> Option<Self> {
        self.node().child_by_field_name(name).map(|node| self.at(node))
    }

    fn named(self, field: &str) -> Option<&'src str> {
        self.field(field).map(Scan::text)
    }

    fn at(self, node: Node<'src>) -> Self {
        Self { node, ..self }
    }

    fn kind_of(self) -> &'static str {
        self.node().kind()
    }

    fn text(self) -> &'src str {
        &self.src()[self.node().byte_range()]
    }

    fn is_comment(&self) -> bool {
        matches!(self.kind_of(), "line_comment" | "block_comment")
    }

    fn is_trivia(&self) -> bool {
        self.is_comment() || self.kind_of() == "attribute_item"
    }
}
