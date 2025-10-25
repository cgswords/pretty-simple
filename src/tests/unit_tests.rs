// Copyright 2025 Cameron Swords
// SPDX-License-Identifier: Apache-2.0

use insta::assert_snapshot;

use crate::*;

#[test]
fn column() {
    let doc = Doc::text("prefix").concat_space(Doc::column(|l| {
        Doc::text("| <- column").concat_space(Doc::text(format!("{l}")))
    }));
    let doc = Doc::vsep(
        vec![0, 4, 8]
            .into_iter()
            .map(|n| Doc::indent(doc.clone(), n)),
    );
    assert_snapshot!(doc.render(20))
}

#[test]
fn nesting() {
    let doc = Doc::text("prefix").concat_space(Doc::nesting(|l| {
        Doc::brackets(Doc::text("Nested:").concat_space(Doc::text(format!("{l}"))))
    }));
    let doc = Doc::vsep(
        vec![0, 4, 8]
            .into_iter()
            .map(|n| Doc::indent(doc.clone(), n)),
    );
    assert_snapshot!(doc.render(20))
}

#[test]
fn stack_stress() {
    // Build a "group" like:
    // flat:  [item0, item1, item2, ...]
    // broke: [item0
    //          , item1
    //          , item2
    //          , ...]
    //
    // Lengths grow so some groups barely fit while others don't.
    fn group_with_k_items(k: usize, base: usize, indent: i16) -> Doc {
        let items: Vec<Doc> = (0..k)
            .map(|i| {
                // Vary the payload length to create tight fit/no-fit edges.
                let len = base + (i % 7) + (k % 5);
                let label = format!("item{}_{}", k, i);
                let payload = "x".repeat(len);
                Doc::hcat([Doc::text(&label), Doc::text(":"), Doc::text(&payload)])
            })
            .collect();

        let comma_space = Doc::text(", ");
        let flat_items = Doc::intersperse(items.iter().cloned(), comma_space.clone());
        let flat = flat_items.brackets();

        let broke_items = {
            let mut v: Vec<Doc> = Vec::new();
            for (i, it) in items.into_iter().enumerate() {
                if i > 0 {
                    // start each subsequent item on a new line, leading comma
                    v.push(Doc::line().concat(Doc::comma()));
                    v.push(Doc::text(" "));
                }
                v.push(it);
            }
            Doc::hcat(v).nest(indent)
        };
        let broke = broke_items.concat(Doc::line()).brackets();

        Doc::alt(flat.flatten(), broke)
    }

    fn build_alt_stress(num_groups: usize) -> Doc {
        let mut blocks: Vec<Doc> = Vec::new();
        for g in 0..num_groups {
            // Increasing item count + base length create rising pressure.
            let k = 3 + (g % 9); // between 3 and 11 items
            let base = 4 + (g % 13); // item payload base length
            let indent = 2 + (g % 6) as i16;

            let grp = group_with_k_items(k, base, indent);

            // Separator that itself can be flat or broken to make probes work harder.
            // flat:  " • "
            // broke: line + same token
            let sep_flat = Doc::text(" • ");
            let sep_broke = Doc::line().concat(Doc::text("- "));
            let sep = Doc::alt(sep_flat, sep_broke);

            if g > 0 {
                blocks.push(sep);
            }
            // Add some deeper nests that still flatten nicely in the flat alternative.
            let deep = {
                let inner = Doc::line().concat(Doc::text("inner")).braces();
                // group the inner too
                Doc::alt(inner.clone().flatten(), inner.nest(2 + (g % 4) as i16))
            };
            blocks.push(Doc::hcat(vec![grp, Doc::space(), deep]));
        }
        Doc::hcat(blocks)
    }

    let doc = build_alt_stress(1000);
    let widths = [20_i16, 32, 48, 64, 96, 140];
    for &w in &widths {
        let cloned = doc.clone();
        let _render = cloned.render(w);
    }
}
#[test]
fn stack_stress_2() {
    // Build a massive, interspersed group
    let msg = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Fusce risus lacus, \
    porttitor id lectus vitae, volutpat imperdiet dolor. Orci varius natoque penatibus et magnis \
    dis parturient montes, nascetur ridiculus mus. Donec malesuada venenatis est at blandit. Duis \
    hendrerit, tortor vitae fermentum cursus, orci metus scelerisque mi, id porta metus erat in \
    ex. Nunc a lacus at ante rutrum pulvinar in non ipsum. Quisque dapibus posuere ante sed \
    consectetur. Nulla facilisi. In ac leo porttitor, mattis erat ac, pulvinar ex. In elementum \
    orci at scelerisque egestas. Cras id sapien leo. Nulla velit diam, tincidunt eget lectus sit \
    amet, varius varius nibh. Nulla facilisi. Fusce sit amet euismod sapien. In luctus congue ex \
    eget viverra. Nullam vitae felis sollicitudin, maximus nulla vel, consectetur magna. Cras orci \
    dui, dignissim eget nisi ut, iaculis lacinia orci. Sed eget quam et lacus luctus posuere quis \
    eu massa. Donec placerat velit justo, a convallis eros feugiat sed. Pellentesque nec feugiat \
    enim, id sagittis neque. Nulla non lectus sed orci ultrices sagittis ut at dolor. Maecenas \
    eget ipsum ultricies, auctor tellus ac, vehicula risus. Nulla malesuada aliquet sem quis \
    aliquam. Maecenas tincidunt sapien mi, a ultrices urna viverra ac. In placerat tellus sit.";

    let mut encoded = vec![];
    while encoded.len() < 100_000 {
        for char in msg.chars() {
            encoded.push(char as u8);
        }
    }

    let doc = Doc::text("[").concat(Doc::intersperse(
        encoded.iter().map(|elem| Doc::text(elem.to_string())),
        Doc::text(",").concat(Doc::space()),
    ));
    let _render = doc.render(100);
}
