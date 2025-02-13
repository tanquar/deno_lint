// Copyright 2020-2021 the Deno authors. All rights reserved. MIT license.
use super::{Context, LintRule, DUMMY_NODE};
use crate::ProgramRef;
use deno_ast::swc::visit::Node;
use deno_ast::swc::visit::Visit;
use derive_more::Display;

#[derive(Debug)]
pub struct NoNonNullAssertion;

const CODE: &str = "no-non-null-assertion";

#[derive(Display)]
enum NoNonNullAssertionMessage {
  #[display(fmt = "do not use non-null assertion")]
  Unexpected,
}

impl LintRule for NoNonNullAssertion {
  fn new() -> Box<Self> {
    Box::new(NoNonNullAssertion)
  }

  fn code(&self) -> &'static str {
    CODE
  }

  fn lint_program<'view>(
    &self,
    context: &mut Context<'view>,
    program: ProgramRef<'view>,
  ) {
    let mut visitor = NoNonNullAssertionVisitor::new(context);
    match program {
      ProgramRef::Module(m) => visitor.visit_module(m, &DUMMY_NODE),
      ProgramRef::Script(s) => visitor.visit_script(s, &DUMMY_NODE),
    }
  }

  #[cfg(feature = "docs")]
  fn docs(&self) -> &'static str {
    include_str!("../../docs/rules/no_non_null_assertion.md")
  }
}

struct NoNonNullAssertionVisitor<'c, 'view> {
  context: &'c mut Context<'view>,
}

impl<'c, 'view> NoNonNullAssertionVisitor<'c, 'view> {
  fn new(context: &'c mut Context<'view>) -> Self {
    Self { context }
  }
}

impl<'c, 'view> Visit for NoNonNullAssertionVisitor<'c, 'view> {
  fn visit_ts_non_null_expr(
    &mut self,
    non_null_expr: &deno_ast::swc::ast::TsNonNullExpr,
    _parent: &dyn Node,
  ) {
    self.context.add_diagnostic(
      non_null_expr.span,
      CODE,
      NoNonNullAssertionMessage::Unexpected,
    );
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn no_non_null_assertion_valid() {
    assert_lint_ok! {
      NoNonNullAssertion,
      "instance.doWork();",
      "foo.bar?.includes('baz')",
      "x;",
      "x.y;",
      "x.y.z;",
      "x?.y.z;",
      "x?.y?.z;",
      "!x;",
    };
  }

  #[test]
  fn no_non_null_assertion_invalid() {
    assert_lint_err! {
      NoNonNullAssertion,

      r#"instance!.doWork()"#: [
      {
        col: 0,
        message: NoNonNullAssertionMessage::Unexpected,
      }],
      r#"foo.bar!.includes('baz');"#: [
      {
        col: 0,
        message: NoNonNullAssertionMessage::Unexpected,
      }],
      r#"x.y.z!?.();"#: [
      {
        col: 0,
        message: NoNonNullAssertionMessage::Unexpected,
      }],
      r#"x!?.y.z;"#: [
      {
        col: 0,
        message: NoNonNullAssertionMessage::Unexpected,
      }],
      r#"x!?.[y].z;"#: [
      {
        col: 0,
        message: NoNonNullAssertionMessage::Unexpected,
      }],
      r#"x.y.z!!();"#: [
      {
        col: 0,
        message: NoNonNullAssertionMessage::Unexpected,
      }],
      r#"x.y!!;"#: [
      {
        col: 0,
        message: NoNonNullAssertionMessage::Unexpected,
      }],
      r#"x!!.y;"#: [
      {
        col: 0,
        message: NoNonNullAssertionMessage::Unexpected,
      }],
      r#"x!!!;"#: [
      {
        col: 0,
        message: NoNonNullAssertionMessage::Unexpected,
      }],
      r#"x.y?.z!();"#: [
      {
        col: 0,
        message: NoNonNullAssertionMessage::Unexpected,
      }],
      r#"x.y.z!();"#: [
      {
        col: 0,
        message: NoNonNullAssertionMessage::Unexpected,
      }],
      r#"x![y]?.z;"#: [
      {
        col: 0,
        message: NoNonNullAssertionMessage::Unexpected,
      }],
      r#"x![y];"#: [
      {
        col: 0,
        message: NoNonNullAssertionMessage::Unexpected,
      }],
      r#"!x!.y;"#: [
      {
        col: 1,
        message: NoNonNullAssertionMessage::Unexpected,
      }],
      r#"x!.y?.z;"#: [
      {
        col: 0,
        message: NoNonNullAssertionMessage::Unexpected,
      }],
      r#"x.y!;"#: [
      {
        col: 0,
        message: NoNonNullAssertionMessage::Unexpected,
      }],
      r#"x!.y;"#: [
      {
        col: 0,
        message: NoNonNullAssertionMessage::Unexpected,
      }],
      r#"x!;"#: [
      {
        col: 0,
        message: NoNonNullAssertionMessage::Unexpected,
      }],

    }
  }
}
