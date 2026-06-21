#[cfg(any(feature = "jpeg2000"))]
mod build_impl {
    use std::fs;
    use std::io;
    use std::path::Path;

    fn extract_zip(zip_path: &Path, out_dir: &Path) {
        if out_dir.exists() {
            return;
        }
        let file = fs::File::open(zip_path).unwrap_or_else(|e| panic!("Failed to open {}: {}", zip_path.display(), e));
        let mut archive =
            zip::ZipArchive::new(file).unwrap_or_else(|e| panic!("Failed to read {}: {}", zip_path.display(), e));
        for i in 0..archive.len() {
            let mut entry = archive.by_index(i).unwrap();
            let name = entry.name().to_string();
            let out_path = out_dir.join(&name);
            if entry.is_dir() {
                fs::create_dir_all(&out_path).unwrap();
            } else {
                if let Some(parent) = out_path.parent() {
                    fs::create_dir_all(parent).unwrap();
                }
                let mut outfile = fs::File::create(&out_path).unwrap();
                io::copy(&mut entry, &mut outfile).unwrap();
            }
        }
        println!("cargo:warning=Extracted {} into {}", zip_path.display(), out_dir.display());
    }

    #[cfg(feature = "jpeg2000")]
    pub fn extract_openjph() {
        extract_zip(Path::new("cpp/OpenJPH.zip"), Path::new("cpp/OpenJPH"));
    }

    fn add_files(build: &mut cc::Build, dir: &Path, ext: &str) {
        for entry in fs::read_dir(dir).unwrap() {
            let p = entry.unwrap().path();
            if p.extension().map_or(false, |e| e == ext) {
                build.file(&p);
            }
        }
    }

