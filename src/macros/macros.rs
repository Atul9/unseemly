#![macro_use]

// TODO: use a real logging framework
macro_rules! log {
    ($($e:expr),*) => {
        // print!( $($e),* );
    }
}

/* Assoc */

macro_rules! expr_ify {
    ($e:expr) => {$e};
}

macro_rules! assoc_n {
    () => { ::util::assoc::Assoc::new() };
    ( $k:tt => $v:expr $(, $k_cdr:tt => $v_cdr:expr)* ) => {
        assoc_n!( $( $k_cdr => $v_cdr ),* ).set(::name::n(expr_ify!($k)), $v)
    };
    ( ($k:expr) => $v:expr $(, $k_cdr:tt => $v_cdr:expr)* ) => {
        assoc_n!( $( $k_cdr => $v_cdr ),* ).set(::name::n($k), $v)
    };
}




/* Beta */

macro_rules! beta_connector {
    ( : ) => { ::beta::Basic };
    ( = ) => { ::beta::SameAs }
}

macro_rules! beta {
    ( [] ) => { ::beta::Nothing };
    ( [* $body:tt ]) => {
        {
            let sub = beta!($body);
            let drivers = sub.names_mentioned();
            ::beta::ShadowAll(Box::new(sub), drivers)
        }
    };
    ( [ forall $name:tt $( $rest:tt )*] ) => {
        ::beta::Shadow(Box::new(::beta::Underspecified(::name::n(expr_ify!($name)))),
               Box::new(beta!( [ $( $rest )* ] )))
    };
    ( [ prot $name:tt $( $rest:tt )*] ) => {
        ::beta::Shadow(Box::new(::beta::Protected(     ::name::n(expr_ify!($name)))),
               Box::new(beta!( [ $( $rest )* ] )))
    };
    ( [ $name:tt $connector:tt $t:tt
        $( $rest:tt )*
         ] ) => {
        ::beta::Shadow(Box::new(beta_connector!($connector)(::name::n(expr_ify!($name)),
                                                    ::name::n(expr_ify!($t)))),
               Box::new(beta!( [ $( $rest )* ] )))
    };
}

macro_rules! ebeta {
    ( [] ) => { ::beta::ExportBeta::Nothing };
    ( [* $body:tt ]) => {
        {
            let sub = ebeta!($body);
            let drivers = sub.names_mentioned();
            ::beta::ExportBeta::ShadowAll(Box::new(sub), drivers)
        }
    };
    ( [ $name:tt $( $rest:tt )*] ) => {
        ::beta::ExportBeta::Shadow(Box::new(::beta::ExportBeta::Use(::name::n(expr_ify!($name)))),
               Box::new(ebeta!( [ $( $rest )* ] )))
    };
}


/* Read */

macro_rules! tokens {
    ($($contents:tt)*) => { TokenTree{t: vec![ $(  t_elt!($contents) ),* ] }}
}

macro_rules! t_elt {
    ( [ $e:expr ;  $( $list:tt )* ] ) => {
        Group(::name::n(concat!($e,"[")), ::read::DelimChar::SquareBracket, tokens!($($list)*))
    };
    ( { $e:expr ;  $( $list:tt )* } ) => {
        Group(::name::n(concat!($e,"{")), ::read::DelimChar::CurlyBracket, tokens!($($list)*))
    };
    ( ( $e:expr ;  $( $list:tt )* ) ) => {
        Group(::name::n(concat!($e,"(")), ::read::DelimChar::Paren, tokens!($($list)*))
    };
    ($e:expr) => { Simple(::name::n($e)) }
}



/* Ast */

macro_rules! ast_shape {
    ($($contents:tt)*) => { ::ast::Shape(vec![ $(  ast!($contents) ),* ] )};
}

