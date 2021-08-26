# flag-edge-matcher
Match flag edges

First, the Rust programming language must be installed with Cargo.

The first stage is generating flag edges. This is done by running `cargo run -p edge-gen --release`.
Then flags are matched together. This is done with `cargo run -p flag-matcher --release`.
As the program runs, it will print the full graph and the number of flags in that graph, saving to "best_graph_found_{flag_count}.json".
Once the program outputs a graph with a sufficient number of flags, they can be combined into an image using `cargo run -p graph-to-flag-img --release`.
