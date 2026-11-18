#!/usr/bin/env node
// Exercise current embedded libraries through the public browser ABI.
import fs from 'node:fs';
import assert from 'node:assert/strict';
const {instance}=await WebAssembly.instantiate(fs.readFileSync(process.argv[2]),{});
const e=instance.exports, enc=new TextEncoder(),dec=new TextDecoder();
function boot(source,expectedOk=true){
  const bytes=enc.encode(source),p=e.mq_alloc(bytes.length);
  new Uint8Array(e.memory.buffer,p,bytes.length).set(bytes);
  const out=e.mq_boot(p,bytes.length);e.mq_dealloc(p,bytes.length);
  const length=new DataView(e.memory.buffer).getUint32(out,true);
  const value=JSON.parse(dec.decode(new Uint8Array(e.memory.buffer,out+4,length)));
  e.mq_dealloc(out,length+4);assert.equal(value.ok,expectedOk,JSON.stringify(value));return expectedOk?value.value:value;
}
const summary=boot(`---
markup: occlusion
import:
  table: lib/table.mq.md
---
# main

\`rows\` =

| @ | team | n |
|---|---|---|
| 1 | a | 1 |
| 2 | b | 4 |
| 3 | a | 2 |

*> table.summarize list=\`rows\` key="team" field="n"*
`);
assert.deepEqual(summary,[{key:'a',count:2,value:3},{key:'b',count:1,value:4}]);
const bytes=boot(`---
markup: occlusion
import:
  bytes: lib/bytes.mq.md
---
# main

*> bytes.from_base64 text="AP+A"*
`);
assert.deepEqual(bytes,{_type:'bytes',base64:'AP+A'});
const doc=boot(`---
markup: occlusion
import:
  document: lib/document.mq.md
---
# main

*> document.sections text="# Notes"*
`);
assert.equal(doc.length,1);assert.equal(doc[0].title,'Notes');
const config=boot(`---
markup: occlusion
import:
  toml: lib/toml.mq.md
---
# main

*> toml.parse text="name='Marqdo'"*
`);
assert.equal(config.name,'Marqdo');
const moment=boot(`---
markup: occlusion
import:
  dt: lib/datetime.mq.md
---
# main

*> dt.parse_local text="2024-11-03 01:30:00" zone="America/New_York" ambiguity="earliest"*
`);
assert.equal(moment.ok,true);assert.equal(moment.value.iso,'2024-11-03T01:30:00-04:00');
console.log('wasm stdlib ok: collection, bytes, document, TOML, IANA/DST');
const chunk=boot(`---
markup: occlusion
import:
  table: lib/table.mq.md
---
# main

\`values\` =

| n |
|---|
| 1 |
| 2 |

*> table.chunk list=\`values\` size=4294967296*
`);
assert.deepEqual(chunk,[[1,2]]);
const sliced=boot(`---
markup: occlusion
import:
  bytes: lib/bytes.mq.md
---
# main

**data = bytes.from_base64 text="AP+A"**
*> bytes.slice data=\`data\` start=4294967296 end=4294967298*
`);
assert.equal(sliced.base64,'');
console.log('wasm 32-bit large-index semantics ok');

const badPut=boot(`---
markup: occlusion
import:
  table: lib/table.mq.md
---
# main

\`values\` =

| n |
|---|
| 1 |
| 2 |

\`indices\` =

| index |
|---|
| 4294967296 |
| 1 |

*> table.put in=\`values\` at=\`indices\` value=9*
`,false);
assert.equal(badPut.ok,false);
console.log('wasm nested index overflow is a diagnostic, not a trap');

for (const [module, entry, parameter] of [
  ['lib/table.mq.md', 'from_pairs', 'pairs'],
  ['lib/表.mq.md', '键值转记录', '键值对'],
]) {
  const result = boot(`---
markup: occlusion
import:
  records: ${module}
---
# main

\`fields\` =

| @ | key | value |
|---|---|---|
| 1 | channel | old |
| 2 | empty | None |
| 3 | channel | "source\\|skill" |

The ordered fields retain real nulls. Return *[Build the record from the visible rows](records.${entry}(${parameter}=\`fields\`))*.
`);
  assert.deepEqual(result, {channel: 'source|skill', empty: null});
}
assert.equal(boot(`# main\n\nReturn *[multiply x*y and retain the product](2 * (3 + 1))* after calculation.\n`), 8);
console.log('wasm narrative libraries ok: bilingual pair records, escaped GFM pipe, inline occlusion product');
