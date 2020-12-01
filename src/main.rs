extern crate csv;
#[macro_use]
extern crate serde_derive;

use std::env;
use std::error::Error;
use std::ffi::OsString;
use std::fs::File;
use std::process;

#[derive(Deserialize, Debug)]
struct Row<'a> {
    address_code: &'a str,
    pref_code: &'a str,
    city_code: &'a str,
    area_code: &'a str,
    zip_code: &'a str,
    company_flag: &'a str,
    stop_flag: &'a str,
    pref_name: &'a str,
    pref_name_kana: &'a str,
    city_name: &'a str,
    city_name_kana: &'a str,
    town_area: &'a str,
    town_area_kana: &'a str,
    town_area_supplement: &'a str,
    kyoto_street_name: &'a str,
    street_name: &'a str,
    street_name_kana: &'a str,
    supplement: &'a str,
    company_name: &'a str,
    company_name_kana: &'a str,
    company_address: &'a str,
    new_address_code: &'a str,
}

#[derive(Debug, PartialEq)]
struct Pref {
    id: u8,
    name: String,
}

#[derive(Debug, PartialEq)]
struct City {
    code: String,
    name: String,
    pref_id: u8,
}

#[derive(Debug, PartialEq)]
struct Town {
    id: u32,
    zip_code: String,
    area_name: String,
    street_name: String,
    city_code: String,
}

fn run() -> Result<(), Box<Error>> {
    let file_path = get_first_args()?;
    let file = File::open(file_path)?;
    let mut rdr = csv::Reader::from_reader(file);
    let mut prefs: Vec<Pref> = vec![];
    let mut cities: Vec<City> = vec![];
    let mut towns: Vec<Town> = vec![];

    for result in rdr.records() {
        let temp = result.unwrap();
        let record: Row = temp.deserialize(None)?;

        let is_company: u8 = record.company_flag.parse().unwrap();

        if is_company == 1 {
            continue;
        }

        let pref = collect_prefs(&record);
        let city = collect_cities(&record);
        let town = collect_towns(&record, &city, &towns);

        if !prefs.contains(&pref) {
            prefs.push(pref);
        }
        if !cities.contains(&city) {
            cities.push(city);
        }
        if !towns.contains(&town) {
            towns.push(town);
        }
    }
    create_prefs_sql(&prefs);
    create_cities_sql(&cities);
    create_towns_sql(&towns);
    Ok(())
}

fn get_first_args() -> Result<OsString, Box<Error>> {
    match env::args_os().nth(1) {
        None => Err(From::from("expected 1 argument, but got none")),
        Some(file_path) => Ok(file_path),
    }
}

fn collect_prefs(record: &Row) -> Pref {
    Pref {
        id: record.pref_code.parse().unwrap(),
        name: record.pref_name.to_string(),
    }
}

fn collect_cities(record: &Row) -> City {
    City {
        code: record.city_code.to_string(),
        name: record.city_name.to_string(),
        pref_id: record.pref_code.parse().unwrap(),
    }
}

fn collect_towns(record: &Row, city: &City, towns: &Vec<Town>) -> Town {
    Town {
        id: towns.len() as u32 + 1,
        zip_code: record.zip_code.to_string(),
        area_name: record.town_area.to_string(),
        street_name: format!("{}{}", record.street_name.to_string(), record.kyoto_street_name),
        city_code: city.code.to_string()
    }
}

fn create_prefs_sql(prefs: &Vec<Pref>) {
    println!("INSERT INTO prefs(id, name) VALUES");
    for pref in prefs {
        print!("({}, '{}')", pref.id, pref.name);
        if prefs.last().unwrap() == pref {
            print!(";\n\n");
        } else {
            print!(",\n");
        }
    }
}

fn create_cities_sql(cities: &Vec<City>) {
    println!("INSERT INTO cities(id, pref_id, code, name) VALUES");
    for city in cities {
        print!("('{}', {}, '{}')", city.code, city.pref_id, city.name);
        if cities.last().unwrap() == city {
            print!(";\n\n");
        } else {
            print!(",\n");
        }
    }
}
fn create_towns_sql(towns: &Vec<Town>) {
    println!("INSERT INTO towns(id, city_id, zip_code, area_name, street_name)");
    for town in towns {
        print!("({}, '{}', '{}', '{}', '{}')", town.id, town.city_code, town.zip_code, town.area_name, town.street_name);
        if towns.last().unwrap() == town {
            print!(";\n\n");
        } else {
            print!(",\n");
        }
    }
}

fn main() {
    if let Err(err) = run() {
        println!("{}", err);
        process::exit(1);
    }
}
