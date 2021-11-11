use std::{collections::HashMap, fs, ops::RangeInclusive};

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

    test_b_and_g_coverage();

    test_wiki_coverage();

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

fn test_b_and_g_coverage() {
    let mut data = MktData::load("data/mkt_data.json").unwrap();

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
}

fn test_wiki_coverage() {
    let data = MktData::load("data/mkt_data_b&g.json").unwrap();
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
    courses.sort_by_key(|c| &c.id);

    fn display_wiki_items(
        data: &MktData,
        favorite_items: &[&ItemRequirement],
        wiki_items: &HashMap<String, String>,
        rarity: Rarity,
        i_type: ItemType,
        lvl_range: RangeInclusive<u8>,
    ) {
        let items = match i_type {
            ItemType::Driver => &data.drivers,
            ItemType::Kart => &data.karts,
            ItemType::Glider => &data.gliders,
        };
        favorite_items
            .iter()
            .filter(|r| lvl_range.contains(&r.lvl))
            .filter_map(|r| items.get(&r.id).map(|i| (r, i)))
            .filter(|(_, i)| i.rarity == rarity)
            .for_each(|(r, i)| {
                println!(
                    "{}{}",
                    wiki_items
                        .get(&i.id)
                        .unwrap_or_else(|| panic!("wiki doesn't have: {}", &i.id)),
                    match r.lvl {
                        3 => "<sup>*</sup>",
                        6 => "<sup>**</sup>",
                        _ => "",
                    }
                )
            });
    }

    for c in courses {
        let mut favorite_items = c.favorite_items.iter().collect_vec();
        favorite_items.sort_by_key(|r| r.id.replace("_", " "));

        println!("----- {} -----", c.name);
        for ((r, l), t) in [Rarity::HighEnd, Rarity::Super, Rarity::Normal]
            .iter()
            .cartesian_product([1..=1, 3..=6])
            .cartesian_product([ItemType::Driver, ItemType::Kart, ItemType::Glider])
        {
            println!("|");
            display_wiki_items(&data, &favorite_items, &wiki_items, *r, t, l);
        }
        println!("----- END {} -----", c.name);
        println!("\n");
    }
}
