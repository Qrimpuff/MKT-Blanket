#![feature(try_blocks)]
#![allow(dead_code)]

use std::convert::TryInto;

use itertools::Itertools;
use mkt_data::*;
use regex::Regex;
use scraper::{ElementRef, Html, Selector};

pub fn update_mkt_item_data(data: &mut MktDatabase, i_type: ItemType) {
    // get data (from Super Mario Wiki)
    let url = match i_type {
        // TODO: the page format changed for drivers, might change as well for karts and gliders
        // ItemType::Driver => "https://www.mariowiki.com/List_of_drivers_in_Mario_Kart_Tour",
        ItemType::Driver => "https://www.mariowiki.com/index.php?title=List_of_drivers_in_Mario_Kart_Tour&oldid=3268498",
        ItemType::Kart => "https://www.mariowiki.com/List_of_karts_in_Mario_Kart_Tour",
        ItemType::Glider => "https://www.mariowiki.com/List_of_gliders_in_Mario_Kart_Tour",
    };

    let resp = reqwest::blocking::get(url).unwrap();
    let content = resp.text().unwrap();

    let document = Html::parse_document(&content);

    // try the old style first
    let items_select = Selector::parse("table tbody tr").unwrap();
    let name_select = Selector::parse("th:nth-of-type(1) a").unwrap();
    let img_select = Selector::parse("td:nth-of-type(1) img").unwrap();
    let rarity_select = Selector::parse("td:nth-of-type(3)").unwrap();

    for (i, item) in document.select(&items_select).enumerate() {
        let _: Option<_> = try {
            let name = item.select(&name_select).next()?.text().next()?.trim();
            let _img_url = item.select(&img_select).next()?.value().attr("src")?;
            let rarity = item.select(&rarity_select).next()?.text().next()?.trim();
            let item = Item::new(i_type, rarity.try_into().ok()?, name.into(), Some(i as u32));

            println!("{:?}", item);
            match i_type {
                ItemType::Driver => &mut data.drivers,
                ItemType::Kart => &mut data.karts,
                ItemType::Glider => &mut data.gliders,
            }
            .insert(item.id.clone(), item);
        };
    }

    // TODO: try the new style, used on drivers
}

pub fn update_mkt_item_coverage_data(data: &mut MktDatabase) {
    let name_rgx = Regex::new("('s icon)? from.*").unwrap();

    // get data (from Super Mario Wiki)
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

            // println!("{:?}", &course);
            // println!("{:?}", &drivers);
            // println!("{:?}", &karts);
            // println!("{:?}", &gliders);

            if let Some(course) = course {
                let course_id = course_id_from_name(&course);
                data.courses
                    .insert(course_id.clone(), Course::new(course.to_string()));
                let course = data.courses.get_mut(&course_id).unwrap();

                // drivers
                let mut drivers_id = vec![];
                for (driver, lvl) in drivers {
                    let driver_id = driver_id_from_name(&driver);
                    let _: Option<_> = try {
                        let driver = data.drivers.get_mut(&driver_id)?;
                        driver
                            .favorite_courses
                            .insert((course_id.clone(), lvl).into());
                        drivers_id.push((driver_id, lvl).into());
                    };
                }
                course.favorite_items.extend(drivers_id);

                // karts
                let mut karts_id = vec![];
                for (kart, lvl) in karts {
                    let kart_id = kart_id_from_name(&kart);
                    let _: Option<_> = try {
                        let kart = data.karts.get_mut(&kart_id)?;
                        kart.favorite_courses
                            .insert((course_id.clone(), lvl).into());
                        karts_id.push((kart_id, lvl).into());
                    };
                }
                course.favorite_items.extend(karts_id);

                // gliders
                let mut gliders_id = vec![];
                for (glider, lvl) in gliders {
                    let glider_id = glider_id_from_name(&glider);
                    let _: Option<_> = try {
                        let glider = data.gliders.get_mut(&glider_id)?;
                        glider
                            .favorite_courses
                            .insert((course_id.clone(), lvl).into());
                        gliders_id.push((glider_id, lvl).into());
                    };
                }
                course.favorite_items.extend(gliders_id);
            }
        }
    }
}
