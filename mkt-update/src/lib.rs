#![feature(try_blocks)]
#![allow(dead_code)]

use std::convert::TryInto;

use itertools::Itertools;
use lazy_static::lazy_static;
use mkt_data::*;
use regex::Regex;
use reqwest::blocking::{Client, ClientBuilder};
use scraper::{Element, ElementRef, Html, Selector};

lazy_static! {
    static ref HTTP_CLIENT: Client = ClientBuilder::new().cookie_store(true).build().unwrap();
}

pub fn update_mkt_item_data(data: &mut MktData, i_type: ItemType) {
    // get data (from Super Mario Wiki)
    let url = match i_type {
        ItemType::Driver => "https://www.mariowiki.com/List_of_drivers_in_Mario_Kart_Tour",
        ItemType::Kart => "https://www.mariowiki.com/List_of_karts_in_Mario_Kart_Tour",
        ItemType::Glider => "https://www.mariowiki.com/List_of_gliders_in_Mario_Kart_Tour",
    };
    match i_type {
        // the page format changed for drivers, might change as well for karts and gliders
        ItemType::Driver => parse_items_new_format(url, data, i_type, 7),
        ItemType::Kart => parse_items_new_format(url, data, i_type, 5),
        ItemType::Glider => parse_items_new_format(url, data, i_type, 5),
    }
}

fn parse_items(url: &str, data: &mut MktData, i_type: ItemType) {
    let resp = HTTP_CLIENT.get(url).send().unwrap();
    let content = resp.text().unwrap();

    let document = Html::parse_document(&content);

    let items_select = Selector::parse("table table tbody tr").unwrap();
    let name_select = Selector::parse("th:nth-of-type(1) a").unwrap();
    let img_select = Selector::parse("td:nth-of-type(1) img").unwrap();
    let rarity_select = Selector::parse("td:nth-of-type(3)").unwrap();

    let mut i = 1;
    for item in document.select(&items_select) {
        let _: Option<_> = try {
            let name = item
                .select(&name_select)
                .next()?
                .text()
                .next()?
                .trim()
                .into();
            let _img_url = item.select(&img_select).next()?.value().attr("src")?;
            let rarity = item
                .select(&rarity_select)
                .next()?
                .text()
                .next()?
                .trim()
                .try_into()
                .ok()?;
            let item = Item::new(i_type, rarity, name, Some(i));

            // println!("{:?}", item);
            match i_type {
                ItemType::Driver => &mut data.drivers,
                ItemType::Kart => &mut data.karts,
                ItemType::Glider => &mut data.gliders,
            }
            .insert(item.id.clone(), item);
            i += 1;
        };
    }
}

fn parse_items_new_format(url: &str, data: &mut MktData, i_type: ItemType, row_num: usize) {
    let name_rgx = Regex::new("<br/?>").unwrap();

    let resp = HTTP_CLIENT.get(url).send().unwrap();
    let content = resp.text().unwrap();

    let document = Html::parse_document(&content);

    let table_select = Selector::parse("h2 + table tbody").unwrap();
    let row_select = Selector::parse("tr").unwrap();
    let cell_select = Selector::parse("th a[title]:first-child, td").unwrap();

    let table = document.select(&table_select).next().unwrap();
    let rows = table.select(&row_select);

    let mut i = 1;
    for mut rs in rows
        .map(|r| r.select(&cell_select))
        .chunks(row_num)
        .into_iter()
    {
        let (names, _, _, rarities) = rs.next_tuple().unwrap();
        for (name, rarity) in names.zip(rarities) {
            let name = Some(name)
                .into_iter()
                .chain(name.next_siblings().filter_map(ElementRef::wrap))
                .map(|n| n.inner_html())
                .join(" ");
            let _: Option<_> = try {
                let name = name_rgx.replace_all(&name, " ").trim().into();
                let rarity = rarity.text().next()?.trim().try_into().ok()?;
                let item = Item::new(i_type, rarity, name, Some(i));

                // println!("{:?}", item);
                match i_type {
                    ItemType::Driver => &mut data.drivers,
                    ItemType::Kart => &mut data.karts,
                    ItemType::Glider => &mut data.gliders,
                }
                .insert(item.id.clone(), item);
                i += 1;
            };
        }
    }
}

