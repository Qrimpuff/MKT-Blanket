use mkt_data::*;
use mkt_update::*;

use unidecode::unidecode;

fn main() {
    println!("MKT Data Update");

    test_b_and_g_coverage();
    return;

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

    let mut rdr = csv::Reader::from_path("tests/coverage.csv").unwrap();

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
