# Hive File Format

Every `.hive` file is a binary file containing HIVE bytecode and associated module metadata.

A `.hive` file consists of a header, a section table, and zero or more sections. Sections may appear in any physical order. Their locations are determined exclusively by the section table.

All structures, encodings, identifiers, and validation rules described in this specification are part of the `.hive` file format.

---

## 1. Integer Encoding

All multi-byte integer fields are encoded in little-endian byte order.

The following integer types are defined:

| Type  | Size |
| ----- | ---: |
| `u8` |  1 B |
| `u16` |  2 B |
| `u32` |  4 B |
| `u64` |  8 B |

The file format does not depend on the native pointer size of the host system.

---

## 2. Header

The file begins with a header.

The header contains the following fields:

| Field         | Size | Type          | Description                      |
| ------------- | ---: | ------------- | -------------------------------- |
| Magic         |  4 B | `char[4]` | File magic `"HIVE"` |
| Version       |  2 B | `u16` | File format version              |
| Type          |  2 B | `u16` | File type                        |
| Flags         |  4 B | `u32` | File flags                       |
| Size          |  8 B | `u64` | Total file size                  |
| Section Table |  8 B | `u64` | File offset of the section table |
| Section Count |  4 B | `u32` | Number of section entries        |
| Entry Point   |  4 B | `FUNCTION_ID` | Executable entry point           |
| Header Size   |  2 B | `u16` | Header size in bytes             |
| Reserved      |  2 B | `u16` | Reserved                         |

The `Header Size` field specifies the complete size of the header, including fields introduced by future format revisions.

A loader must use `Header Size` when locating the section table and must not assume that the header has a fixed size across all format versions.

---

### 2.1. Magic

The first four bytes of the file must contain:

```text
HIVE
```

A file with a different magic value is invalid.

---

### 2.2. Version

`Version` identifies the binary file format version.

The version applies to the complete `.hive` file format.

A loader must reject versions that it does not support.

---

### 2.3. File Types

`Type` identifies the kind of `.hive` file.

The following values are defined:

```text
0x0001  EXECUTABLE
0x0002  MODULE
0x0003  LIBRARY
0x0004  OBJECT
```

Unknown file types are invalid.

---

### 2.4. File Flags

`Flags` is a bit field.

The following flags are defined:

```text
EXECUTABLE
POSITION_INDEPENDENT
HAS_RELOCATIONS
HAS_DEBUG
HAS_JIT_INFO
```

Undefined flag bits must be ignored unless a future format revision assigns them a defined meaning.

---

## 3. File Layout

The file consists of:

```text
Header
Section Table
Section Data
```

The physical order of section data is not significant.

The section table is authoritative for locating sections.

Sections may appear before or after any other section, provided that all offsets and alignment requirements are valid.

---

## 4. Section Table

The section table contains one descriptor for every section.

Each section descriptor has a fixed size of 32 bytes.

| Field     | Size | Type  | Description                |
| --------- | ---: | ----- | -------------------------- |
| Type      |  4 B | `u32` | Section type               |
| Flags     |  4 B | `u32` | Section flags              |
| Offset    |  8 B | `u64` | File offset of the section |
| Size      |  8 B | `u64` | Section size in bytes      |
| Alignment |  4 B | `u32` | Required alignment         |
| Reserved  |  4 B | `u32` | Reserved                   |

The section table contains exactly `Section Count` entries.

Therefore:

```text
Section Table Size = Section Count × 32
```

The section table itself must be fully contained within the file.

---

### 4.1. Section Types

The following section types are defined:

```text
0x00000001  METADATA
0x00000002  STRINGS
0x00000003  SYMBOLS
0x00000004  TYPES
0x00000005  ATOMS
0x00000006  FUNCTIONS
0x00000008  CODE
0x00000009  DATA
0x0000000A  RELOCATIONS
0x0000000B  DEBUG
0x0000000C  JIT_INFO
0x0000000D  MODULES
```

Section types are globally defined by the file format.

A section type must not occur more than once unless explicitly permitted by a future format revision.

---

### 4.2. Section Flags

Section flags are represented as a bit field.

The following flags are defined:

```text
READ
WRITE
EXEC
ALLOC
OPTIONAL
```

