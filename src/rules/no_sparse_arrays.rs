// Copyright 2020-2021 the Deno authors. All rights reserved. MIT license.
use super::{Context, LintRule, DUMMY_NODE};
use crate::ProgramRef;
use deno_ast::swc::visit::noop_visit_type;
use deno_ast::swc::visit::Node;
use deno_ast::swc::visit::Visit;
use derive_more::Display;

#[derive(Debug)]
pub struct NoSparseArrays;

const CODE: &str = "no-sparse-arrays";

#[derive(Display)]
enum NoSparseArraysMessage {
  #[display(fmt = "Sparse arrays are not allowed")]
  Disallowed,
}

impl LintRule for NoSparseArrays {
  fn new() -> Box<Self> {
    Box::new(NoSparseArrays)
  }

  fn code(&self) -> &'static str {
    CODE
  }

  fn lint_program<'view>(
    &self,
    context: &mut Context<'view>,
    program: ProgramRef<'view>,
  ) {
    let mut visitor = NoSparseArraysVisitor::new(context);
    match program {
      ProgramRef::Module(m) => visitor.visit_module(m, &DUMMY_NODE),
      ProgramRef::Script(s) => visitor.visit_script(s, &DUMMY_NODE),
    }
  }

  #[cfg(feature = "docs")]
  fn docs(&self) -> &'static str {
    include_str!("../../docs/rules/no_sparse_arrays.md")
  }
}

struct NoSparseArraysVisitor<'c, 'view> {
  context: &'c mut Context<'view>,
}

impl<'c, 'view> NoSparseArraysVisitor<'c, 'view> {
  fn new(context: &'c mut Context<'view>) -> Self {
    Self { context }
  }
}

impl<'c, 'view> Visit for NoSparseArraysVisitor<'c, 'view> {
  noop_visit_type!();

  fn visit_array_lit(
    &mut self,
    array_lit: &deno_ast::swc::ast::ArrayLit,
    _parent: &dyn Node,
  ) {
    if array_lit.elems.iter().any(|e| e.is_none()) {
      self.context.add_diagnostic(
        array_lit.span,
        CODE,
        NoSparseArraysMessage::Disallowed,
      );
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn no_sparse_arrays_valid() {
    assert_lint_ok! {
      NoSparseArrays,
      "const sparseArray1 = [1,null,3];",
    };
  }

  #[test]
  fn no_sparse_arrays_invalid() {
    assert_lint_err! {
      NoSparseArrays,
      r#"const sparseArray = [1,,3];"#: [
      {
        col: 20,
        message: NoSparseArraysMessage::Disallowed,
      }],
    }
  }
}
