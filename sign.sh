#!/bin/bash

# Sign the target files package
# usage: ./sign.sh <PATH_TO_PACKAGE> <OUTPUT>
# This script determines the Lineage OS version from source tree,
# and assumes that the otatools target has already been builtk,
# so do not use it out of the tree.

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )

source "$SCRIPT_DIR/../patches/lib/message.sh"
colorize

if [ "$#" -ne 2 ]; then
    error "usage: $0 <PATH_TO_PACKAGE> <OUTPUT>"
    exit 1
fi

source_package="$(realpath "$1")"
output_package="$(realpath "$2")"

if [ -z "$ANDROID_BUILD_TOP" ]; then
    ANDROID_BUILD_TOP="$(realpath "$SCRIPT_DIR/..")"
fi

# Check Lineage OS Version
LINEAGE_VERSION_MAJOR=$(grep '^PRODUCT_VERSION_MAJOR' "$ANDROID_BUILD_TOP/vendor/lineage/config/version.mk" | awk -F ' = ' '{print $2}')
LINEAGE_VERSION_MINOR=$(grep '^PRODUCT_VERSION_MINOR' "$ANDROID_BUILD_TOP/vendor/lineage/config/version.mk" | awk -F ' = ' '{print $2}')
LINEAGE_VERSION="$LINEAGE_VERSION_MAJOR.$LINEAGE_VERSION_MINOR"

apexes=()
apexapks=()

