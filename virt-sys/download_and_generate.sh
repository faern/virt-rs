#!/usr/bin/env bash

# Downloads a libvirt release tarball, unpacks it, generats headers
# and calls generate_bindings.sh on it to generate the bindings
#
# Call with the desired libvirt version as the first and only argument

set -eu

version=$1

pushd libvirt-c/
curl -O https://libvirt.org/sources/libvirt-$version.tar.xz
tar xf libvirt-$version.tar.xz
rm libvirt-$version.tar.xz

cd libvirt-$version/
mkdir -p build
cd build
../configure
cp include/libvirt/libvirt-common.h ../include/libvirt/

popd

./generate_bindings.sh libvirt-c/libvirt-$version src/libvirt_${version//./_}.rs