macro_rules! ast {
    ( (trivial) ) => { ::ast::Trivial };
    ( (++ $pos:tt $sub:tt) ) => {
        ::ast::QuoteMore(Box::new(ast!($sub)), $pos)
    };
    ( (-- $depth:tt $sub:tt ) ) => {
        ::ast::QuoteLess(Box::new(ast!($sub)), $depth)
    };
    ( (import $beta:tt $sub:tt) ) => {
        ::ast::ExtendEnv(Box::new(ast!($sub)), beta!($beta))
    };
    /* // not sure we'll need this
    ( (* $env:expr => $new_env:ident / $($n:expr),* ; $($sub_ar"gs:tt)*) ) => {
        {
            let mut res = vec![];

            for $new_env in $env.march_all( &vec![$($n),*] ) {
                res.push(ast!($sub))
            }
            res.reverse();
            Shape(res)
        }
    };*/
    ( (vr $var:expr) ) => { ::ast::VariableReference(::name::n($var)) };
    ( (, $interpolate:expr)) => { $interpolate };
    // TODO: maybe we should use commas for consistency:
    ( ( $( $list:tt )* ) ) => { ast_shape!($($list)*)};
    ( { - $($mbe_arg:tt)* } ) => {
        ::ast::IncompleteNode(mbe!( $($mbe_arg)* ))
    };
    ( { $nt:tt $form:tt => $beta:tt : $($mbe_arg:tt)*} ) => {
        ::ast::Node(::core_forms::find_core_form($nt, $form), mbe!( $($mbe_arg)* ),
                    ebeta!($beta))
    };
    ( { $form:expr => $beta:tt ; $($mbe_arg:tt)*} ) => {
        ::ast::Node($form, mbe!( $($mbe_arg)* ), ebeta!($beta))
    };
    ( { $form:expr; [ $($mbe_arg:tt)* ] }) => {
        ast!( { $form ; $($mbe_arg)* } )
    };
    ( { $form:expr; $($mbe_arg:tt)* }) => {
        ::ast::Node($form, mbe!( $($mbe_arg)* ), ::beta::ExportBeta::Nothing)
    };
    ( { $nt:tt $form:tt : $($mbe_arg:tt)* }) => {
        ::ast::Node(::core_forms::find_core_form($nt, $form), mbe!( $($mbe_arg)* ),
                    ::beta::ExportBeta::Nothing)
    };
    ($e:expr) => { ::ast::Atom(::name::n($e))}
}

/* Ty */

// Note that interpolations into this have to be `Ast`, not `Ty`.
// This isn't ideal, but the macrology involved in fixing that is a bridge too far for me
macro_rules! ty {
    ( $($contents:tt)* ) => { ::ty::Ty::new(ast!($($contents)*)) }
}

// These construct spanned type errors (so, for type synthesis, not subtyping)

macro_rules! ty_err_val {
    ( $name:tt ( $($arg:expr),* ) at $loc:expr) => {
        ::util::err::sp(::ty::TyErr::$name( $($arg),* ), $loc.clone())
    }
}

macro_rules! ty_err {
    ( $name:tt ( $($arg:expr),* ) at $loc:expr) => {
        return Err(ty_err_val!( $name ( $($arg),* ) at $loc));
    }
}

macro_rules! ty_exp { // type expectation
    ( $got:expr, $expected:expr, $loc:expr) => {
        if $got != $expected {
            ty_err!(Mismatch((*$got).clone(), (*$expected).clone()) at $loc)
        }
    }
}

macro_rules! ty_err_p { // type error pattern
    ( $name:tt ( $($arg:pat),* ) ) => {
        Err( ::util::err::Spanned { body: ::ty::TyErr::$name( $($arg),* ), loc: _ } )
    }
}

/* EnvMBE */

/* These macros generate `EnvMBE<Ast>`s, not arbitrary `EnvMBE`s,
    which is a little un-abstract, but is the main usage. */

/*
 * Wait a second, I'm writing in Rust right now! I'll use an MBE macro to implement an MBE literal!
 */