pub fn update_mkt_item_coverage_data(data: &mut MktData) {
    let name_rgx = Regex::new("('s icon)? from.*").unwrap();

    let urls = [
        ("", "https://www.mariowiki.com/List_of_favored_and_favorite_new_courses_in_Mario_Kart_Tour"),
        ("", "https://www.mariowiki.com/List_of_favored_and_favorite_remix_courses_in_Mario_Kart_Tour"),
        ("SNES", "https://www.mariowiki.com/List_of_favored_and_favorite_Super_Mario_Kart_(SNES)_courses_in_Mario_Kart_Tour"),
        ("N64", "https://www.mariowiki.com/List_of_favored_and_favorite_Mario_Kart_64_(N64)_courses_in_Mario_Kart_Tour"),
        ("GBA", "https://www.mariowiki.com/List_of_favored_and_favorite_Mario_Kart:_Super_Circuit_(GBA)_courses_in_Mario_Kart_Tour"),
        ("GCN", "https://www.mariowiki.com/List_of_favored_and_favorite_Mario_Kart:_Double_Dash!!_(GCN)_courses_in_Mario_Kart_Tour"),
        ("DS", "https://www.mariowiki.com/List_of_favored_and_favorite_Mario_Kart_DS_(DS)_courses_in_Mario_Kart_Tour"),
        ("Wii", "https://www.mariowiki.com/List_of_favored_and_favorite_Mario_Kart_Wii_(Wii)_courses_in_Mario_Kart_Tour"),
        ("3DS", "https://www.mariowiki.com/List_of_favored_and_favorite_Mario_Kart_7_(3DS)_courses_in_Mario_Kart_Tour"),
    ];

    for (prefix, url) in urls {
        // get data (from Super Mario Wiki)
        let resp = HTTP_CLIENT.get(url).send().unwrap();
        let content = resp.text().unwrap();

        let document = Html::parse_document(&content);
        let courses_select = Selector::parse("h2 + table").unwrap();
        let row_select = Selector::parse("tr").unwrap();
        let cell_select = Selector::parse("td").unwrap();
        let item_select = Selector::parse("a").unwrap();
        let course_name_select = Selector::parse("a[title]").unwrap();

        let mut i = 1;
        for course in document.select(&courses_select) {
            for (high_ends, supers, normals) in course.select(&row_select).skip(1).tuples() {
                let i_types = Some(None).into_iter().chain(
                    IntoIterator::into_iter([ItemType::Driver, ItemType::Kart, ItemType::Glider])
                        .map(Option::Some)
                        .cycle(),
                );
                let levels = Some(1).into_iter().chain(
                    Some(1)
                        .into_iter()
                        .cycle()
                        .take(3)
                        .chain(Some(0).into_iter().cycle().take(3))
                        .cycle(),
                );

                let course: Option<_> = try {
                    let name = course
                        .prev_sibling_element()?
                        .select(&course_name_select)
                        .next()?
                        .inner_html();
                    let variant = high_ends
                        .first_element_child()?
                        .text()
                        .next()?
                        .split_once(' ')?
                        .0
                        .replace("Normal", "");
                    let mut course = String::new();
                    course += prefix;
                    if !course.is_empty() {
                        course += " ";
                    }
                    course += &name;
                    if name.chars().last()?.is_ascii_digit() {
                        course += &variant;
                    } else if !variant.is_empty() {
                        course += " ";
                        course += &variant;
                    }
                    course
                };

                let mut drivers = vec![];
                let mut karts = vec![];
                let mut gliders = vec![];

                for (name, i_type, lvl) in IntoIterator::into_iter([high_ends, supers, normals])
                    .flat_map(|r| r.select(&cell_select))
                    .zip(i_types)
                    .zip(levels)
                    .flat_map(|((c, t), l)| {
                        c.select(&item_select)
                            .map(move |i| {
                                let mut l = l;
                                if let Some(s) = i.next_sibling() {
                                    if let Some(e) = ElementRef::wrap(s) {
                                        if e.value().name() == "sup" {
                                            // * = 3, ** = 6, *** = 8
                                            l = match e
                                                .inner_html()
                                                .chars()
                                                .filter(|c| *c == '*')
                                                .count()
                                            {
                                                1 => 3,
                                                2 => 6,
                                                3 => 8,
                                                _ => 0,
                                            };
                                        }
                                    }
                                }
                                (i, l)
                            })
                            .flat_map(|(i, l)| {
                                if let Some(n) = i.value().attr("title") {
                                    Some((n, l))
                                } else if let Some(n) = i.select(&course_name_select).next() {
                                    n.value().attr("alt").map(|n| (n, l))
                                } else {
                                    None
                                }
                            })
                            .map(|(n, l)| (name_rgx.replace(n, ""), l))
                            .map(move |(n, l)| (n, t, l))
                        // .filter(|(_, _, l)| *l > 0)
                    })
                {
                    match i_type {
                        None => {}
                        Some(ItemType::Driver) => drivers.push((name, lvl as ItemLvl)),
                        Some(ItemType::Kart) => karts.push((name, lvl as ItemLvl)),
                        Some(ItemType::Glider) => gliders.push((name, lvl as ItemLvl)),
                    }
                }

                // println!("{:?}", &course);
                // println!("{:?}", &drivers);
                // println!("{:?}", &karts);
                // println!("{:?}", &gliders);

                if let Some(course) = course {
                    let course_id = course_id_from_name(&course);
                    data.courses
                        .insert(course_id.clone(), Course::new(course.to_string(), Some(i)));
                    let course = data.courses.get_mut(&course_id).unwrap();
                    i += 1;

                    // drivers
                    let mut drivers_id: Vec<ItemRequirement> = vec![];
                    for (driver, lvl) in drivers {
                        let driver_id = driver_id_from_name(&driver);
                        let _: Option<_> = try {
                            let driver = data.drivers.get_mut(&driver_id)?;
                            if lvl > 0 {
                                driver
                                    .favorite_courses
                                    .insert((course_id.clone(), lvl).into());
                            } else {
                                driver
                                    .favored_courses
                                    .insert((course_id.clone(), lvl).into());
                            }
                            drivers_id.push((driver_id, lvl).into());
                        };
                    }
                    course
                        .favorite_items
                        .extend(drivers_id.iter().filter(|r| r.lvl > 0).cloned());
                    course
                        .favored_items
                        .extend(drivers_id.iter().filter(|r| r.lvl == 0).cloned());

                    // karts
                    let mut karts_id: Vec<ItemRequirement> = vec![];
                    for (kart, lvl) in karts {
                        let kart_id = kart_id_from_name(&kart);
                        let _: Option<_> = try {
                            let kart = data.karts.get_mut(&kart_id)?;
                            if lvl > 0 {
                                kart.favorite_courses
                                    .insert((course_id.clone(), lvl).into());
                            } else {
                                kart.favored_courses.insert((course_id.clone(), lvl).into());
                            }
                            karts_id.push((kart_id, lvl).into());
                        };
                    }
                    course
                        .favorite_items
                        .extend(karts_id.iter().filter(|r| r.lvl > 0).cloned());
                    course
                        .favored_items
                        .extend(karts_id.iter().filter(|r| r.lvl == 0).cloned());

                    // gliders
                    let mut gliders_id: Vec<ItemRequirement> = vec![];
                    for (glider, lvl) in gliders {
                        let glider_id = glider_id_from_name(&glider);
                        let _: Option<_> = try {
                            let glider = data.gliders.get_mut(&glider_id)?;
                            if lvl > 0 {
                                glider
                                    .favorite_courses
                                    .insert((course_id.clone(), lvl).into());
                            } else {
                                glider
                                    .favored_courses
                                    .insert((course_id.clone(), lvl).into());
                            }
                            gliders_id.push((glider_id, lvl).into());
                        };
                    }
                    course
                        .favorite_items
                        .extend(gliders_id.iter().filter(|r| r.lvl > 0).cloned());
                    course
                        .favored_items
                        .extend(gliders_id.iter().filter(|r| r.lvl == 0).cloned());
                }
            }
        }
    }
}

