if not exist piped\node_modules (
    pnpm -C piped install
)

if not exist piped\dist (
    pnpm -C piped build
)