The numeric values of the flags are defined by the HIVE file format.

---

### 4.3. Section Alignment

`Alignment` specifies the required alignment of the section's file offset.

If `Alignment` is non-zero, the following condition must hold:

```text
Offset % Alignment == 0
```

`Alignment` must be a power of two.

An alignment value of zero means that no additional alignment is required.

---

## 5. File Offsets

A file offset is an unsigned 64-bit integer:

```text
FILE_OFFSET = u64
```

A file offset identifies a byte position within the `.hive` file.

File offsets are not runtime addresses.

Every referenced file range must be fully contained within the file.

For a section:

```text
Offset + Size <= File Size
```

must hold.

All offset arithmetic must be validated without integer overflow.

---

## 6. Identifier Types

The following identifier types are defined:

| Identifier    | Type  | Description          |
| ------------- | ----- | -------------------- |
| `STRING_ID` | `u32` | String table index   |
| `SYMBOL_ID` | `u32` | Symbol table index   |
| `TYPE_ID` | `u32` | Type table index     |
| `ATOM_ID` | `u32` | Atom table index     |
| `FUNCTION_ID` | `u32` | Function table index |
| `WORKER_ID` | `u32` | Worker table index   |
| `MODULE_ID` | `u32` | Module table index   |

An identifier normally represents the zero-based index of an entry in its corresponding table.

Unless explicitly stated otherwise, identifiers are local to the `.hive` file.

A `SYMBOL_ID` used as a cross-module reference is interpreted in the context of the referenced module.

---

### 6.1. Invalid Identifier

The value:

```text
0xFFFFFFFF
```

is reserved as `INVALID_ID` for all 32-bit identifier types.

Unless explicitly permitted by a field definition, `INVALID_ID` must not be used as a valid table reference.

---

## 7. Metadata Section

The `METADATA` section contains module-level metadata.

| Field           | Size | Type  | Description                |
| --------------- | ---: | ----- | -------------------------- |
| Runtime Version |  2 B | `u16` | Required runtime version   |
| Flags           |  4 B | `u32` | Runtime capability flags   |
| Worker Count    |  4 B | `u32` | Number of worker entries   |
| Function Count  |  4 B | `u32` | Number of function entries |
| Type Count      |  4 B | `u32` | Number of type entries     |
| Atom Count      |  4 B | `u32` | Number of atom entries     |
| String Count    |  4 B | `u32` | Number of string entries   |

The count fields describe the number of entries in their corresponding sections.

If a corresponding optional section is absent, its count must be zero.

The counts must match the actual number of entries in the corresponding sections.

---

## 8. Tables

### 8.1. String Table

The `STRINGS` section contains immutable UTF-8 strings.

Each string is identified by a `STRING_ID` .

The section consists of a string entry table followed by string data.

Each string entry contains:

| Field  | Size | Type  | Description                      |
| ------ | ---: | ----- | -------------------------------- |
| Offset |  8 B | `u64` | Offset into the string data area |

`Offset` is relative to the beginning of the string data area within the `STRINGS` section.

Strings are required to be null terminated.

Every string must contain valid UTF-8 data.

---

### 8.2. Symbol Table

The `SYMBOLS` section contains symbol entries.

Each symbol entry contains:

| Field   | Size | Type        | Description    |
| ------- | ---: | ----------- | -------------- |
| Name    |  4 B | `STRING_ID` | Symbol name    |
| Value   |  8 B | `u64` | Symbol value   |
| Size    |  4 B | `u32` | Symbol size    |
| Type    |  2 B | `u16` | Symbol type    |
| Binding |  2 B | `u16` | Symbol binding |
| Flags   |  4 B | `u32` | Symbol flags   |

The interpretation of `Value` depends on the symbol type and binding.

A symbol referenced by a `SYMBOL_ID` is identified by its zero-based index within the `SYMBOLS` section of the relevant module.

---

#### 8.2.1. Symbol Types

The following symbol types are defined:

```text
0x0001 FUNCTION
0x0003 DATA
0x0004 ATOM
0x0005 TYPE
0x0006 EXTERNAL
```

If the symbol type is `EXTERNAL` the `Value` is a pointer to a struct with these fields:
`4B Module ID`
`4B Symbol ID in external module`