macro_rules! mbe_one_name {
    // `elt` ought to have an interpolation that references `new_env`
    // TODO: maybe this (and the parser!) ought to add 0-times-repeated leaves to `leaf_locations`
    /* TYPE PUN ALERT: $env has to be something with a `march_all` method;
        there's no trait enforcing this.

       But wait, isn't preventing this kind of nonsense the whole point of this project?

       Well, you know the old saying:
        "While the mice are implementing the cat, the mice will play."
    */
    ($k:tt => [* $env:expr =>($($n:expr),*) $new_env:ident : $elt:tt]) => {
        {
            let mut v = vec![];
            let marchee = vec![$(::name::n($n)),*];
            for $new_env in $env.march_all(&marchee) {
                v.push( mbe_one_name!($k => $elt));
            }
            ::util::mbe::EnvMBE::new_from_anon_repeat(v)
        }
    };

    ($k:tt => [@ $n:tt $($elt:tt),*]) => {
        ::util::mbe::EnvMBE::new_from_named_repeat(
            ::name::n(expr_ify!($n)),
            vec![ $( mbe_one_name!($k => $elt) ),* ]
        )
    };

    ($k:tt => [...($elt_rep:tt)... $(, $elt_post:tt)*]) => { // (the stanza below won't parse it)
        ::util::mbe::EnvMBE::new_from_anon_repeat_ddd(
            vec![ mbe_one_name!($k => $elt_rep) ,
                    $( mbe_one_name!($k => $elt_post) ),* ],
              Some(0)
        )
    };

    ($k:tt => [$($elt_pre:tt),* ...($elt_rep:tt)... $(, $elt_post:tt)*]) => {
        ::util::mbe::EnvMBE::new_from_anon_repeat_ddd(
            vec![ $( mbe_one_name!($k => $elt_pre) ),* ,
                  mbe_one_name!($k => $elt_rep) ,
                  $( mbe_one_name!($k => $elt_post) ),* ],
              Some( vec![$( {$elt_pre; ()} ),*].len() ) // count the things in `$elt_pre`
        )
    };

    ($k:tt => [$($elt:tt),*]) => {
        ::util::mbe::EnvMBE::new_from_anon_repeat(
            vec![ $( mbe_one_name!($k => $elt) ),* ])
    };

    // since `Ast`s go on the RHS, we have to have a distinctive interpolation syntax
    ($k:tt => (,seq $e:expr)) => {
        {
            let mut v = vec![];
            for elt in $e {
                v.push(::util::mbe::EnvMBE::new_from_leaves(assoc_n!($k => elt)))
            }
            ::util::mbe::EnvMBE::new_from_anon_repeat(v)
        }
    };

    ($k:tt => (@ $rep_n:tt ,seq $e:expr)) => {
        {
            let mut v = vec![];
            for elt in $e {
                v.push(::util::mbe::EnvMBE::new_from_leaves(assoc_n!($k => elt)))
            }
            ::util::mbe::EnvMBE::new_from_named_repeat(::name::n(expr_ify!($rep_n)), v)
        }
    };

    // For parsing reasons, we only accept expressions that are TTs.
    // It's hard to generalize the `mbe!` interface so that it accepts exprs
    // or `[]`-surrounded trees of them.
    ($k:tt => $leaf:tt) => {
        ::util::mbe::EnvMBE::new_from_leaves(assoc_n!($k => ast!($leaf)))
    }
}


// Eventually, this ought to support more complex structures
macro_rules! mbe {
    ( $( $lhs:tt => $rhs:tt ),* ) => {{
        let single_name_mbes = vec![ $( mbe_one_name!($lhs => $rhs) ),*];
        let mut res = ::util::mbe::EnvMBE::new();
        for m in &single_name_mbes {
            res = res.merge(m);
        }
        res
    }}
}



/* FormPat */

