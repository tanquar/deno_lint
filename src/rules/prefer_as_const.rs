// Copyright 2020-2021 the Deno authors. All rights reserved. MIT license.
use super::{Context, LintRule, DUMMY_NODE};
use crate::ProgramRef;
use deno_ast::swc::ast::{
  ArrayPat, BindingIdent, Expr, Lit, ObjectPat, Pat, TsAsExpr, TsLit, TsType,
  TsTypeAnn, TsTypeAssertion, VarDecl,
};
use deno_ast::swc::common::{Span, Spanned};
use deno_ast::swc::visit::Node;
use deno_ast::swc::visit::{VisitAll, VisitAllWith};
use derive_more::Display;

const CODE: &str = "prefer-as-const";

#[derive(Display)]
enum PreferAsConstMessage {
  #[display(
    fmt = "Expected a `const` assertion instead of a literal type annotation"
  )]
  ExpectedConstAssertion,
}

#[derive(Display)]
enum PreferAsConstHint {
  #[display(fmt = "Remove a literal type annotation and add `as const`")]
  AddAsConst,
}

#[derive(Debug)]
pub struct PreferAsConst;

impl LintRule for PreferAsConst {
  fn new() -> Box<Self> {
    Box::new(PreferAsConst)
  }

  fn tags(&self) -> &'static [&'static str] {
    &["recommended"]
  }

  fn code(&self) -> &'static str {
    CODE
  }

  fn lint_program<'view>(
    &self,
    context: &mut Context<'view>,
    program: ProgramRef<'view>,
  ) {
    let mut visitor = PreferAsConstVisitor::new(context);

    match program {
      ProgramRef::Module(m) => m.visit_all_with(&DUMMY_NODE, &mut visitor),
      ProgramRef::Script(s) => s.visit_all_with(&DUMMY_NODE, &mut visitor),
    }
  }

  #[cfg(feature = "docs")]
  fn docs(&self) -> &'static str {
    include_str!("../../docs/rules/prefer_as_const.md")
  }
}

struct PreferAsConstVisitor<'c, 'view> {
  context: &'c mut Context<'view>,
}

impl<'c, 'view> PreferAsConstVisitor<'c, 'view> {
  fn new(context: &'c mut Context<'view>) -> Self {
    Self { context }
  }

  fn add_diagnostic_helper(&mut self, span: Span) {
    self.context.add_diagnostic_with_hint(
      span,
      CODE,
      PreferAsConstMessage::ExpectedConstAssertion,
      PreferAsConstHint::AddAsConst,
    );
  }

  fn compare(&mut self, type_ann: &TsType, expr: &Expr, span: Span) {
    if let TsType::TsLitType(lit_type) = &*type_ann {
      if let Expr::Lit(expr_lit) = &*expr {
        match (expr_lit, &lit_type.lit) {
          (Lit::Str(value_literal), TsLit::Str(type_literal)) => {
            if value_literal.value == type_literal.value {
              self.add_diagnostic_helper(span)
            }
          }
          (Lit::Num(value_literal), TsLit::Number(type_literal)) => {
            if (value_literal.value - type_literal.value).abs() < f64::EPSILON {
              self.add_diagnostic_helper(span)
            }
          }
          _ => return,
        }
      }
    }
  }
}

impl<'c, 'view> VisitAll for PreferAsConstVisitor<'c, 'view> {
  fn visit_ts_as_expr(&mut self, as_expr: &TsAsExpr, _: &dyn Node) {
    self.compare(&as_expr.type_ann, &as_expr.expr, as_expr.type_ann.span());
  }

  fn visit_ts_type_assertion(
    &mut self,
    type_assertion: &TsTypeAssertion,
    _: &dyn Node,
  ) {
    self.compare(
      &type_assertion.type_ann,
      &type_assertion.expr,
      type_assertion.type_ann.span(),
    );
  }