---

#### 8.2.2. Symbol Bindings

The following symbol bindings are defined:

```text
LOCAL
GLOBAL
WEAK
IMPORT
EXPORT
```

---

### 8.3. Type Table

The `TYPES` section contains type definitions.

A type is identified by a `TYPE_ID` , which is the zero-based index of the corresponding type entry.

Each type entry has a fixed size.

| Field         | Size | Type        | Description                  |
| ------------- | ---: | ----------- | ---------------------------- |
| Kind          |  2 B | `u16` | Type kind                    |
| Flags         |  2 B | `u16` | Type flags                   |
| Size          |  4 B | `u32` | Size of the type in bytes    |
| Alignment     |  4 B | `u32` | Required alignment           |
| Element Count |  4 B | `u32` | Number of elements           |
| Data Offset   |  8 B | `u64` | Offset to type-specific data |
| Name          |  4 B | `STRING_ID` | Optional type name           |

The `TYPE_ID` itself is not stored in the entry.

---

#### 8.3.1. Type Kinds

The following type kinds are defined:

```text
VOID
ANY
I8
I16
I32
I64
I128
U8
U16
U32
U64
U128
F32
F64
WORD
DWORD
QUAD
PTR
ARRAY
TUPLE
STRUCT
FUNCTION
```

---

#### 8.3.2. Primitive Types

Primitive types do not require type-specific data.

For primitive types:

```text
Data Offset = 0
```

---

#### 8.3.3. Pointer Type

`PTR` represents a pointer-sized runtime value.

The representation of the pointer value is defined by the runtime ABI.

A native process address must not be stored as a persistent value in a `.hive` file.

---

#### 8.3.4. Tuple Types

For `TUPLE` , `Element Count` specifies the number of tuple elements.

The type-specific data contains an array of:

```text
TYPE_ID
```

with exactly `Element Count` entries.

Tuple element ordering is significant.

---

#### 8.3.5. Struct Types

For `STRUCT` , `Element Count` specifies the number of fields.

The type-specific data contains an array of field descriptors.

Each field descriptor contains:

| Field  | Size | Type        | Description  |
| ------ | ---: | ----------- | ------------ |
| Name   |  4 B | `STRING_ID` | Field name   |
| Type   |  4 B | `TYPE_ID` | Field type   |
| Offset |  4 B | `u32` | Field offset |

Fields are stored in declaration order unless otherwise specified by a future format revision.

---

#### 8.3.6. Array Types

For `ARRAY` , `Element Count` specifies the number of array elements.

The type-specific data contains:

| Field        | Size | Type      | Description  |
| ------------ | ---: | --------- | ------------ |
| Element Type |  4 B | `TYPE_ID` | Element type |

The type's `Size` is the total size of the complete array.

---

#### 8.3.7. Function Types

For `FUNCTION` , `Element Count` is reserved and must be zero.

The type-specific data contains the function signature.

The signature consists of:

```text
Parameter Type ID
Return Type ID
```

Each parameter and return type is represented by a `TYPE_ID` .

---

#### 8.3.8. Type-Specific Data

`Data Offset` is relative to the beginning of the `TYPES` section.

The referenced data must be fully contained within the `TYPES` section.

Primitive types do not have type-specific data.

Composite types use type-specific data according to their type kind.

---

### 8.4. Atom Table

The `ATOMS` section contains interned symbolic values.

Each atom entry contains:

| Field | Size | Type        | Description |
| ----- | ---: | ----------- | ----------- |
| Name  |  4 B | `STRING_ID` | Atom name   |
| Flags |  4 B | `u32` | Atom flags  |

The zero-based entry index is the corresponding `ATOM_ID` .

Runtime-global atom identifiers are not stored as persistent file values.

---

### 8.5. Function Table

The `FUNCTIONS` section contains one entry for every function.

Each function entry contains:

| Field            | Size | Type        | Description                  |
| ---------------- | ---: | ----------- | ---------------------------- |
| Symbol           |  4 B | `SYMBOL_ID` | Function symbol              |
| Code Offset      |  8 B | `u64` | Offset within `CODE` |
| Code Size        |  4 B | `u32` | Function bytecode size       |
| Frame Size       |  4 B | `u32` | Required frame size          |
| Flags            |  4 B | `u32` | Function flags               |
| Parameter Type   |  4 B | `u32` | Type Id of params (tuple if multiple parameter)         |
| Return Type      |  4 B | `u32` | Type Id of return type            |

