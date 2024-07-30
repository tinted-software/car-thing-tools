use anyhow::Result;
use blake3::Hash;
use cpio::write_cpio;
use cpio::NewcBuilder;
use gix::url::parse as parse_url;
use mlua::prelude::*;
use mlua::Lua;
use std::collections::HashMap;
use std::fs::File;
use std::io::Cursor;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

struct Package {
    name: String,
    src: Vec<Source>,
}

struct Source {
    uri: String,
    hash: Option<Hash>,
}

type PackageStore = Arc<Mutex<HashMap<String, Package>>>;

fn make_package(lua: &Lua, store: PackageStore) -> LuaResult<LuaFunction> {
    lua.create_function(move |lua, table: LuaTable| {
        let name = table.get::<_, String>("name")?;
        let src = table.get::<_, LuaTable>("src")?;
        let build = table.get::<_, LuaFunction>("build")?;

        let mut sources = Vec::new();
        for pair in src.pairs::<LuaValue, LuaTable>() {
            let (_, source_table) = pair?;
            let uri = source_table.get::<_, String>("uri")?;
            let hash = source_table
                .get::<_, Option<String>>("hash")?
                .map(|h| blake3::Hash::from_hex(&h).unwrap());
            sources.push(Source { uri, hash });
        }

        let builder = lua.create_any_userdata(ArchiveWriter {
            name: name.clone(),
            files: Vec::new(),
        })?;

        build.call::<_, ()>(builder)?;

        let package = Package {
            name: name.clone(),
            src: sources,
        };

        let store = store.clone();

        tokio::spawn(async move {
            let mut store = store.lock().await;
            store.insert(name, package);
        });

        Ok(())
    })
}

fn clone_repo(repo_url: &str, dst: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    unsafe {
        gix::interrupt::init_handler(1, || {})?;
    }
    std::fs::create_dir_all(dst)?;
    let url = parse_url(repo_url.into())?;

    println!("Url: {:?}", url.to_bstring());
    let mut prepare_clone = gix::prepare_clone(url, dst)?;

    println!("Cloning {repo_url:?} into {dst:?}...");
    let (mut prepare_checkout, _) = prepare_clone
        .fetch_then_checkout(gix::progress::Discard, &gix::interrupt::IS_INTERRUPTED)?;

    println!(
        "Checking out into {:?} ...",
        prepare_checkout.repo().work_dir().expect("should be there")
    );

    let (repo, _) =
        prepare_checkout.main_worktree(gix::progress::Discard, &gix::interrupt::IS_INTERRUPTED)?;
    println!(
        "Repo cloned into {:?}",
        repo.work_dir().expect("directory pre-created")
    );

    let remote = repo
        .find_default_remote(gix::remote::Direction::Fetch)
        .expect("always present after clone")?;

    println!(
        "Default remote: {} -> {}",
        remote.name().expect("default remote is always named").as_bstr(),
        remote.url(gix::remote::Direction::Fetch).expect("should be the remote URL").to_bstring(),
    );

    Ok(())
}

pub struct ArchiveWriter {
    pub name: String,
    pub files: Vec<(NewcBuilder, Cursor<Vec<u8>>)>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let store = PackageStore::new(Mutex::new(HashMap::new()));

    let lua = Lua::new();

    lua.register_userdata_type::<ArchiveWriter>(move |reg| {
        reg.add_method_mut(
            "add_text_file",
            move |_lua, this: &mut ArchiveWriter, (path, content): (String, String)| {
                let builder = NewcBuilder::new(&path).uid(1000).gid(1000).mode(0o100644);
                this.files.push((builder, Cursor::new(content.as_bytes().to_vec())));

                Ok(())
            },
        );

        reg.add_method_mut("finish", move |_lua, this: &mut ArchiveWriter, ()| {
            let file = File::create(format!("/tix/store/{}", this.name))?;
            let mut encoder = zstd::stream::Encoder::new(file, 22).unwrap();

            write_cpio(this.files.clone().into_iter(), &mut encoder).unwrap();

            encoder.finish().unwrap();

            Ok(())
        });
    })?;

    let make_package_fn = make_package(&lua, store.clone())?;
    lua.globals().set("package", make_package_fn)?;

    // Example usage
    lua.load(
        r#"
        local hello = package {
            name = 'hello',
            src = {
            },
            build = function(builder)
                builder:add_text_file(
                    'bin/hello',
                    [[
                    #!/bin/sh
                    echo "Hello World!"
                    ]]
                )

                builder:finish()
            end,
        }
    "#,
    )
    .exec()?;

    Ok(())
}