case "$LINEAGE_VERSION" in
    22.2)
        apexapks+=(
            AdServicesApk FederatedCompute HalfSheetUX HealthConnectBackupRestore HealthConnectController
            OsuLogin SafetyCenterResources ServiceConnectivityResources ServiceUwbResources ServiceWifiResources WifiDialog
        )
        apexes+=(
            com.android.adbd com.android.adservices com.android.adservices.api com.android.appsearch
            com.android.appsearch.apk com.android.art com.android.bluetooth com.android.btservices com.android.cellbroadcast
            com.android.compos com.android.configinfrastructure com.android.connectivity.resources com.android.conscrypt
            com.android.devicelock com.android.extservices com.android.graphics.pdf com.android.hardware.authsecret
            com.android.hardware.biometrics.face.virtual com.android.hardware.biometrics.fingerprint.virtual
            com.android.hardware.boot com.android.hardware.cas com.android.hardware.neuralnetworks
            com.android.hardware.rebootescrow com.android.hardware.wifi com.android.healthfitness
            com.android.hotspot2.osulogin com.android.i18n com.android.ipsec com.android.media com.android.media.swcodec
            com.android.mediaprovider com.android.nearby.halfsheet com.android.networkstack.tethering
            com.android.neuralnetworks com.android.nfcservices com.android.ondevicepersonalization com.android.os.statsd
            com.android.permission com.android.profiling com.android.resolv com.android.rkpd com.android.runtime
            com.android.safetycenter.resources com.android.scheduling com.android.sdkext com.android.support.apexer
            com.android.telephony com.android.telephonymodules com.android.tethering com.android.tzdata com.android.uwb
            com.android.uwb.resources com.android.virt com.android.vndk.current com.android.vndk.current.on_vendor
            com.android.wifi com.android.wifi.dialog com.android.wifi.resources com.google.pixel.camera.hal
            com.google.pixel.vibrator.hal com.qorvo.uwb
        )
        ;;
    23.0 | 23.1)
        apexapks+=(
            AdServicesApk FederatedCompute HalfSheetUX HealthConnectBackupRestore HealthConnectController
            OsuLogin SafetyCenterResources ServiceConnectivityResources ServiceUwbResources ServiceWifiResources WifiDialog
        )
        apexes+=(
            com.android.adbd com.android.adservices com.android.adservices.api com.android.appsearch
            com.android.appsearch.apk com.android.art com.android.bluetooth com.android.btservices com.android.cellbroadcast
            com.android.compos com.android.configinfrastructure com.android.connectivity.resources com.android.conscrypt
            com.android.devicelock com.android.extservices com.android.graphics.pdf com.android.hardware.authsecret
            com.android.hardware.biometrics.face.virtual com.android.hardware.biometrics.fingerprint.virtual
            com.android.hardware.boot com.android.hardware.cas com.android.hardware.neuralnetworks
            com.android.hardware.rebootescrow com.android.hardware.wifi com.android.healthfitness
            com.android.hotspot2.osulogin com.android.i18n com.android.ipsec com.android.media com.android.media.swcodec
            com.android.mediaprovider com.android.nearby.halfsheet com.android.networkstack.tethering
            com.android.neuralnetworks com.android.nfcservices com.android.ondevicepersonalization com.android.os.statsd
            com.android.permission com.android.profiling com.android.resolv com.android.rkpd com.android.runtime
            com.android.safetycenter.resources com.android.scheduling com.android.sdkext com.android.support.apexer
            com.android.telephony com.android.telephonymodules com.android.tethering com.android.tzdata com.android.uwb
            com.android.uwb.resources com.android.virt com.android.vndk.current com.android.vndk.current.on_vendor
            com.android.wifi com.android.wifi.dialog com.android.wifi.resources com.google.pixel.camera.hal
            com.google.pixel.vibrator.hal com.qorvo.uwb
            # Added in 23
            com.android.bt com.android.crashrecovery com.android.hardware.contexthub com.android.hardware.dumpstate
            com.android.hardware.gatekeeper.nonsecure com.android.hardware.power com.android.hardware.thermal
            com.android.hardware.threadnetwork com.android.hardware.uwb com.android.hardware.vibrator com.android.uprobestats
        )
        ;;
    23.2)
        apexapks+=(
            AdServicesApk FederatedCompute HalfSheetUX HealthConnectBackupRestore HealthConnectController
            OsuLogin SafetyCenterResources ServiceConnectivityResources ServiceUwbResources ServiceWifiResources WifiDialog
        )
        apexes+=(
            com.android.adbd com.android.adservices com.android.adservices.api com.android.appsearch
            com.android.appsearch.apk com.android.art com.android.bluetooth com.android.btservices com.android.cellbroadcast
            com.android.compos com.android.configinfrastructure com.android.connectivity.resources com.android.conscrypt
            com.android.devicelock com.android.extservices com.android.graphics.pdf com.android.hardware.authsecret
            com.android.hardware.biometrics.face.virtual com.android.hardware.biometrics.fingerprint.virtual
            com.android.hardware.boot com.android.hardware.cas com.android.hardware.neuralnetworks
            com.android.hardware.rebootescrow com.android.hardware.wifi com.android.healthfitness
            com.android.hotspot2.osulogin com.android.i18n com.android.ipsec com.android.media com.android.media.swcodec
            com.android.mediaprovider com.android.nearby.halfsheet com.android.networkstack.tethering
            com.android.neuralnetworks com.android.nfcservices com.android.ondevicepersonalization com.android.os.statsd
            com.android.permission com.android.profiling com.android.resolv com.android.rkpd com.android.runtime
            com.android.safetycenter.resources com.android.scheduling com.android.sdkext com.android.support.apexer
            com.android.telephony com.android.telephonymodules com.android.tethering com.android.tzdata com.android.uwb
            com.android.uwb.resources com.android.virt com.android.vndk.current com.android.vndk.current.on_vendor
            com.android.wifi com.android.wifi.dialog com.android.wifi.resources com.google.pixel.camera.hal
            com.google.pixel.vibrator.hal com.qorvo.uwb
            # Added in 23
            com.android.bt com.android.crashrecovery com.android.hardware.contexthub com.android.hardware.dumpstate
            com.android.hardware.gatekeeper.nonsecure com.android.hardware.power com.android.hardware.thermal
            com.android.hardware.threadnetwork com.android.hardware.uwb com.android.hardware.vibrator com.android.uprobestats
            # Added in 23.2
            com.android.telephonycore
        )
        ;;
    *)
        error "LineageOS $LINEAGE_VERSION is not supported"
        exit 2
        ;;
esac

extra_apex_apks="$(printf -- "--extra_apks %s.apk=$HOME/.android-certs/releasekey\n" "${apexapks[@]}")"
extra_apexes+="$(for k in "${apexes[@]}"; do printf -- "--extra_apks %s.apex=$HOME/.android-certs/%s\n" "$k" "$k"; done)"
extra_apex_payload_keys="$(for k in "${apexes[@]}"; do printf -- "--extra_apex_payload_key %s.apex=$HOME/.android-certs/%s.pem\n" "$k" "$k"; done)"


cd "$ANDROID_BUILD_TOP"
"out/host/linux-x86/bin/sign_target_files_apks" -o -d ~/.android-certs \
    $extra_apex_apks \
    $extra_apexes \
    $extra_apex_payload_keys \
    "$source_package" \
    "$output_package"

