# ASCII Schemas v1.0.0

JSON Schemas for ASCII art, text analysis, and Unicode-aware string handling utilities.

## Schemas

### string-analysis.schema.json

**Purpose**: Analysis results for string properties with Unicode awareness

**Key Properties**:

- `length`: Total character length (integer, required)
- `width`: Display width accounting for Unicode (integer, required)
- `hasUnicode`: Whether string contains Unicode characters (boolean, required)
- `lineCount`: Number of lines (integer, required)
- `wordCount`: Number of words (integer, required)
- `isMultiline`: Whether string is multiline (boolean, required)

**Use Cases**:

- Terminal output formatting
- Table column width calculations
- Text wrapping and alignment
- Unicode emoji handling

### box-chars.schema.json

**Purpose**: Character set definitions for drawing ASCII boxes and borders

**Key Properties**:

- `topLeft`: Top-left corner character (string, single char, required)
- `topRight`: Top-right corner character (string, single char, required)
- `bottomLeft`: Bottom-left corner character (string, single char, required)
- `bottomRight`: Bottom-right corner character (string, single char, required)
- `horizontal`: Horizontal line character (string, single char, required)
- `vertical`: Vertical line character (string, single char, required)
- `cross`: Cross/intersection character (string, single char, required)

**Use Cases**:

- Drawing tables and borders
- Creating box-drawing CLI interfaces
- ASCII art generation
- Terminal UI components

## Usage

These schemas are used across Fulmen libraries:

- `gofulmen/ascii`: Go implementation
- `tsfulmen/ascii`: TypeScript implementation
- `pyfulmen/ascii`: Python implementation (future)

**Purpose**:

- Validate text analysis results
- Ensure consistent box-drawing character sets
- Type generation for ASCII utilities
- Cross-language API consistency

## Versioning

Schemas follow semantic versioning. Breaking changes will increment the major version number.

## Naming Conventions

All property names use **camelCase** for consistency with JSON/JavaScript conventions.
