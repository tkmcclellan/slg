use regex::Regex;

pub struct LineManager {
    size: usize,
    lines: Vec<String>,
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

    pub fn filter(&mut self, filter: String) -> Vec<String> {
        let re = Regex::new(&filter).unwrap();

        self.lines
            .iter()
            .cloned()
            .filter(move |x| re.is_match(x))
            .collect()
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

        let filter = String::from("New");

        assert_eq!(manager.filter(filter), vec!["New line!", "New line 2!"]);
    }
}