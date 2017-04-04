#![macro_use]

use parse::FormPat;
use name::*;
use std::fmt::{Debug,Formatter,Error};
use util::assoc::Assoc;
use std::rc::Rc;
use ast_walk::WalkRule;
use ty::Ty;
use ast::Ast;
use runtime::eval::Value;

pub type NMap<T> = Assoc<Name, T>;

/// BiDirectionalWalkRule: a walk rule, abstracted over whether the walk is positive or negative
pub type BiDiWR<Mode, NegMode> = EitherPN<WalkRule<Mode>, WalkRule<NegMode>>;

custom_derive! {
    /// Unseemly language form. This is what tells us what a particular `Node` actually does.
    #[derive(Reifiable)]
    pub struct Form {
        /// The name of the form. Mainly for internal use.
        pub name: Name,
        /** The grammar the programmer should use to invoke this form. 
         * This contains information about bindings and syntax extension; this is where it belongs!
         */
        pub grammar: Rc<FormPat>,
        /** From a type environment, construct the type of this term. */
        pub synth_type: BiDiWR<::ty::SynthTy, ::ty::UnpackTy>,
        /** From a value environment, evaluate this term.*/
        pub eval: BiDiWR<::runtime::eval::Eval, ::runtime::eval::Destructure>,
        /** At runtime, pick up code to use it as a value */
        pub quasiquote: BiDiWR<::runtime::eval::QQuote, ::runtime::eval::QQuoteDestr>,
        pub relative_phase: Assoc<Name, i32> /* 2^31 macro phases ought to be enough for anybody */
    }
}

custom_derive! {
    /// The distinction between `Form`s with positive and negative walks is documented at `Mode`.
    #[derive(Reifiable)]
    pub enum EitherPN<L, R> {
        Positive(L),
        Negative(R),
        Both(L, R)
        // Maybe instead of WalkRule::NotWalked, we need EitherPN::Neither
    }
}
pub use self::EitherPN::*;


impl<L, R> EitherPN<L, R> {
    pub fn pos(&self) -> &L {
        match *self { 
            Positive(ref l) | Both(ref l, _) => l, 
            Negative(_) => panic!("ICE: wanted positive walk"),
        }
    }
    pub fn neg(&self) -> &R {
        match *self {
            Negative(ref r) | Both(_, ref r)=> r, 
            Positive(_) => panic!("ICE: wanted negative walk"),
        }
    }
    pub fn is_pos(&self) -> bool { match *self { Negative(_) => false, _ => true }}
    pub fn is_neg(&self) -> bool { match *self { Positive(_) => false, _ => true }}
}


impl PartialEq for Form {
    /// pointer equality on the underlying structure!
    fn eq(&self, other: &Form) -> bool { 
        self as *const Form == other as *const Form
    }
}


impl Debug for Form {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), Error> {
        formatter.write_str(format!("[FORM {:?}]", self.name).as_str())
    }
}


pub fn simple_form(form_name: &str, p: FormPat) -> Rc<Form> {
    Rc::new(Form {
            name: n(form_name),
            grammar: Rc::new(p),
            relative_phase: Assoc::new(), 
            synth_type: ::form::Positive(WalkRule::NotWalked),
            eval: ::form::Positive(WalkRule::NotWalked),
            quasiquote: ::form::Both(WalkRule::LiteralLike, WalkRule::LiteralLike)
        })
}
