---
title: lib/csv
description: RFC4180-ish CSV parse/stringify (Mid M4). First row = headers → list of maps.
---

CSV parse and stringify. First row is headers; rows become a list of maps.

## parse
    + `text`

Parse CSV text into a list of maps.

Caller: [csv.parse] text=`raw`

*> host_csv_parse text=`text`*

## stringify
    + `rows`

Serialize a list of maps to CSV text.

*> host_csv_stringify rows=`rows`*