// TODO #8: `ast!` and `form_pat!` are inconsistent with each other.
macro_rules! form_pat {
    ((lit $e:expr)) => { ::grammar::FormPat::Literal(::name::n($e)) };
    ((lit_by_name $e:expr)) => { ::grammar::FormPat::Literal($e) };
    ((anyways $a:tt)) => { ::grammar::FormPat::Anyways(ast!($a)) };
    ((impossible)) => { ::grammar::FormPat::Impossible };
    (at) => { ::grammar::FormPat::AnyToken };
    (aat) => { ::grammar::FormPat::AnyAtomicToken };
    (varref) => { ::grammar::FormPat::VarRef };
    ((delim $n:expr, $d:expr, $body:tt)) => {
        ::grammar::FormPat::Delimited(::name::n($n), ::read::delim($d),
                          ::std::rc::Rc::new(form_pat!($body)))
    };
    ((star $body:tt)) => { ::grammar::FormPat::Star(::std::rc::Rc::new(form_pat!($body))) };
    ((plus $body:tt)) => { ::grammar::FormPat::Plus(::std::rc::Rc::new(form_pat!($body))) };
    ((alt $($body:tt),* )) => { ::grammar::FormPat::Alt(vec![
        $( ::std::rc::Rc::new(form_pat!($body)) ),* ] )};
    ((biased $lhs:tt, $rhs:tt)) => {
        ::grammar::FormPat::Biased(::std::rc::Rc::new(form_pat!($lhs)),
                                 ::std::rc::Rc::new(form_pat!($rhs))) };
    ((call $n:expr)) => { ::grammar::FormPat::Call(::name::n($n)) };
    ((call_by_name $n:expr)) => { ::grammar::FormPat::Call($n) };
    ((scope $f:expr)) => { ::grammar::FormPat::Scope($f, ::beta::ExportBeta::Nothing) };
    ((scope $f:expr, $ebeta:tt)) => { ::grammar::FormPat::Scope($f, ebeta!($ebeta)) };
    ((named $n:expr, $body:tt)) => {
        ::grammar::FormPat::Named(::name::n($n), ::std::rc::Rc::new(form_pat!($body)))
    };
    ((import $beta:tt, $body:tt)) => {
        ::grammar::FormPat::NameImport(::std::rc::Rc::new(form_pat!($body)), beta!($beta))
    };
    ((++ $pos:tt $body:tt)) => { // `pos` should be an expr, but I didn't want a comma. Name it.
        ::grammar::FormPat::QuoteDeepen(::std::rc::Rc::new(form_pat!($body)), $pos)
    };
    ((-- $depth:tt $body:tt)) => {
        ::grammar::FormPat::QuoteEscape(::std::rc::Rc::new(form_pat!($body)), $depth)
    };
    ((extend $lhs:tt, $n:expr, $f:expr)) => {
        ::grammar::FormPat::SynImport(::std::rc::Rc::new(form_pat!($lhs)), ::name::n($n),
            ::grammar::SyntaxExtension(::std::rc::Rc::new(Box::new($f))))
    };
    ( [$($body:tt),*] ) => {
        ::grammar::FormPat::Seq(vec![ $( ::std::rc::Rc::new(form_pat!($body)) ),* ])}
}



/* utility, for core_forms and core_type_forms */
// This has to be a macro for type reasons involving sizedness I don't understand.
macro_rules! cust_rc_box {
    ($contents:expr) => { Custom(::std::rc::Rc::new(Box::new($contents))) }
}



/* Form */

macro_rules! basic_typed_form {
    ( $p:tt, $gen_type:expr, $eval:expr ) => {
        Rc::new(Form {
            name: ::name::n("unnamed form"),
            grammar: Rc::new(form_pat!($p)),
            type_compare: ::form::Positive(::ast_walk::WalkRule::NotWalked),
            synth_type: ::form::Positive($gen_type),
            quasiquote: ::form::Both(::ast_walk::WalkRule::LiteralLike,
                                     ::ast_walk::WalkRule::LiteralLike),
            eval: ::form::Positive($eval)
        })
    }
}

macro_rules! typed_form {
    ( $name:expr, $p:tt, $gen_type:expr, $eval:expr ) => {
        Rc::new(Form {
            name: ::name::n($name),
            grammar: Rc::new(form_pat!($p)),
            type_compare: ::form::Positive(::ast_walk::WalkRule::NotWalked),
            synth_type: ::form::Positive($gen_type),
            quasiquote: ::form::Both(::ast_walk::WalkRule::LiteralLike,
                                     ::ast_walk::WalkRule::LiteralLike),
            eval: ::form::Positive($eval)
        })
    }
}

macro_rules! negative_typed_form {
    ( $name:expr, $p:tt, $gen_type:expr, $eval:expr ) => {
        Rc::new(Form {
            name: ::name::n($name),
            grammar: Rc::new(form_pat!($p)),
            type_compare: ::form::Positive(::ast_walk::WalkRule::NotWalked),
            synth_type: ::form::Negative($gen_type),
            quasiquote: ::form::Both(::ast_walk::WalkRule::LiteralLike,
                                     ::ast_walk::WalkRule::LiteralLike),
            eval: ::form::Negative($eval)
        })
    }
}



