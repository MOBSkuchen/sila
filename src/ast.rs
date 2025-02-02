use crate::lexer::CodePosition;

pub enum BinOp {
    Add,
    Sub,
    Div,
    Mul,
    Mod,
    Or,
    And,
    Xor,
    Eq,
    Neq,
    Not,
    As,
    Gt,
    Ls,
    Gte,
    Lse
}

pub enum NumberType {
    i8,
    i16,
    i32,
    i64,
    u8,
    u16,
    u32,
    u64,
    f32,
    f64,
    f128
}

pub enum ExpressionType {
    FunctionCall,
    BinaryExpression,

    String,
    Number,
    Identifier
}

pub enum NodeType {
    StringExpr,
    NumberExpr,
    IdentifierExpr,
    FuncCallExpr,
    BinOpExpr,
    FunctionDef,
    VarSet,
    Unary,
    Cast
}

// TODO : Implement for all members
pub trait Represent {}

pub trait AstNodeTrait {
    fn get_codepos(&self) -> &CodePosition;
    fn get_nodetype(&self) -> NodeType;
}
pub trait ExpressionTrait: AstNodeTrait {}

pub trait FunctionCallExprTrait: AstNodeTrait {
    fn get_fname(&self) -> &String;
    fn get_cargs(&self) -> &Vec<impl ExpressionTrait>;
}

pub trait IdentifierExprTrait: AstNodeTrait {
    fn get_sym_name(&self) -> &String;
}

pub trait BinOpExprTrait: AstNodeTrait {
    fn get_lhs(&self) -> &impl ExpressionTrait;
    fn get_rhs(&self) -> &impl ExpressionTrait;
    fn get_opc(&self) -> &BinOp;
}

pub trait StringExprTrait: AstNodeTrait {
    fn get_content(&self) -> &String;
}

pub trait NumberExprTrait: AstNodeTrait {
    fn get_num_as_str(&self) -> &String;
    fn get_num_type(&self) -> &NumberType;
}

pub trait FunctionDefTrait: AstNodeTrait {
    fn get_body(&self) -> &Vec<impl AstNodeTrait>;
    fn get_name(&self) -> &String;
    fn get_out_type(&self) -> &impl IdentifierExprTrait;
    fn get_args(&self) -> &Vec<(impl IdentifierExprTrait, impl IdentifierExprTrait)>;
}

pub trait VariableSetTrait: AstNodeTrait {
    fn get_name(&self) -> &String;
    fn is_mut(&self) -> bool;
    fn get_content(&self) -> &impl ExpressionTrait;
    fn get_type(&self) -> &Option<impl IdentifierExprTrait>;
    fn dec_type(&self) -> bool;
}

pub trait CastExprTrait: AstNodeTrait {
    fn get_into_type(&self) -> &impl IdentifierExprTrait;
    fn get_expr(&self) -> &impl ExpressionTrait;
}

pub trait UnaryExprTrait: AstNodeTrait {
    fn get_inner(&self) -> &impl ExpressionTrait;
}

pub struct UnaryExpr<Expr: ExpressionTrait> {
    expr: Expr,
    code_position: CodePosition,
}

pub struct CastExpr<Expr: ExpressionTrait, Ident: IdentifierExprTrait> {
    into_type: Ident,
    expr: Expr,
    code_position: CodePosition
}

impl<Expr: ExpressionTrait> AstNodeTrait for UnaryExpr<Expr> {
    fn get_codepos(&self) -> &CodePosition {
        &self.code_position
    }

    fn get_nodetype(&self) -> NodeType {
        NodeType::Unary
    }
}

impl<Expr: ExpressionTrait> UnaryExprTrait for UnaryExpr<Expr> {
    fn get_inner(&self) -> &impl ExpressionTrait {
        &self.expr
    }
}

impl<Expr: ExpressionTrait, Ident: IdentifierExprTrait> AstNodeTrait for CastExpr<Expr, Ident> {
    fn get_codepos(&self) -> &CodePosition {
        &self.code_position
    }

    fn get_nodetype(&self) -> NodeType {
        NodeType::Cast
    }
}

impl<Expr: ExpressionTrait, Ident: IdentifierExprTrait> CastExprTrait for CastExpr<Expr, Ident> {
    fn get_into_type(&self) -> &impl IdentifierExprTrait {
        &self.into_type
    }

    fn get_expr(&self) -> &impl ExpressionTrait {
        &self.expr
    }
}

pub struct FunctionCallExpr<Expr: ExpressionTrait> {
    function_name: String,
    call_args: Vec<Expr>,
    code_position: CodePosition
}

impl<Expr: ExpressionTrait> AstNodeTrait for FunctionCallExpr<Expr> {
    fn get_codepos(&self) -> &CodePosition {
        &self.code_position
    }

    fn get_nodetype(&self) -> NodeType {
        NodeType::FuncCallExpr
    }
}

impl<Expr: ExpressionTrait> FunctionCallExprTrait for FunctionCallExpr<Expr> {
    fn get_fname(&self) -> &String {
        &self.function_name
    }

    fn get_cargs(&self) -> &Vec<impl ExpressionTrait> {
        &self.call_args
    }
}

