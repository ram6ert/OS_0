user:
    cd user && cargo build --release
    mkdir -p kernel/artifacts
    cp ./user/target/x86_64-os0-user/release/user kernel/artifacts/user
kernel: user
    cd kernel && cargo build --release
kernel-dev: user
    cd kernel && cargo build
image: kernel
    mkdir -p build
    cd image_builder && cargo run --release -- ../kernel/target/x86_64-os0/release/kernel ../build/
image-dev: kernel-dev
    mkdir -p build/dev
    cd image_builder && cargo run --release -- ../kernel/target/x86_64-os0/debug/kernel ../build/dev
qemu: image
    qemu-system-x86_64 -drive format=raw,file=build/bios.img -serial stdio -no-reboot
qemu-debug: image-dev
    qemu-system-x86_64 -drive format=raw,file=build/dev/bios.img -s -S -nographic
clean:
    rm -r build/* kernel/target/* image_builder/target/* user/target/*