/* Value */

macro_rules! val {
    (i $i:expr) => { ::runtime::eval::Value::Int(::num::bigint::BigInt::from($i)) };
    (b $b:expr) => {
        ::runtime::eval::Value::Enum( ::name::n(if $b {"True"} else {"False"}), vec![])
    };
    (cons $a:tt, $d:tt) => { ::runtime::eval::Value::Cons(Rc::new(val!($a)), Rc::new(val! $d )) };
    (f $body:tt, $params:expr, $env:tt) => {
        ::runtime::eval::Value::Function(
            Rc::new(::runtime::eval::Closure(ast!($body), $params, assoc_n! $env)))
    };
    (bif $f:expr) => {
        ::runtime::eval::Value::BuiltInFunction(::runtime::eval::BIF(Rc::new($f)))
    };
    (ast $body:tt) => {
        ::runtime::eval::Value::AbstractSyntax(ast!($body))
    };
    (struct $( $k:tt => $v:tt ),* ) => {
        ::runtime::eval::Value::Struct(assoc_n!( $( $k => val! $v),* ))
    };
    (enum $nm:expr, $($v:tt),*) => {
        ::runtime::eval::Value::Enum(::name::n($nm), vec![ $( val! $v ),* ])
    };
    (seq $($v:tt)*) => {
        ::runtime::eval::Value::Sequence(vec![ $( Rc::new(val! $v) ),* ])
    };
    (, $interpolate:expr) => { $interpolate }
}



/* core_values stuff */

macro_rules! mk_type { // TODO: maybe now use find_core_form and un-thread $se?
    ( [ ( $( $param:tt ),* )  -> $ret_t:tt ] ) => {
        ast!( { ::core_forms::find_core_form("Type", "fn") ;
                  "param" => [ $((, mk_type!($param) )),*],
                  "ret" => (, mk_type!($ret_t))
        })
    };
    ( $n:tt ) => { ast!({ "Type" $n : }) }; // atomic type
}

/* Define a typed function */
macro_rules! tf {
    (  [ ( $($param_t:tt),* ) -> $ret_t:tt ] ,
       ( $($param_p:pat),* ) => $body:expr) => {
        TypedValue {
            ty: mk_type!([ ( $($param_t),* ) -> $ret_t ] ),
            val: core_fn!( $($param_p),* => $body )
        }
    };
    (  $n:tt, $e:expr ) => {
        TypedValue {
            ty: mk_type!( $n ),
            val: $e
        }
    }
}

/* Like `tf!`, but actually uses `ast!`, which is more flexible than `mk_type!` */
macro_rules! tyf {
    ( $t:tt, ( $($param_p:pat),* ) => $body:expr ) => {
        TypedValue { ty: ast!($t), val: core_fn!($($param_p),* => $body) }
    }
}

macro_rules! bind_patterns {
    ( $iter:expr; () => $body:expr ) => { $body };
    ( $iter:expr; ($p_car:pat, $($p_cdr:pat,)* ) => $body:expr ) => {
        match $iter.next() {
            Some($p_car) => {
                bind_patterns!($iter; ($( $p_cdr, )*) => $body)
            }
            None => { panic!("ICE: too few arguments"); }
            Some(ref other) => { panic!("Type ICE in argument: {:#?}", other); }
        }
    }
}

macro_rules! core_fn {
    ( $($p:pat),* => $body:expr ) => {
        BuiltInFunction(BIF(Rc::new(
            move | args | {
                let mut argi = args.into_iter();
                bind_patterns!(argi; ($( $p, )*) => $body )
            }
        )))
    }
}


/* Alpha */
macro_rules! without_freshening {
    ($( $body:tt )*) => {{
        let mut orig: bool = false;
        ::alpha::freshening_enabled.with(|f| {
            orig = *f.borrow();
            *f.borrow_mut() = false;
        });
        { $( $body )* }
        ::alpha::freshening_enabled.with(|f| {
            *f.borrow_mut() = orig;
        });
    }}
}


/* for core_forms */

