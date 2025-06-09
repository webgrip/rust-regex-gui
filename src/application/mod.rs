use std::io;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::domain::Rule;
use crate::telemetry::Logger;
use regex::Regex;
use walkdir::WalkDir;

#[derive(Clone)]
pub struct FileEntry {
    pub path: PathBuf,
    pub is_dir: bool,
}

pub trait FileSystem: Send + Sync {
    fn find_matches(&self, pattern: &Regex) -> io::Result<Vec<FileEntry>>;
    fn move_file(&self, from: &Path, to: &Path) -> io::Result<()>;
}

pub struct StdFileSystem;

impl FileSystem for StdFileSystem {
    fn find_matches(&self, pattern: &Regex) -> io::Result<Vec<FileEntry>> {
        let mut matches = Vec::new();
        for entry in WalkDir::new(".") {
            let entry = entry?;
            let abs = entry.path().canonicalize()?;
            let path_str = abs.to_string_lossy();
            if pattern.is_match(&path_str) {
                matches.push(FileEntry {
                    path: entry.path().to_path_buf(),
                    is_dir: entry.file_type().is_dir(),
                });
            }
        }
        Ok(matches)
    }

    fn move_file(&self, from: &Path, to: &Path) -> io::Result<()> {
        if let Some(parent) = to.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::rename(from, to)
    }
}

pub struct Renamer {
    logger: Arc<dyn Logger>,
    fs: Arc<dyn FileSystem>,
}

impl Renamer {
    pub fn new(logger: Arc<dyn Logger>, fs: Arc<dyn FileSystem>) -> Self {
        Self { logger, fs }
    }

    pub fn count_matches(&self, rule: &mut Rule) -> io::Result<usize> {
        let re =
            Regex::new(&rule.from).map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
        let matches = self.fs.find_matches(&re)?;
        let file_count = matches.iter().filter(|m| !m.is_dir).count();
        let dir_count = matches.iter().filter(|m| m.is_dir).count();
        rule.file_match_count = Some(file_count);
        rule.dir_match_count = Some(dir_count);
        self.logger.log(&format!(
            "Found {} files and {} directories for '{}'",
            file_count, dir_count, rule.from
        ));
        Ok(file_count + dir_count)
    }

    pub fn count_all_matches(&self, rules: &mut [Rule]) -> io::Result<()> {
        for rule in rules {
            self.count_matches(rule)?;
        }
        Ok(())
    }

