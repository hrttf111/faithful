BASE=/opt/sandbox/pop/data/

for f in plsdata.dat plssphr.dat; do
    rm ./$f
    cargo run --bin pop_res --release -- pls $BASE/$f > ./$f
done
