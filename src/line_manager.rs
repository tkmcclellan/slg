use regex::Regex;

pub struct LineManager {
    size: usize,
    lines: Vec<String>,
    scroll: usize,
}

impl LineManager {
    pub fn new(size: usize) -> LineManager {
        LineManager {
            lines: Vec::new(),
            size,
            scroll: 0,
        }
    }

    pub fn add_line(&mut self, new_line: String) {
        if self.lines.len() >= self.size {
            self.lines.remove(0);
        }

        self.lines.push(new_line);

        if self.scroll > 0 {
            self.scroll_down(1)
        }
    }

    pub fn scroll_up(&mut self, scroll: usize) {
        if scroll == 0 {
            return;
        } else if self.scroll < scroll {
            self.scroll = 0;
        } else {
            self.scroll -= scroll;
        }
    }

    pub fn scroll_down(&mut self, scroll: usize) {
        self.scroll += scroll;
    }

    pub fn scroll_to_top(&mut self) {
        self.scroll = 0
    }

    pub fn scroll_to_bottom(&mut self) {
        if self.lines.len() > 0 {
            self.scroll = self.lines.len() - 1
        }
    }

    pub fn filter(&self, filter: &Regex, items: usize) -> Vec<String> {
        self.lines[..(self.lines.len() - self.scroll - 1)]
            .iter()
            .cloned()
            .rev()
            .filter(|x| filter.is_match(x))
            .take(items)
            .collect()
    }

    pub fn count(&self) -> usize {
        self.lines.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_new_line_manager() {
        let manager = LineManager::new(10);

        assert_eq!(manager.size, 10);
        assert_eq!(manager.lines.len(), 0);
    }

    #[test]
    fn adds_new_line() {
        let mut manager = LineManager::new(10);

        manager.add_line(String::from("New line!"));

        assert_eq!(manager.lines, vec![String::from("New line!")]);
    }

    #[test]
    fn pops_at_capacity() {
        let mut manager = LineManager::new(2);
        manager.add_line(String::from("New line!"));
        manager.add_line(String::from("New line 2!"));
        manager.add_line(String::from("New line 3!"));

        assert_eq!(manager.lines.len(), 2);
        assert_eq!(
            manager.lines,
            vec![String::from("New line 2!"), String::from("New line 3!")]
        )
    }

    #[test]
    fn filters_lines() {
        let mut manager = LineManager::new(5);
        manager.add_line(String::from("New line!"));
        manager.add_line(String::from("New line 2!"));
        manager.add_line(String::from("The line 3!"));

        let filter = Regex::new("New").unwrap();

        assert_eq!(manager.filter(&filter, 3), vec!["New line 2!", "New line!"]);
    }

    #[test]
    fn gets_count() {
        let mut manager = LineManager::new(5);
        manager.add_line(String::from("New line!"));
        manager.add_line(String::from("New line 2!"));
        manager.add_line(String::from("The line 3!"));

        assert_eq!(manager.count(), 3);
    }
}
