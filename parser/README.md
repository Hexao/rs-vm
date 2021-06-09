# Variable declaration

## Syntax

```
<type> <name> = <value>
```

## Type

The variable type can be one of: `u8` or `u16`

## Name

The variable name should start by a letter and can contains any alphanumeric character or underscore

## Value

- `u8` and `u16`
  - The value can be represented in differents bases as follow:
    - Prefix `0b` for binary representation
    - Prefix `0o` for octal representation
    - Without prefix for decimal representation
    - Prefix `0x` for hexadecimal representation

# Structure definition

## Syntax

Use the `struct` keyword to declare a structure followed by his name

```
struct <name> {
    <field_type> <field_name>,
    [ ... ],
    <field_type> <field_name>,
}
```

The *name* and *field_name* follow the same requirement as [variable name](#name) and the *field_type* are the same as [value type](#type). Fields are comma separated and the last comma is optional
