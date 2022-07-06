#!/bin/sh
# Creates a distributable tarball for a local platform or for
# a given target triple. To build for the local platform, do not
# pass any arguments or pass in `native`:
#
#     ./build.sh
#
# The result is a tarball with a name like `ibisibi-0.1.0-linux-x86_64.tar.gz`,
# which is assembled from uname info.
#
# To cross-compile with an ARM toolchain to get a tarball like
# `release/ibisibi-<version>-arm-unknown-linux-gnueabihf.tar.gz`,
# invoke the script with the desired target triple:
#
#     ./build.sh arm-unknown-linux-gnueabihf
#
# You can also build releases for multiple target architectures with
# one invocation, e.g. to build for the native architecture and
# additionally for ARMv6 and AArch64 do:
#
#     ./build.sh native arm-unknown-linux-gnueabihf aarch64-unknown-linux-musl
#
# Make sure that a cross-compile toolchain is available for the target platform,
# e.g. aarch64 needs a package `aarch64-unknown-linux-musl` on Arch Linux.
# For 32bit ARM you can install a armhf package, or if none is available, install
# binaries for `arm-linux-gnueabihf-gcc` and `arm-linux-gnueabihf-ar` manually into
# `~/Development/rpi-newer-crosstools/x64-gcc-6.3.1/arm-rpi-linux-gnueabihf/bin/`
#
# Please also note that `arm-unknown-linux-gnueabihf` needs some more
# special setup. Home directory must contain
# `Development/vendor/arm-unknown-linux-gnueabihf/libudev`
# with builds of libudev and pkgconfig (pc file).
# No such special setup is required for other targets.

METADATA=$(head -n 4 Cargo.toml | sed -n 's/^.*"\([^"]*\)".*$/\1/p')
CRATE=$(echo "$METADATA" | head -n 1)
VERSION=$(echo "$METADATA" | sed -n '2p')

HOST_TARGET_TRIPLE=$(rustc -vV | grep "host: " | cut -b 7-)

function clean {
    rm -rf $RELEASE_DIR && \
    mkdir -p $RELEASE_DIR
}

function copy_assets {
    cp AUTHORS.md $RELEASE_DIR && \
    cp LICENSE $RELEASE_DIR && \
    cp README.md $RELEASE_DIR && \
    cp install.sh $RELEASE_DIR && \
    cp -r examples $RELEASE_DIR
}

function generate_source_link {
    echo "The source code is publicly hosted at GitHub:
https://github.com/tapirbug/ibisibi" > $RELEASE_DIR/SOURCE
}

function info {
    echo "$@" 1>&2
}

function error {
    echo "error: $@" 1>&2
}

if [ -z "$1" ]
then
    # Build for host architecture if nno target triple specified
    TARGET_TRIPLES=("native")
else
    TARGET_TRIPLES=("$@")
fi

for TARGET_TRIPLE in "${TARGET_TRIPLES[@]}"
do
    if [ "$TARGET_TRIPLE" = "native" ]
    then
        # Set to host architecture printed by rustc to build native
        TARGET_TRIPLE="$HOST_TARGET_TRIPLE"
    fi

    # ARMv7 for older raspberries as a cross target => customize build
    if [ "$TARGET_TRIPLE" != "$HOST_TARGET_TRIPLE" ] && [ "$TARGET_TRIPLE" = "arm-unknown-linux-gnueabihf" ]
    then
        # Bring cross-compile toolchain into path
        OLD_PATH="$PATH"
        PATH="$PATH:$HOME/Development/rpi-newer-crosstools/x64-gcc-6.3.1/arm-rpi-linux-gnueabihf/bin/"

        # set up library search paths for libudev
        # When cross-compoiling serialport on linux, help the build script
        # find the library for cross-compiling.
        # See: https://github.com/dcuddeback/libudev-sys#cross-compiling
        export PKG_CONFIG_SYSROOT_DIR="$HOME/Development/vendor/arm-unknown-linux-gnueabihf/libudev"
        export PKG_CONFIG_LIBDIR="${PKG_CONFIG_SYSROOT_DIR}/usr/lib/pkgconfig:${PKG_CONFIG_SYSROOT_DIR}/usr/share/pkgconfig:${PKG_CONFIG_SYSROOT_DIR}/usr/lib/arm-linux-gnueabihf/pkgconfig"
        export PKG_CONFIG_ALLOW_CROSS=1
    fi

    # Actual build after customization
    CARGO_ARGS="--release"
    RELEASE_DIR_NAME="$CRATE-$VERSION-$TARGET_TRIPLE"
    RELEASE_DIR="release/$RELEASE_DIR_NAME"
    RELEASE_TAR="$RELEASE_DIR_NAME.tar.gz" # ibisibi-0.1.0-arm-unknown-linux-gnueabihf.tar.gz
    if [ "$TARGET_TRIPLE" = "$HOST_TARGET_TRIPLE" ]
    then
        info "Building for $TARGET_TRIPLE..."
        BINARY="target/release/$CRATE"
    else
        info "Cross-compiling for $TARGET_TRIPLE ..."
        CARGO_ARGS="$CARGO_ARGS --target=$TARGET_TRIPLE"
        BINARY="target/$TARGET_TRIPLE/release/$CRATE"
    fi

    info "Building $CRATE into $RELEASE_DIR"
    cargo build $CARGO_ARGS && \
    info "Clearing output directory ..." && \
    clean && \
    info "Copying binary ..." && \
    cp $BINARY $RELEASE_DIR || cp $BINARY.exe $RELEASE_DIR && \
    info "Copying static assets ..." && \
    copy_assets && \
    generate_source_link && \
    info "Writing compressed tarball $RELEASE_TAR ..." && \
    cd release && \
    tar -zcf $RELEASE_TAR $RELEASE_DIR_NAME && \
    cd .. && \
    echo $RELEASE_DIR_NAME || \
    FAILURE="true"

    if [ "$FAILURE" = "true" ]
    then
        OVERALL_FAILURE="true"
        error "compilation failed for target triple $TARGET_TRIPLE"
    fi


    # ARMv7 for older raspberries as cross target => uncustomize build again
    if [ "$TARGET_TRIPLE" != "$HOST_TARGET_TRIPLE" ] && [ "$TARGET_TRIPLE" = "arm-unknown-linux-gnueabihf" ]
    then
        # remove cross-compile toolchain from path again
        PATH="$OLD_PATH"
        # remove library search paths for libudev again
        unset PKG_CONFIG_SYSROOT_DIR
        unset PKG_CONFIG_LIBDIR
        unset PKG_CONFIG_ALLOW_CROSS
    fi
done

if [ "$OVERALL_FAILURE" = "true" ]
then
    exit 1
fi
