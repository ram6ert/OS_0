kernel:
    cd kernel && cargo build --release
image: kernel
    mkdir -p build
    cd image_builder && cargo run --release -- ../kernel/target/x86_64-os0/release/kernel ../build/
qemu: image
    qemu-system-x86_64 -drive format=raw,file=build/bios.img -serial stdio
clean:
    rm -r build/* kernel/target/* image_builder/target/*
