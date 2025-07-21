# libvex-rs: Building and Running on macOS (Sonoma and Later)

This guide explains how to build and run the Rust libvex bindings and examples on macOS Sonoma (or later), including all necessary environment variables, patches, and troubleshooting tips.

---

## 1. Prerequisites

- **Homebrew** (for installing dependencies)
- **Rust toolchain** (`rustup`, `cargo`)
- **Automake** (for `aclocal`)
- **Patched Valgrind/VEX** for Sonoma: [LouisBrunner/valgrind-macos](https://github.com/LouisBrunner/valgrind-macos)

Install dependencies:
```sh
brew install automake
cargo install bindgen-cli
```

---

## 2. Clone the Patched Valgrind Source

```sh
git clone https://github.com/LouisBrunner/valgrind-macos.git ~/Projects/valgrind-macos
```

---

## 3. Set Environment Variables

Set these variables in your shell **before building**:

```sh
export VEX_SRC=$HOME/Projects/valgrind-macos
export VEX_HEADERS=$HOME/Projects/valgrind-macos/VEX:$HOME/Projects/valgrind-macos
export VEX_LIBS=$HOME/Projects/valgrind-macos/VEX
```

You can add these to your shell profile for convenience.

---

## 4. Build the Rust Bindings

From the root of your `libvex-rs` workspace:

```sh
cargo clean
cargo build -p vex-sys
```

---

## 5. Run the Example

```sh
cargo run --example irsb
```

You should see IR Super Block (IRSB) output if everything is working.

### Example Output

```
IRSB {
   t0:I64   t1:I64   t2:I64   t3:I64   t4:I64   t5:I64   t6:I64   t7:I64
   t8:I64   t9:I64   t10:I64   t11:I64   t12:I64   t13:I64   t14:I64   t15:I64
   t16:I64   t17:I64   t18:I64   t19:I64   t20:I64   t21:I64   t22:I64   t23:I64
   t24:I64   t25:I64   t26:I32   t27:I64   t28:I64   t29:I64   t30:I32   t31:I64
   t32:I64   t33:I64   t34:I32   t35:I64   t36:I64   t37:I64   t38:I64   t39:I64
   t40:I64   

   ------ IMark(0xCAAA5C0, 1, 0) ------
   t0 = GET:I64(56)
   t15 = GET:I64(48)
   t14 = Sub64(t15,0x8:I64)
   PUT(48) = t14
   STle(t14) = t0
   ------ IMark(0xCAAA5C1, 3, 0) ------
   PUT(56) = t14
   ------ IMark(0xCAAA5C4, 7, 0) ------
   t2 = Sub64(t14,0x3D0:I64)
   PUT(144) = 0x8:I64
   PUT(152) = t14
   PUT(160) = 0x3D0:I64
   PUT(168) = 0x0:I64
   PUT(48) = t2
   PUT(184) = 0x10CAAA5CB:I64
   ------ IMark(0xCAAA5CB, 10, 0) ------
   t17 = Add64(t14,0xFFFFFFFFFFFFFE74:I64)
   STle(t17) = 0x402:I32
   PUT(184) = 0x10CAAA5D5:I64
   ------ IMark(0xCAAA5D5, 10, 0) ------
   t19 = Add64(t14,0xFFFFFFFFFFFFFE78:I64)
   STle(t19) = 0x402:I32
   PUT(184) = 0x10CAAA5DF:I64
   ------ IMark(0xCAAA5DF, 10, 0) ------
   t21 = Add64(t14,0xFFFFFFFFFFFFFE7C:I64)
   STle(t21) = 0x601:I32
   PUT(184) = 0x10CAAA5E9:I64
   ------ IMark(0xCAAA5E9, 6, 0) ------
   t23 = Add64(t14,0xFFFFFFFFFFFFFE74:I64)
   t26 = LDle:I32(t23)
   t25 = 32Uto64(t26)
   PUT(64) = t25
   PUT(184) = 0x10CAAA5EF:I64
   ------ IMark(0xCAAA5EF, 6, 0) ------
   t27 = Add64(t14,0xFFFFFFFFFFFFFE78:I64)
   t30 = LDle:I32(t27)
   t29 = 32Uto64(t30)
   PUT(32) = t29
   PUT(184) = 0x10CAAA5F5:I64
   ------ IMark(0xCAAA5F5, 6, 0) ------
   t31 = Add64(t14,0xFFFFFFFFFFFFFE7C:I64)
   t34 = LDle:I32(t31)
   t33 = 32Uto64(t34)
   PUT(24) = t33
   ------ IMark(0xCAAA5FB, 7, 0) ------
   t35 = Add64(t14,0xFFFFFFFFFFFFFD38:I64)
   PUT(72) = t35
   PUT(184) = 0x10CAAA602:I64
   ------ IMark(0xCAAA602, 5, 0) ------
   t37 = Sub64(t2,0x8:I64)
   PUT(48) = t37
   STle(t37) = 0x10CAAA607:I64
   t39 = Sub64(t37,0x80:I64)
   ====== AbiHint(t39, 128, 0x10CAADDE0:I64) ======
   PUT(184) = 0x10CAADDE0:I64; exit-Call
}

IRSB {
   t0:I32   t1:I32   t2:I32   t3:I32   

   ------ IMark(0xF16B11B2, 6, 0) ------
   t0 = LDle:I32(0xF00BABA:I32)
   STle(0xF00ABBA:I32) = Sub32(Add32(t0,t0),0x20:I32)
   t1 = GET:I32(48)
   IR-NoOp
   ====== AbiHint(t2, 128, t3) ======
   PUT(184) = t1; exit-Call
}
```

---

## 6. Patch Applied

- **Fixed non-exhaustive match in `src/ir.rs`:**
  - Added a wildcard arm to the `match` on `IRConstTag` to handle new/unknown variants:
    ```rust
    _ => todo!("Unimplemented IRConstTag variant: {:?}", co.tag),
    ```

---

## 7. Troubleshooting

- **Header not found errors:**
  - Ensure `VEX_HEADERS` includes both the `VEX` directory and the Valgrind root, separated by a colon (`:`).
- **Static library not found:**
  - Confirm `libvex-amd64-darwin.a` exists in `$VEX_LIBS`.
- **Bindgen errors:**
  - Make sure `bindgen` is installed and the include paths are correct.
- **macOS version errors:**
  - Use the patched Valgrind from [LouisBrunner/valgrind-macos](https://github.com/LouisBrunner/valgrind-macos).

---

## 8. Notes

- Warnings about unused fields or dead code are safe to ignore for basic usage.
- For further development, update match arms as new VEX enum variants are added.

---

**Maintainer:** Update this file as the build process or dependencies change. 