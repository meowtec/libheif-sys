use std::path::PathBuf;

fn main() {
    if std::env::var("DOCS_RS").is_ok() {
        // Don't link with libheif in case of building documentation for docs.rs.
        println!("cargo:rustc-cfg=docs_rs");
        return;
    }

    #[allow(unused_mut)]
    #[allow(unused_variables)]
    #[allow(unused_assignments)]
    let mut include_dirs: Vec<String> = Vec::new();

    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());

    // build vendored library using cmake
    #[cfg(feature = "use-vendor")]
    {
        let libde265_output = cmake::Config::new("libde265")
            .out_dir(out_dir.join("libde265"))
            .define("ENABLE_SDL", "OFF")
            .define("BUILD_SHARED_LIBS", "OFF")
            .define("ENABLE_ENCODER", "OFF")
            .define("ENABLE_DECODER", "OFF")
            .build();

        println!(
            "cargo:rustc-link-search=native={}",
            libde265_output.join("lib").display()
        );
        println!("cargo:rustc-link-lib=static=de265");

        let libwebp_output = cmake::Config::new("libwebp")
            .out_dir(out_dir.join("libwebp"))
            .define("BUILD_SHARED_LIBS", "OFF")
            // .build_target("sharpyuv")
            .build();

        println!(
            "cargo:rustc-link-search=native={}",
            libwebp_output.join("lib").display()
        );
        println!("cargo:rustc-link-lib=static=sharpyuv");

        let x265_output = cmake::Config::new("x265/source")
            .out_dir(out_dir.join("x265"))
            .define("ENABLE_SDL", "OFF")
            .define("BUILD_SHARED_LIBS", "OFF")
            .define("ENABLE_SHARED", "OFF")
            .define("ENABLE_ENCODER", "OFF")
            .build();

        println!("cargo:rustc-link-search=native={}", x265_output.join("lib").display());
        println!("cargo:rustc-link-lib=static=x265");

        let libheif_output = cmake::Config::new("libheif")
            .out_dir(out_dir.join("libheif"))
            .configure_arg("--preset=release-noplugins")
            .define("BUILD_SHARED_LIBS", "OFF")
            .define(
                "CMAKE_PREFIX_PATH",
                format!(
                    "{};{};{}",
                    libde265_output.display(),
                    x265_output.display(),
                    libwebp_output.display()
                ),
            )
            .define("WITH_AOM_DECODER", "OFF")
            .define("WITH_AOM_ENCODER", "OFF")
            .define("WITH_JPEG_DECODER", "OFF")
            .define("WITH_JPEG_ENCODER", "OFF")
            .define("WITH_EXAMPLES", "OFF")
            .build();

        println!(
            "cargo:rustc-link-search=native={}",
            libheif_output.join("lib").display()
        );
        println!("cargo:rustc-link-lib=static=heif");

        include_dirs = vec![libheif_output, libde265_output, x265_output, libwebp_output]
            .iter()
            .map(|dir| dir.join("include").to_string_lossy().to_string())
            .collect();

        // link c++. (TODO: libstdc++ vs libc++?)
        println!("cargo:rustc-link-lib=dylib=c++");
    }

    // not vendored, should find and link libheif via pkg_config
    #[cfg(not(feature = "use-vendor"))]
    {
        #[cfg(not(target_os = "windows"))]
        {
            println!("cargo:warning=start probe heif");
            match pkg_config::Config::new()
                .atleast_version("1.16")
                .probe("libheif")
            {
                Ok(library) => {
                    include_dirs = library
                        .include_paths
                        .iter()
                        .map(|dir| dir.to_string_lossy().to_string())
                        .collect();
                }
                Err(err) => {
                    println!("cargo:warning={}", err);
                    std::process::exit(1);
                }
            }
            println!("cargo:warning=end probe heif");

            pkg_config::Config::new()
                // .statik(true)
                .probe("libsharpyuv")
                .expect("sharpyuv not found");
        }

        #[cfg(target_os = "windows")]
        {
            let vcpkg_lib = vcpkg::Config::new()
                .emit_includes(true)
                .find_package("libheif");
            match vcpkg_lib {
                Ok(lib) => {
                    // https://users.rust-lang.org/t/bindgen-cant-find-included-file/62687
                    use walkdir::WalkDir;
                    for path in lib.include_paths {
                        for subdir in WalkDir::new(path)
                            .into_iter()
                            .filter_entry(|e| e.file_type().is_dir())
                        {
                            let dir = subdir.unwrap().path().to_string_lossy().to_string();
                            include_dirs.push(dir);
                        }
                    }
                }
                Err(err) => {
                    println!("cargo:warning={}", err);
                    std::process::exit(1);
                }
            }
        }
    }

    #[cfg(feature = "use-bindgen")]
    {
        use std::env;
        // The bindgen::Builder is the main entry point
        // to bindgen, and lets you build up options for
        // the resulting bindings.
        let mut builder = bindgen::Builder::default()
            // The input header we would like to generate
            // bindings for.
            .header("wrapper.h")
            .generate_comments(true)
            .generate_cstr(true)
            .ctypes_prefix("libc")
            .allowlist_function("heif_.*")
            .allowlist_type("heif_.*")
            .size_t_is_usize(true)
            .clang_args([
                "-fparse-all-comments",
                "-fretain-comments-from-system-headers",
            ]);

        if !include_dirs.is_empty() {
            dbg!(&include_dirs);
            builder = builder.clang_args(
                include_dirs
                    .iter()
                    .map(|dir| format!("--include-directory={}", dir)),
            );
        }

        // Finish the builder and generate the bindings.
        let bindings = builder
            .generate()
            // Unwrap the Result and panic on failure.
            .expect("Unable to generate bindings");

        // Write the bindings to the $OUT_DIR/bindings.rs file.
        let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
        bindings
            .write_to_file(out_path.join("bindings.rs"))
            .expect("Couldn't write bindings!");
    }
}