`Code Offset` is relative to the beginning of the `CODE` section.

`Parameter Offset` and `Return Offset` are relative to the beginning of the `FUNCTIONS` section.

The parameter area contains `Parameter Count` consecutive `TYPE_ID` values.

The return area contains `Return Count` consecutive `TYPE_ID` values.

If the `SYMBOLS` section is present, `Symbol` must reference a symbol of type `FUNCTION` .

---

#### 8.5.1. Function Flags

Function flags are a bit field.

The following flags are defined:

```text
ENTRY
EXPORTED
IMPORTED
VARIADIC
NO_RETURN
```
If `NO_RETURN` is set, the function is able to behave like a worker. So you are able to do a worker call

---

### 8.7. Relocation Table

The `RELOCATIONS` section contains relocation entries.

Each relocation entry contains:

| Field    | Size | Type        | Description                  |
| -------- | ---: | ----------- | ---------------------------- |
| Section  |  4 B | `u32` | Target section type          |
| Offset   |  8 B | `u64` | Offset within target section |
| Symbol   |  4 B | `SYMBOL_ID` | Referenced symbol            |
| Type     |  2 B | `u16` | Relocation type              |
| Reserved |  2 B | `u16` | Reserved                     |
| Addend   |  8 B | `u64` | Relocation addend            |

The relocation target is identified by:

```text
Section + Offset
```

`Offset` is relative to the beginning of the target section.

A relocation symbol refers to a symbol in the current module unless the relocation type explicitly defines an external module reference.

---

#### 8.7.1. Relocation Types

The following relocation types are defined:

```text
ABSOLUTE
RELATIVE
FUNCTION
DATA
TYPE
ATOM
```

The exact encoding of a relocation result is determined by the relocation type and the target field.

---

### 8.8. Module Table

The `MODULES` section contains references to external modules.

Each module entry contains:

| Field     | Size | Type     | Description                                |
| --------- | ---: | -------- | ------------------------------------------ |
| Reference | 32 B | `u8[32]` | BLAKE3 hash of the referenced `.hive` file |
| Module Name| 4 B | `STRING_ID` | Optional module name (e. g. for debugging)|
The zero-based entry index is the corresponding `MODULE_ID` .

The module reference identifies the referenced `.hive` file by its BLAKE3 hash.
The module hash exists of a normalised module id. The `.hive` extension is removed and subfolders can be referenced with a `/`
examples:
foo/bar.hive -> blake3("foo/bar")
Foo/BAR.hive -> blake3("foo/bar")
baz.hive -> blake3("foo")

The BLAKE3 hash is stored as 32 raw bytes.

---

## 9. Sections

### 9.1. Code Section

The `CODE` section contains HIVE bytecode.

The code section is an executable representation and is not an intermediate representation.

Each instruction consists of:

```text
OPCODE
OPERANDS
```

The opcode is encoded as:

```text
u16
```

Operand encoding is instruction-specific.

The complete opcode and operand specification is defined separately by the HIVE opcode specification.

---

#### 9.1.1. Local Identifiers

Instructions may contain function-local identifiers.

```text
LOCAL_ID = u16
```

A `LOCAL_ID` refers to a function-local value or slot according to the instruction that uses it.

`LOCAL_ID` values are not file offsets and are not runtime addresses.

---

### 9.2. Data Section

The `DATA` section contains static data associated with the module.

Data entries may represent:

```text
CONSTANT
STATIC
READONLY
GLOBAL
```

The binary representation of data entries is determined by their associated type and flags.

---

### 9.3. Debug Section

The `DEBUG` section is optional.

Each debug entry contains:

| Field           | Size | Type          | Description        |
| --------------- | ---: | ------------- | ------------------ |
| Function        |  4 B | `FUNCTION_ID` | Function           |
| Source File     |  4 B | `STRING_ID` | Source file        |
| Source Offset   |  4 B | `u32` | Source byte offset |
| Bytecode Offset |  8 B | `u64` | Bytecode offset    |
| Line            |  4 B | `u32` | Source line        |
| Column          |  4 B | `u32` | Source column      |