  fn visit_var_decl(&mut self, var_decl: &VarDecl, _: &dyn Node) {
    for decl in &var_decl.decls {
      if let Some(init) = &decl.init {
        if let Pat::Array(ArrayPat { type_ann, .. })
        | Pat::Object(ObjectPat { type_ann, .. })
        | Pat::Ident(BindingIdent { type_ann, .. }) = &decl.name
        {
          if let Some(TsTypeAnn { type_ann, .. }) = &type_ann {
            self.compare(type_ann, init, type_ann.span());
          }
        }
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn prefer_as_const_valid() {
    assert_lint_ok! {
      PreferAsConst,
      "let foo = 'baz' as const;",
      "let foo = 1 as const;",
      "let foo = { bar: 'baz' as const };",
      "let foo = { bar: 1 as const };",
      "let foo = { bar: 'baz' };",
      "let foo = { bar: 2 };",
      "let foo = <bar>'bar';",
      "let foo = <string>'bar';",
      "let foo = 'bar' as string;",
      "let foo = `bar` as `bar`;",
      "let foo = `bar` as `foo`;",
      "let foo = `bar` as 'bar';",
      "let foo: string = 'bar';",
      "let foo: number = 1;",
      "let foo: 'bar' = baz;",
      "let foo = 'bar';",
      "class foo { bar: 'baz' = 'baz'; }",
      "class foo { bar = 'baz'; }",
      "let foo: 'bar';",
      "let foo = { bar };",
      "let foo: 'baz' = 'baz' as const;",

      // https://github.com/denoland/deno_lint/issues/567
      "const",
      "let",
      "var",
    };
  }

  #[test]
  fn prefer_as_const_invalid() {
    assert_lint_err! {
      PreferAsConst,
      "let foo = { bar: 'baz' as 'baz' };": [
        {
          col: 26,
          message: PreferAsConstMessage::ExpectedConstAssertion,
          hint: PreferAsConstHint::AddAsConst,
        }
      ],
      "let foo = { bar: 1 as 1 };": [
        {
          col: 22,
          message: PreferAsConstMessage::ExpectedConstAssertion,
          hint: PreferAsConstHint::AddAsConst,
        }
      ],
      "let [x]: 'bar' = 'bar';": [
        {
          col: 9,
          message: PreferAsConstMessage::ExpectedConstAssertion,
          hint: PreferAsConstHint::AddAsConst,
        }
      ],
      "let {x}: 'bar' = 'bar';": [
        {
          col: 9,
          message: PreferAsConstMessage::ExpectedConstAssertion,
          hint: PreferAsConstHint::AddAsConst,
        }
      ],
      "let foo: 'bar' = 'bar';": [
        {
          col: 9,
          message: PreferAsConstMessage::ExpectedConstAssertion,
          hint: PreferAsConstHint::AddAsConst,
        }
      ],
      "let foo: 2 = 2;": [
        {
          col: 9,
          message: PreferAsConstMessage::ExpectedConstAssertion,
          hint: PreferAsConstHint::AddAsConst,
        }
      ],
      "let foo: 'bar' = 'bar' as 'bar';": [
        {
          col: 26,
          message: PreferAsConstMessage::ExpectedConstAssertion,
          hint: PreferAsConstHint::AddAsConst,
        }
      ],
      "let foo = <'bar'>'bar';": [
        {
          col: 11,
          message: PreferAsConstMessage::ExpectedConstAssertion,
          hint: PreferAsConstHint::AddAsConst,
        }
      ],
      "let foo = <4>4;": [
        {
          col: 11,
          message: PreferAsConstMessage::ExpectedConstAssertion,
          hint: PreferAsConstHint::AddAsConst,
        }
      ],
      "let foo = 'bar' as 'bar';": [
        {
          col: 19,
          message: PreferAsConstMessage::ExpectedConstAssertion,
          hint: PreferAsConstHint::AddAsConst,
        }
      ],
      "let foo = 5 as 5;": [
        {
          col: 15,
          message: PreferAsConstMessage::ExpectedConstAssertion,
          hint: PreferAsConstHint::AddAsConst,
        }
      ],
      "let foo: 1.23456 = 1.23456;": [
        {
          col: 9,
          message: PreferAsConstMessage::ExpectedConstAssertion,
          hint: PreferAsConstHint::AddAsConst,
        }
      ],
      "let foo: 2 = 2, bar: 3 = 3;": [
        {
          col: 9,
          message: PreferAsConstMessage::ExpectedConstAssertion,
          hint: PreferAsConstHint::AddAsConst,
        },
        {
          col: 21,
          message: PreferAsConstMessage::ExpectedConstAssertion,
          hint: PreferAsConstHint::AddAsConst,
        }
      ],

      // nested
      "let foo = () => { let x: 'x' = 'x'; };": [
        {
          col: 25,
          message: PreferAsConstMessage::ExpectedConstAssertion,
          hint: PreferAsConstHint::AddAsConst,
        }
      ],
    };
  }
}
