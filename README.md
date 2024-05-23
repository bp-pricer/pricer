# BP-Pricer

This project is very work in progress. Its supposed to automatically create price suggestions based on backpack.tf listings,
it does so by syncing the listings from the websocket & snapshots into a redis database and then using that data to create price suggestions.

The plan is also to create a web interface for the price suggestions & to visualize the data.

You can also write your own pricing logic using this Project, the main working part of this is the sync of listings inside the database, you can then fetch those listings and calculate the price yourself.



## Installation
1. Setup the database, either run `docker-compose` with the provided `docker-compose.yml` or setup redis on your system.
2. Setup the `.env` file to your liking, the values are pretty self explainatory
3. Run the pricer using `cargo run --release`
4. ???
5. Profit.

## Contributing

If you want to contribute to this project you should contact me, as the project is very much in the early stages and I have a lot of plans for it.
