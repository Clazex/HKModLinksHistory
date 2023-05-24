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

const NEW_TEMPLATE_NAME: &str = "new";
const NEW_TEMPLATE: &str = include_str!("../assets/new-template.txt");

const UPDATED_TEMPLATE_NAME: &str = "updated";
const UPDATED_TEMPLATE: &str = include_str!("../assets/updated-template.txt");

const CHANGED_TEMPLATE_NAME: &str = "changed";
const CHANGED_TEMPLATE: &str = include_str!("../assets/changed-template.txt");

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
    handlebars.register_template_string(UPDATED_TEMPLATE_NAME, UPDATED_TEMPLATE)?;
    handlebars.register_template_string(NEW_TEMPLATE_NAME, NEW_TEMPLATE)?;
    handlebars.register_template_string(CHANGED_TEMPLATE_NAME, CHANGED_TEMPLATE)?;
    let handlebars = handlebars;

    let mut new_mods = Vec::with_capacity(MONTH_DAYS);
    let mut updated_mods = Vec::with_capacity(MONTH_DAYS);
    let mut changed_mods = Vec::with_capacity(MONTH_DAYS);
    for (i, (new, old)) in modlinks.iter().tuple_windows().enumerate() {
        let changelog = new.changelog_since(old);

        fs::write(
            format!("{BASE_DIR}/Changelog-{}.md", i + 1),
            changelog.to_markdown()?,
        )?;

        let new = format_mod_list(handlebars.render(NEW_TEMPLATE_NAME, changelog.json())?);
        fs::write(format!("{BASE_DIR}/NewMods-{}.txt", i + 1), new.as_str())?;
        new_mods.push(new);

        let updated = format_mod_list(handlebars.render(UPDATED_TEMPLATE_NAME, changelog.json())?);
        fs::write(
            format!("{BASE_DIR}/UpdatedMods-{}.txt", i + 1),
            updated.as_str(),
        )?;
        updated_mods.push(updated);

        let changed = format_mod_list(handlebars.render(CHANGED_TEMPLATE_NAME, changelog.json())?);
        fs::write(
            format!("{BASE_DIR}/ChangedMods-{}.txt", i + 1),
            changed.as_str(),
        )?;
        changed_mods.push(changed);
    }
    let new_mods = new_mods;
    let updated_mods = updated_mods;

    fs::write(
        format!("{BASE_DIR}/Changelog-Week.md"),
        modlinks[0]
            .changelog_since(&modlinks[WEEK_DAYS - 1])
            .to_markdown()?,
    )?;
    fs::write(
        format!("{BASE_DIR}/NewMods-Week.txt"),
        merge_mod_list(new_mods.iter().take(WEEK_DAYS - 1)),
    )?;
    fs::write(
        format!("{BASE_DIR}/UpdatedMods-Week.txt"),
        merge_mod_list(updated_mods.iter().take(WEEK_DAYS - 1)),
    )?;
    fs::write(
        format!("{BASE_DIR}/ChangedMods-Week.txt"),
        merge_mod_list(changed_mods.iter().take(WEEK_DAYS - 1)),
    )?;

    fs::write(
        format!("{BASE_DIR}/Changelog-Month.md"),
        modlinks[0]
            .changelog_since(&modlinks[MONTH_DAYS - 1])
            .to_markdown()?,
    )?;
    fs::write(
        format!("{BASE_DIR}/NewMods-Month.txt"),
        merge_mod_list(new_mods.iter()),
    )?;
    fs::write(
        format!("{BASE_DIR}/UpdatedMods-Month.txt"),
        merge_mod_list(updated_mods.iter()),
    )?;
    fs::write(
        format!("{BASE_DIR}/ChangedMods-Month.txt"),
        merge_mod_list(changed_mods.iter()),
    )?;

    fs::write(format!("{BASE_DIR}/modlinks.json"), modlinks[0].to_json()?)?;

    Ok(())
}

fn format_mod_list(mods: String) -> String {
    mods.lines()
        .unique()
        .sorted()
        .skip_while(|s| s.is_empty())
        .map(|s| format!("{s}\n"))
        .join("")
}

fn merge_mod_list<'a>(mods: impl Iterator<Item = &'a String> + DoubleEndedIterator) -> String {
    mods.into_iter()
        .rev()
        .flat_map(|s| s.lines())
        .filter(|s| !s.is_empty())
        .map(|s| format!("{s}\n"))
        .join("")
}
