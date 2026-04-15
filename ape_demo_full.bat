@echo off
echo ================================================================================
echo                    APE TRUST KERNEL END-TO-END DEMO
echo ================================================================================
echo.
echo [1] Local deterministic demo from real AI workflow data
cargo run --manifest-path ape\Cargo.toml --bin ape -- demo --mode local --seed 42 --action transfer_100_tokens
echo.
echo [2] Start sidecar separately if needed, then run HTTP trust-kernel demo
echo     cargo run --manifest-path coh-node\Cargo.toml -p coh-sidecar
echo.
echo [3] Sidecar HTTP demo
cargo run --manifest-path ape\Cargo.toml --bin ape -- demo --mode sidecar --sidecar-url http://127.0.0.1:3030 --seed 42 --action transfer_100_tokens
echo.
echo [4] Performance benchmarks (CLI path)
cargo run --manifest-path ape\Cargo.toml --bin ape -- bench --iterations 1000
echo.
echo [5] Performance benchmarks (sidecar path)
cargo run --manifest-path ape\Cargo.toml --bin ape -- bench --iterations 200 --with-sidecar --sidecar-url http://127.0.0.1:3030
echo.
echo Output artifacts written under ape\output
echo ================================================================================
