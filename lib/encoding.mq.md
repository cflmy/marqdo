---
title: lib/encoding
description: Base64, hex, and base32 encode/decode (Mid M1 + Mid2 M8). UTF-8 text round-trip.
---

## base64_encode
    + `text`

**> host_encoding_base64_encode text=`text`**

## base64_decode
    + `text`

**> host_encoding_base64_decode text=`text`**

## hex_encode
    + `text`

**> host_encoding_hex_encode text=`text`**

## hex_decode
    + `text`

**> host_encoding_hex_decode text=`text`**

## base32_encode
    + `text`

RFC 4648 alphabet; padding with `=`.

**> host_encoding_base32_encode text=`text`**

## base32_decode
    + `text`

**> host_encoding_base32_decode text=`text`**
