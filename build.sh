npm run build
# The rust compiler has a bug where it leaves some panicking code intact.
# This results in a "Can't resolve 'env'" error, so
# we absolutely must strip it.
file=target/wasm32-unknown-unknown/release/gadget_up_2.wasm
wasm-snip --snip-rust-panicking-code -o $file $file
# And rebuild with the new wasm file
npm run build