use std::fmt::Write;
use std::{collections::HashMap, fs};

use itertools::Itertools;
use mkt_data::*;
use mkt_update::*;

use regex::Regex;
use unidecode::unidecode;

fn main() {
    println!("MKT Data Update");

    let mut data = update_data();
    println!("courses: {}", data.courses.len());
    println!("drivers: {}", data.drivers.len());
    println!("karts: {}", data.karts.len());
    println!("gliders: {}", data.gliders.len());

    // don't overwrite with bad data
    if !data.courses.is_empty()
        && !data.drivers.is_empty()
        && !data.karts.is_empty()
        && !data.gliders.is_empty()
    {
        let hash =
            MktItemHashes::load("data/mkt_hash.json").unwrap_or_else(|_| MktItemHashes::new());
        data.merge_hashes(&hash);

        data.save("data/mkt_data.json").unwrap();
    } else {
        panic!(
            "some data are empty. courses: {}, drivers: {}, karts: {}, gliders: {}",
            data.courses.len(),
            data.drivers.len(),
            data.karts.len(),
            data.gliders.len()
        );
    }

    if std::env::var("MKT_B_G_WIKI_TEST").is_ok() {
        let bg_data = _test_b_and_g_coverage(&data);

        _test_wiki_coverage(&bg_data);
    }

    println!("Done");
}

fn update_data() -> MktData {
    let mut data = MktData::new();
    // uses b&g coverage, instead of wiki
    update_mkt_item_and_course_data(&mut data);
    update_mkt_mii_data(&mut data);
    _test_b_and_g_coverage(&data)
}

fn _test_b_and_g_coverage(data: &MktData) -> MktData {
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

    // csv version of the sheet Coverage Lookup
    let mut rdr = csv::Reader::from_path("tmp/coverage.csv").unwrap();

    for result in rdr.records() {
        let record = result.unwrap();
        let item_name = &record[0];
        let course_name_lvl1 = &record[1];
        let course_name_lvl3 = &record[2];
        let course_name_lvl6 = &record[3];
        let course_name_lvl8 = &record[4];
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
        } else if !course_name_lvl8.is_empty() {
            course_name = course_name_lvl8;
            lvl = 8;
        }

        if item_name.is_empty() || course_name.is_empty() {
            continue;
        }

        let item_name = item_name.to_uppercase();
        let course_name = course_name.to_uppercase();

        let mut item = data
            .drivers
            .values_mut()
            .chain(data.karts.values_mut())
            .chain(data.gliders.values_mut())
            .find(|i| i.get_bgr_name() == item_name);

        // exact match first
        let mut course = data
            .courses
            .values_mut()
            .find(|c| unidecode(&c.name).to_uppercase().replace("R/T", "RT") == course_name);
        // match without console
        if course.is_none() {
            course = data.courses.values_mut().find(|c| {
                !c.name.starts_with("RMX")
                    && unidecode(&c.name)
                        .to_uppercase()
                        .replace("R/T", "RT")
                        .ends_with(&course_name)
            });
        }

        if let (Some(item), Some(course)) = (item.as_mut(), course.as_mut()) {
            course.favorite_items.insert((item.id.clone(), lvl).into());
            item.favorite_courses
                .insert((course.id.clone(), lvl).into());
            // println!("{}, {} lvl {}", item.name, course.name, lvl);
        } else {
            if item.is_none() {
                println!("ERROR: missing item: {item_name:?}");
            }
            if course.is_none() {
                println!("ERROR: missing course: {course_name:?}");
            }
        }
    }

    data.save("data/mkt_data_b&g.json").unwrap();
    data
}

fn _test_wiki_coverage(data: &MktData) {
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
            item.name.replace('(', r"\(").replace(')', r"\)")
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
                            8 => "<sup>***</sup>",
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
        favorite_items.sort_by_key(|r| r.id.replace('_', " "));

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
            for (l, t) in [|l| l == 1, |l| (3..=8).contains(&l) || l == 0]
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
