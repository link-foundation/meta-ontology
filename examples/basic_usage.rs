//! Load the meta-ontology and walk a few cycles.
//!
//! Run with: `cargo run --example basic_usage`

use meta_ontology::loader::load_default;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ontology = load_default()?;
    println!("loaded {} concepts", ontology.len());

    let thing = ontology.find("thing").ok_or("seed data missing `thing`")?;
    println!("\n`thing` definitions:");
    for def in &thing.definitions {
        println!("  - {}", def.words.join(" "));
    }

    println!("\nwalking thing -> concept -> thing (cycle):");
    let mut path = Vec::new();
    let mut current = "thing".to_string();
    let target = "thing".to_string();
    path.push(current.clone());
    for step in 0..4 {
        let next = ontology
            .neighbors(&current)
            .find(|c| {
                if step == 0 {
                    c.name != current
                } else {
                    c.name == target
                }
            })
            .map(|c| c.name.clone());
        match next {
            Some(n) => {
                path.push(n.clone());
                current = n;
                if step > 0 && current == target {
                    break;
                }
            }
            None => break,
        }
    }
    println!("  {}", path.join(" -> "));

    println!("\nlanguages with declared exponents:");
    for code in ontology.languages() {
        println!("  - {code}");
    }

    Ok(())
}