impl<Expr: ExpressionTrait> ExpressionTrait for FunctionCallExpr<Expr> {}

pub struct IdentifierExpr {
    name: String,
    code_position: CodePosition
}

impl IdentifierExpr {
    pub fn new(name: String, code_position: CodePosition) -> Self {
        Self {name, code_position}
    }
}

impl AstNodeTrait for IdentifierExpr {
    fn get_codepos(&self) -> &CodePosition {
        &self.code_position
    }

    fn get_nodetype(&self) -> NodeType {
        NodeType::IdentifierExpr
    }
}

impl IdentifierExprTrait for IdentifierExpr {
    fn get_sym_name(&self) -> &String {
        &self.name
    }
}

impl ExpressionTrait for IdentifierExpr {}

pub struct StringExpr {
    content: String,
    code_position: CodePosition
}

impl AstNodeTrait for StringExpr {
    fn get_codepos(&self) -> &CodePosition {
        &self.code_position
    }

    fn get_nodetype(&self) -> NodeType {
        NodeType::StringExpr
    }
}

impl StringExprTrait for StringExpr {
    fn get_content(&self) -> &String {
        &self.content
    }
}

impl ExpressionTrait for StringExpr {}

pub struct NumberExpr {
    number_raw_content: String,
    number_type: NumberType,
    code_position: CodePosition
}

impl AstNodeTrait for NumberExpr {
    fn get_codepos(&self) -> &CodePosition {
        &self.code_position
    }

    fn get_nodetype(&self) -> NodeType {
        NodeType::NumberExpr
    }
}

impl NumberExprTrait for NumberExpr {
    fn get_num_as_str(&self) -> &String {
        &self.number_raw_content
    }

    fn get_num_type(&self) -> &NumberType {
        &self.number_type
    }
}
impl ExpressionTrait for NumberExpr {}

pub struct BinOpExpr<Expr: ExpressionTrait> {
    lhs: Expr,
    rhs: Expr,
    opc: BinOp,
    code_position: CodePosition
}

impl<Expr: ExpressionTrait> BinOpExpr<Expr> {
    pub fn new(lhs: Expr, rhs: Expr, opc: BinOp, code_position: CodePosition) -> Self {
        Self {lhs, rhs, opc, code_position}
    }
}

impl<Expr: ExpressionTrait> AstNodeTrait for BinOpExpr<Expr> {
    fn get_codepos(&self) -> &CodePosition {
        &self.code_position
    }

    fn get_nodetype(&self) -> NodeType {
        NodeType::BinOpExpr
    }
}

impl<Expr: ExpressionTrait> BinOpExprTrait for BinOpExpr<Expr> {
    fn get_lhs(&self) -> &impl ExpressionTrait {
        &self.lhs
    }

    fn get_rhs(&self) -> &impl ExpressionTrait {
        &self.rhs
    }

    fn get_opc(&self) -> &BinOp {
        &self.opc
    }
}

impl<Expr: ExpressionTrait> ExpressionTrait for BinOpExpr<Expr> {}

pub struct FunctionDef<Node: AstNodeTrait, Ident: IdentifierExprTrait> {
    body: Vec<Node>,
    name: String,
    out_type: Ident,
    args: Vec<(Ident, Ident)>,
    code_position: CodePosition
}

impl<Node: AstNodeTrait, Ident: IdentifierExprTrait> AstNodeTrait for FunctionDef<Node, Ident> {
    fn get_codepos(&self) -> &CodePosition {
        &self.code_position
    }

    fn get_nodetype(&self) -> NodeType {
        NodeType::FunctionDef
    }
}

impl<Node: AstNodeTrait, Ident: IdentifierExprTrait> FunctionDefTrait for FunctionDef<Node, Ident> {
    fn get_body(&self) -> &Vec<impl AstNodeTrait> {
        &self.body
    }

    fn get_name(&self) -> &String {
        &self.name
    }

    fn get_out_type(&self) -> &impl IdentifierExprTrait {
        &self.out_type
    }

    fn get_args(&self) -> &Vec<(impl IdentifierExprTrait, impl IdentifierExprTrait)> {
        &self.args
    }
}

pub struct VariableSet<Expr: ExpressionTrait, Ident: IdentifierExprTrait> {
    name: String,
    mutable: bool,
    content: Expr,
    dec_type: Option<Ident>,
    code_position: CodePosition
}

impl<Expr: ExpressionTrait, Ident: IdentifierExprTrait> AstNodeTrait for VariableSet<Expr, Ident> {
    fn get_codepos(&self) -> &CodePosition {
        &self.code_position
    }

    fn get_nodetype(&self) -> NodeType {
        NodeType::VarSet
    }
}

impl<Expr: ExpressionTrait, Ident: IdentifierExprTrait> VariableSetTrait for VariableSet<Expr, Ident> {
    fn get_name(&self) -> &String {
        &self.name
    }

    fn is_mut(&self) -> bool {
        self.mutable
    }

    fn get_content(&self) -> &impl ExpressionTrait {
        &self.content
    }

    fn get_type(&self) -> &Option<impl IdentifierExprTrait> {
        &self.dec_type
    }

    fn dec_type(&self) -> bool {
        self.dec_type.is_some()
    }
}