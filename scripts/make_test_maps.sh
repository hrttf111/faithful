LANDSCAPE_TYPES="k g t h j"
LEVELS="2 6 23"

make_map() {
    LEVEL=$1
    TYPE=$2
    MOVE=$3
    cargo run --bin pop_res --release -- globe $MOVE $LEVEL > img/${i}_${TYPE}.bmp
    for t in $LANDSCAPE_TYPES; do
        cargo run --bin pop_res --release -- ${TYPE} $MOVE --landtype=$t $LEVEL > img/${LEVEL}_${t}_${TYPE}.bmp
    done
}

rm -rf ./img
mkdir ./img

make_map 1 "minimap" --move="400;400"
make_map 1 "globe" --move="400;400"
#make_map 1 "land" --move="2400;2400"
for i in $LEVELS; do
    make_map $i "globe"
    make_map $i "minimap"
    #make_map $i "land"
done

#cargo run --bin pop_res -- land 2 | feh -
