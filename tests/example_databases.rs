//! Execute the documented SQLite initializer against disposable site roots.
use serde_json::{json, Value};
use std::{
    fs,
    io::Write,
    path::PathBuf,
    process::{Command, Stdio},
};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

struct Scratch(PathBuf);
impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn article_initializers_use_real_resources_and_preserve_user_rows_on_both_backends() {
    for (site, module, table, function, expected, has_slug) in [
        ("web-site", "index", "articles", "open", 8, true),
        ("web-site-zh", "index", "articles", "open", 8, true),
        ("web-net-site", "index", "articles", "open", 3, false),
        ("web-net-site-zh", "index", "articles", "打开", 3, false),
        ("marqdo-blog", "posts", "posts", "open", 2, true),
    ] {
        for backend in ["tree", "bytecode"] {
            let work = Scratch(std::env::temp_dir().join(format!(
                "mq-article-init-{}-{site}-{backend}", std::process::id()
            )));
            fs::create_dir_all(work.0.join("db")).unwrap();
            let leaf=if site=="marqdo-blog" {"db/posts.tables.md"} else {"db/articles.tables.md"};
            fs::copy(root().join("examples").join(site).join(leaf),work.0.join(leaf)).unwrap();
            let driver = work.0.join("verify.mq.md");
            fs::write(&driver, format!(
                "---\nmarkup: occlusion\nimport:\n  site: {}/examples/{site}/db/{module}.mq.md\n---\n\n## main\n\nUse `url` for the owned database and `rows` for a business write.\n\nWe **store = site.{function}(url=`url`)** and **before = [Observe seeds](> `store`.count table=\"{table}\")**.\nWe **[Insert a user article](> `store`.insert table=\"{table}\" rows=`rows`)** and **again = site.{function}(url=`url`)** and **after = [Observe preserved writes](> `again`.count table=\"{table}\")**.\n\n`counts` =\n\n| before | after |\n|---|---|\n| `before` | `after` |\n\nReturn *counts*.\n",
                root().display()
            )).unwrap();
            let library = if cfg!(target_os = "macos") {"libweb.dylib"} else if cfg!(windows) {"web.dll"} else {"libweb.so"};
            let mut child = Command::new(env!("CARGO_BIN_EXE_marqdo"))
                .current_dir(root())
                .env("MARQDO_LIB", root().join("lib"))
                .env("MARQDO_EXT", root().join("ext"))
                .env("MARQDO_WEB_PLUGIN", root().join("plugins/web/build").join(library))
                .env("MARQDO_CALL_SESSION_DIR", work.0.join("sessions"))
                .args(["call", driver.to_str().unwrap(), "main", "--backend", backend,
                    "--args-json", "-", "--effects", "plugin,fs.write", "--fs-root",
                    work.0.to_str().unwrap(), "--json"])
                .stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped())
                .spawn().unwrap();
            let mut row = if site=="marqdo-blog" {json!({"title":"Retained user article","content":"User text"})} else {json!({"title":"Retained user article","body":"User text"})};
            if has_slug { row["slug"] = json!("retained-user"); }
            let args = json!({"url":format!("sqlite:{}", work.0.join("site.db").display()),"rows":row});
            child.stdin.take().unwrap().write_all(args.to_string().as_bytes()).unwrap();
            let out=child.wait_with_output().unwrap();
            assert!(out.status.success(), "{site} [{backend}] {}\n{}",
                String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr));
            let envelope:Value=serde_json::from_slice(&out.stdout).unwrap();
            assert_eq!(envelope["result"]["before"],expected,"{envelope}");
            assert_eq!(envelope["result"]["after"],expected+1,"{envelope}");
        }
    }
}
