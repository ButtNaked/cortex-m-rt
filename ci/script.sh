set -euxo pipefail

main() {
    cargo check --target $TARGET

    cargo check --target $TARGET --features device

    if [ $TARGET = x86_64-unknown-linux-gnu ]; then
        ( cd macros && cargo check && cargo test )

        cargo test --test compiletest
    fi

    local examples=(
        alignment
        divergent-default-handler
        divergent-exception
        entry-static
        main
        minimal
        override-exception
        pre_init
        qemu
        rand
        state
        unsafe-default-handler
        unsafe-entry
        unsafe-exception
        unsafe-hard-fault
    )
    local fail_examples=(
        data_overflow
    )
    if [ $TARGET != x86_64-unknown-linux-gnu ]; then
        # linking with GNU LD
        for ex in "${examples[@]}"; do
            cargo rustc --target $TARGET --example $ex -- \
                  -C linker=arm-none-eabi-ld

            cargo rustc --target $TARGET --example $ex --release -- \
                  -C linker=arm-none-eabi-ld
        done
        for ex in "${fail_examples[@]}"; do
            ! cargo rustc --target $TARGET --example $ex -- \
                  -C linker=arm-none-eabi-ld

            ! cargo rustc --target $TARGET --example $ex --release -- \
                  -C linker=arm-none-eabi-ld
        done

        cargo rustc --target $TARGET --example device --features device -- \
              -C linker=arm-none-eabi-ld

        cargo rustc --target $TARGET --example device --features device --release -- \
              -C linker=arm-none-eabi-ld

        # linking with rustc's LLD
        for ex in "${examples[@]}"; do
            cargo rustc --target $TARGET --example $ex
            cargo rustc --target $TARGET --example $ex --release
        done
        for ex in "${fail_examples[@]}"; do
            ! cargo rustc --target $TARGET --example $ex
            ! cargo rustc --target $TARGET --example $ex --release
        done

        cargo rustc --target $TARGET --example device --features device
        cargo rustc --target $TARGET --example device --features device --release
    fi

    case $TARGET in
        thumbv6m-none-eabi|thumbv7m-none-eabi)
            cargo run --target $TARGET --example qemu | grep "x = 42"
            cargo run --target $TARGET --example qemu --release | grep "x = 42"
            ;;
    esac

    if [ $TARGET = x86_64-unknown-linux-gnu ]; then
        ./check-blobs.sh
    fi
}

main