pub fn update_mkt_item_and_course_data(data: &mut MktData) {
    // get data (from Super Mario Wiki)
    let resp = HTTP_CLIENT
        .get("https://www.mariowiki.com/Template:MKT")
        .send()
        .unwrap();
    let content = resp.text().unwrap();

    let document = Html::parse_document(&content);
    let rows_select = Selector::parse("tr").unwrap();
    let item_name_select = Selector::parse("td a").unwrap();

    let rows = document.select(&rows_select).collect_vec();

    let rarities = [Rarity::Normal, Rarity::Super, Rarity::HighEnd];

    // drivers
    let mut i = 0;
    for (drivers, rarity) in rows[2..5].iter().zip(rarities.iter()) {
        for driver in drivers.select(&item_name_select) {
            let name = driver.text().collect::<String>();
            if name.contains("Mii") {
                continue;
            }

            i += 1;
            let item = Item::new(ItemType::Driver, *rarity, name, Some(i));
            data.drivers.insert(item.id.clone(), item);
        }
    }

    // karts
    let mut i = 0;
    for (karts, rarity) in rows[6..9].iter().zip(rarities.iter()) {
        for kart in karts.select(&item_name_select) {
            i += 1;
            let name = kart.text().collect::<String>();
            let item = Item::new(ItemType::Kart, *rarity, name, Some(i));
            data.karts.insert(item.id.clone(), item);
        }
    }

    // gliders
    let mut i = 0;
    for (gliders, rarity) in rows[9..12].iter().zip(rarities.iter()) {
        for glider in gliders.select(&item_name_select) {
            i += 1;
            let name = glider.text().collect::<String>();
            let item = Item::new(ItemType::Glider, *rarity, name, Some(i));
            data.gliders.insert(item.id.clone(), item);
        }
    }

    // courses
    let mut i = 0;
    for (courses, prefix) in rows[14..26].iter().zip([
        "", "", "", "SNES", "N64", "GBA", "GCN", "DS", "Wii", "3DS", "", "",
    ]) {
        for course in courses.select(&item_name_select) {
            let name = course.text().collect::<String>();
            if name == "tour appearances" {
                continue;
            }

            if let Some((name, rt)) = name.split_once(" (") {
                for name in Some("")
                    .into_iter()
                    .chain(rt.trim_end_matches(')').split(',').map(str::trim))
                    .map(|rt| {
                        let mut course = String::new();
                        course += prefix;
                        if !course.is_empty() {
                            course += " ";
                        }
                        course += &name;
                        if name.chars().last().unwrap_or_default().is_ascii_digit() {
                            course += &rt;
                        } else if !rt.is_empty() {
                            course += " ";
                            course += &rt;
                        }
                        course
                    })
                {
                    i += 1;
                    let course = Course::new(name, Some(i));
                    data.courses.insert(course.id.clone(), course);
                }
            } else {
                i += 1;
                let name = format!("{prefix} {name}").trim().into();
                let course = Course::new(name, Some(i));
                data.courses.insert(course.id.clone(), course);
            }
        }
    }
}

pub fn update_mkt_mii_data(data: &mut MktData) {
    // get data (from Super Mario Wiki)
    let resp = HTTP_CLIENT
        .get("https://www.mariowiki.com/Mii")
        .send()
        .unwrap();
    let content = resp.text().unwrap();

    let document = Html::parse_document(&content);
    let span_select = Selector::parse("#Mario_Kart_Tour").unwrap();
    let row_select = Selector::parse("tr").unwrap();
    let cell_select = Selector::parse("td b").unwrap();

    let table = document
        .select(&span_select) // span
        .next()
        .unwrap()
        .parent() // h4
        .unwrap()
        .next_siblings()
        .find(|e| e.value().as_element().map(|e| e.name()).unwrap_or_default() == "table")
        .map(ElementRef::wrap)
        .unwrap()
        .unwrap();

    let rows = table.select(&row_select);

    let mut i = data.drivers.len() as u32;
    for row in rows.skip(2).step_by(4) {
        for cell in row.select(&cell_select) {
            i += 1;
            let name = cell.text().next().unwrap();
            let item = Item::new(ItemType::Driver, Rarity::HighEnd, name.into(), Some(i));
            data.drivers.insert(item.id.clone(), item);
        }
    }
}
