# Matrix Language Programming Guide

Matrix Language is a functional, matrix-oriented scripting language designed for physics simulation and GPU acceleration.

## Table of Contents
1. [Installation & Setup](#installation--setup)
2. [Basic Syntax](#basic-syntax)
3. [Data Types](#data-types)
4. [Functions](#functions)
5. [Control Flow](#control-flow)
6. [Structs](#structs)
7. [Arrays and Matrices](#arrays-and-matrices)
8. [Compilation & Execution](#compilation--execution)
9. [Examples](#examples)

## Installation & Setup

### Prerequisites
- Rust (latest stable version)
- Cargo (comes with Rust)

### Building the Compiler
```bash
cd /path/to/matrix-lang
cargo build --release
```

The compiled binary will be available at `target/release/matrix-lang`.

## Basic Syntax

### Comments
```matrix
// Single line comment
-- Alternative single line comment
```

### Variables
```matrix
let x = 42;                    // Integer
let y = 3.14;                  // Float
let name = "Hello World";      // String
let flag = true;               // Boolean
```

### Type Annotations (Optional)
```matrix
let x: Int = 42;
let y: Float = 3.14;
let name: String = "Hello";
let flag: Bool = true;
```

## Data Types

Matrix Language supports the following basic types:

- `Int` - 64-bit signed integers
- `Float` - 64-bit floating point numbers
- `Bool` - Boolean values (true/false)
- `String` - UTF-8 strings
- `Array<T>` - Dynamic arrays
- `Matrix<T>` - 2D matrices
- User-defined structs

## Functions

### Function Definition
```matrix
// Simple function
let add = (a: Int, b: Int) => a + b;

// Function with type annotations
let multiply: (Int, Int) -> Int = (x: Int, y: Int) => x * y;

// Function with block body
let factorial = (n: Int) => {
    if n <= 1 {
        1
    } else {
        n * factorial(n - 1)
    }
};
```

### Function Calls
```matrix
let result = add(5, 3);          // Returns 8
let product = multiply(4, 6);    // Returns 24
let fact5 = factorial(5);        // Returns 120
```

### Higher-Order Functions
```matrix
let apply_twice = (f: (Int) -> Int, x: Int) => f(f(x));
let increment = (x: Int) => x + 1;
let result = apply_twice(increment, 5);  // Returns 7
```

## Control Flow

### Conditional Expressions
```matrix
// If-else expression
let result = if x > 0 { "positive" } else { "non-positive" };

// Nested conditionals
let sign = if x > 0 {
    "positive"
} else if x < 0 {
    "negative"
} else {
    "zero"
};
```

### Let-In Expressions
```matrix
// Single binding
let x = 10 in x * 2;

// Multiple bindings
let x = 5 in
let y = 10 in
x + y;
```

## Structs

### Struct Definition
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
```

### Struct Creation
```matrix
let origin = Point { x: 0.0, y: 0.0 };

let player = GameObject {
    name: "Player",
    position: Point { x: 10.0, y: 5.0 },
    velocity: Point { x: 0.0, y: 0.0 },
    mass: 1.0
};
```

### Field Access
```matrix
let x_pos = player.position.x;
let player_name = player.name;
```

## Arrays and Matrices

### Array Creation
```matrix
let numbers = [1, 2, 3, 4, 5];
let mixed = [1, 2.5, 3];  // Type inferred as Array<Float>
let empty: Array<Int> = [];
```

### Matrix Creation
```matrix
let identity = [[1, 0], [0, 1]];
let matrix3x3 = [
    [1, 2, 3],
    [4, 5, 6],
    [7, 8, 9]
];
```

### Array/Matrix Operations
```matrix
let first = numbers[0];        // Index access
let length = len(numbers);     // Length function
```

## Compilation & Execution

### Command Line Usage

#### Execute a Matrix Language file:
```bash
matrix-lang script.matrix
```

#### Parse only (syntax check):
```bash
matrix-lang --parse-only script.matrix
```

#### Interactive REPL:
```bash
matrix-lang --repl
# or simply
matrix-lang
```

#### GUI Mode (if available):
```bash
matrix-lang --gui
```

### REPL Commands
- `help` - Show help message
- `exit` or `quit` - Exit REPL
- `clear` - Clear screen

## Examples

### Example 1: Basic Arithmetic
Create a file `arithmetic.matrix`:
```matrix
// Basic arithmetic operations
let a = 10;
let b = 5;

let sum = a + b;
let difference = a - b;
let product = a * b;
let quotient = a / b;

print("Sum: " + sum);
print("Difference: " + difference);
print("Product: " + product);
print("Quotient: " + quotient);
```

Run it:
```bash
matrix-lang arithmetic.matrix
```

### Example 2: Functions and Recursion
Create a file `functions.matrix`:
```matrix
// Function definitions
let square = (x: Int) => x * x;
let cube = (x: Int) => x * x * x;

// Recursive factorial
let factorial = (n: Int) => {
    if n <= 1 {
        1
    } else {
        n * factorial(n - 1)
    }
};

let result1 = square(5);      // 25
let result2 = cube(3);        // 27
let result3 = factorial(5);   // 120

print(result1);
print(result2);
print(result3);
```

### Example 3: Structs and Objects
Create a file `structs.matrix`:
```matrix
// Define a 2D point struct
struct Vector2 {
    x: Float,
    y: Float
}

// Define a game object
struct GameObject {
    name: String,
    position: Vector2,
    velocity: Vector2
}

// Create instances
let origin = Vector2 { x: 0.0, y: 0.0 };
let player_pos = Vector2 { x: 10.0, y: 5.0 };
let player_vel = Vector2 { x: 1.0, y: 0.0 };

let player = GameObject {
    name: "Player",
    position: player_pos,
    velocity: player_vel
};

print("Player at: (" + player.position.x + ", " + player.position.y + ")");
```

### Example 4: Matrix Operations
Create a file `matrices.matrix`:
```matrix
// Create matrices
let matrix_a = [[1, 2], [3, 4]];
let matrix_b = [[5, 6], [7, 8]];

// Identity matrix
let identity = [[1, 0], [0, 1]];

print("Matrix A:");
print(matrix_a);
print("Matrix B:");
print(matrix_b);
print("Identity:");
print(identity);
```

### Example 5: Physics Simulation (if physics functions are available)
Create a file `physics.matrix`:
```matrix
// Simple physics simulation
let world = create_physics_world();

// Add objects to the world
let sphere = add_rigid_body(world, "sphere", 1.0, 2.0, [0.0, 5.0, 0.0]);
let ground = add_rigid_body(world, "box", 0.0, 100.0, [0.0, -10.0, 0.0]);

// Set gravity
set_gravity(world, [0.0, -9.81, 0.0]);

// Simulate physics step
physics_step(world);

print("Physics simulation step completed!");
```

## Standard Library Functions

Matrix Language includes several built-in functions:

- `print(value)` - Print a value to console
- `abs(number)` - Absolute value
- `sqrt(number)` - Square root
- `len(array)` - Length of array
- Physics functions (if available):
  - `create_physics_world()`
  - `add_rigid_body(world, shape, mass, size, position)`
  - `set_gravity(world, gravity_vector)`
  - `physics_step(world)`

## Tips and Best Practices

1. **Use type annotations** for clarity, especially in function parameters
2. **Prefer expressions over statements** when possible
3. **Use `let...in` expressions** for local bindings within expressions
4. **Structure your code with functions** for reusability
5. **Use descriptive variable names** for better readability
6. **Test your code** with the REPL before writing complete programs

## Error Handling

The Matrix Language compiler provides helpful error messages:

- **Lexical errors**: Issues with tokens (invalid characters, etc.)
- **Parse errors**: Syntax errors in your code
- **Type errors**: Type mismatches and incompatibilities
- **Runtime errors**: Division by zero, undefined variables, etc.

Use `--parse-only` flag to check syntax without execution.

## Development Workflow

1. Write your Matrix Language code in a `.matrix` file
2. Test syntax with: `matrix-lang --parse-only yourfile.matrix`
3. Execute with: `matrix-lang yourfile.matrix`
4. Use the REPL for quick testing: `matrix-lang --repl`
5. Debug by adding `print()` statements

Happy coding with Matrix Language! 🚀
