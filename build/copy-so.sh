AARC64_TARGET_DIR=../target/aarch64-linux-android
AARC64_DIST_DIR=app/src/main/jniLibs/arm64-v8a/

ARMV7_TARGET_DIR=../target/armv7-linux-androideabi
ARMV7_DIST_DIR=app/src/main/jniLibs/armeabi-v7a/

i686_TARGET_DIR=../target/i686-linux-android
i686_DIST_DIR=app/src/main/jniLibs/x86/

TARGET_NAME=librust_nes.so

# cargo build --target aarch64-linux-android -q --lib
# cargo build --target armv7-linux-androideabi -q --lib
cargo build --target i686-linux-android -q --lib


echo "build finish"

# cp $AARC64_TARGET_DIR/debug/$TARGET_NAME ./android/$AARC64_DIST_DIR
# cp $ARMV7_TARGET_DIR/debug/$TARGET_NAME ./android/$ARMV7_DIST_DIR
cp $i686_TARGET_DIR/debug/$TARGET_NAME ./android/$i686_DIST_DIR


cd ./android
./gradlew installDebug


cd ../