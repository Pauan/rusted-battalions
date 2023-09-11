import { nodeResolve } from "@rollup/plugin-node-resolve";
import commonjs from "@rollup/plugin-commonjs";
import rust from "@wasm-tool/rollup-plugin-rust";
import serve from "rollup-plugin-serve";
import livereload from "rollup-plugin-livereload";
import terser from "@rollup/plugin-terser";

const is_watch = !!process.env.ROLLUP_WATCH;

export default {
    input: {
        client: "./crates/client-web/Cargo.toml",
    },
    output: {
        dir: "dist/js",
        format: "esm",
        sourcemap: true,
    },
    plugins: [
        nodeResolve({
            preferBuiltins: false,
            //modulesOnly: true,
        }),

        commonjs(),

        rust({
            serverPath: "js/",
            watchPatterns: ["src/**", "../engine/**", "../game/**"],

            // Makes debug behave similarly to release, except with debug symbols
            //
            // https://doc.rust-lang.org/cargo/reference/profiles.html#release
            /*cargoArgs: [
                "--config", "profile.dev.opt-level=3",
                "--config", "profile.dev.debug=false",
                "--config", "profile.dev.debug-assertions=false",
                "--config", "profile.dev.overflow-checks=false",
                "--config", "profile.dev.lto=true",
                "--config", "profile.dev.codegen-units=1",
            ],*/

            //debug: false,
        }),

        is_watch && serve({
            contentBase: "dist",
            open: true,
        }),

        is_watch && livereload("dist"),

        !is_watch && terser(),
    ],
};
