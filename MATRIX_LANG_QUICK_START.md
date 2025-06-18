# How to Write and Compile Matrix Language Code

## Quick Start Guide

Matrix Language is a functional programming language designed for physics simulation and GPU acceleration. Here's everything you need to know to get started.

## 🚀 Getting Started

### 1. Build the Compiler
```bash
cd /path/to/matrix-lang
cargo build --release
```

### 2. Your First Matrix Language Program

Create a file called `hello.matrix`:
```matrix
-- Comments start with double dashes
let message = "Hello, Matrix Language!"
let number = 42
let result = number * 2
```

### 3. Compile and Run
```bash
# Run the program
./target/release/matrix-lang hello.matrix

# Or using cargo:
cargo run hello.matrix

# Parse only (syntax check):
cargo run -- --parse-only hello.matrix
```

## 📝 Matrix Language Syntax

### Key Syntax Rules:
- **No semicolons required** at the end of statements
- **Comments** use `--` (not `//`)
- **Variables** declared with `let`
- **Functions** use `=>` arrow syntax
- **Type annotations** are optional but recommended

### Basic Examples:

#### Variables and Basic Operations
```matrix
-- Variables
let x = 10
let y = 20
let sum = x + y
let name = "Matrix Lang"
let flag = true
```

#### Functions
```matrix
-- Simple function
let add = (a: Int, b: Int) => a + b
let result = add(5, 3)

-- Function with complex logic
let factorial = (n: Int) => {
    if n <= 1 { 1 } else { n * factorial(n - 1) }
}
let fact5 = factorial(5)
```

#### Higher-Order Functions
```matrix
let apply_twice = (f: (Int) -> Int, x: Int) => f(f(x))
let increment = (x: Int) => x + 1
let result = apply_twice(increment, 10)  -- Returns 12
```

#### Structs
```matrix
struct Point {
    x: Float,
    y: Float
}

struct GameObject {
    name: String,
    position: Point,
    velocity: Point,
    mass: Float
}

let origin = Point { x: 0.0, y: 0.0 }
let player = GameObject {
    name: "Player",
    position: Point { x: 10.0, y: 5.0 },
    velocity: Point { x: 1.0, y: 0.0 },
    mass: 1.0
}
```

#### Arrays and Matrices
```matrix
-- Arrays
let numbers = [1, 2, 3, 4, 5]
let mixed = [1, 2.5, 3]

-- Matrices
let matrix2x2 = [[1, 2], [3, 4]]
let matrix3x3 = [
    [1, 2, 3],
    [4, 5, 6],
    [7, 8, 9]
]
```

#### Conditional Expressions
```matrix
let x = 10
let sign = if x > 0 { "positive" } else { "non-positive" }

-- Nested conditions
let category = if x > 0 {
    "positive"
} else if x < 0 {
    "negative"
} else {
    "zero"
}
```

## 🔧 Compiler Usage

### Command Line Options

```bash
# Execute a Matrix Language file
matrix-lang script.matrix

# Parse only (syntax check)
matrix-lang --parse-only script.matrix

# Interactive REPL
matrix-lang --repl
# or simply:
matrix-lang

# GUI Mode (if available)
matrix-lang --gui
```

### Development Workflow

1. **Write** your Matrix Language code in a `.matrix` file
2. **Check syntax** with: `matrix-lang --parse-only yourfile.matrix`
3. **Execute** with: `matrix-lang yourfile.matrix`
4. **Debug** using the REPL: `matrix-lang --repl`

## 📚 Complete Examples

### Example 1: Basic Math Operations
Create `math.matrix`:
```matrix
-- Basic arithmetic
let a = 15
let b = 4

let sum = a + b
let difference = a - b
let product = a * b
let quotient = a / b

-- Functions for calculations
let square = (x: Int) => x * x
let cube = (x: Int) => x * x * x

let squared = square(5)
let cubed = cube(3)
```

Run it:
```bash
cargo run math.matrix
```

### Example 2: Working with Functions
Create `functions.matrix`:
```matrix
-- Function definitions
let add = (a: Int, b: Int) => a + b
let multiply = (x: Int, y: Int) => x * y

-- Recursive function
let fibonacci = (n: Int) => {
    if n <= 1 {
        n
    } else {
        fibonacci(n - 1) + fibonacci(n - 2)
    }
}

let result1 = add(10, 5)
let result2 = multiply(6, 7)
let fib10 = fibonacci(10)
```

### Example 3: Data Structures
Create `structs.matrix`:
```matrix
-- Define structures
struct Vector3 {
    x: Float,
    y: Float,
    z: Float
}

struct Particle {
    position: Vector3,
    velocity: Vector3,
    mass: Float
}

-- Create instances
let origin = Vector3 { x: 0.0, y: 0.0, z: 0.0 }
let velocity = Vector3 { x: 1.0, y: 0.5, z: 0.0 }

let particle = Particle {
    position: Vector3 { x: 10.0, y: 5.0, z: 2.0 },
    velocity: velocity,
    mass: 1.5
}
```

### Example 4: Matrix Operations
Create `matrices.matrix`:
```matrix
-- Create matrices
let identity2x2 = [[1, 0], [0, 1]]
let matrix_a = [[1, 2], [3, 4]]
let matrix_b = [[5, 6], [7, 8]]

-- 3D transformation matrix
let transform = [
    [1.0, 0.0, 0.0, 10.0],
    [0.0, 1.0, 0.0, 5.0],
    [0.0, 0.0, 1.0, 0.0],
    [0.0, 0.0, 0.0, 1.0]
]
```

## 🎯 Key Features

### Type System
- **Static typing** with type inference
- **Optional type annotations** for clarity
- **Strong type checking** at compile time

### Functional Programming
- **First-class functions** and higher-order functions
- **Immutable data** by default
- **Expression-oriented** language design

### Matrix Operations
- **Native matrix literals** with `[[1, 2], [3, 4]]` syntax
- **GPU acceleration** support (when available)
- **Physics simulation** integration

### Development Features
- **Interactive REPL** for testing
- **Parse-only mode** for syntax checking
- **Detailed error messages** with line/column information
- **AST inspection** for debugging

## 🐛 Debugging Tips

1. **Use the REPL** to test expressions quickly
2. **Check syntax first** with `--parse-only`
3. **Read error messages carefully** - they include line/column info
4. **Start simple** and build complexity gradually
5. **Use type annotations** to catch type errors early

## ⚡ Performance Tips

1. **Use appropriate data types** (Int vs Float)
2. **Leverage matrix operations** for numerical computing
3. **Consider GPU acceleration** for heavy computations
4. **Profile with the built-in tools** when available

## 🔄 REPL Commands

When using the interactive REPL:
- `help` - Show help message
- `exit` or `quit` - Exit REPL
- `clear` - Clear screen

## 📦 Standard Library

Matrix Language includes built-in functions:
- `print(value)` - Output to console
- `abs(number)` - Absolute value
- `sqrt(number)` - Square root
- Physics functions (when available)

## 🎮 Next Steps

1. **Explore the test files** in `tests/debug_scripts/` for more examples
2. **Try the physics simulation** features
3. **Experiment with GPU acceleration** (if available)
4. **Build your own matrix operations** and physics simulations

Happy coding with Matrix Language! 🚀✨
