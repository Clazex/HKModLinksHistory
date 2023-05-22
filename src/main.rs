use std::error::Error;
use std::fs;

use handlebars::Handlebars;

use itertools::Itertools;

use hk_modlinks::ModLinks;

const MODLINKS_LINK: &str =
    "https://raw.githubusercontent.com/hk-modding/modlinks/main/ModLinks.xml";
const BASE_DIR: &str = "dist";
const WEEK_DAYS: usize = 7;
const MONTH_DAYS: usize = 30;

const TEMPLATE_NAME: &str = "template";
const TEMPLATE: &str = include_str!("../assets/template.txt");

fn main() -> Result<(), Box<dyn Error>> {
    for i in (1..MONTH_DAYS).rev() {
        fs::rename(
            format!("{BASE_DIR}/ModLinks-{i}.xml"),
            format!("{BASE_DIR}/ModLinks-{}.xml", i + 1),
        )?;
    }

    fs::write(
        format!("{BASE_DIR}/ModLinks-1.xml"),
        ureq::get(MODLINKS_LINK).call()?.into_string()?,
    )?;

    let modlinks = (1..=MONTH_DAYS)
        .map(|i| fs::read_to_string(format!("{BASE_DIR}/ModLinks-{i}.xml")))
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .map(|modlinks| ModLinks::from_xml(&modlinks))
        .collect::<Result<Vec<_>, _>>()?;

    let mut handlebars = Handlebars::new();
    handlebars.set_strict_mode(true);
    handlebars
        .register_template_string(TEMPLATE_NAME, TEMPLATE)
        .unwrap();
    let handlebars = handlebars;

    let mut changed_mods = Vec::with_capacity(MONTH_DAYS);
    for (i, (new, old)) in modlinks.iter().tuple_windows().enumerate() {
        let changelog = new.changelog_since(old);

        fs::write(
            format!("{BASE_DIR}/Changelog-{}.md", i + 1),
            changelog.to_markdown()?,
        )?;

        let changed = handlebars
            .render(TEMPLATE_NAME, changelog.json())?
            .lines()
            .unique()
            .sorted()
            .skip_while(|s| s.is_empty())
            .map(|s| format!("{s}\n"))
            .join("");

        fs::write(
            format!("{BASE_DIR}/ChangedMods-{}.txt", i + 1),
            changed.as_str(),
        )?;
        changed_mods.push(changed);
    }
    let changed_mods = changed_mods;

    fs::write(
        format!("{BASE_DIR}/Changelog-Week.md"),
        modlinks[0]
            .changelog_since(&modlinks[WEEK_DAYS - 1])
            .to_markdown()?,
    )?;
    fs::write(
        format!("{BASE_DIR}/ChangedMods-Week.txt"),
        changed_mods
            .iter()
            .take(WEEK_DAYS - 1)
            .rev()
            .flat_map(|s| s.lines())
            .filter(|s| !s.is_empty())
            .map(|s| format!("{s}\n"))
            .join(""),
    )?;

    fs::write(
        format!("{BASE_DIR}/Changelog-Month.md"),
        modlinks[0]
            .changelog_since(&modlinks[MONTH_DAYS - 1])
            .to_markdown()?,
    )?;
    fs::write(
        format!("{BASE_DIR}/ChangedMods-Month.txt"),
        changed_mods
            .iter()
            .rev()
            .flat_map(|s| s.lines())
            .filter(|s| !s.is_empty())
            .map(|s| format!("{s}\n"))
            .join(""),
    )?;

    fs::write(format!("{BASE_DIR}/modlinks.json"), modlinks[0].to_json()?)?;

    Ok(())
}
