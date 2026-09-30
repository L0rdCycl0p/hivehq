<!--
 * @file            docs/src/format/index.md
 * @description     
 * @author          TrollMii <trollmii@proton.me>
 * @createTime      2026-09-20 15:48:29
 * @lastModified    2026-09-29 20:05:00
 * Copyright ©L0rdCycl0p
                    HIVE (Hive Is Very Efficient)
Copyright (C) 2026 L0rdCycl0p

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program. If not, see <https://www.gnu.org/licenses/>.

-->

# Hive File Format

Every `.hive` file is a binary format.

The format uses a fixed header followed by a section table. The actual data sections may appear anywhere after the header and are located through offsets recorded in the section table.

## Magic and Version

| Name | Size | Description |
|---------------|-----:|---------------------------------------------|
| Magic | 4 B | `"HIVE"` |
| Version | 2 B | Version of the file format |
| Type | 2 B | Type of this `.hive` file |
| Flags | 4 B | File flags |
| Size | 8 B | Total size of the `.hive` file |
| Section Table | 8 B | File offset of the section table |
| Section Count | 4 B | Number of section entries |
| Entry Point | 4 B | `FUNCTION_ID` of the executable entry point |
| Reserved | 4 B | Reserved for future use |

## Rest

The rest is data, version and type specific
