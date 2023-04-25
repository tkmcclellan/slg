use regex::Regex;

pub struct LineManager {
    size: usize,
    pub lines: Vec<String>,
}

impl LineManager {
    pub fn new(size: usize) -> LineManager {
        LineManager {
            lines: Vec::new(),
            size,
        }
    }

    pub fn add_line(&mut self, new_line: String) {
        if self.lines.len() >= self.size {
            self.lines.remove(0);
        }

        self.lines.push(new_line);
    }

    pub fn filter(&self, filter: &Regex, items: usize) -> Vec<String> {
        self.lines
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
