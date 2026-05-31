#![deny(clippy::all)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AstNode {
    Program(Vec<AstNode>),
    // Declarations
    EntityDef {
        name: String,
        components: Vec<ComponentRef>,
        events: Vec<EventHandler>,
    },
    ComponentDef {
        name: String,
        fields: Vec<FieldDecl>,
        methods: Vec<FnDef>,
    },
    EventDef {
        name: String,
        params: Vec<Param>,
    },
    FnDef {
        name: String,
        params: Vec<Param>,
        return_type: Option<Type>,
        is_coroutine: bool,
        body: Box<AstNode>,
    },
    // Statements
    VarDecl {
        name: String,
        mutable: bool,
        var_type: Option<Type>,
        initializer: Box<AstNode>,
    },
    Assignment {
        target: Box<AstNode>,
        op: AssignOp,
        value: Box<AstNode>,
    },
    IfStmt {
        condition: Box<AstNode>,
        then_branch: Box<AstNode>,
        else_branch: Option<Box<AstNode>>,
    },
    WhileStmt {
        condition: Box<AstNode>,
        body: Box<AstNode>,
    },
    ForRangeStmt {
        var: String,
        start: Box<AstNode>,
        end: Box<AstNode>,
        body: Box<AstNode>,
    },
    ReturnStmt {
        value: Option<Box<AstNode>>,
    },
    Break,
    Block(Vec<AstNode>),
    // Expressions
    Int(i32),
    Float(f64),
    Bool(bool),
    String(String),
    Null,
    Identifier(String),
    BinaryOp {
        left: Box<AstNode>,
        op: BinaryOpKind,
        right: Box<AstNode>,
    },
    UnaryOp {
        op: UnaryOpKind,
        expr: Box<AstNode>,
    },
    Call {
        callee: Box<AstNode>,
        args: Vec<AstNode>,
    },
    MemberAccess {
        object: Box<AstNode>,
        member: String,
    },
    Index {
        object: Box<AstNode>,
        index: Box<AstNode>,
    },
    Vec3(f64, f64, f64),
    ComponentRef {
        name: String,
        fields: Vec<(String, Type, AstNode)>,
    },
    EventHandler {
        name: String,
        params: Vec<Param>,
        body: Box<AstNode>,
    },
    StateDecl {
        name: String,
        var_type: Type,
        initializer: Box<AstNode>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Type {
    Int,
    Float,
    Bool,
    String,
    Void,
    Vec2,
    Vec3,
    Vec4,
    Quat,
    Entity,
    Named(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BinaryOpKind {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    Neq,
    Lt,
    Gt,
    Le,
    Ge,
    And,
    Or,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UnaryOpKind {
    Neg,
    Not,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AssignOp {
    Assign,
    AddAssign,
    SubAssign,
    MulAssign,
    DivAssign,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Param {
    pub name: String,
    pub param_type: Type,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FieldDecl {
    pub name: String,
    pub field_type: Type,
    pub default: Option<AstNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ComponentRef {
    pub name: String,
    pub fields: Vec<(String, AstNode)>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EventHandler {
    pub name: String,
    pub params: Vec<Param>,
    pub body: Vec<AstNode>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_program_ast() {
        let prog = AstNode::Program(vec![
            AstNode::Int(42),
            AstNode::Float(3.14),
            AstNode::Bool(true),
            AstNode::String("hello".to_string()),
            AstNode::Null,
        ]);
        assert!(matches!(prog, AstNode::Program(_)));
    }

    #[test]
    fn test_binary_op() {
        let expr = AstNode::BinaryOp {
            left: Box::new(AstNode::Int(1)),
            op: BinaryOpKind::Add,
            right: Box::new(AstNode::Int(2)),
        };
        assert!(matches!(expr, AstNode::BinaryOp { .. }));
    }

    #[test]
    fn test_entity_def() {
        let entity = AstNode::EntityDef {
            name: "Player".to_string(),
            components: vec![],
            events: vec![],
        };
        assert!(matches!(entity, AstNode::EntityDef { .. }));
    }
}
