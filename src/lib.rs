use rusqlite::Connection;
use rusqlite::Result;
use comfy_table::Row;
use std::io;
use rusqlite::Error;

#[derive(Debug)]
pub struct Tp {
    id: u32,
    name: String,
}

impl Tp {
    pub fn new(id: u32, name: String) -> Tp {
        Tp {
            id,
            name
        }
    }
    pub fn get_name(&self) -> String {
        self.name.clone()
    }
    pub fn get_id(&self) -> u32 {
        self.id.clone()
    }
    
}

impl Into<Row> for Tp {
    fn into(self) -> Row {
        Row::from([self.id.to_string(), self.name])
    }
}

pub fn get_tps(conn: &Connection) -> Result<Vec<Tp>> {
    let mut stmt = conn.prepare("SELECT ID, name FROM tp_list")?;
    stmt.query_map([], |row|{
        Ok(Tp {
            id: row.get(0)?,
            name: row.get(1)?
        })
    }).map(|i| i.filter_map(Result::ok).collect::<Vec<Tp>>())
}

pub fn get_tp(conn: &Connection,id: u32) -> Result<Option<Tp>> {
    conn.query_row("SELECT ID, name FROM tp_list WHERE ID=?",[id],|row| {
        Ok(Some(Tp {
            id: row.get(0)?,
            name: row.get(1)?
        }))
    }).or_else(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => Ok(None),
        _   => Err(e),
    })
}

pub fn set_tp(conn: &Connection, name: &String) -> Result<()> {
    conn.execute("INSERT INTO tp_list (name,sheet_id,admin_id) VALUES (?,'','');", [name])?;
    Ok(())
}

pub fn remove_tp(conn: &Connection, id: u32) -> Result<()> {
    conn.execute("DELETE FROM tp_list WHERE ID = ?;", [id])?;
    Ok(())
}

#[derive(Debug,Clone)]
pub struct Bi {
    id:     u32,
    first:  Student,
    second: Student
}

impl Into<Row> for Bi {
    fn into(self) -> Row {
        let mut row = Row::new();
        row.add_cell(self.id.into())
            .add_cell(format!("{}\n--\n{}",self.first.id,self.second.id).into())
            .add_cell(format!("{}\n--\n{}",self.first.name,self.second.name).into())
            .add_cell(format!("{}\n--\n{}",self.first.masar,self.second.masar).into())
            .add_cell(format!("{}\n--\n{}",self.first.ed,self.second.ed).into());
        row
    }
}

impl Bi {
    pub fn new(id: u32,first: Student, second: Student) -> Bi {
        Bi { 
            id,
            first,
            second
        }
    }
    pub fn get_id(&self) -> u32 {
        self.id
    }
    pub fn get_f_student_name(&self) -> String {
        self.first.name.clone()
    }
    pub fn get_s_student_name(&self) -> String {
        self.second.name.clone()
    }
    pub fn get_f_student_masar(&self) -> String {
        self.first.masar.clone()
    }
    pub fn get_s_student_masar(&self) -> String {
        self.second.masar.clone()
    }
    pub fn get_f_student_ed(&self) -> String {
        self.first.ed.clone()
    }
    pub fn get_s_student_ed(&self) -> String {
        self.second.ed.clone()
    }
}

#[derive(Debug,Clone)]
pub struct Student {
    id: u32,
    name: String,
    masar: String,
    ed: String
}

impl Student {
    pub fn new(id: u32,name: String,masar: String, ed: String) -> Self {
        Student { 
            id,
            name,
            masar,
            ed 
        }
    }
}

pub fn remove_bi(conn: &Connection, id: u32) -> Result<bool> {
    match conn.execute("DELETE FROM bi_list WHERE ID = ?;",[id]) {
        Ok(r) => Ok(if r == 0 { false } else { true }),
        Err(e) => Err(e),
    }
}

pub fn get_bis(conn: &Connection, id: u32) -> Result<Option<Vec<Bi>>> {
    let mut stmt = conn.prepare("SELECT ID FROM bi_list WHERE TP_ID = ?")?;
    let r = stmt.query_map([id], |row| {
        let id : Result<u32> = row.get(0);
        id
    })?.filter_map(Result::ok)
       .filter_map(|id| {
        let mut stmt = conn.prepare(" SELECT ID, name, masar, ed  FROM st_list WHERE bi_id = ?").ok()?;
        let bi = stmt.query([id]).ok()?
            .mapped(|row| Ok(Student::new(row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)))
            .filter_map(Result::ok)
            .collect::<Vec<Student>>();
        Some(Bi::new(id, bi[0].clone(), bi[1].clone()))
    }).collect::<Vec<Bi>>();
    if r.is_empty() {
        return Ok(None);
    }
    Ok(Some(r))
}

pub fn set_bi(conn: &Connection, id: u32, f_id: &String,s_id: &String) -> Result<String> {
    let (f_name,f_masar) = {
        match conn.query_row("SELECT name FROM bi_list INNER JOIN st_list \
                           ON st_list.bi_id = bi_list.ID and st_list.ed = ?1 \
                           WHERE TP_ID = ?2",
                           [f_id,&id.to_string()],
                           |row| row.get::<usize,String>(0)) {
            Ok(n)   => return Ok(n),
            Err(e)  => match e {
                Error::QueryReturnedNoRows => (),
                _ => return Err(e),
            },
        }
        let mut name = String::new();
        eprint!("insert the name of ({f_id})>");
        io::stdin().read_line(&mut name).expect("not expected");
        let mut masar = String::new();
        eprint!("insert the masar code of ({f_id})>");
        io::stdin().read_line(&mut masar).expect("not expected");
        (name.trim().to_string(),masar.trim().to_string())
    };
    let (s_name,s_masar) = {
        match conn.query_row("SELECT name FROM bi_list INNER JOIN st_list \
                           ON st_list.bi_id = bi_list.ID and st_list.ed = ?1 \
                           WHERE TP_ID = ?2",
                           [s_id,&id.to_string()],
                           |row| row.get::<usize,String>(0)) {
            Ok(n)   => return Ok(n),
            Err(e)  => match e {
                Error::QueryReturnedNoRows => (),
                _ => return Err(e),
            },
        }
        let mut name = String::new();
        eprint!("insert the name of ({s_id})>");
        io::stdin().read_line(&mut name).expect("not expected");
        let mut masar = String::new();
        eprint!("insert the masar code of ({s_id})>");
        io::stdin().read_line(&mut masar).expect("not expected");
        (name.trim().to_string(),masar.trim().to_string())
    };
    conn.execute("INSERT INTO bi_list (TP_ID) VALUES (?1);",[id])
        .and_then(|_| {
            let id = conn.last_insert_rowid();
            match conn.execute("INSERT INTO st_list (name,masar,ed,bi_id) VALUES (?1,?2,?3,?4),(?5,?6,?7,?8);",
                         [&f_name,&f_masar,f_id,&id.to_string(),&s_name,&s_masar,s_id,&id.to_string()]) {
                Ok(_)   => Ok(String::new()),
                Err(e)  => Err(e),
            }
        })
}
