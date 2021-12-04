use std::fmt::Write;
use std::{collections::HashMap, fs};

use itertools::Itertools;
use mkt_data::*;
use mkt_update::*;

use regex::Regex;
use unidecode::unidecode;

fn main() {
    println!("MKT Data Update");

    let mut data = test_update_data();

    // don't overwrite with bad data
    if !data.courses.is_empty()
        && !data.drivers.is_empty()
        && !data.karts.is_empty()
        && !data.gliders.is_empty()
    {
        let hash =
            MktItemHashes::load("data/mkt_hash.json").unwrap_or_else(|_| MktItemHashes::new());
        dbg!(&hash);
        data.merge_hashes(&hash);

        data.save("data/mkt_data.json").unwrap();
    }

    let bg_data = test_b_and_g_coverage(&data);

    test_wiki_coverage(&bg_data);

    println!("Done");
}

fn test_update_data() -> MktData {
    let mut data = MktData::new();
    update_mkt_item_data(&mut data, ItemType::Driver);
    update_mkt_item_data(&mut data, ItemType::Kart);
    update_mkt_item_data(&mut data, ItemType::Glider);
    update_mkt_item_coverage_data(&mut data);
    data
}

fn test_b_and_g_coverage(data: &MktData) -> MktData {
    let mut data = data.clone();

    data.courses
        .values_mut()
        .for_each(|c| c.favorite_items = Default::default());
    data.drivers
        .values_mut()
        .for_each(|c| c.favorite_courses = Default::default());
    data.karts
        .values_mut()
        .for_each(|c| c.favorite_courses = Default::default());
    data.gliders
        .values_mut()
        .for_each(|c| c.favorite_courses = Default::default());

    let mut rdr = csv::Reader::from_path("tmp/coverage.csv").unwrap();

    for result in rdr.records() {
        let record = result.unwrap();
        let item_name = &record[0];
        let course_name_lvl1 = &record[1];
        let course_name_lvl3 = &record[2];
        let course_name_lvl6 = &record[3];
        let mut course_name = "";
        let mut lvl = 0;
        if !course_name_lvl1.is_empty() {
            course_name = course_name_lvl1;
            lvl = 1;
        } else if !course_name_lvl3.is_empty() {
            course_name = course_name_lvl3;
            lvl = 3;
        } else if !course_name_lvl6.is_empty() {
            course_name = course_name_lvl6;
            lvl = 6;
        }

        if item_name.is_empty() || course_name.is_empty() {
            continue;
        }

        let item_name = item_name.to_lowercase();
        let course_name = course_name.replace("RT", "R/T").to_lowercase();

        let item = data
            .drivers
            .values_mut()
            .chain(data.karts.values_mut())
            .chain(data.gliders.values_mut())
            .find(|i| unidecode(&i.name).to_lowercase() == item_name);
        let course = data.courses.values_mut().find(|c| {
            if c.name.starts_with("RMX") {
                unidecode(&c.name).to_lowercase() == course_name
            } else {
                unidecode(&c.name).to_lowercase().ends_with(&course_name)
            }
        });

        if let (Some(item), Some(course)) = (item, course) {
            course.favorite_items.insert((item.id.clone(), lvl).into());
            item.favorite_courses
                .insert((course.id.clone(), lvl).into());
            println!("{}, {} lvl {}", item.name, course.name, lvl);
        }
    }

    data.save("data/mkt_data_b&g.json").unwrap();
    data
}

fn test_wiki_coverage(data: &MktData) {
    let wiki = fs::read_to_string("tmp/wiki.txt").unwrap();

    let mut wiki_items = HashMap::new();

    // build wiki items
    for item in data
        .drivers
        .values()
        .chain(data.karts.values())
        .chain(data.gliders.values())
    {
        let rgx = format!(
            r"(.*\[File:.*[=|]{}\][^<\n]*)",
            item.name.replace("(", r"\(").replace(")", r"\)")
        );
        let re = Regex::new(&rgx).unwrap();
        if let Some(c) = re.captures_iter(&wiki).next() {
            if let Some(m) = c.get(0) {
                println!("{}", m.as_str());
                wiki_items.insert(item.id.to_string(), m.as_str().trim().to_string());
            }
        } else {
            println!("ERROR {}", item.name);
        };
    }
    println!("\n\n\n\n");

    let mut courses = data.courses.values().collect_vec();
    courses.sort_by_key(|c| course_parts_from_id(&c.id));

    fn display_wiki_items(
        wiki_new: &mut String,
        data: &MktData,
        favorite_items: &[&ItemRequirement],
        wiki_items: &HashMap<String, String>,
        rarity: Rarity,
        i_type: ItemType,
        lvl_range: fn(ItemLvl) -> bool,
    ) {
        let items = match i_type {
            ItemType::Driver => &data.drivers,
            ItemType::Kart => &data.karts,
            ItemType::Glider => &data.gliders,
        };
        favorite_items
            .iter()
            .filter(|r| lvl_range(r.lvl))
            .filter_map(|r| items.get(&r.id).map(|i| (r, i)))
            .filter(|(_, i)| i.rarity == rarity)
            .for_each(|(r, i)| {
                if let Some(wiki_item) = wiki_items.get(&i.id) {
                    writeln!(
                        wiki_new,
                        "{}{}",
                        wiki_item,
                        match r.lvl {
                            3 => "<sup>*</sup>",
                            6 => "<sup>**</sup>",
                            _ => "",
                        }
                    )
                    .unwrap();
                } else {
                    eprintln!("[ERROR] wiki doesn't have: {}", &i.id);
                }
            });
    }

    let mut wiki_new = String::new();
    for c in courses {
        let mut favorite_items = c
            .favorite_items
            .iter()
            .chain(
                c.favored_items
                    .iter()
                    .filter(|r1| !c.favorite_items.iter().any(|r2| r1.id == r2.id)),
            )
            .collect_vec();
        favorite_items.sort_by_key(|r| r.id.replace("_", " "));

        writeln!(&mut wiki_new, "----- {} -----", c.name).unwrap();
        for r in [Rarity::HighEnd, Rarity::Super, Rarity::Normal] {
            match r {
                Rarity::HighEnd => writeln!(
                    &mut wiki_new,
                    "|rowspan=3 style=\"background-color:#FEFEFE;\"|\n!!! COURSE NAME HERE !!!"
                )
                .unwrap(),
                Rarity::Super => {
                    writeln!(&mut wiki_new, "|-style=\"background-color:#FEEB80\"").unwrap()
                }
                Rarity::Normal => {
                    writeln!(&mut wiki_new, "|-style=\"background-color:#E3E3E3\"").unwrap()
                }
            }
            for (l, t) in [|l| l == 1, |l| (3..=6).contains(&l) || l == 0]
                .iter()
                .cartesian_product([ItemType::Driver, ItemType::Kart, ItemType::Glider])
            {
                writeln!(&mut wiki_new, "|").unwrap();
                display_wiki_items(&mut wiki_new, data, &favorite_items, &wiki_items, r, t, *l);
            }
        }
        writeln!(&mut wiki_new, "----- END {} -----", c.name).unwrap();
        writeln!(&mut wiki_new, "\n").unwrap();
    }

    fs::write("tmp/wiki_new.txt", wiki_new).unwrap();
}
