#![macro_use]

use num::bigint::BigInt;
use util::assoc::Assoc;
use name::*;
use std::rc::Rc;
use ast::Ast;
use ast_walk::{walk, WalkRule, LazyWalkReses};
use walk_mode::{WalkMode, NegativeWalkMode};
use form::Form;
use std;

/**
 * Values in Unseemly.
 */

#[derive(Debug,Clone,PartialEq)]
pub enum Value {
    Int(BigInt),
    Sequence(Vec<Rc<Value>>), // TODO: switch to a different core sequence type
    Function(Rc<Closure>), // TODO: unsure if this Rc is needed
    BuiltInFunction(BIF),
    AbstractSyntax(Ast), // Unsure if this needs an Rc.
    Struct(Assoc<Name, Value>),
    Enum(Name, Vec<Value>) // A real compiler would probably tag with numbers...
}

pub use self::Value::*;

#[derive(Debug, Clone, PartialEq)]
pub struct Closure {
    pub body: Ast,
    pub params: Vec<Name>,
    pub env: Assoc<Name, Value>
}

// Built-in function
pub struct BIF(pub Rc<(dyn Fn(Vec<Value>) -> Value)>);

impl PartialEq for BIF {
    fn eq(&self, other: &BIF) -> bool {
        self as *const BIF == other as *const BIF
    }
}

impl Clone for BIF {
    fn clone(&self) -> BIF {
        BIF(self.0.clone())
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match *self {
            Int(ref bi) => { write!(f, "{}", bi) }
            Sequence(ref seq) => {
                for elt in seq { try!(write!(f, "{}", &*elt)); }; Ok(())
            }
            Function(_) => { write!(f, "[closure]") }
            BuiltInFunction(_) => { write!(f, "[built-in function]") }
            AbstractSyntax(ref ast) => { write!(f, "'[{}]'", ast) }
            Struct(ref parts) => {
                try!(write!(f, "*["));
                for (k,v) in parts.iter_pairs() {
                    try!(write!(f, "{}: {} ", k, v));
                }
                write!(f, "]*")
            }
            Enum(n, ref parts) => {
                try!(write!(f, "+[{}", n));
                for p in parts.iter() { try!(write!(f, " {}", p)); }
                write!(f, "]+")
            }
        }
    }
}

impl std::fmt::Debug for BIF {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        formatter.write_str("[built-in function]")
    }
}

impl ::walk_mode::WalkElt for Value {
    fn from_ast(a: &Ast) -> Value {  AbstractSyntax(a.clone()) }
    fn to_ast(&self) -> Ast {
        match *self {
            AbstractSyntax(ref a) => a.clone(),
            _ => panic!("Type error: {} is not syntax", self)
        }
    }

    fn core_env() -> Assoc<Name, Self> { ::runtime::core_values::core_values() }
}



custom_derive!{
    #[derive(Copy, Clone, Debug, Reifiable)]
    pub struct Eval {}
}
custom_derive!{
    #[derive(Copy, Clone, Debug, Reifiable)]
    pub struct Destructure {}
}

impl WalkMode for Eval {
    fn name() -> &'static str { "Evalu" }

    type Elt = Value;
    type Negated = Destructure;
    type Err = ();
    type D = ::walk_mode::Positive<Eval>;
    type ExtraInfo = ();

    fn get_walk_rule(f: &Form) -> WalkRule<Eval> { f.eval.pos().clone() }
    fn automatically_extend_env() -> bool { true }

    fn walk_var(n: Name, cnc: &LazyWalkReses<Eval>) -> Result<Value, ()> {
        Ok(cnc.env.find(&n).expect("Undefined var; did you use a type name as a value?").clone())
    }

    // TODO: maybe keep this from being called?
    fn underspecified(_: Name) -> Value { val!(enum "why is this here?", ) }
}

impl WalkMode for Destructure {
    fn name() -> &'static str { "Destr" }

    type Elt = Value;
    type Negated = Eval;
    type Err = ();
    type D = ::walk_mode::Negative<Destructure>;
    type ExtraInfo = ();

