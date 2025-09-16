// Copyright 2025 Cameron Swords
// SPDX-License-Identifier: Apache-2.0

use insta::assert_snapshot;

use crate::*;

#[test]
fn column() {
    let doc = Doc::text("prefix").concat_space(Doc::column(|l| Doc::text("| <- column").concat_space(Doc::text(format!("{l}")))));
    let doc = Doc::vsep(vec![0,4,8].into_iter().map(|n| Doc::indent(doc.clone(), n)));
    assert_snapshot!(doc.render(20))
}

#[test]
fn nesting() {
    let doc = Doc::text("prefix").concat_space(Doc::nesting(|l| Doc::brackets(Doc::text("Nested:").concat_space(Doc::text(format!("{l}"))))));
    let doc = Doc::vsep(vec![0,4,8].into_iter().map(|n| Doc::indent(doc.clone(), n)));
    assert_snapshot!(doc.render(20))
}
