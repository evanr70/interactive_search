//! Customizes the rendering of the elements.
use std::{fmt, io};

use dialoguer::console::Term;

/// Implements a theme for dialoguer.
pub struct Theme;

impl Theme {
    /// Formats an input prompt after selection.
    #[inline]
    fn format_input_prompt_selection(
        &self,
        f: &mut dyn fmt::Write,
        prompt: &str,
        sel: &str,
    ) -> fmt::Result {
        write!(f, "{}: {}", prompt, sel)
    }

    /// Formats a fuzzy select prompt item.
    fn format_fuzzy_select_prompt_item(
        &self,
        f: &mut dyn fmt::Write,
        text: &str,
        active: bool,
    ) -> fmt::Result {
        write!(f, "{} ", if active { ">" } else { " " })?;
        write!(f, "{}", text)
    }

    /// Formats a fuzzy select prompt.
    fn format_fuzzy_select_prompt(
        &self,
        f: &mut dyn fmt::Write,
        prompt: &str,
        search_term: &str,
        cursor_pos: usize,
    ) -> fmt::Result {
        if !prompt.is_empty() {
            write!(f, "{}: ", prompt,)?;
        }

        if cursor_pos < search_term.len() {
            let st_head = search_term[0..cursor_pos].to_string();
            let st_tail = search_term[cursor_pos..search_term.len()].to_string();
            let st_cursor = "|".to_string();
            write!(f, "{}{}{}", st_head, st_cursor, st_tail)
        } else {
            let cursor = "|".to_string();
            write!(f, "{}{}", search_term, cursor)
        }
    }
}

/// Helper struct to conveniently render a theme of a term.
pub(crate) struct TermThemeRenderer<'a> {
    term: &'a Term,
    theme: Theme,
    height: usize,
    prompt_height: usize,
    prompts_reset_height: bool,
}

impl<'a> TermThemeRenderer<'a> {
    pub fn new(term: &'a Term, theme: Theme) -> TermThemeRenderer<'a> {
        TermThemeRenderer {
            term,
            theme,
            height: 0,
            prompt_height: 0,
            prompts_reset_height: true,
        }
    }

    fn write_formatted_line<
        F: FnOnce(&mut TermThemeRenderer, &mut dyn fmt::Write) -> fmt::Result,
    >(
        &mut self,
        f: F,
    ) -> io::Result<()> {
        let mut buf = String::new();
        f(self, &mut buf).map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;
        self.height += buf.chars().filter(|&x| x == '\n').count() + 1;
        self.term.write_line(&buf)
    }

    fn write_formatted_prompt<
        F: FnOnce(&mut TermThemeRenderer, &mut dyn fmt::Write) -> fmt::Result,
    >(
        &mut self,
        f: F,
    ) -> io::Result<()> {
        self.write_formatted_line(f)?;
        if self.prompts_reset_height {
            self.prompt_height = self.height;
            self.height = 0;
        }
        Ok(())
    }

    pub fn fuzzy_select_prompt_item(&mut self, text: &str, active: bool) -> io::Result<()> {
        self.write_formatted_line(|this, buf| {
            this.theme
                .format_fuzzy_select_prompt_item(buf, text, active)
        })
    }

    pub fn fuzzy_select_prompt(
        &mut self,
        prompt: &str,
        search_term: &str,
        cursor_pos: usize,
    ) -> io::Result<()> {
        self.write_formatted_prompt(|this, buf| {
            this.theme
                .format_fuzzy_select_prompt(buf, prompt, search_term, cursor_pos)
        })
    }

    pub fn input_prompt_selection(&mut self, prompt: &str, sel: &str) -> io::Result<()> {
        self.write_formatted_prompt(|this, buf| {
            this.theme.format_input_prompt_selection(buf, prompt, sel)
        })
    }

    pub fn clear(&mut self) -> io::Result<()> {
        self.term
            .clear_last_lines(self.height + self.prompt_height)?;
        self.height = 0;
        Ok(())
    }

    pub fn clear_preserve_prompt(&mut self, size_vec: &[usize]) -> io::Result<()> {
        let mut new_height = self.height;
        //Check each item size, increment on finding an overflow
        for size in size_vec {
            if *size > self.term.size().1 as usize {
                new_height += 1;
            }
        }

        self.term.clear_last_lines(new_height)?;
        self.height = 0;
        Ok(())
    }
}
