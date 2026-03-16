fn main() {
    println!("grove-ts-gen — TypeScript types are generated via ts-rs derives.");
    println!();
    println!("Run: pnpm generate:types");
    println!();
    println!("This runs `cargo test -p grove-domain --features ts` to export");
    println!("bindings, copies them to src/generated/, and creates a barrel file.");
    println!("Generation also runs automatically before `pnpm check` and `pnpm test`.");
}
