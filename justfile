set positional-arguments

code-review: git-status disallowed check-format build clippy test check-docs git-status

check:
    cargo check --all-targets --all-features

build:
    cargo build --all-targets --all-features

run:
    cargo run --bin spelldawn

run-firestore:
    cargo run --bin spelldawn -- --firestore

test:
    cargo test

disallowed:
    ! grep -r --include '*.rs' 'ERROR_PANIC: bool = true'

# Build a local docker image
docker-build:
    docker build -t spelldawn .

# Run local docker image
docker-run:
    docker run -it --rm -p 50052:50052 --name spelldawn spelldawn

# Submit docker build to Google Cloud Build
cloud-build:
    gcloud builds submit --region=us-central1 --config cloudbuild.yaml

# Updates current google cloud image
cloud-update:
    gcloud compute instances update-container spelldawn-trunk --zone us-central1-a --container-image us-central1-docker.pkg.dev/spelldawn/spelldawn/spelldawn:latest

protos:
    cargo run --bin gen_protos
    rm src/protos/src/google.protobuf.rs

    - rm proto/*.cs
    - rm -r proto/bin
    - rm -r proto/obj
    dotnet clean proto/protos.csproj
    dotnet build proto/protos.csproj
    mv proto/*.cs Assets/Spelldawn/Protos/
    dotnet clean proto/protos.csproj
    rm -r proto/bin
    rm -r proto/obj

@dropbox-ignore:
    find . -name '*conflicted*' -delete
    mkdir -p Library
    mkdir -p Logs
    mkdir -p log
    mkdir -p obj
    mkdir -p UserSettings
    mkdir -p Temp
    mkdir -p out
    mkdir -p out_BurstDebugInformation_DoNotShip/
    mkdir -p ServerData
    mkdir -p target
    xattr -w com.dropbox.ignored 1 Library/
    xattr -w com.dropbox.ignored 1 Logs/
    xattr -w com.dropbox.ignored 1 log/
    xattr -w com.dropbox.ignored 1 obj/
    xattr -w com.dropbox.ignored 1 UserSettings/
    xattr -w com.dropbox.ignored 1 Temp/
    xattr -w com.dropbox.ignored 1 out/
    xattr -w com.dropbox.ignored 1 out_BurstDebugInformation_DoNotShip/
    xattr -w com.dropbox.ignored 1 target/
    xattr -w com.dropbox.ignored 1 db/
    xattr -w com.dropbox.ignored 1 ServerData/
    - rm -r 'Temp (Ignored Item Conflict)'
    - rm -r 'out (Ignored Item Conflict)'

screenshots-message:
    @ echo "\nRunning Screenshot Tests"
    @ sleep 1
    @ echo "\n(this would be a good time to grab a snack)"
    @ sleep 1
    @ echo "\nPlease Stand By...\n"
    @ sleep 3

rsync:
    mkdir -p /tmp/spelldawn
    rsync -a . --delete --exclude=Temp --exclude=target --exclude=out /tmp/spelldawn

build_flag := if os() == "macos" {
    "-buildOSXUniversalPlayer"
  } else {
    "OS not supported"
  }

app_path := if os() == "macos" {
    "/tmp/spelldawn/out/spelldawn.app"
  } else {
    "OS not supported"
  }

bin_path := if os() == "macos" {
    "/tmp/spelldawn/out/spelldawn.app/Contents/MacOS/Spelldawn"
  } else {
    "OS not supported"
  }

screenshot_path := if os() == "macos" {
    join(app_path, "Contents", "Screenshots")
  } else {
    "OS not supported"
  }

unity := if os() == "macos" {
    "/Applications/Unity/Hub/Editor/2021.3.3f1/Unity.app/Contents/MacOS/Unity"
  } else {
    "ERROR: Add unity path"
  }

# You can't run tests on a project you have open in Unity, so we rsync the project to a tmp dir
# before running end to end tests.
run-screenshots: screenshots-message rsync
    rm -rf /tmp/spelldawn/out/
    mkdir -p /tmp/spelldawn/out/
    "{{unity}}" -batchMode -quit -projectPath "/tmp/spelldawn" {{build_flag}} "{{app_path}}"
    "{{bin_path}}" -test -monitor 2 -screen-width 1334 -screen-height 750 -screen-quality "High" -screen-fullscreen 0

finish-screenshots: run-screenshots
    #!/usr/bin/env sh
    for file in `ls "{{screenshot_path}}"`; do
        magick "{{screenshot_path}}/$file" -resize '50%' "{{screenshot_path}}/$file"
    done

screenshot-tests: finish-screenshots
  #!/usr/bin/env sh
  image_diffs="/tmp/spelldawn/image_diffs"
  rm -r $image_diffs
  mkdir $image_diffs
  failed=0
  for file in `ls "{{screenshot_path}}"`; do
    result=`magick compare -metric mse "{{screenshot_path}}/$file" "./ScreenshotTests/$file" "$image_diffs/$file" 2>&1`
    difference=`echo $result | cut -f 1 -d ' ' -`
    echo "Image difference is $difference for $file"
    if awk "BEGIN {exit !($difference >= 1)}"; then
        echo "\n>>> Test Failed: $file"
        echo "See open $image_diffs/$file {{screenshot_path}}/$file ./ScreenshotTests/$file"
        failed=1
    fi
  done
  exit $failed

record: finish-screenshots
    rm -rf ScreenshotTests
    mkdir -p ScreenshotTests
    cp "{{screenshot_path}}"/*.png ScreenshotTests/

plugin_out := "Assets/Plugins"
target_arm := "aarch64-apple-darwin"
target_x86 := "x86_64-apple-darwin"

clean-plugin:
    rm -r Assets/Plugins/

mac-plugin:
    # you may need to run codesign --deep -s - -f spelldawn.app before running
    cargo build -p plugin --release --target={{target_arm}}
    cargo build -p plugin --release --target={{target_x86}}
    # lib prefix breaks on mac standalone
    lipo -create -output plugin.bundle \
        target/{{target_arm}}/release/libplugin.dylib \
        target/{{target_x86}}/release/libplugin.dylib
    mkdir -p {{plugin_out}}/macOS/
    mv plugin.bundle {{plugin_out}}/macOS/

target_windows := "x86_64-pc-windows-gnu"

# You may need to install mingw, e.g. via brew install mingw-w64
# Note that the plugin name cannot conflict with any .asmdef file
windows-plugin:
    # Note that you cannot use IL2CPP when cross-compiling for windows
    cargo build --release -p plugin --target {{target_windows}}
    mkdir -p {{plugin_out}}/Windows/
    cp target/{{target_windows}}/release/plugin.dll {{plugin_out}}/Windows/

# install via rustup target add aarch64-linux-android
target_android := "aarch64-linux-android"

# Android NDK path
# e.g. /Users/name/Library/Android/sdk/ndk/24.0.8215888
# e.g. /Applications/Unity/Hub/Editor/2021.3.3f1/PlaybackEngines/AndroidPlayer/NDK
android_ndk := env_var_or_default("ANDROID_NDK", "")

llvm_toolchain := if os() == "macos" {
        "darwin-x86_64"
    } else if os() == "linux" {
        "linux-x86_64"
    } else {
        "OS not supported"
    }

# If you get an error about libgcc not being found, see here:
# https://github.com/rust-lang/rust/pull/85806
# "Find directories containing file libunwind.a and create a text file called libgcc.a with the text INPUT(-lunwind)"

clang := "aarch64-linux-android21-clang"
toolchains := "toolchains/llvm/prebuilt"
export CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER := join(android_ndk, toolchains, llvm_toolchain, "bin", clang)

android-plugin:
    # Note: builds for Android that use native plugins must use IL2CPP
    # This is only arm64, need to do arm7 at some point too
    @ echo "Using linker $CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER"
    cargo build --release --target={{target_android}}
    mkdir -p {{plugin_out}}/Android/ARM64
    # You see, standalone osx builds *do not* want the lib prefix but android fails *without* it...
    cp target/{{target_android}}/release/libplugin.so {{plugin_out}}/Android/ARM64/

target_x86_sim := "x86_64-apple-ios"

ios-simulator-plugin:
    cargo build -p plugin --release --target={{target_x86_sim}}
    mkdir -p {{plugin_out}}/iOS/Simulator
    cp target/{{target_x86_sim}}/release/libplugin.a {{plugin_out}}/iOS/Simulator

target_ios := "aarch64-apple-ios"

ios-plugin:
    cargo build -p plugin --release --target={{target_ios}}
    mkdir -p {{plugin_out}}/iOS/Device
    cp target/{{target_ios}}/release/libplugin.a {{plugin_out}}/iOS/Device

plugin: mac-plugin windows-plugin ios-plugin android-plugin

doc:
    cargo doc

fix: git-status fmt clippy-fix fix-lints

watch:
    cargo watch -c -w src -x run

watch-firestore:
    cargo watch -c -w src -x run -- --firestore

fix-amend: git-status fmt git-amend1 clippy-fix git-amend2 fix-lints git-amend3

clippy:
    cargo clippy --workspace --exclude "protos" -- -D warnings -D clippy::all

clippy-fix:
    cargo clippy --fix --allow-dirty -- -D warnings -D clippy::all

# Reformats code. Requires nightly because several useful options (e.g. imports_granularity) are
# nightly-only
fmt:
    cargo +nightly fmt

check-format:
    cargo +nightly fmt -- --check

fix-lints:
    cargo fix --allow-dirty --all-targets

snapshots:
    cargo insta review

update-cards:
    cargo run --bin update_cards

benchmark *args='':
    cargo criterion --no-run -p tests
    if [[ "$OSTYPE" == "darwin"* ]]; then \
      echo "Signing benchmark binary"; \
      codesign -f -s - `find target/release/deps -name '*benchmarks*'`; \
    fi
    cargo criterion -p tests -- "$@"

# Checks documentation lints, haven't figured out how to do this with a single command
check-docs:
    RUSTDOCFLAGS="-D rustdoc::broken-intra-doc-links -D rustdoc::private-intra-doc-links -D rustdoc::bare-urls" cargo doc --all

# Need to run
# rustup target add x86_64-unknown-linux-gnu
# brew tap SergioBenitez/osxct
# brew install x86_64-unknown-linux-gnu
build-linux-from-osx:
    CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=x86_64-unknown-linux-gnu-gcc cargo build --target=x86_64-unknown-linux-gnu

outdated:
    # Check for outdated dependencies, consider installing cargo-edit and running 'cargo upgrade' if this fails
    cargo outdated --exit-code 1

upgrade:
    cargo upgrade --workspace

machete:
    cargo machete --fix

remove-unused-deps: machete

time-passes: clean-dropbox
    cargo +nightly rustc -p spelldawn --bin spelldawn -- -Z time-passes

timings: clean-dropbox
    cargo build --timings

# Builds .gcda files used for code coverage
gen-gcda: clean-dropbox
    #!/usr/bin/env sh
    set -euxo pipefail
    export LLVM_PROFILE_FILE='spelldawn-%p-%m.profraw'
    export RUSTFLAGS='-Zinstrument-coverage'
    cargo +nightly build
    cargo +nightly test # Generates .profraw files in the current working directory
    export CARGO_INCREMENTAL=0
    export RUSTFLAGS='-Zprofile -Ccodegen-units=1 -Copt-level=0 -Clink-dead-code -Coverflow-checks=off -Zpanic_abort_tests -Cpanic=abort'
    export RUSTDOCFLAGS="-Cpanic=abort"
    cargo +nightly build # Build .gcno files in ./target/debug/deps/
    cargo +nightly test # Build .gcda files in ./target/debug/deps/

# Displays test coverage information in a web browser
coverage: gen-gcda
    grcov . -s . \
        --binary-path ./target/debug/ \
        -t html \
        --branch \
        --ignore-not-existing \
        -o ./target/debug/coverage
    open target/debug/coverage/index.html

# Checks for uncommitted repository changes
git-status:
    git diff-index --quiet HEAD --
    git ls-files --exclude-standard --others

git-amend1:
    git commit -a --amend -C HEAD

git-amend2:
    git commit -a --amend -C HEAD

git-amend3:
    git commit -a --amend -C HEAD

@nim *args='':
    cargo run --bin run_nim -- $@

@matchup *args='':
    cargo run --bin run_matchup -- $@

clean-dropbox:
    rm -f target/.rustc_info.json
    cargo clean
    mkdir target
    xattr -w com.dropbox.ignored 1 target
    find . -name "*.profraw" -delete

just paths:
    echo "Asset Bundle Downloads: ~/Library/Caches/com.spelldawn.Spelldawn/"
    echo "Log Files: ~/Library/Logs/Spelldawn/Spelldawn"
    echo "Game Data: ~/Library/Application Support/Spelldawn/Spelldawn"

# GCloud
#
# Log into GCloud:
# gcloud auth login
# Settting GCloud Project:
# gcloud config set project spelldawn
# Creating GCloud Artifact:
# gcloud artifacts repositories create spelldawn --repository-format=docker
#     --location=us-central1 --description="Spelldawn repository"
# Authorizing Docker:
# gcloud auth configure-docker us-central1-docker.pkg.dev
# SSH into server:
# gcloud compute ssh --zone "us-central1-a" "spelldawn-trunk"  --project
#     "spelldawn"
# Running GCloud Build:
# gcloud builds submit --region=us-central1 --tag
#     us-central1-docker.pkg.dev/spelldawn/spelldawn/spelldawn:latest

# IP Addresses
# gcloud compute addresses list
# gcloud compute addresses create trunk-regional --region=us-central1
# gcloud compute instances describe spelldawn-trunk
# gcloud compute instances delete-access-config spelldawn-trunk --access-config-name "External NAT"
# gcloud compute instances add-access-config spelldawn-trunk  --access-config-name='trunk' --address "34.29.1.131"

# Docker
# List Containers:
# docker container ls
# List Images:
# docker image ls
# Show Container Logs:
# docker logs --tail 50 --follow --timestamps ca675fc6b778
# Attach to container:
# docker attach 184dfc90b290
# Adding Docker Tag:
# docker tag spelldawn:latest
#     us-central1-docker.pkg.dev/spelldawn/spelldawn/spelldawn:latest
# Pushing Local Docker Image:
# docker push us-central1-docker.pkg.dev/spelldawn/spelldawn/spelldawn:latest
# Running commands on container:
#  docker exec 7458a883c1b9 rm -r db
# Restart container
#  docker restart 7458a883c1b9

# GRPC CLI
# grpc_cli ls 35.184.200.62:50052 -l
# grpc_cli ls trunk.spelldawn.com:80 -l

# iOS App Store
# Uploading: https://help.apple.com/xcode/mac/current/#/dev442d7f2ca
