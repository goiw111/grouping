use clap::Parser;
use rusqlite::{Connection, Result};
use csv::Writer;
use serde::Serialize;
use std::fs::File;
use std::io::Write;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {

    #[arg(short = 'i')]
    insert: bool,

    #[arg(short = 'a', long)]
    name_a: Option<String>,

    #[arg(long)]
    aid: Option<String>,

    #[arg(short = 'b', long)]
    name_b: Option<String>,

    #[arg(long)]
    bid: Option<String>,

    #[arg(short = 'l')]
    ls: bool,

    #[arg(short = 'r', default_value_t = -1i32)]
    remove: i32,

    #[arg(short = 'o')]
    output: bool,

    #[arg(short = 'd')]
    outdir: Option<std::path::PathBuf>,
}

#[derive(Debug, Serialize)]
struct Person {
    #[serde(rename = "ID")]
    id: i32,
    #[serde(rename = "Name")]
    name_a: String,
    #[serde(rename = "CNE")]
    ida: Option<String>,
    #[serde(rename = "Name")]
    name_b: String,
    #[serde(rename = "CNE")]
    idb: Option<String>
}

fn main() -> Result<()> {
    let args = Args::parse();

    let conn = Connection::open("groups.db")?;

    //println!("{:?}", args);

    if args.insert && (args.name_a.is_some() && args.name_b.is_some()) {
        conn.execute("INSERT INTO binome (name_a, ida, name_b, idb) \
                     VALUES (?1, ?2, ?3, ?4)",
        (&args.name_a, &args.aid.unwrap_or(String::from("")),
        &args.name_b, &args.bid.unwrap_or(String::from(""))))?;
    }

    if args.ls {
        let mut stmt = conn.prepare("SELECT id, name_a, ida, name_b, idb \
                                    FROM binome")?;
        let person_iter = stmt.query_map([], |row| {
            Ok(Person {
                id: row.get(0)?,
                name_a: row.get(1)?,
                ida: row.get(2)?,
                name_b: row.get(3)?,
                idb: row.get(4)?
            })
        })?;
        println!(
        "{0: <3} | {1: <23} | {2: <20} | {3: <23} | {4: <20}",
        "id", "name", "CNE", "name" , "CNE");
        for person in person_iter {
            let person = person?;
            println!(
            "{0: <3} | {1: <23} | {2: <20} | {3: <23} | {4: <20}",
            person.id, person.name_a, person.ida.unwrap_or(String::from("")),
            person.name_b, person.idb.unwrap_or(String::from("")));
        }
    }

    if args.remove > 0 {
        conn.execute("DELETE FROM binome WHERE id = ?1", (&args.remove,))?;
    }
    if args.output {
        let mut wtr = Writer::from_writer(vec![]);
        let mut stmt = conn.prepare("SELECT id, name_a, ida, name_b, idb \
                                    FROM binome")?;
        let person_iter = stmt.query_map([], |row| {
            Ok(Person {
                id: row.get(0)?,
                name_a: row.get(1)?,
                ida: row.get(2)?,
                name_b: row.get(3)?,
                idb: row.get(4)?
            })
        })?;
        let mut count = 1;

        for person in person_iter {
            let mut person = person?;
            person.id = count;
                
            let _ = wtr.serialize(person);
            count += 1;
        }
        if let Some(path) = args.outdir {
            let mut file = File::create(path).unwrap();
            file.write_all(&wtr.into_inner().unwrap());
        } else {
            let mut file = File::create("list.csv").unwrap();
            file.write_all(&wtr.into_inner().unwrap());
        }
    }

    Ok(())
}