    #[cfg(feature = "jpeg2000")]
    pub fn build_openjph() {
        let core = Path::new("cpp/OpenJPH");

        let codestream_dir = core.join("codestream");
        let coding_dir = core.join("coding");
        let transform_dir = core.join("transform");
        let others_dir = core.join("others");
        let openjph_dir = core.join("openjph");

        let is_x86 =
            std::env::var("TARGET").map_or(false, |t| t.contains("x86_64") || t.contains("i686") || t.contains("i386"));

        // ── Base C++ sources (no SIMD variants) ──
        let mut base_build = cc::Build::new();
        base_build
            .cpp(true)
            .include(&codestream_dir)
            .include(&coding_dir)
            .include(&transform_dir)
            .include(&others_dir)
            .include(&openjph_dir)
            .include(core)
            .flag("-fPIC")
            .opt_level(3)
            .warnings(false)
            .define("_FILE_OFFSET_BITS", "64")
            .file(core.join("ojph_wrapper.cpp"));

        // Add all base .cpp files (excluding SIMD variants and wasm)
        for dir in &[&codestream_dir, &coding_dir, &transform_dir, &others_dir] {
            if let Ok(entries) = fs::read_dir(dir) {
                for entry in entries.filter_map(|e| e.ok()) {
                    let p = entry.path();
                    let name = p.file_name().unwrap().to_str().unwrap();
                    let ext = p.extension().map_or("", |e| e.to_str().unwrap());
                    if ext == "cpp" || ext == "c" {
                        if name.contains("_sse.")
                            || name.contains("_ssse3.")
                            || name.contains("_avx.")
                            || name.contains("_avx2.")
                            || name.contains("_avx512.")
                            || name.contains("_wasm.")
                        {
                            continue;
                        }
                        if ext == "c" {
                            continue;
                        }
                        base_build.file(&p);
                    }
                }
            }
        }

        base_build.compile("openjph_base");

        // ── C file (ojph_mem_c.c) ──
        let mut c_build = cc::Build::new();
        c_build.include(&openjph_dir).flag("-fPIC").opt_level(3).warnings(false).file(others_dir.join("ojph_mem_c.c"));
        c_build.compile("openjph_c");

        // ── SIMD sources (x86 or ARM via sse2neon) ──
        let sse2neon_dir = core.join("sse2neon");

        // SSE
        {
            let mut b = cc::Build::new();
            b.cpp(true)
                .include(&codestream_dir)
                .include(&coding_dir)
                .include(&transform_dir)
                .include(&others_dir)
                .include(&openjph_dir)
                .flag("-fPIC")
                .opt_level(3)
                .warnings(false)
                .file(codestream_dir.join("ojph_codestream_sse.cpp"))
                .file(transform_dir.join("ojph_colour_sse.cpp"))
                .file(transform_dir.join("ojph_transform_sse.cpp"));
            if is_x86 {
                b.flag("-msse");
            } else {
                b.include(&sse2neon_dir).define("OJPH_ARCH_X86_64", "1");
            }
            b.compile("openjph_sse");
        }

        // SSE2
        {
            let mut b = cc::Build::new();
            b.cpp(true)
                .include(&codestream_dir)
                .include(&coding_dir)
                .include(&transform_dir)
                .include(&others_dir)
                .include(&openjph_dir)
                .flag("-fPIC")
                .opt_level(3)
                .warnings(false)
                .file(codestream_dir.join("ojph_codestream_sse2.cpp"))
                .file(transform_dir.join("ojph_colour_sse2.cpp"));
            if is_x86 {
                b.flag("-msse2").file(transform_dir.join("ojph_transform_sse2.cpp"));
            } else {
                b.include(&sse2neon_dir).define("OJPH_ARCH_X86_64", "1");
            }
            b.compile("openjph_sse2");
        }

        // SSSE3
        {
            let mut b = cc::Build::new();
            b.cpp(true)
                .include(&codestream_dir)
                .include(&coding_dir)
                .include(&transform_dir)
                .include(&others_dir)
                .include(&openjph_dir)
                .flag("-fPIC")
                .opt_level(3)
                .warnings(false)
                .file(coding_dir.join("ojph_block_decoder_ssse3.cpp"));
            if is_x86 {
                b.flag("-mssse3");
            } else {
                b.include(&sse2neon_dir).define("OJPH_ARCH_X86_64", "1");
            }
            b.compile("openjph_ssse3");
        }

        // AVX (x86 only)
        if is_x86 {
            let mut b = cc::Build::new();
            b.cpp(true)
                .include(&codestream_dir)
                .include(&coding_dir)
                .include(&transform_dir)
                .include(&others_dir)
                .include(&openjph_dir)
                .flag("-fPIC")
                .opt_level(3)
                .warnings(false)
                .flag("-mavx")
                .file(codestream_dir.join("ojph_codestream_avx.cpp"))
                .file(transform_dir.join("ojph_colour_avx.cpp"))
                .file(transform_dir.join("ojph_transform_avx.cpp"))
                .compile("openjph_avx");
        }

        // AVX2 (x86 only)
        if is_x86 {
            let mut b = cc::Build::new();
            b.cpp(true)
                .include(&codestream_dir)
                .include(&coding_dir)
                .include(&transform_dir)
                .include(&others_dir)
                .include(&openjph_dir)
                .flag("-fPIC")
                .opt_level(3)
                .warnings(false)
                .flag("-mavx2")
                .file(codestream_dir.join("ojph_codestream_avx2.cpp"))
                .file(coding_dir.join("ojph_block_decoder_avx2.cpp"))
                .file(coding_dir.join("ojph_block_encoder_avx2.cpp"))
                .file(transform_dir.join("ojph_colour_avx2.cpp"))
                .file(transform_dir.join("ojph_transform_avx2.cpp"))
                .compile("openjph_avx2");
        }

        // AVX512 (x86 only)
        if is_x86 {
            let mut b = cc::Build::new();
            b.cpp(true)
                .include(&codestream_dir)
                .include(&coding_dir)
                .include(&transform_dir)
                .include(&others_dir)
                .include(&openjph_dir)
                .flag("-fPIC")
                .opt_level(3)
                .warnings(false)
                .flag("-mavx512f")
                .flag("-mavx512cd")
                .file(coding_dir.join("ojph_block_encoder_avx512.cpp"))
                .file(transform_dir.join("ojph_transform_avx512.cpp"))
                .compile("openjph_avx512");
        }

        println!("cargo:rustc-link-lib=dylib=c++");

        println!("cargo:rerun-if-changed=build.rs");
    }
}

fn main() {
    #[cfg(feature = "jpeg2000")]
    build_impl::extract_openjph();

    #[cfg(feature = "jpeg2000")]
    build_impl::build_openjph();
}
