use std::io::{self, stdin, stdout, Write};

use std::collections::HashSet;
use interns::Interner;

pub fn main() -> io::Result<()> {
    let mut interner: Interner<str> = Interner::new();

    let mut words = HashSet::new();
    let mut line = String::new();
    loop {
        line.clear();
        print!("> ");
        stdout().flush()?;

        stdin().read_line(&mut line)?;
        let linet = line.trim();
        if linet.is_empty() { break }
        let sym = interner.get_or_intern(linet);
        if words.contains(&sym) {
            println!("String '{linet}' already interned as {sym:?}");
        } else {
            words.insert(sym);
            println!("'{linet}' = {sym:?}");
        }
    }

    println!("\n== Interned symbols ==");
    let mut syms = words.iter().collect::<Vec<_>>();
    syms.sort();
    for sym in syms {
        let s = interner.resolve(*sym).unwrap();
        println!("{sym:?} = '{s}'");
    }

    Ok(())
}
