use std::io;
use std::ops::Rem;

use dialoguer::console::{Key, Term};
use glob::glob;

use crate::theme::{TermThemeRenderer, Theme};

mod theme;

fn main() {
    let mut items = glob_to_strings("*");

    let result = launch(
        &mut items,
        0,
        Theme,
        &Term::stderr(),
        "Type glob, get file".to_string(),
        true,
    )
    .unwrap()
    .unwrap();

    println!("Selected: {}", items[result]);
}

fn glob_to_strings(search_term: &str) -> Vec<String> {
    let paths = glob(search_term).expect("asd");
    let mut items = vec![];
    for path in paths.flatten() {
        items.push(path.to_str().unwrap().to_string())
    }
    items
}

fn launch(
    items: &mut Vec<String>,
    default: usize,
    theme: Theme,
    term: &Term,
    prompt: String,
    report: bool,
) -> io::Result<Option<usize>> {
    // Place cursor at the end of the search term
    let mut position = 0;
    let mut search_term = String::new();

    let mut render = TermThemeRenderer::new(term, theme);
    let mut sel = default;

    let mut size_vec = Vec::new();
    for items in items.iter().as_slice() {
        let size = &items.len();
        size_vec.push(*size);
    }

    // Subtract -2 because we need space to render the prompt.
    let visible_term_rows = (term.size().0 as usize).max(3) - 2;
    // Variable used to determine if we need to scroll through the list.
    let mut starting_row = 0;

    term.hide_cursor()?;

    loop {
        render.clear()?;
        render.fuzzy_select_prompt(prompt.as_str(), &search_term, position)?;

        *items = glob_to_strings(&search_term);
        let filtered_list: Vec<_> = items.iter().map(|s| s.as_str()).collect();

        for (idx, item) in filtered_list
            .iter()
            .enumerate()
            .skip(starting_row)
            .take(visible_term_rows)
        {
            render.fuzzy_select_prompt_item(item, idx == sel)?;
            term.flush()?;
        }

        match term.read_key()? {
            Key::ArrowUp | Key::BackTab if !filtered_list.is_empty() => {
                if sel == 0 {
                    starting_row = filtered_list.len().max(visible_term_rows) - visible_term_rows;
                } else if sel == starting_row {
                    starting_row -= 1;
                }
                if sel == !0 {
                    sel = filtered_list.len() - 1;
                } else {
                    sel = ((sel as i64 - 1 + filtered_list.len() as i64)
                        % (filtered_list.len() as i64)) as usize;
                }
                term.flush()?;
            }
            Key::ArrowDown | Key::Tab if !filtered_list.is_empty() => {
                if sel == !0 {
                    sel = 0;
                } else {
                    sel = (sel as u64 + 1).rem(filtered_list.len() as u64) as usize;
                }
                if sel == visible_term_rows + starting_row {
                    starting_row += 1;
                } else if sel == 0 {
                    starting_row = 0;
                }
                term.flush()?;
            }
            Key::ArrowLeft if position > 0 => {
                position -= 1;
                term.flush()?;
            }
            Key::ArrowRight if position < search_term.len() => {
                position += 1;
                term.flush()?;
            }
            Key::Enter if !filtered_list.is_empty() => {
                render.clear()?;

                if report {
                    render.input_prompt_selection(prompt.as_str(), filtered_list[sel])?;
                }

                let sel_string = filtered_list[sel];
                let sel_string_pos_in_items = items.iter().position(|item| item.eq(sel_string));

                term.show_cursor()?;
                return Ok(sel_string_pos_in_items);
            }
            Key::Backspace if position > 0 => {
                position -= 1;
                search_term.remove(position);
                term.flush()?;
            }
            Key::Char(chr) if !chr.is_ascii_control() => {
                search_term.insert(position, chr);
                position += 1;
                term.flush()?;
                sel = 0;
                starting_row = 0;
            }

            _ => {}
        }

        render.clear_preserve_prompt(&size_vec)?;
    }
}
