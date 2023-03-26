use clap::{Parser, Subcommand, ValueEnum};
use rusqlite::{Result, Connection};
use grouping::*;
use comfy_table::Table;
use comfy_table::Row;
use xlsxwriter::*;
use std::path::Path;


#[derive(Debug, Parser)] // requires `derive` feature
#[command(name = "grouping")]
#[command(about = "A fictional versioning CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    #[arg(long, require_equals = true,)]
    path: Option<String>,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Bi {
        #[arg(required = true)]
        #[arg(value_name = "TD ID",required = true)]
        tp: Option<u32>,
        #[arg(value_name = "FIRST ID")]
        f_id: Option<String>,
        #[arg(value_name = "SECEND ID")]
        s_id: Option<String>,
        #[arg(short = 'r')]
        remove: Option<u32>,
        #[arg(short = 'd',
              value_enum,
              value_name = "DUMP_TYPE")]
        dump: Option<Dumpargs>
    },
    Tp {
        #[arg(value_name = "NAME")]
        name: Option<String>,
        #[arg(short = 'r')]
        remove: Option<u32>
    },
    Init {
        #[arg(short= 'p')]
        path: String,
    }
}

#[derive(ValueEnum, Copy, Clone, Debug, PartialEq, Eq)]
enum Dumpargs {
    Stdout,
    Xlsx
}

fn  main() -> Result<()> {
    let args = Cli::parse();

    let conn = Connection::open("groups.db")?;
    match args.command {
        Commands::Init { path } => println!("test {:?}", Path::new(&path)),
        Commands::Bi { tp, f_id, s_id , remove, dump } => {
            if let Some(id) = remove {
                match remove_bi(&conn, id) {
                    Ok(r)   => if r { 
                        println!("the pare successfully removed") 
                    } else { 
                        println!("there is no pare with that id : {id}") 
                    },
                    Err(e)  =>  println!("{e}"),
                }
            } 
            match (f_id,s_id,tp) {
                (Some(f_id), Some(s_id), Some(id)) => {
                    match set_bi(&conn, id, &f_id, &s_id) {
                        Ok(n)   => if n.is_empty() { 
                            println!("{f_id} and {s_id} are successfully added") 
                        } else {
                            println!("{n} is alredy exist");
                        },
                        Err(e)  => eprintln!("{e}"),
                    }
                },
                (None,Some(_),_) | (Some(_),None,_) => println!("you should add the both pares"),
                _ => (),
            }
            if let Some(id) = tp {
                match get_bis(&conn, id) {
                    Ok(Some(r)) => {
                        match dump {
                            Some(Dumpargs::Stdout) => {
                                println!("{} Row",r.len());
                                let mut table = Table::new();
                                table.set_header(["BIN","NOM ET PRENOM","N MASAR","N INSCIPT"]);
                                r.into_iter()
                                    .fold(1,|c: u32,s| {
                                        let mut row = Row::new();
                                        row.add_cell(c.into())
                                            .add_cell(format!("{}\n--\n{}",s.get_f_student_name(),s.get_s_student_name()).into())
                                            .add_cell(format!("{}\n--\n{}",s.get_f_student_masar(),s.get_s_student_masar()).into())
                                            .add_cell(format!("{}\n--\n{}",s.get_f_student_ed(),s.get_s_student_ed()).into());
                                        table.add_row(row);
                                        c + 1
                                });
                                println!("{table}");
                            },
                            Some(Dumpargs::Xlsx) => {
                                match Workbook::new(&format!("table_{}.xlsx",id)) {
                                    Ok(f) => {
                                        let mut sheet = f.add_worksheet(None).expect("non expected");
                                        sheet.set_column(1,3,20.0,None).expect("unexpected");
                                        let format_bin = f.add_format()
                                            .set_align(FormatAlignment::Center)
                                            .set_align(FormatAlignment::VerticalCenter);
                                        r.into_iter()
                                            .fold(0, |c: u32,s| {
                                                sheet.merge_range(c , 0,c + 1, 0, &((c/2) + 1).to_string(), Some(&format_bin))
                                                    .expect("unexpected");
                                                sheet.write_string(c, 1,&s.get_f_student_masar(), None)
                                                    .expect("unexpected");
                                                sheet.write_string(c + 1, 1,&s.get_s_student_masar(), None)
                                                    .expect("unexpected");
                                                sheet.write_string(c, 2,&s.get_f_student_ed(), None)
                                                    .expect("unexpected");
                                                sheet.write_string(c + 1, 2,&s.get_s_student_ed(), None)
                                                    .expect("unexpected");
                                                sheet.write_string(c, 3, &s.get_f_student_name(), None)
                                                    .expect("unexpected");
                                                sheet.write_string(c + 1, 3, &s.get_s_student_name(), None)
                                                    .expect("unexpected");
                                                c + 2
                                            });
                                        f.close().expect("unexpected");
                                    },
                                    Err(e) => println!("{e}"),
                                }
                            },
                            None => {
                                let list = r.clone();
                                let mut table = Table::new();
                                table.set_header(["pare id","student id","name","masar","ed"])
                                .add_rows(r);
                                println!("{table}");
                                println!("{} Row",list.len());
                            },
                        }
                                            
                    },
                    Ok(None) => println!("it's a empty list"),
                    Err(e) => println!("{e}"),
                }
            }
        }
        Commands::Tp { name, remove } => {
            if let Some(id) = remove {
                match get_tp(&conn, id) {
                    Ok(Some(tp)) => {
                        match remove_tp(&conn, tp.get_id()) {
                            Ok(_)   => println!("{} successfully removed",tp.get_name()),
                            Err(e)  => eprintln!("{e}"),
                        }
                    },
                    Ok(None) => println!("there is no TP with that ID : {id}"),
                    Err(e) => println!("{:?}",e),
                }
                
            }
            if let Some(name) = name { 
                match set_tp(&conn, &name) {
                    Ok(_)   => println!("{name} successfully added"),
                    Err(e)  => println!("{e}"),
                }
            } else {
                let list = get_tps(&conn)?;
                let mut table = Table::new();
                table.set_header(["ID","NAME"])
                    .add_rows(list);
                println!("{table}");
            }
        }
    }
    Ok(())
}