/* Unpacking `Ast`s into environments is a pain, so here's a macro for it*/
macro_rules! expect_node {
    ( ($node:expr ; $form:expr) $env:ident ; $body:expr ) => (
        // This is tied to the signature of `Custom`
        if let Node(ref f, ref $env, _) = $node {
            if *f == $form {
                $body
            } else {
                // TODO: make it possible to specify which one
                panic!("ICE or type error: Expected a {:#?} node, got {:#?}, which is {:#?}.",
                       $form, $node, *f)
            }
        } else {
            panic!("ICE or type error: Expected a {:#?} node, got {:#?}, which isn't a node.",
                   $form, $node)
        }
    )
}

macro_rules! expect_ty_node {
    ( ($node:expr ; $form:expr ; $loc:expr) $env:ident ; $body:expr ) => ({
        // This is tied to the signature of `Custom`
        let $env = $node.destructure($form, $loc)?;
        $body
    })
}


// TODO: this ought to have some MBE support
macro_rules! destructure_node {
    ( ($node:expr ; $form:expr) $( $n:ident = $name:expr ),* ; $body:expr ) => (
        expect_node!( ($node ; $form) env ; {
            let ( $( $n ),* ) = ( $( env.get_leaf_or_panic(&::name::n($name)) ),* );
            $body
        })
    )
}

macro_rules! forms_to_form_pat {
    ( $( $form:expr ),* ) => {
        form_pat!((alt $( (scope $form) ),* ))
    }
}

macro_rules! forms_to_form_pat_export {
    ( $( $form:expr => $export:tt),* ) => {
        form_pat!((alt $( (scope $form, $export) ),* ))
    }
}


/* panicking destructor (when the type system should offer protection) */

macro_rules! extract {
    (($v:expr) $( $expected:path = ( $( $sub:pat ),* ) => $body:expr);* ) => {
        match * $v {
            $( $expected ( $($sub),* ) => { $body } )*
            _ => { panic!("ICE: {:#?} isn't a {:#?}", $v, stringify!( $($expected),* )) }
        }
    }
}


/* Reification helper (doesn't work on parameterized types...) */

macro_rules! cop_out_reifiability {
    ( $underlying_type:ty, $ty_name:tt ) => {
        impl Reifiable for $underlying_type {
            fn ty_name() -> Name { n(stringify!($ty_name)) }

            fn reify(&self) -> Value { Value::Smuggled(self.clone()) }

            fn reflect(v: &Value) -> Self {
                extract!((v) Value::Smuggled = (ref s) =>
                    s.downcast_ref::<Self>().expect("Smuggling has gone terribly wrong!").clone())
            }
        }
    }
}

/* Testing */

macro_rules! assert_m {
    ($got:expr, $expected:pat, $body:expr) => {{
        let got = $got;
        match got.clone() {
            $expected => { assert!($body) }
            _ => { assert!(false, "{:#?} does not match {:#?}", got, quote!($expected).as_str()) }
        }
    }};
    ($got:expr, $expected:pat) => {
        assert_m!($got, $expected, true)
    }
}

macro_rules! layer_watch {
    {$layer:ident : $( $body:stmt );* } => {
        $layer.with(|l| *l.borrow_mut() += 1);
        let res = {
            $( $body );*
        };
        $layer.with(|l| *l.borrow_mut() -= 1);
        res
    }
}

// "Layer debug"
macro_rules! ld {
    ($layer:ident, $template:tt, $($arg:expr),*) => {{
        let layers = $layer.with(|l| *l.borrow()) - 1;
        for i in 1..layers {
            if i % 2 == 0 {
                print!("║ ")
            } else {
                print!("│ ");
            }
        }
        if layers > 0 {
            if layers % 2 == 0 {
                print!("╠═")
            } else {
                print!("├─");
            }
        }
        println!($template, $($arg),*);
    }}
}

// "Layer debug, continued"
macro_rules! lc {
    ($layer:ident, $template:tt, $($arg:expr),*) => {{
        let layers = $layer.with(|l| *l.borrow()) - 1;
        for i in 1..(layers+1) {
            if i % 2 == 0 {
                print!("║ ")
            } else {
                print!("│ ");
            }
        }
        println!($template, $($arg),*);
    }}
}
