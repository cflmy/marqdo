---
markup: occlusion
title: Data resources — database, cache, storage and forms
import:
  cache_api: ext/data/cache.mq.md
  db_api: ext/data/db.mq.md
  fs: lib/fs.mq.md
  path: lib/path.mq.md
---

Start here when a program needs persistent records, short-lived values, object storage or forms.
The public resource files remain the implementation: [database](db.mq.md) owns schemas and
transactions, [cache](cache.mq.md) owns expiry, [storage](storage.mq.md) owns blobs and upload
validation, and [forms](form.mq.md) separates validation, rendering and submission.

These modules use the native web plugin. In this source checkout build it with
scripts/build-web-plugin.sh; installed packages follow the web package installation route.
Commands below run at the repository root. The cache example uses memory only. The transaction example creates a SQLite file
in an owned temporary directory. Prepare an existing disposable directory and export MQ_WORK to its
absolute path; the transaction report returns work_dir for cleanup after this process exits.
Neither example needs a cloud account, credentials or an external service.

```sh
marqdo call ext/data/index.mq.md memory_cache 'key="manual"' 'value="ready"' --effects plugin --json
marqdo call ext/data/index.mq.md transaction_roundtrip "root=\"$MQ_WORK\"" --effects plugin,fs.write --fs-root "$MQ_WORK" --json
```

A resource constructor creates an object; its methods need that actual receiver. Use the free
workflow tools below for a single CLI call, or import the resource file and keep the object in
your program. Do not pass an instance method to call without its receiver or reconstruct a
transaction from guessed fields. Native resource URLs select their own backend: the marqdo
filesystem root does not rewrite a SQLite, S3 or Redis URL. Grant plugin access deliberately.

For development, keep backend operations in the native web resource implementation and expose
small, documented compositions here. Specify row/filter shapes, transaction ownership, conflicts,
expiry units and cleanup. Add an operation to its responsible leaf file and exercise it through
this facade or a real author program. Run the source-handbook regression and database/cache gold
cases on both backends. A successful transport record is not proof that a business update matched.

## memory_cache

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| key | text | "manual" | Key owned by this short-lived demonstration |
| value | any | "ready" | Value written and then read back from the in-process cache |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Actual cache get result; value contains the stored value |

Use `key`="manual" and `value`="ready" unless the caller supplies different data. This
workflow uses memory: storage, so values belong to the current native plugin process.
It removes its key after reading; do not use a production key for an illustrative roundtrip.

First **cache = [Create a process-local cache](cache_api.cache(url="memory:"))**.
We then **[Store the supplied value with a sixty-second expiry](> `cache`.set key=`key` value=`value` ttl=60 )**
and retain **stored = [Read the same cache entry](> `cache`.get key=`key` )**.
Once the value has been captured, **[Remove the demonstration key](> `cache`.del key=`key` )**
so another operation does not inherit it. Return *stored* as observed, including its backend fields.

## transaction_roundtrip

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| root | text | required | Absolute disposable root, matching --fs-root; native resource URLs need an absolute path |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | The committed row list and native commit/rollback outcomes; the second insert is rolled back |

Use the absolute `root` supplied for --fs-root. This isolated-file example shows the whole ownership sequence: create a schema, begin a transaction,
write through its receiver, finish it, and read outside it. A second transaction is explicitly
rolled back. The standalone txn constructor wraps the actual native identifier and connection URL;
it does not begin another transaction or invent an identifier.

First **directory = [Create an owned temporary directory under the granted root](fs.temp_dir(prefix="data-manual"))**.
The filesystem helper returns a root-relative name. We retain **work_dir = [Resolve that owned name against the explicit root](path.join(a=`root`, b=`directory`))**
and **[Check that this absolute directory is inside the granted filesystem root](fs.stat(path=`work_dir`))**
before giving any path to the native database. We select **database_path = [Choose the SQLite file inside that directory](path.join(a=`work_dir`, b="manual.db"))**
and select **database_url = "sqlite:" + database_path** before creating
the receiver **store = [Open only that explicitly selected SQLite resource](db_api.db(url=`database_url`))**.
The table schema is ordinary readable data; both columns and their nullability remain visible.

`fields` =

| 字段 | 类型 | 可空 |
|---|---|---|
| title | text | False |
| body | text | True |

We **[Create the isolated demonstration table](> `store`.init name="manual_notes" fields=`fields` )**
before borrowing the transaction connection. Next **native = [Begin the first transaction](> `store`.事务 )**.
To retain the exact identifier, **identifier = [Read the returned transaction identifier](access(native, "txn"))**
and **connection = [Read its connection URL](access(native, "url"))**.
Now **transaction = [Wrap those actual fields as a transaction receiver](db_api.txn(txn=`identifier`, url=`connection`))**.

`row` =

| title | body |
|---|---|
| committed | kept |

We **[Insert the first row inside its transaction](> `transaction`.insert table="manual_notes" rows=`row` )**
and keep **committed = [Commit that transaction](> `transaction`.提交 )**.
The second **cancelled = [Begin an independent transaction](> `store`.事务 )** is a native
transaction receiver, which can be used directly because its type and resource fields are retained.
We **[Insert a row that will be discarded](> `cancelled`.insert table="manual_notes" rows=`row` )**
then capture **rolled_back = [Roll back the second transaction](> `cancelled`.回滚 )**.
Finally **rows = [Read the committed state outside a transaction](> `store`.select table="manual_notes" order="id" )**.

`report` =

| rows | committed | rolled_back | work_dir |
|---|---|---|---|
| `rows` | `committed` | `rolled_back` | `work_dir` |

This example ends with one committed row. Real programs must also arrange rollback when a later
operation fails. This plugin pools open connections, so remove the owned work_dir only after this
CLI process exits and the connection is closed, including on failure. Do not assume sqlite::memory:
is in-memory through this ABI: relative resource paths are resolved against the module directory.
Return *report* with the real row data, completion records and cleanup location.
