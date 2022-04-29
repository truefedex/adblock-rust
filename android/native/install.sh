cargo build --target aarch64-linux-android --release
cargo build --target armv7-linux-androideabi --release
#cargo build --target i686-linux-android --release
cargo build --target x86_64-linux-android --release

cd ../library/src/main
if [ ! -d jniLibs ]; then
    mkdir -p jniLibs
    mkdir jniLibs/arm64
    mkdir jniLibs/armeabi
    mkdir jniLibs/x86_64
fi

#ln -s /home/fedex/Projects/phlox/adblock-rust/android/native/target/aarch64-linux-android/release/libadblock_rs.so jniLibs/arm64/libadblock_rs.so
#ln -s /home/fedex/Projects/phlox/adblock-rust/android/native/target/armv7-linux-androideabi/release/libadblock_rs.so jniLibs/armeabi/libadblock_rs.so
#ln -s /home/fedex/Projects/phlox/adblock-rust/android/native/target/x86_64-linux-android/release/libadblock_rs.so jniLibs/x86_64/libadblock_rs.so

cp --remove-destination ../../../native/target/aarch64-linux-android/release/libadblock_rs.so jniLibs/arm64/libadblock_rs.so
cp --remove-destination ../../../native/target/armv7-linux-androideabi/release/libadblock_rs.so jniLibs/armeabi/libadblock_rs.so
cp --remove-destination ../../../native/target/x86_64-linux-android/release/libadblock_rs.so jniLibs/x86_64/libadblock_rs.so
