use super::SoupSource;
use crate::soup::model::Soup;
use quick_xml::events::Event;
use quick_xml::Reader;
use serde_json::json;
use std::{
    collections::{BTreeSet, HashMap},
    io
};

pub struct CsProj {}

impl<R> SoupSource<R> for CsProj
where
    R: io::BufRead,
{
    fn soups(reader: R) -> BTreeSet<Soup> {
        let mut reader = Reader::from_reader(reader);
        reader.trim_text(true);
        reader.expand_empty_elements(true);

        let mut soups: BTreeSet<Soup> = BTreeSet::new();
        let mut buf = Vec::new();
        loop {
            match reader.read_event(&mut buf) {
                Ok(Event::Start(ref e)) => match e.name() {
                    b"PackageReference" => {
                        let mut attributes_by_key = e.attributes()
                            .filter_map(|attribute| attribute.ok())
                            .map(|attribute| (
                                String::from_utf8(attribute.key.to_vec()).unwrap(),
                                String::from_utf8(attribute.value.to_vec()).unwrap()
                            ))
                            .collect::<HashMap<String, String>>();
                        if attributes_by_key.contains_key("Include") && attributes_by_key.contains_key("Version") {
                            soups.insert(Soup {
                                name: attributes_by_key.remove("Include").unwrap(),
                                version: attributes_by_key.remove("Version").unwrap(),
                                meta: json!({})
                            });
                        }
                    }
                    _ => {}
                },
                Ok(Event::Eof) => break,
                Err(e) => {
                    panic!("Error: {}", e);
                }
                _ => {}
            }
        }
        buf.clear();
        soups
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn single_dependency() {
        let content: &[u8] = r#"
<Project Sdk="Microsoft.NET.Sdk.Web">
    <ItemGroup>
        <PackageReference Include="Azure.Messaging.ServiceBus" Version="7.2.1" />
    </ItemGroup>
</Project>
        "#
        .as_bytes();

        let soups = CsProj::soups(content);
        assert_eq!(1, soups.len());
        let expected_soup = Soup {
            name: "Azure.Messaging.ServiceBus".to_owned(),
            version: "7.2.1".to_owned(),
            meta: json!({}),
        };
        assert_eq!(true, soups.contains(&expected_soup));
    }

    #[test]
    fn multiple_dependencies() {
        let content: &[u8] = r#"
<Project Sdk="Microsoft.NET.Sdk.Web">
    <ItemGroup>
        <PackageReference Include="Azure.Messaging.ServiceBus" Version="7.2.1" />
        <PackageReference Include="Swashbuckle.AspNetCore" Version="6.3.1" />
    </ItemGroup>
</Project>
        "#
        .as_bytes();

        let soups = CsProj::soups(content);
        assert_eq!(2, soups.len());
        let expected_soups = vec![
            Soup { name: "Azure.Messaging.ServiceBus".to_owned(), version: "7.2.1".to_owned(), meta: json!({}) },
            Soup { name: "Swashbuckle.AspNetCore".to_owned(), version: "6.3.1".to_owned(), meta: json!({}) }
        ].into_iter().collect::<BTreeSet<Soup>>();
        assert_eq!(expected_soups, soups);
    }

    #[test]
    fn no_dependencies() {
        let content = r#"
<Project Sdk="Microsoft.NET.Sdk.Web">
    <ItemGroup>
    </ItemGroup>
</Project>
        "#.as_bytes();

        let soups = CsProj::soups(content);
        assert_eq!(0, soups.len());
    }
}