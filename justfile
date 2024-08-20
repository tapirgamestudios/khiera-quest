GAME_CRATE_NAME := "built-to-scale"
ROM_TITLE := "khieras-quest"
ITCH_PROJECT := "setsquare/khieras-quest"

export CARGO_TARGET_DIR := justfile_directory() + "/target"


create-release-files: player build-release
    rm -rf target/export
    mkdir target/export
    cp -r player/build target/export/html

    agb-gbafix -tBUILTSCALE -cBTS, -mGC "$CARGO_TARGET_DIR/thumbv4t-none-eabi/release/{{GAME_CRATE_NAME}}" -o "target/export/{{ROM_TITLE}}.gba" --debug
    cp target/export/"{{ROM_TITLE}}.gba" target/export/html/built-to-scale.gba

player:
    (cd player && npm install --no-save --prefer-offline --no-audit && npm run build)

build-release:
    (cd built-to-scale && cargo build --release --no-default-features)

publish: create-release-files
    butler push "target/export/html" "{{ITCH_PROJECT}}:html"
    butler push "target/export/{{ROM_TITLE}}.gba" "{{ITCH_PROJECT}}:gba"