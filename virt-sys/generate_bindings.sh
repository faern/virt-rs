#!/usr/bin/env bash

# give C library dir as first argument and output binding as second.
# Example:
#  $ ./generate_bindings.sh ./libvirt-5.0.0/ src/libvirt_5_0_0.rs

set -ue

LIB_PATH=$1
BINDING_OUTPUT_PATH=$2

echo "Writing the result to $BINDING_OUTPUT_PATH"

bindgen \
    --no-doc-comments \
    --use-core \
    --no-prepend-enum-name \
    --no-layout-tests \
    --whitelist-function '^vir[A-Z].+$' \
    --whitelist-type '^_?vir[A-Z].+$' \
    --whitelist-var '^(LIB)?VIR_.+$' \
    --blacklist-type '^__.+$' \
    --blacklist-type 'size_t' \
    --raw-line 'use core::option::Option;' \
    -o $BINDING_OUTPUT_PATH \
    $LIB_PATH/include/libvirt/virterror.h \
    -- \
    -I$LIB_PATH/include
    #--ctypes-prefix 'libc' \
    #--raw-line 'use libc::{c_char, c_int, c_void, c_longlong};' \

# Tidy up and correct things I could not manage to configure bindgen to do for me
#sed -i 's/libc::\(c_[a-z]*\)/\1/g'  $BINDING_OUTPUT_PATH
sed -i 's/::core::option::Option/Option/g' $BINDING_OUTPUT_PATH
#sed -i 's/_bindgen_ty_[0-9]\+/u32/g' $BINDING_OUTPUT_PATH
#sed -i 's/pub type u32 = u32;//g' $BINDING_OUTPUT_PATH
sed -i '/#\[derive(Debug, Copy, Clone)\]/d' $BINDING_OUTPUT_PATH
sed -i 's/size_t/usize/g'  $BINDING_OUTPUT_PATH

# Change struct bodies to (c_void);
#   Search regex: {\n +_unused: \[u8; 0],\n}
#   Replace string: (c_void);\n
sed -i -e '/^pub struct .* {$/ {
    N;N
    s/ {\n *_unused: \[u8; 0\],\n}/(::std::os::raw::c_void);\n/
}' "$BINDING_OUTPUT_PATH"


# Remove all }\nextern "C" { to condense code a bit
#   Search regex: }\nextern "C" {
#   Replace string:
sed -i -e '/^extern "C" {$/ {
    :loop
    n
    /^}$/! b loop
    /^}$/ {
        N
        t reset_condition_flags
        :reset_condition_flags
        s/}\nextern "C" {//
        t loop
    }
}' "$BINDING_OUTPUT_PATH"

# Add bindgen version to comment at start of file
#sed -i "1s/bindgen/$(bindgen --version)/" $BINDING_OUTPUT_PATH

rustfmt $BINDING_OUTPUT_PATH
