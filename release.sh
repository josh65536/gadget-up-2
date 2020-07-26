if [ $# -eq 0 ]
then
    echo "Pass in a commit message" 1>&2
    exit 1
fi

./build.sh

# Remove the wasm file; it has a different hash
rm prod/*.wasm

cp -r dist/* prod

cd prod
git add .
# TODO: Fix
git commit -m "$1"
git push
cd ..