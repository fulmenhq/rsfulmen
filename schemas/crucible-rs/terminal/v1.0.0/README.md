# Terminal Width Override Schema v1.0.0

This schema defines the structure for terminal-specific character width override configurations used by FulmenHQ tools like goneat.

## Purpose

Different terminal emulators handle Unicode character widths inconsistently. This schema provides a standardized way to define width overrides for specific terminals to ensure proper rendering of ASCII art and boxes.

## Schema Structure

- `version`: Must be "1.0.0"
- `terminals`: Object mapping terminal identifiers to configurations
- `test_sequences`: Optional sequences for calibration testing

## Terminal Configuration

Each terminal entry has:

- `name`: Human-readable name
- `overrides`: Map of character → width (optional)
- `notes`: Additional information (optional)

## Catalog

The `catalog/` directory contains empirically tested configurations for popular terminals. Contributions welcome!

## Usage

This schema is used by:

- goneat: For ASCII box rendering
- Other FulmenHQ tools: For consistent terminal output

## Contributing

To add a new terminal:

1. Run calibration: `goneat terminal calibrate`
2. Test rendering: `goneat ascii box <file>`
3. Add config to `catalog/<terminal>.yaml`
4. Submit PR

## Validation

Use JSON Schema validators to validate configurations against `schema.yaml`.