    pub fn execute(&self, rules: &[Rule]) -> io::Result<()> {
        for rule in rules {
            self.logger
                .log(&format!("Mapping '{}' -> '{}'", rule.from, rule.to));
            let re = Regex::new(&rule.from)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
            for entry in self.fs.find_matches(&re)? {
                let path_str = entry.path.to_string_lossy();
                let dest_str = re.replace(&path_str, &rule.to).to_string();
                self.fs.move_file(&entry.path, Path::new(&dest_str))?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::sync::{Arc, Mutex};

    struct MockFs {
        entries: Vec<FileEntry>,
        moved: Arc<Mutex<Vec<(PathBuf, PathBuf)>>>,
    }

    impl FileSystem for MockFs {
        fn find_matches(&self, pattern: &Regex) -> io::Result<Vec<FileEntry>> {
            Ok(self
                .entries
                .iter()
                .cloned()
                .filter(|e| pattern.is_match(&e.path.to_string_lossy()))
                .collect())
        }

        fn move_file(&self, from: &Path, to: &Path) -> io::Result<()> {
            self.moved
                .lock()
                .unwrap()
                .push((from.to_path_buf(), to.to_path_buf()));
            Ok(())
        }
    }

    struct TestLogger {
        messages: Arc<Mutex<Vec<String>>>,
    }

    impl Logger for TestLogger {
        fn log(&self, message: &str) {
            self.messages.lock().unwrap().push(message.to_string());
        }
    }

    #[test]
    fn execute_logs_each_mapping() {
        let messages = Arc::new(Mutex::new(Vec::new()));
        let logger = Arc::new(TestLogger {
            messages: Arc::clone(&messages),
        });
        let fs = Arc::new(MockFs {
            entries: vec![],
            moved: Arc::new(Mutex::new(Vec::new())),
        });
        let renamer = Renamer::new(logger, fs);

        let rules = vec![
            Rule {
                from: "src".into(),
                to: "dst".into(),
                file_match_count: None,
                dir_match_count: None,
            },
            Rule {
                from: "foo".into(),
                to: "bar".into(),
                file_match_count: None,
                dir_match_count: None,
            },
        ];

        renamer.execute(&rules).unwrap();

        let collected = messages.lock().unwrap().clone();
        assert_eq!(
            collected,
            vec![
                "Mapping 'src' -> 'dst'".to_string(),
                "Mapping 'foo' -> 'bar'".to_string(),
            ]
        );
    }

    #[test]
    fn count_matches_sets_rule_match_count() {
        let messages = Arc::new(Mutex::new(Vec::new()));
        let logger = Arc::new(TestLogger {
            messages: Arc::clone(&messages),
        });
        let fs = Arc::new(MockFs {
            entries: vec![
                FileEntry {
                    path: PathBuf::from("a.txt"),
                    is_dir: false,
                },
                FileEntry {
                    path: PathBuf::from("b.rs"),
                    is_dir: false,
                },
            ],
            moved: Arc::new(Mutex::new(Vec::new())),
        });
        let renamer = Renamer::new(logger, fs);

        let mut rule = Rule {
            from: ".*\\.txt$".into(),
            to: "".into(),
            file_match_count: None,
            dir_match_count: None,
        };

        renamer.count_matches(&mut rule).unwrap();
        assert_eq!(rule.file_match_count, Some(1));
        assert_eq!(rule.dir_match_count, Some(0));
    }

    #[test]
    fn count_all_updates_all_rules() {
        let logger = Arc::new(TestLogger {
            messages: Arc::new(Mutex::new(Vec::new())),
        });
        let fs = Arc::new(MockFs {
            entries: vec![
                FileEntry {
                    path: PathBuf::from("a.txt"),
                    is_dir: false,
                },
                FileEntry {
                    path: PathBuf::from("b.txt"),
                    is_dir: false,
                },
            ],
            moved: Arc::new(Mutex::new(Vec::new())),
        });
        let renamer = Renamer::new(logger, fs);

        let mut rules = vec![
            Rule {
                from: ".*a\\.txt".into(),
                to: "".into(),
                file_match_count: None,
                dir_match_count: None,
            },
            Rule {
                from: ".*b\\.txt".into(),
                to: "".into(),
                file_match_count: None,
                dir_match_count: None,
            },
        ];

        renamer.count_all_matches(&mut rules).unwrap();

        assert_eq!(rules[0].file_match_count, Some(1));
        assert_eq!(rules[1].file_match_count, Some(1));
    }

    #[test]
    fn execute_moves_matching_files() {
        let messages = Arc::new(Mutex::new(Vec::new()));
        let logger = Arc::new(TestLogger {
            messages: Arc::clone(&messages),
        });
        let moved = Arc::new(Mutex::new(Vec::new()));
        let fs = Arc::new(MockFs {
            entries: vec![
                FileEntry {
                    path: PathBuf::from("foo/a.txt"),
                    is_dir: false,
                },
                FileEntry {
                    path: PathBuf::from("foo/b.txt"),
                    is_dir: false,
                },
            ],
            moved: Arc::clone(&moved),
        });
        let renamer = Renamer::new(logger, fs);

        let rules = vec![Rule {
            from: "foo/(.*)\\.txt".into(),
            to: "bar/$1.md".into(),
            file_match_count: None,
            dir_match_count: None,
        }];

        renamer.execute(&rules).unwrap();

        let moved_files = moved.lock().unwrap().clone();
        assert_eq!(
            moved_files,
            vec![
                (PathBuf::from("foo/a.txt"), PathBuf::from("bar/a.md")),
                (PathBuf::from("foo/b.txt"), PathBuf::from("bar/b.md")),
            ]
        );
    }
}
