# Hive File Format

Every `.hive` file is a binary format.

The format uses a fixed header followed by a section table. The actual data sections may appear anywhere after the header and are located through offsets recorded in the section table.

## Magic and Version

| Name          | Size | Description                                 |
|---------------|-----:|---------------------------------------------|
| Magic         |  4 B | `"HIVE"`                                    |
| Version       |  2 B | Version of the file format                  |
| Type          |  2 B | Type of this `.hive` file                   |
| Flags         |  4 B | File flags                                  |
| Size          |  8 B | Total size of the `.hive` file              |
| Section Table |  8 B | File offset of the section table            |
| Section Count |  4 B | Number of section entries                   |
| Entry Point   |  4 B | `FUNCTION_ID` of the executable entry point |
| Reserved      |  4 B | Reserved for future use                     |
## Rest
The rest is data, version and type specific