`Bytecode Offset` is relative to the beginning of the referenced function's bytecode.

---

### 9.4. JIT Information

The `JIT_INFO` section is optional.

The section contains JIT metadata records.

Defined record categories are:

```text
FUNCTION_METADATA
REGISTER_HINTS
SAFEPOINTS
CALL_SITES
STACK_MAPS
```

The encoding of individual records is defined by the HIVE JIT metadata format.

---

## 10. Entry Point

The `Entry Point` field in the file header contains a `FUNCTION_ID` .

For non-executable file types, the entry point must contain `INVALID_ID` .

For executable files, the entry point must reference an existing function entry.

The referenced function must have the `ENTRY` flag set.

---

## 11. Runtime Addresses

Native runtime addresses are not part of the portable `.hive` format.

The following values are not valid persistent file references:

```text
native pointers
native function addresses
heap addresses
stack addresses
scheduler addresses
process addresses
```

Persistent references must use file offsets or defined identifiers.

---

## 12. Required Sections

Executable files must contain:

```text
METADATA
STRINGS
SYMBOLS
TYPES
FUNCTIONS
CODE
```

Other sections are optional unless required by the file type or by a file flag.

---

## 13. Optional Sections

The following sections may be omitted:

```text
ATOMS
DATA
RELOCATIONS
DEBUG
JIT_INFO
MODULES
```

An optional section may be absent without making the file invalid.

If a section is absent, all fields that would reference entries in that section must either contain `INVALID_ID` or satisfy the specific rules defined for that field.

---

## 14. Section Ordering

No semantic meaning is attached to the physical order of sections.

A loader must locate sections exclusively through the section table.

A section may appear before or after any other section.

---

## 15. File Validation

A structurally valid file must satisfy all of the following:

* Magic is valid.
* Version is supported.
* Header size is valid.
* Header is fully contained within the file.
* File size matches the actual file size.
* Section table offset and size are valid.
* Section table is fully contained within the file.
* Section count matches the section table size.
* Every section is fully contained within the file.
* Every section satisfies its declared alignment.
* Section types are known or explicitly permitted as optional extensions.
* Required sections are present.
* Section counts match the actual number of table entries.
* Table references are within their corresponding table bounds.
* Type-specific data is fully contained within the `TYPES` section.
* Function code ranges are fully contained within the `CODE` section.
* Function parameter and return ranges are fully contained within the `FUNCTIONS` section.
* Relocation targets reference valid sections.
* Relocation offsets are within their target sections.
* Referenced symbols exist.
* Referenced types exist.
* Referenced strings exist.
* Referenced atoms exist.
* Referenced functions exist.
* Referenced workers exist.
* Referenced modules exist.
* Referenced imports exist.
* Imported module references are valid.
* Imported symbol references are valid in their referenced module.
* The entry point is valid for executable files.
* Reserved fields contain valid reserved values.
* No required range calculation overflows.

A file failing any required validation is malformed.

---

## 16. Reserved Fields

Reserved fields must be written as zero unless a later format revision defines a different value.

A reader must ignore reserved fields whose semantics are not defined by the supported format version.

---

## 17. Format Extensibility

The format is designed to be extended without changing the meaning of existing structures.

New functionality may be introduced through:

* new file flags, 
* new section types, 
* new section flags, 
* new type kinds, 
* new relocation types, 
* new metadata fields, 
* new optional sections.

Existing loaders may ignore unknown optional sections.

Unknown mandatory structures or values must cause loading to fail.

---

## 18. Binary Structure Summary

The complete file structure is:

```text
HIVE File

│
├── Header
│
├── Section Table
│
└── Sections
    │
    ├── METADATA
    ├── STRINGS
    ├── SYMBOLS
    ├── TYPES
    ├── ATOMS
    ├── FUNCTIONS
    ├── CODE
    ├── DATA
    ├── RELOCATIONS
    ├── DEBUG
    ├── JIT_INFO
    ├── MODULES
```

The format is defined entirely in terms of fixed-width fields, section-relative data, file offsets, and defined identifiers.

Runtime memory addresses and runtime execution state are not serialized by the `.hive` file format.
