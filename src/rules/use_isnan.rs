// Copyright 2020-2021 the Deno authors. All rights reserved. MIT license.
use super::{Context, LintRule, DUMMY_NODE};
use crate::ProgramRef;
use deno_ast::swc::visit::noop_visit_type;
use deno_ast::swc::visit::Node;
use deno_ast::swc::visit::Visit;
use derive_more::Display;

#[derive(Debug)]
pub struct UseIsNaN;

const CODE: &str = "use-isnan";

#[derive(Display)]
enum UseIsNaNMessage {
  #[display(fmt = "Use the isNaN function to compare with NaN")]
  Comparison,

  #[display(
    fmt = "'switch(NaN)' can never match a case clause. Use Number.isNaN instead of the switch"
  )]
  SwitchUnmatched,

  #[display(
    fmt = "'case NaN' can never match. Use Number.isNaN before the switch"
  )]
  CaseUnmatched,
}

impl LintRule for UseIsNaN {
  fn new() -> Box<Self> {
    Box::new(UseIsNaN)
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
    let mut visitor = UseIsNaNVisitor::new(context);
    match program {
      ProgramRef::Module(m) => visitor.visit_module(m, &DUMMY_NODE),
      ProgramRef::Script(s) => visitor.visit_script(s, &DUMMY_NODE),
    }
  }

  #[cfg(feature = "docs")]
  fn docs(&self) -> &'static str {
    include_str!("../../docs/rules/use_isnan.md")
  }
}

struct UseIsNaNVisitor<'c, 'view> {
  context: &'c mut Context<'view>,
}

impl<'c, 'view> UseIsNaNVisitor<'c, 'view> {
  fn new(context: &'c mut Context<'view>) -> Self {
    Self { context }
  }
}

fn is_nan_identifier(ident: &deno_ast::swc::ast::Ident) -> bool {
  ident.sym == *"NaN"
}

impl<'c, 'view> Visit for UseIsNaNVisitor<'c, 'view> {
  noop_visit_type!();

  fn visit_bin_expr(
    &mut self,
    bin_expr: &deno_ast::swc::ast::BinExpr,
    _parent: &dyn Node,
  ) {
    if bin_expr.op == deno_ast::swc::ast::BinaryOp::EqEq
      || bin_expr.op == deno_ast::swc::ast::BinaryOp::NotEq
      || bin_expr.op == deno_ast::swc::ast::BinaryOp::EqEqEq
      || bin_expr.op == deno_ast::swc::ast::BinaryOp::NotEqEq
      || bin_expr.op == deno_ast::swc::ast::BinaryOp::Lt
      || bin_expr.op == deno_ast::swc::ast::BinaryOp::LtEq
      || bin_expr.op == deno_ast::swc::ast::BinaryOp::Gt
      || bin_expr.op == deno_ast::swc::ast::BinaryOp::GtEq
    {
      if let deno_ast::swc::ast::Expr::Ident(ident) = &*bin_expr.left {
        if is_nan_identifier(ident) {
          self.context.add_diagnostic(
            bin_expr.span,
            CODE,
            UseIsNaNMessage::Comparison,
          );
        }
      }
      if let deno_ast::swc::ast::Expr::Ident(ident) = &*bin_expr.right {
        if is_nan_identifier(ident) {
          self.context.add_diagnostic(
            bin_expr.span,
            CODE,
            UseIsNaNMessage::Comparison,
          );
        }
      }
    }
  }

  fn visit_switch_stmt(
    &mut self,
    switch_stmt: &deno_ast::swc::ast::SwitchStmt,
    _parent: &dyn Node,
  ) {
    if let deno_ast::swc::ast::Expr::Ident(ident) = &*switch_stmt.discriminant {
      if is_nan_identifier(ident) {
        self.context.add_diagnostic(
          switch_stmt.span,
          CODE,
          UseIsNaNMessage::SwitchUnmatched,
        );
      }
    }

    for case in &switch_stmt.cases {
      if let Some(expr) = &case.test {
        if let deno_ast::swc::ast::Expr::Ident(ident) = &**expr {
          if is_nan_identifier(ident) {
            self.context.add_diagnostic(
              case.span,
              CODE,
              UseIsNaNMessage::CaseUnmatched,
            );
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
  fn use_isnan_invalid() {
    assert_lint_err! {
      UseIsNaN,
      "42 === NaN": [
      {
        col: 0,
        message: UseIsNaNMessage::Comparison,
      }],
      r#"
switch (NaN) {
  case NaN:
    break;
  default:
    break;
}
        "#: [
      {
        line: 2,
        col: 0,
        message: UseIsNaNMessage::SwitchUnmatched,
      },
      {
        line: 3,
        col: 2,
        message: UseIsNaNMessage::CaseUnmatched,
      }],
    }
  }
}
