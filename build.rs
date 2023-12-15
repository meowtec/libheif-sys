use std::path::{Path, PathBuf};

fn write_file(path: &Path, content: &str) -> () {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(path, content).unwrap();
}

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
    let target_arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    let is_wasm = target_arch.contains("wasm");

    // build vendored library using cmake
    // #[cfg(feature = "use-vendor")]
    {
        // cmake & pkg_config have some problem when target to wasm
        if is_wasm {
            let libde265_sources = vec![
                "alloc_pool.cc",
                "bitstream.cc",
                "cabac.cc",
                "configparam.cc",
                "contextmodel.cc",
                "de265.cc",
                "deblock.cc",
                "decctx.cc",
                "dpb.cc",
                "en265.cc",
                "fallback-dct.cc",
                "fallback-motion.cc",
                "fallback.cc",
                "image-io.cc",
                "image.cc",
                "intrapred.cc",
                "md5.cc",
                "motion.cc",
                "nal-parser.cc",
                "nal.cc",
                "pps.cc",
                "quality.cc",
                "refpic.cc",
                "sao.cc",
                "scan.cc",
                "sei.cc",
                "slice.cc",
                "sps.cc",
                "threads.cc",
                "transform.cc",
                "util.cc",
                "visualize.cc",
                "vps.cc",
                "vui.cc",
            ];

            cc::Build::new()
                .warnings(false)
                .files(
                    libde265_sources
                        .iter()
                        .map(|s| format!("libde265/libde265/{}", s)),
                )
                .include("libde265")
                .include("libde265/extra")
                .define("HAVE_POSIX_MEMALIGN", "1")
                .compile("de265");

            let x265_sources = vec![
                "common/primitives.cpp",
                "common/pixel.cpp",
                "common/dct.cpp",
                "common/lowpassdct.cpp",
                "common/ipfilter.cpp",
                "common/intrapred.cpp",
                "common/loopfilter.cpp",
                "common/constants.cpp",
                "common/cpu.cpp",
                "common/version.cpp",
                "common/threading.cpp",
                "common/threadpool.cpp",
                "common/wavefront.cpp",
                "common/md5.cpp",
                "common/bitstream.cpp",
                "common/yuv.cpp",
                "common/shortyuv.cpp",
                "common/picyuv.cpp",
                "common/common.cpp",
                "common/param.cpp",
                "common/frame.cpp",
                "common/framedata.cpp",
                "common/cudata.cpp",
                "common/slice.cpp",
                "common/lowres.cpp",
                "common/piclist.cpp",
                "common/predict.cpp",
                "common/scalinglist.cpp",
                "common/quant.cpp",
                "common/deblock.cpp",
                "common/scaler.cpp",
                "encoder/analysis.cpp",
                "encoder/search.cpp",
                "encoder/bitcost.cpp",
                "encoder/motion.cpp",
                "encoder/slicetype.cpp",
                "encoder/frameencoder.cpp",
                "encoder/framefilter.cpp",
                "encoder/level.cpp",
                "encoder/nal.cpp",
                "encoder/sei.cpp",
                "encoder/sao.cpp",
                "encoder/entropy.cpp",
                "encoder/dpb.cpp",
                "encoder/ratecontrol.cpp",
                "encoder/reference.cpp",
                "encoder/encoder.cpp",
                "encoder/api.cpp",
                "encoder/weightPrediction.cpp",
            ];

            let x265_config_path = out_dir.join("x265/x265_config.h");
            write_file(
                &x265_config_path,
                &std::fs::read_to_string("x265/source/x265_config.h.in")
                    .unwrap()
                    .replace("${X265_BUILD}", "198"),
            );

            cc::Build::new()
                .warnings(false)
                .std("c++11")
                .define("EXPORT_C_API", "1")
                .define("HAVE_STRTOK_R", "1")
                .define("X265_NS", "x265")
                // NOT support HIGH_BIT_DEPTH
                .define("HIGH_BIT_DEPTH", "0")
                .define("X265_DEPTH", "8")
                .files(
                    x265_sources
                        .iter()
                        .map(|s| format!("x265/source/{}", s)),
                )
                .include(out_dir.join("x265"))
                .include("x265/source")
                .include("x265/source/common")
                .include("x265/source/encoder")
                .compile("x265");

            let libheif_sources = vec![
                "bitstream.cc",
                "box.cc",
                "error.cc",
                "heif.cc",
                "context.cc",
                "file.cc",
                "pixelimage.cc",
                "hevc.cc",
                "avif.cc",
                "plugin_registry.cc",
                "heif_plugin.cc",
                "nclx.cc",
                "init.cc",
                "mask_image.cc",
                "metadata_compression.cc",
                "common_utils.cc",
                "region.cc",
                "heif_regions.cc",
                "heif_properties.cc",
                "color-conversion/colorconversion.cc",
                "color-conversion/rgb2yuv.cc",
                "color-conversion/rgb2yuv_sharp.cc",
                "color-conversion/yuv2rgb.cc",
                "color-conversion/rgb2rgb.cc",
                "color-conversion/monochrome.cc",
                "color-conversion/hdr_sdr.cc",
                "color-conversion/alpha.cc",
                "color-conversion/chroma_sampling.cc",
                "jpeg.cc",
                "jpeg2000.cc",
                "vvc.cc",
                "plugins/encoder_mask.cc",
                "plugins/decoder_libde265.cc",
                "plugins/encoder_x265.cc",
            ];

            write_file(
                &out_dir.join("heif/libheif/heif_version.h"),
                &std::fs::read_to_string("libheif/libheif/heif_version.h.in")
                    .unwrap()
                    .replace("@PROJECT_VERSION_MAJOR@", "1")
                    .replace("@PROJECT_VERSION_MINOR@", "17")
                    .replace("@PROJECT_VERSION_PATCH@", "5"),
            );

            cc::Build::new()
                .warnings(false)
                .std("c++11")
                .define("HAVE_VISIBILITY", "1")
                .define("HAVE_LIBDE265", "1")
                .define("HAVE_X265", "1")
                .files(
                    libheif_sources
                        .iter()
                        .map(|s| format!("libheif/libheif/{}", s)),
                )
                .include(out_dir.join("heif"))
                .include(out_dir.join("x265"))
                .include("libde265")
                .include("libde265/extra")
                .include("x265/source")
                .include("libheif")
                .compile("heif");

            include_dirs = vec![
                "libheif".to_string(),
                out_dir.join("heif").to_string_lossy().to_string(),
            ];
        } else {
            const PKG_CONFIG_PATH: &str = "PKG_CONFIG_PATH";

            fn probe_pkg_config_in(name: &str, dir: &str) -> pkg_config::Library {
                // store original PKG_CONFIG_PATH
                let env_pkg_config_path = std::env::var(PKG_CONFIG_PATH);

                std::env::set_var(PKG_CONFIG_PATH, dir);

                let library = pkg_config::Config::new()
                    .statik(true)
                    .probe(name)
                    .expect(&format!("library `{}` not found", name));

                // restore original PKG_CONFIG_PATH
                match env_pkg_config_path {
                    Ok(var) => {
                        std::env::set_var(PKG_CONFIG_PATH, var);
                    }
                    Err(_) => {
                        std::env::remove_var(PKG_CONFIG_PATH);
                    }
                }
                library
            }

            let libde265_output = cmake::Config::new("libde265")
                .out_dir(out_dir.join("libde265"))
                .define("ENABLE_SDL", "OFF")
                .define("BUILD_SHARED_LIBS", "OFF")
                .define("ENABLE_ENCODER", "OFF")
                .define("ENABLE_DECODER", "OFF")
                .build();

            let libwebp_output = if !target_arch.contains("wasm") {
                Some(
                    cmake::Config::new("libwebp")
                        .out_dir(out_dir.join("libwebp"))
                        .define("BUILD_SHARED_LIBS", "OFF")
                        .build(),
                )
            } else {
                None
            };

            let mut x265 = cmake::Config::new("x265/source");

            x265.out_dir(out_dir.join("x265"))
                .define("ENABLE_SDL", "OFF")
                .define("BUILD_SHARED_LIBS", "OFF")
                .define("ENABLE_SHARED", "OFF")
                .define("ENABLE_ENCODER", "OFF");

            if is_wasm {
                x265.define("ENABLE_ASSEMBLY", "OFF");
            }

            let x265_output = x265.build();

            let output_paths = [
                Some(&libde265_output),
                Some(&x265_output),
                libwebp_output.as_ref(),
            ]
            .iter()
            .filter_map(|x| x.to_owned())
            .collect::<Vec<_>>();

            let output = cmake::Config::new("libheif")
                .out_dir(out_dir.join("libheif"))
                .configure_arg("--preset=release-noplugins")
                .define("BUILD_SHARED_LIBS", "OFF")
                .define(
                    "CMAKE_PREFIX_PATH",
                    output_paths
                        .iter()
                        .map(|x| x.to_string_lossy().to_string())
                        .collect::<Vec<_>>()
                        .join(";"),
                )
                .define("WITH_AOM_DECODER", "OFF")
                .define("WITH_AOM_ENCODER", "OFF")
                .define("WITH_JPEG_DECODER", "OFF")
                .define("WITH_JPEG_ENCODER", "OFF")
                .define("WITH_EXAMPLES", "OFF")
                .define("WITH_DEFLATE_HEADER_COMPRESSION", "OFF")
                .build();

            println!("cargo:warning=output: {:?}", output);

            include_dirs.push(output.join("include").to_string_lossy().to_string());

            let mut all_output_paths = output_paths.clone();
            all_output_paths.push(&output);

            let library = probe_pkg_config_in(
                "libheif",
                all_output_paths
                    .iter()
                    .map(|output| output.join("lib/pkgconfig").to_string_lossy().to_string())
                    .collect::<Vec<_>>()
                    .join(":")
                    .as_str(),
            );

            println!("cargo:warnings=library: {:?}", library);
        }
    }

    // not vendored, should find and link libheif via pkg_config
    // #[cfg(not(feature = "use-vendor"))]
    #[cfg(any())]
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
            .allowlist_function("heif_.*")
            .allowlist_type("heif_.*")
            .size_t_is_usize(true)
            .clang_args([
                "-fparse-all-comments",
                "-fretain-comments-from-system-headers",
                "-fvisibility=default",
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
