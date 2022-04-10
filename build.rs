use static_files::resource_dir;

fn main() -> std::io::Result<()> {
    resource_dir("./static").build()?;
    // trigger recompilation when a new migration is added
    println!("cargo:rerun-if-changed=migrations");
    Ok(())
}
