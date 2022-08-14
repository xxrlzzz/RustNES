AARC64_TARGET=aarch64-linux-android
AARC64_DIST_DIR=app/src/main/jniLibs/arm64-v8a/

ARMV7_TARGET=armv7-linux-androideabi
ARMV7_DIST_DIR=app/src/main/jniLibs/armeabi-v7a/

i686_TARGET=i686-linux-android
i686_DIST_DIR=app/src/main/jniLibs/x86/

TARGET_NAME=librust_nes.so
BUILD_TYPE=release
BUILD_FLAG="-q --lib"
if (( BUILD_TYPE == release )); then
  BUILD_FLAG="$BUILD_FLAG --release"
fi

# echo rust flag is: $BUILD_FLAG 
cargo build --target $AARC64_TARGET $BUILD_FLAG
# cargo build --target $ARMV7_TARGET $BUILD_FLAG
# cargo build --target $i686_TARGET $BUILD_FLAG


echo "build finish"

cp ../target/$AARC64_TARGET/$BUILD_TYPE/$TARGET_NAME ./android/$AARC64_DIST_DIR
# cp ../target/$ARMV7_TARGET/$BUILD_TYPE/$TARGET_NAME ./android/$ARMV7_DIST_DIR
# cp ../target/$i686_TARGET/$BUILD_TYPE/$TARGET_NAME ./android/$i686_DIST_DIR


cd ./android
./gradlew installDebug


cd ../