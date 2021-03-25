#![feature(bool_to_option)]

use itertools::Itertools;
use lazy_static::lazy_static;
use seisreinas2::{a_star, Board, Graph, Node};
use std::fmt::Write;

use cursive::{
    event::Key,
    traits::{Nameable, Resizable, Scrollable},
    views::{Dialog, ListView, Panel, TextView},
};

fn main() {
    let mut siv = cursive::default();

    fn scrollable_list(name: &str, list: ListView) -> impl cursive::View {
        Panel::new(
            list.with_name(name)
                .scrollable()
                .scroll_x(true)
                .fixed_height(3),
        )
    }

    fn non_wrapping_text() -> TextView {
        TextView::empty().no_wrap()
    }

    siv.add_layer(
        Dialog::around(
            ListView::new()
                .child("Open queue", scrollable_list("open", ListView::new()))
                .child("Closed set", scrollable_list("closed", ListView::new()))
                .child(
                    "Tentative set",
                    scrollable_list("tentative", ListView::new()),
                )
                .child(
                    "Current",
                    scrollable_list(
                        "current",
                        ListView::new().child("Node", non_wrapping_text().with_name("best")),
                    ),
                )
                .child("Board", non_wrapping_text().with_name("board")),
        )
        .title("Seis reinas"),
    );

    let mut steps = a_star(&GRAPH).peekable();

    let mut next_step = move |siv: &mut cursive::Cursive| {
        fn list_update<T>(
            nodes: impl Iterator<Item = T>,
            content_for: impl Fn(T) -> String,
        ) -> impl FnOnce(&mut ListView) {
            move |list: &mut ListView| {
                list.clear();
                nodes.enumerate().for_each(|(i, node)| {
                    list.add_child(
                        &format!("#{}", i + 1),
                        non_wrapping_text().content(&content_for(node)),
                    );
                });
            }
        }

        fn path_to((cost, path): (u32, Vec<Board>)) -> String {
            format!("${}: {}", cost, path.iter().join(" -> "))
        }

        fn format_node(node: &Node) -> String {
            format!(
                "{} = {} + {} | {}",
                node.f(),
                node.g(),
                node.h(),
                node.board()
            )
        }

        if let Some(step) = steps.next() {
            siv.call_on_name("best", |best: &mut TextView| {
                best.set_content(&format!("{}", step.current()));
            });

            let open = step.clone_open_sorted();

            siv.call_on_name("open", list_update(open.iter(), format_node));
            siv.call_on_name("closed", list_update(step.closed(), format_node));
            siv.call_on_name(
                "tentative",
                list_update(
                    open.iter().map(|node| step.traceback(node.board())),
                    path_to,
                ),
            );

            if steps.peek().is_none() {
                siv.call_on_name("current", |best_list: &mut ListView| {
                    best_list.add_child(
                        "Solution",
                        non_wrapping_text().content(&path_to(step.traceback(step.current()))),
                    );
                });
            }

            let mut board_art = String::from("╭───┬───┬───┬───┬───┬───╮\n");

            let bits = step.current().bits();
            let mut bits = bits.chunks_exact(6).peekable();

            while let Some(row) = bits.next() {
                write!(
                    &mut board_art,
                    "│ {} │\n",
                    row.iter()
                        .map(|b| b.then_some('♛').unwrap_or(' '))
                        .join(" │ ")
                )
                .unwrap();

                if bits.peek().is_some() {
                    board_art.push_str("├───┼───┼───┼───┼───┼───┤\n");
                } else {
                    board_art.push_str("╰───┴───┴───┴───┴───┴───╯");
                }
            }

            siv.call_on_name("board", |board: &mut TextView| {
                board.set_content(&board_art);
            });
        }
    };

    next_step(&mut siv);

    siv.add_global_callback(Key::Enter, next_step);
    siv.add_global_callback('q', |siv| siv.quit());

    siv.run();
}

lazy_static! {
    static ref GRAPH: Graph = Graph::valid_boards_from_empty();
}
