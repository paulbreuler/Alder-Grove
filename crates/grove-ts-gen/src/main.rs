fn main() {
    println!("grove-ts-gen — TypeScript types are generated via ts-rs derives.");
    println!();
    println!("Run: pnpm generate:types");
    println!();
    println!("This runs `cargo test -p grove-domain --features ts` with");
    println!("TS_RS_EXPORT_DIR=src/generated, then generates a barrel file.");
}
