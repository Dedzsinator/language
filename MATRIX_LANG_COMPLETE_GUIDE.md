## 🎯 **MATRIX LANGUAGE - COMPLETE GUIDE & REFERENCE**

### **✅ Quick Start - Write Your First Program**

1. **Create a file** `hello.matrix`:
```matrix
-- My first Matrix Language program
let greeting = "Hello, Matrix Language!"
let number = 42
let result = number * 2
```

2. **Compile and run**:
```bash
cd /path/to/matrix-lang
cargo run hello.matrix
```

### **📝 Verified Working Syntax**

#### **Core Language Rules:**
- ✅ **Comments**: Use `--` (not `//`)
- ✅ **No semicolons** required at end of statements
- ✅ **Program structure**: Must end with a `let` statement
- ✅ **Type annotations**: Optional but recommended for functions

#### **✅ Data Types**
```matrix
let integer = 42                    -- Int
let decimal = 3.14159              -- Float
let text = "Hello World"           -- String
let flag = true                    -- Bool
let numbers = [1, 2, 3, 4, 5]      -- Array
let matrix = [[1, 2], [3, 4]]      -- Matrix
```

#### **✅ Functions**
```matrix
-- Simple functions
let add = (a: Int, b: Int) => a + b
let square = (x: Float) => x * x

-- Functions with blocks
let complex_calc = (x: Float, y: Float) => {
    let dx = x * 2.0
    let dy = y * 3.0
    dx + dy
}

-- Higher-order functions
let apply_twice = (f: (Int) -> Int, x: Int) => f(f(x))
let increment = (x: Int) => x + 1
let result = apply_twice(increment, 10)  -- Returns 12
```

#### **✅ Structures**
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

let player = GameObject {
    name: "Player",
    position: Point { x: 10.0, y: 5.0 },
    velocity: Point { x: 1.0, y: 0.0 },
    mass: 1.0
}
```

#### **✅ Arrays and Matrices**
```matrix
-- Arrays
let numbers = [1, 2, 3, 4, 5]
let floats = [1.0, 2.5, 3.14]

-- 2D Matrices
let matrix2x2 = [[1, 2], [3, 4]]
let identity = [[1.0, 0.0], [0.0, 1.0]]

-- 3D Matrices
let matrix3x3 = [
    [1, 2, 3],
    [4, 5, 6],
    [7, 8, 9]
]

-- 4x4 Transformation matrix
let transform = [
    [1.0, 0.0, 0.0, 10.0],
    [0.0, 1.0, 0.0, 5.0],
    [0.0, 0.0, 1.0, 0.0],
    [0.0, 0.0, 0.0, 1.0]
]
```

### **🔧 Compilation Commands**

#### **Build the Compiler**
```bash
cd matrix-lang/
cargo build --release
```

#### **Execute Programs**
```bash
# Run a .matrix file
cargo run program.matrix

# Using built binary
./target/release/matrix-lang program.matrix

# Parse-only (syntax check)
cargo run -- --parse-only program.matrix

# Interactive REPL
cargo run -- --repl
```

#### **REPL Usage**
```bash
cargo run
>> let x = 10
10
>> let y = 20
20
>> let sum = x + y
30
>> exit
Goodbye!
```

### **📚 Complete Working Examples**

#### **Example 1: Basic Math**
**File: `basic_math.matrix`**
```matrix
-- Basic mathematical operations
let a = 15
let b = 4
let sum = a + b
let product = a * b
let square = (x: Int) => x * x
let result = square(sum)
```
**Run**: `cargo run basic_math.matrix`

#### **Example 2: Game Physics**
**File: `physics.matrix`**
```matrix
-- Game object with physics properties
struct Vector3 {
    x: Float,
    y: Float,
    z: Float
}

struct GameObject {
    name: String,
    position: Vector3,
    velocity: Vector3,
    mass: Float
}

let create_vector = (x: Float, y: Float, z: Float) => Vector3 { x: x, y: y, z: z }

let player = GameObject {
    name: "Player",
    position: create_vector(0.0, 0.0, 0.0),
    velocity: create_vector(5.0, 0.0, 0.0),
    mass: 1.0
}

let final_result = player
```

#### **Example 3: Matrix Operations**
**File: `matrices.matrix`**
```matrix
-- Matrix operations for graphics
let identity4x4 = [
    [1.0, 0.0, 0.0, 0.0],
    [0.0, 1.0, 0.0, 0.0],
    [0.0, 0.0, 1.0, 0.0],
    [0.0, 0.0, 0.0, 1.0]
]

let translation = [
    [1.0, 0.0, 0.0, 10.0],
    [0.0, 1.0, 0.0, 5.0],
    [0.0, 0.0, 1.0, 0.0],
    [0.0, 0.0, 0.0, 1.0]
]

let final_matrix = translation
```

### **🎯 Key Features**

✅ **Static typing** with type inference
✅ **Functional programming** paradigms
✅ **Native matrix operations** for graphics/physics
✅ **Interactive REPL** for development
✅ **Detailed error messages** with location info
✅ **Parse-only mode** for syntax validation
✅ **GPU acceleration** support (when available)

### **🐛 Common Issues & Solutions**

| Issue | Solution |
|-------|----------|
| `Parse error: expected struct, typeclass, instance, let` | End program with a `let` statement, not bare expression |
| `Unexpected token: found /` | Use `--` for comments, not `//` |
| `expected RightBracket, found Semicolon` | Remove semicolons from statements |
| `Type error: Unknown identifier` | Check variable/function names and scoping |

### **⚡ Development Workflow**

1. **Write** your `.matrix` file
2. **Check syntax**: `cargo run -- --parse-only file.matrix`
3. **Test in REPL**: `cargo run -- --repl`
4. **Execute**: `cargo run file.matrix`
5. **Debug**: Add intermediate `let` statements to inspect values

### **🚀 Next Steps**

- Explore the `tests/debug_scripts/` directory for more examples
- Try physics simulation features (if available)
- Experiment with the GUI mode: `cargo run -- --gui`
- Build complex matrix operations for graphics
- Integrate with GPU acceleration features

**Matrix Language** is ready for physics simulations, game development, and mathematical computing! 🎮🔬✨
