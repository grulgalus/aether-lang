# Aether Programming Language 🌌

A modern, fast, and memory-safe programming language with fearless concurrency.

## 🚀 Vision
Aether combines the readability of Python with the performance and safety of Rust. It compiles to native code via LLVM and features a built-in Actor model for effortless multithreading without the overhead of a Garbage Collector.

## ✨ Key Features
- **No Garbage Collector:** Uses compile-time optimized Automatic Reference Counting (ARC).
- **Actor Model Concurrency:** Safe and intuitive multithreading using isolated actors and message passing.
- **LLVM Backend:** Compiles to x86_64, ARM64, WebAssembly (Wasm), and RISC-V.
- **GitHub First:** Built-in package manager `orb` fetches dependencies directly from GitHub repositories.

## 💻 Syntax Example
```aether
actor Worker {
    fn process_data(data: String) -> Result<String, Error> {
        return Ok(f"Processed: {data}")
    }
}

fn main() {
    let my_worker = Worker.spawn()
    let result = my_worker ! process_data("Hello from Aether")
}
🛠️ Building
To build the compiler, you will need Rust and Cargo installed.

cargo build --release
