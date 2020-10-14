use std::collections::hash_map::RandomState;
use std::collections::{BTreeMap, HashSet};
use walkdir::WalkDir;

pub struct Framework {
    pub name: String,
    pub path: String,
    // for find the projects
    pub relative_path: String,
    // in some languages has different framework file
    // |   languages |   files    |
    // |-------------|------------|
    // | Java        | build.gradle, settings.gradle |
    pub framework_files: Vec<String>,
    // in JVM projects, has different languages, such as Java, Groovy, Kotlin...
    pub language: Vec<String>,
}

pub struct FrameworkDetector<'a> {
    pub tags: BTreeMap<&'a str, bool>,
    pub names: Vec<String>,
    pub frameworks: Vec<Framework>,
}

impl<'a> FrameworkDetector<'a> {
    pub fn new() -> Self {
        FrameworkDetector {
            tags: Default::default(),
            names: vec![],
            frameworks: vec![],
        }
    }

    pub fn run(&mut self, path: String) {
        self.light_detector(path)
    }

    fn deep_detector(&mut self, _path: String) {}

    fn light_detector(&mut self, path: String) {
        let name_set = FrameworkDetector::build_level_one_name_set(path);
        self.tags
            .insert("workspace.java.gradle", name_set.contains("build.gradle"));
        self.tags.insert(
            "workspace.java.gradle.composite",
            name_set.contains("build.gradle") && name_set.contains("settings.gradle"),
        );
        self.tags
            .insert("workspace.java.pom", name_set.contains("pom.xml"));

        self.tags.insert(
            "workspace.bower",
            name_set.contains("bower.json") || name_set.contains("bower_components"),
        );

        self.tags.insert(
            "workspace.npm",
            name_set.contains("package.json") || name_set.contains("node_modules"),
        );

        self.tags
            .insert("workspace.rust.cargo", name_set.contains("Cargo.toml"));
    }

    pub fn build_level_one_name_set(path: String) -> HashSet<String, RandomState> {
        let mut name_sets: HashSet<String> = HashSet::new();
        let walk_dir = WalkDir::new(path);
        for dir_entry in walk_dir.max_depth(1).into_iter() {
            if dir_entry.is_err() {
                continue;
            }

            let entry = dir_entry.unwrap();
            if entry.path().file_name().is_none() {
                println!("none file_name {:?}", entry.path());
                continue;
            }

            let file_name = entry.path().file_name().unwrap().clone();
            name_sets.insert(file_name.to_str().unwrap().to_string());
        }

        name_sets
    }
}

#[cfg(test)]
mod tests {
    use crate::framework_detector::FrameworkDetector;
    use std::path::PathBuf;

    #[test]
    fn should_detect_java_gradle_project() {
        let root_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .to_path_buf();

        let test_project_dir = root_dir
            .clone()
            .join("fixtures")
            .join("projects")
            .join("java")
            .join("simple");

        let mut detector = FrameworkDetector::new();
        detector.run(test_project_dir.display().to_string());

        assert!(detector.tags.get("workspace.java.gradle").unwrap());
        assert!(detector
            .tags
            .get("workspace.java.gradle.composite")
            .unwrap());
        assert_eq!(&false, detector.tags.get("workspace.npm").unwrap());
    }
}