    /// The whole point of program evaluation is that the enviornment
    ///  isn't generateable from the source tree.
    /// Does that make sense? I suspect it does not.
    fn get_walk_rule(f: &Form) -> WalkRule<Destructure> { f.eval.neg().clone() }
    fn automatically_extend_env() -> bool { true } // TODO: think about this
}

impl NegativeWalkMode for Destructure {
    fn needs_pre_match() -> bool { false } // Values don't have binding (in this mode!)
}

impl ::walk_mode::WalkElt for Ast {
    fn from_ast(a: &Ast) -> Ast { a.clone() }
    fn to_ast(&self) -> Ast { self.clone() }
}


pub fn eval_top(expr: &Ast) -> Result<Value, ()> {
    eval(expr, Assoc::new())
}

pub fn eval(expr: &Ast, env: Assoc<Name, Value>) -> Result<Value, ()> {
    walk::<Eval>(expr, &LazyWalkReses::new_wrapper(env))
}

pub fn neg_eval(pat: &Ast, env: Assoc<Name, Value>)
        -> Result<Assoc<Name, Value>,()> {
    walk::<Destructure>(pat, &LazyWalkReses::new_wrapper(env))
}

custom_derive!{
    #[derive(Copy, Clone, Debug, Reifiable)]
    pub struct QQuote {}
}
custom_derive!{
    #[derive(Copy, Clone, Debug, Reifiable)]
    pub struct QQuoteDestr {}
}

impl WalkMode for QQuote {
    fn name() -> &'static str { "QQuote" }

    // Why not `Ast`? Because QQuote and Eval need to share environments.
    type Elt = Value;
    type Negated = QQuoteDestr;
    type Err = ();
    type D = ::walk_mode::Positive<QQuote>;
    type ExtraInfo = ();

    fn walk_var(n: Name, _: &LazyWalkReses<Self>) -> Result<Value, ()> {
        let n_sp = &n.sp();
        Ok(val!(ast (vr n_sp)))
    }
    fn walk_atom(n: Name, _: &LazyWalkReses<Self>) -> Result<Value, ()> {
        let n_sp = &n.sp();
        Ok(val!(ast n_sp))
    }
    fn get_walk_rule(f: &Form) -> WalkRule<QQuote> { f.quasiquote.pos().clone() }
    fn automatically_extend_env() -> bool { false }
}

impl WalkMode for QQuoteDestr {
    fn name() -> &'static str { "QQDes" }

    type Elt = Value;
    type Negated = QQuote;
    type Err = ();
    type D = ::walk_mode::Negative<QQuoteDestr>;
    type ExtraInfo = ();

    fn walk_var(n: Name, cnc: &LazyWalkReses<Self>) -> Result<Assoc<Name, Value>, ()> {
        let n_sp = &n.sp();
        if cnc.context_elt() == &val!(ast (vr n_sp)) {
            Ok(Assoc::<Name, Value>::new())
        } else {
            Err(Self::qlit_mismatch_error(val!(ast (vr n_sp)), cnc.context_elt().clone()))
        }
    }
    fn walk_atom(n: Name, cnc: &LazyWalkReses<Self>) -> Result<Assoc<Name, Value>, ()> {
        let n_sp = &n.sp();
        if cnc.context_elt() == &val!(ast n_sp) {
            Ok(Assoc::<Name, Value>::new())
        } else {
            Err(Self::qlit_mismatch_error(val!(ast (vr n_sp)), cnc.context_elt().clone()))
        }
    }
    fn get_walk_rule(f: &Form) -> WalkRule<QQuoteDestr> { f.quasiquote.neg().clone() }
    fn automatically_extend_env() -> bool { false }
}

impl NegativeWalkMode for QQuoteDestr {
    fn needs_pre_match() -> bool { true } // Quoted syntax does have binding!
}


// `env` is a trap! We want a shifted `LazyWalkReses`!
/*
pub fn qquote(expr: &Ast, env: Assoc<Name, Value>) -> Result<Value, ()> {
    walk::<QQuote>(expr, &LazyWalkReses::new_wrapper(env))
}

pub fn qquote_destr(pat: &Ast, env: Assoc<Name, Value>)
        -> Result<Assoc<Name, Value>,()> {
    walk::<QQuoteDestr>(pat, &LazyWalkReses::new_wrapper(env))
}
*/
