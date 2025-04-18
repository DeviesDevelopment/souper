use super::SoupParse;
use crate::soup::model::{Soup, SoupSourceParseError};
use serde::Deserialize;
use serde_json::{Map, Value};
use std::collections::{BTreeSet, HashMap};

pub struct PackageJson {}

#[derive(Deserialize)]
struct Content {
    dependencies: Option<HashMap<String, String>>,
}

impl SoupParse for PackageJson {
    fn soups(
        &self,
        content: &str,
        default_meta: &Map<String, Value>,
    ) -> Result<BTreeSet<Soup>, SoupSourceParseError> {
        let parse_result: Content = match serde_json::from_str(content) {
            Ok(content) => content,
            Err(e) => {
                return Err(SoupSourceParseError {
                    message: format!("Invalid package.json structure ({})", e),
                });
            }
        };

        let soups = match parse_result.dependencies {
            None => BTreeSet::new(),
            Some(dependencies) => dependencies
                .into_iter()
                .map(|(key, value)| Soup {
                    name: key,
                    version: value,
                    meta: default_meta.clone(),
                })
                .collect::<BTreeSet<Soup>>(),
        };
        Ok(soups)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test]
    fn single_dependency() {
        let content = r#"{
            "dependencies": {
                "some-lib": "^1.0.0"
            }
        }"#;
        let result = PackageJson {}.soups(content, &Map::new());
        assert_eq!(true, result.is_ok());
        let soups = result.unwrap();
        assert_eq!(1, soups.len());
        let expected_soup = Soup {
            name: "some-lib".to_owned(),
            version: "^1.0.0".to_owned(),
            meta: Map::new(),
        };
        assert_eq!(true, soups.contains(&expected_soup));
    }

    #[test]
    fn multiple_dependencies() {
        let content = r#"{
            "dependencies": {
                "some-lib": "^1.0.0",
                "another-lib": "6.6.6"
            }
        }"#;
        let result = PackageJson {}.soups(content, &Map::new());
        assert_eq!(true, result.is_ok());
        let soups = result.unwrap();
        assert_eq!(2, soups.len());
        let expected_soups = vec![
            Soup {
                name: "some-lib".to_owned(),
                version: "^1.0.0".to_owned(),
                meta: Map::new(),
            },
            Soup {
                name: "another-lib".to_owned(),
                version: "6.6.6".to_owned(),
                meta: Map::new(),
            },
        ]
        .into_iter()
        .collect::<BTreeSet<Soup>>();
        assert_eq!(expected_soups, soups);
    }

    #[test_case(
        r#"{
"dependencies": {}
    }"#
    )]
    #[test_case("{}")]
    fn no_dependencies(input: &str) {
        let result = PackageJson {}.soups(input, &Map::new());
        assert_eq!(true, result.is_ok());
        let soups = result.unwrap();
        assert_eq!(0, soups.len());
    }

    #[test_case(r#"{"#)]
    #[test_case("")]
    fn fail_on_bad_json(input: &str) {
        let result = PackageJson {}.soups(input, &Map::new());
        assert_eq!(true, result.is_err());
    }
}
