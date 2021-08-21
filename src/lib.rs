#![allow(dead_code)]
pub mod data;
pub mod screenshot;

pub use data::*;
use itertools::Itertools;
use regex::Regex;
use scraper::{ElementRef, Html, Selector};
use screenshot::*;

use image::RgbImage;

pub fn update_mkt_item_coverage_data(data: &mut MktDatabase) {
    let name_rgx = Regex::new("('s icon)? from.*").unwrap();

    // TODO: get data (from Super Mario Wiki?)
    let resp = reqwest::blocking::get(
        "https://www.mariowiki.com/List_of_favored_and_favorite_courses_in_Mario_Kart_Tour",
    )
    .unwrap();
    let content = resp.text().unwrap();

    let document = Html::parse_document(&content);
    let courses_select = Selector::parse("h3 + table").unwrap();
    let row_select = Selector::parse("tr").unwrap();
    let cell_select = Selector::parse("td").unwrap();
    let item_select = Selector::parse("a").unwrap();
    let course_name_select = Selector::parse("img[alt]").unwrap();

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

            let mut course = None;
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
                                        // * = 3, ** = 6
                                        l = e.inner_html().chars().filter(|c| *c == '*').count()
                                            * 3;
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
                        .filter(|(_, _, l)| *l > 0)
                })
            {
                match i_type {
                    None => course = Some(name),
                    Some(ItemType::Driver) => drivers.push((name, lvl as ItemLvl)),
                    Some(ItemType::Kart) => karts.push((name, lvl as ItemLvl)),
                    Some(ItemType::Glider) => gliders.push((name, lvl as ItemLvl)),
                }
            }

            println!("{:?}", &course);
            println!("{:?}", &drivers);
            println!("{:?}", &karts);
            println!("{:?}", &gliders);

            if let Some(course) = course {
                let course_id = course_id_from_name(&course);
                // drivers
                let mut drivers_id: Vec<(String, u8)> = vec![];
                for (driver, lvl) in drivers {
                    let driver_id = driver_id_from_name(&driver);
                    let driver = data.drivers.entry(driver_id.clone()).or_insert_with(|| {
                        Item::new(ItemType::Driver, Rarity::Normal, driver.into())
                        // TODO: fix rarity
                    });
                    driver.favorite_courses.insert((course_id.clone(), lvl));
                    drivers_id.push((driver_id, lvl));
                }
                data.courses
                    .entry(course_id.clone())
                    .or_insert_with(|| Course::new(course.to_string()))
                    .favorite_items
                    .extend(drivers_id);

                // karts
                let mut karts_id: Vec<(String, u8)> = vec![];
                for (kart, lvl) in karts {
                    let kart_id = kart_id_from_name(&kart);
                    let kart = data
                        .karts
                        .entry(kart_id.clone())
                        .or_insert_with(|| Item::new(ItemType::Kart, Rarity::Normal, kart.into())); // TODO: fix rarity
                    kart.favorite_courses.insert((course_id.clone(), lvl));
                    karts_id.push((kart_id, lvl));
                }
                data.courses
                    .entry(course_id.clone())
                    .or_insert_with(|| Course::new(course.to_string()))
                    .favorite_items
                    .extend(karts_id);

                // gliders
                let mut gliders_id: Vec<(String, u8)> = vec![];
                for (glider, lvl) in gliders {
                    let glider_id = glider_id_from_name(&glider);
                    let glider = data.gliders.entry(glider_id.clone()).or_insert_with(|| {
                        Item::new(ItemType::Glider, Rarity::Normal, glider.into())
                        // TODO: fix rarity
                    });
                    glider.favorite_courses.insert((course_id.clone(), lvl));
                    gliders_id.push((glider_id, lvl));
                }
                data.courses
                    .entry(course_id.clone())
                    .or_insert_with(|| Course::new(course.to_string()))
                    .favorite_items
                    .extend(gliders_id);
            }
        }
    }
}

fn import_screenshot(inv: &mut MktInventory, img: RgbImage, data: &MktDatabase) {
    // TODO: might be a client side function

    // TODO: get the picture from browser

    // update inventory
    inv.update_inventory(screenshots_to_inventory(vec![img], data).0);

    // TODO: save inventory to browser
}
